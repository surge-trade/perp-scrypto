import qrcode
import io
import radix_engine_toolkit as ret
import asyncio
import datetime
import json
from os.path import dirname, join, realpath
from os import makedirs, chdir, environ
from aiohttp import ClientSession, TCPConnector
from subprocess import run
from dotenv import load_dotenv
load_dotenv()

from tools.gateway import Gateway
from tools.accounts import new_account, load_account
from tools.manifests import lock_fee, deposit_all, mint_owner_badge, mint_authority, create_base, create_keeper_reward

timestamp = datetime.datetime.now().strftime("%Y%m%d%H")

def clean(name: str) -> None:
    path = join(dirname(dirname(realpath(__file__))), name)
    print(f'Clean: {path}')
    run(['cargo', 'clean'], cwd=path, check=True)

def build(name: str, envs: list, network: str) -> (bytes, bytes):
    path = join(dirname(dirname(realpath(__file__))), name)
    print(f'Build: {path}')
    
    # env = environ.copy()
    # env.update({str(key): str(value) for key, value in envs})
    # run(['scrypto', 'build'], env=env, cwd=path, check=True)

    run(['docker', 'run', 
        '-v', f'/root/surge-scrypto/{name}:/src',
        '-v', f'/root/surge-scrypto/common:/common', 
        '-v', f'/root/surge-scrypto/account:/account',
        '-v', f'/root/surge-scrypto/pool:/pool'] + 
    [item for pair in [[f'-e', f'{key}={value}'] for key, value in envs] for item in pair] + 
    ['radixdlt/scrypto-builder:v1.1.1'],        
        check=True
    )

    code, definition = None, None
    with open(join(path, f'target/wasm32-unknown-unknown/release/{name}.wasm'), 'rb') as f:
        code = f.read()
    with open(join(path, f'target/wasm32-unknown-unknown/release/{name}.rpd'), 'rb') as f:
        definition = f.read()

    release_path = join(dirname(dirname(realpath(__file__))), 'releases')
    makedirs(release_path, exist_ok=True)

    release_path = join(release_path, timestamp + '_' + network)
    makedirs(release_path, exist_ok=True)

    with open(join(release_path, f'{name}.wasm'), 'wb') as f:
        f.write(code)
    with open(join(release_path, f'{name}.rpd'), 'wb') as f:
        f.write(definition)
    return code, definition

async def main():
    path = dirname(realpath(__file__))
    chdir(path)

    async with ClientSession(connector=TCPConnector(ssl=False)) as session:
        clean('common')
        clean('token_wrapper')
        clean('account')
        clean('config')
        clean('pool')
        clean('fee_distributor')
        clean('fee_delegator')
        clean('exchange')

        gateway = Gateway(session)
        network_config = await gateway.network_configuration()
        account_details = load_account(network_config['network_id'])
        if account_details is None:
            account_details = new_account(network_config['network_id'])
        private_key, public_key, account = account_details

        balance = await gateway.get_xrd_balance(account)
        if balance < 1000:
            print('FUND ACCOUNT:', account.as_str())
            qr = qrcode.QRCode()
            qr.add_data(account.as_str())
            f = io.StringIO()
            qr.print_ascii(out=f)
            f.seek(0)
            print(f.read())
        while balance < 1000:
            await asyncio.sleep(5)
            balance = await gateway.get_xrd_balance(account)

        config_path = join(path, 'config.json')
        with open(config_path, 'r') as config_file:
            config_data = json.load(config_file)
        print('Config loaded:', config_data)

        owner_resource = config_data['OWNER_RESOURCE']
        authority_resource = config_data['AUTHORITY_RESOURCE']
        base_resource = config_data['BASE_RESOURCE']
        keeper_reward_resource = config_data['KEEPER_REWARD_RESOURCE']
        config_component = config_data['CONFIG_COMPONENT']
        pool_component = config_data['POOL_COMPONENT']
        oracle_component = config_data['ORACLE_COMPONENT']
        fee_distributor_component = config_data['FEE_DISTRIBUTOR_COMPONENT']
        fee_delegator_component = config_data['FEE_DELEGATOR_COMPONENT']
        exchange_package = config_data['EXCHANGE_PACKAGE']
        exchange_component = config_data['EXCHANGE_COMPONENT']

        owner_role = ret.OwnerRole.UPDATABLE(ret.AccessRule.require(ret.ResourceOrNonFungible.RESOURCE(ret.Address(owner_resource))))
        manifest_owner_role = ret.ManifestBuilderValue.ENUM_VALUE(2, 
            [ret.ManifestBuilderValue.ENUM_VALUE(2, 
                [ret.ManifestBuilderValue.ENUM_VALUE(0, 
                    [ret.ManifestBuilderValue.ENUM_VALUE(0, 
                        [ret.ManifestBuilderValue.ENUM_VALUE(1, 
                            [ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(owner_resource)))]
                        )]
                    )]
                )]
            )]
        )

        envs = [
            ('NETWORK_ID', network_config['network_id']),
            ('AUTHORITY_RESOURCE', authority_resource),
            ('BASE_RESOURCE', base_resource),
            ('KEEPER_REWARD_RESOURCE', keeper_reward_resource),
        ]

        code, definition = build('exchange', envs, network_config['network_name'])
        payload, intent = await gateway.build_publish_transaction(
            account,
            code,
            definition,
            owner_role,
            public_key,
            private_key,
        )
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        exchange_package = addresses[0]
        envs.append(('EXCHANGE_PACKAGE', exchange_package))
        print('EXCHANGE_PACKAGE:', exchange_package)

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(exchange_component)),
            "withdraw_authority",
            []
        )
        builder = builder.take_all_from_worktop(
            ret.Address(authority_resource),
            ret.ManifestBuilderBucket("authority")
        )
        builder = builder.call_function(
            ret.ManifestBuilderAddress.STATIC(ret.Address(exchange_package)),
            'Exchange',
            'new',
            [
                manifest_owner_role, 
                ret.ManifestBuilderValue.BUCKET_VALUE(ret.ManifestBuilderBucket("authority")),
                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(config_component))),
                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(pool_component))),
                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(oracle_component))),
                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(fee_distributor_component))),
                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(fee_delegator_component))),
            ]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        old_exchange_component = exchange_component
        exchange_component = addresses[0]
        print('EXCHANGE_COMPONENT:', exchange_component)

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.account_create_proof_of_amount(
            account,
            ret.Address(owner_resource),
            ret.Decimal('1')
        )
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(old_exchange_component)),
            'signal_upgrade',
            []
        )

        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Signal upgrade:', status)

        print('---------- DEPLOY COMPLETE ----------')

        print('EXCHANGE_PACKAGE:', exchange_package)
        print('EXCHANGE_COMPONENT:', exchange_component)

        config_data['EXCHANGE_PACKAGE'] = exchange_package
        config_data['EXCHANGE_COMPONENT'] = exchange_component

        release_path = join(dirname(dirname(realpath(__file__))), 'releases')
        release_path = join(release_path, timestamp + '_' + network_config['network_name'])
        
        with open(join(release_path, f'config.json'), 'w') as config_file:
            json.dump(config_data, config_file, indent=4)
        with open(join(path, f'config.json'), 'w') as config_file:
            json.dump(config_data, config_file, indent=4)
        print(f'Config saved')

        print('-------------------------------------')

        # withdraw_account = input("Please enter your address to withdraw: ")
        # balance = await gateway.get_xrd_balance(account)
        # builder = ManifestBuilder()
        # builder = lock_fee(builder, account, 100)
        # builder = builder.account_withdraw(
        #     account,
        #     Address(owner_resource),
        #     Decimal('9')
        # )
        # builder = builder.account_withdraw(
        #     account,
        #     Address(network_config['xrd']),
        #     Decimal(str(balance - 1))
        # )
        # builder = deposit_all(builder, Address(withdraw_account))

        # payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        # await gateway.submit_transaction(payload)

        # print('WITHDRAW SUBMITTED:', intent)

if __name__ == '__main__':
    asyncio.run(main())
