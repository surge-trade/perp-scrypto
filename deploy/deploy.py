from radix_engine_toolkit import *
import asyncio
import datetime
import json
from os.path import dirname, join, realpath
from os import makedirs, chdir
from aiohttp import ClientSession, TCPConnector
from subprocess import run
from dotenv import load_dotenv
load_dotenv()

from tools.gateway import Gateway
from tools.accounts import new_account, load_account
from tools.manifests import lock_fee, deposit_all, mint_owner_badge, mint_authority, create_base, create_keeper_reward

def clean(name: str) -> None:
    path = join(dirname(dirname(realpath(__file__))), name)
    print(f'Clean: {path}')
    run(['cargo', 'clean'], cwd=path, check=True)

def build(name: str, envs: list, network: str) -> (bytes, bytes):
    # path = join(dirname(dirname(realpath(__file__))), name)
    # print(f'Build: {path}')
    # run(['scrypto', 'build'] + [f'{key}={value}' for key, value in envs], cwd=path, check=True)

    run(['docker', 'run', 
        '-v', f'/root/surge-scrypto/{name}:/src',
        '-v', f'/root/surge-scrypto/utils:/utils', 
        '-v', f'/root/surge-scrypto/account:/account',
        '-v', f'/root/surge-scrypto/pool:/pool'] + 
    [item for pair in [[f'-e', f'{key}={value}'] for key, value in envs] for item in pair] + 
    ['radixdlt/scrypto-builder:v1.1.1'],        
        check=True
    )

    path = join(dirname(dirname(realpath(__file__))), name)
    code, definition = None, None
    with open(join(path, f'target/wasm32-unknown-unknown/release/{name}.wasm'), 'rb') as f:
        code = f.read()
    with open(join(path, f'target/wasm32-unknown-unknown/release/{name}.rpd'), 'rb') as f:
        definition = f.read()

    release_path = join(dirname(dirname(realpath(__file__))), 'releases')
    makedirs(release_path, exist_ok=True)

    timestamp = datetime.datetime.now().strftime("%Y%m%d%H")
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
        clean('utils')
        clean('account')
        clean('pool')
        clean('referrals')
        clean('token_wrapper')
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
        while balance < 1000:
            await asyncio.sleep(5)
            balance = await gateway.get_xrd_balance(account)

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = mint_owner_badge(builder)
        builder = deposit_all(builder, account)

        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        owner_resource = addresses[0]
        print('OWNER_RESOURCE:', owner_resource)

        owner_role = OwnerRole.UPDATABLE(AccessRule.require(ResourceOrNonFungible.RESOURCE(Address(owner_resource))))
        manifest_owner_role = ManifestBuilderValue.ENUM_VALUE(2, 
            [ManifestBuilderValue.ENUM_VALUE(2, 
                [ManifestBuilderValue.ENUM_VALUE(0, 
                    [ManifestBuilderValue.ENUM_VALUE(0, 
                        [ManifestBuilderValue.ENUM_VALUE(1, 
                            [ManifestBuilderValue.ADDRESS_VALUE(ManifestBuilderAddress.STATIC(Address(owner_resource)))]
                        )]
                    )]
                )]
            )]
        )

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = mint_authority(builder)
        builder = deposit_all(builder, account)
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        authority_resource = addresses[0]
        print('AUTHORITY_RESOURCE:', authority_resource)

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = create_base(builder, owner_role, authority_resource)
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        base_resource = addresses[0]
        print('BASE_RESOURCE:', base_resource)

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = create_keeper_reward(builder, owner_role, authority_resource)
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        keeper_reward_resource = addresses[0]
        print('KEEPER_REWARD_RESOURCE:', keeper_reward_resource)

        envs = [
            ('NETWORK_ID', network_config['network_id']),
            ('AUTHORITY_RESOURCE', authority_resource),
            ('BASE_RESOURCE', base_resource),
            ('KEEPER_REWARD_RESOURCE', keeper_reward_resource),
        ]

        code, definition = build('token_wrapper', envs, network_config['network_name'])
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
        token_wrapper_package = addresses[0]
        envs.append(('TOKEN_WRAPPER_PACKAGE', token_wrapper_package))
        print('TOKEN_WRAPPER_PACKAGE:', token_wrapper_package)

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.account_withdraw(
            account,
            Address(authority_resource),
            Decimal('0.000000000000000001')
        )
        builder = builder.take_from_worktop(
            Address(authority_resource),
            Decimal('0.000000000000000001'),
            ManifestBuilderBucket("authority")
        )
        builder = builder.call_function(
            ManifestBuilderAddress.STATIC(Address(token_wrapper_package)),
            'TokenWrapper',
            'new',
            [manifest_owner_role, ManifestBuilderValue.BUCKET_VALUE(ManifestBuilderBucket("authority"))]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        token_wrapper_component = addresses[0]
        print('TOKEN_WRAPPER_COMPONENT:', token_wrapper_component)

        code, definition = build('account', envs, network_config['network_name'])
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
        account_package = addresses[0]
        envs.append(('ACCOUNT_PACKAGE', account_package))
        print('ACCOUNT_PACKAGE:', account_package)

        code, definition = build('pool', envs, network_config['network_name'])
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
        pool_package = addresses[0]
        envs.append(('POOL_PACKAGE', pool_package))
        print('POOL_PACKAGE:', pool_package)

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_function(
            ManifestBuilderAddress.STATIC(Address(pool_package)),
            'MarginPool',
            'new',
            [manifest_owner_role]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        pool_component, lp_resource = addresses[0], addresses[1]
        print('POOL_COMPONENT:', pool_component)
        print('LP_RESOURCE:', lp_resource)

        code, definition = build('oracle', envs, network_config['network_name'])
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
        oracle_package = addresses[0]
        envs.append(('ORACLE_PACKAGE', oracle_package))
        print('ORACLE_PACKAGE:', oracle_package)

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_function(
            ManifestBuilderAddress.STATIC(Address(oracle_package)),
            'Oracle',
            'new',
            [manifest_owner_role]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        oracle_component = addresses[0]
        print('ORACLE_COMPONENT:', oracle_component)

        code, definition = build('referrals', envs, network_config['network_name'])
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
        referrals_package = addresses[0]
        envs.append(('REFERRALS_PACKAGE', referrals_package))
        print('REFERRALS_PACKAGE:', referrals_package)

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_function(
            ManifestBuilderAddress.STATIC(Address(referrals_package)),
            'Referrals',
            'new',
            [manifest_owner_role]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        referrals_component = addresses[0]
        print('REFERRALS_COMPONENT:', referrals_component)

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

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.account_withdraw(
            account,
            Address(authority_resource),
            Decimal('0.999999999999999999')
        )            
        builder = builder.take_from_worktop(
            Address(authority_resource),
            Decimal('0.999999999999999999'),
            ManifestBuilderBucket("authority")
        )
        builder = builder.call_function(
            ManifestBuilderAddress.STATIC(Address(exchange_package)),
            'Exchange',
            'new',
            [
                manifest_owner_role, 
                ManifestBuilderValue.BUCKET_VALUE(ManifestBuilderBucket("authority")),
                ManifestBuilderValue.ADDRESS_VALUE(ManifestBuilderAddress.STATIC(Address(pool_component))),
                ManifestBuilderValue.ADDRESS_VALUE(ManifestBuilderAddress.STATIC(Address(oracle_component))),
                ManifestBuilderValue.ADDRESS_VALUE(ManifestBuilderAddress.STATIC(Address(referrals_component))),
            ]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        exchange_component = addresses[0]
        print('EXCHANGE_COMPONENT:', exchange_component)

        print('---------- DEPLOY COMPLETE ----------')

        print('OWNER_RESOURCE:', owner_resource)
        print('AUTHORITY_RESOURCE:', authority_resource)
        print('BASE_RESOURCE:', base_resource)
        print('KEEPER_REWARD_RESOURCE:', keeper_reward_resource)

        print('TOKEN_WRAPPER_PACKAGE:', token_wrapper_package)
        print('ACCOUNT_PACKAGE:', account_package)
        print('POOL_PACKAGE:', pool_package)
        print('ORACLE_PACKAGE:', oracle_package)
        print('REFERRALS_PACKAGE:', referrals_package)
        print('EXCHANGE_PACKAGE:', exchange_package)

        print('TOKEN_WRAPPER_COMPONENT:', token_wrapper_component)
        print('POOL_COMPONENT:', pool_component)
        print('ORACLE_COMPONENT:', oracle_component)
        print('REFERRALS_COMPONENT:', referrals_component)
        print('EXCHANGE_COMPONENT:', exchange_component)

        config_data = {
            'OWNER_RESOURCE': owner_resource,
            'AUTHORITY_RESOURCE': authority_resource,
            'BASE_RESOURCE': base_resource,
            'KEEPER_REWARD_RESOURCE': keeper_reward_resource,
            'TOKEN_WRAPPER_PACKAGE': token_wrapper_package,
            'ACCOUNT_PACKAGE': account_package,
            'POOL_PACKAGE': pool_package,
            'ORACLE_PACKAGE': oracle_package,
            'REFERRALS_PACKAGE': referrals_package,
            'EXCHANGE_PACKAGE': exchange_package,
            'TOKEN_WRAPPER_COMPONENT': token_wrapper_component,
            'POOL_COMPONENT': pool_component,
            'ORACLE_COMPONENT': oracle_component,
            'REFERRALS_COMPONENT': referrals_component,
            'EXCHANGE_COMPONENT': exchange_component
        }

        release_path = join(dirname(dirname(realpath(__file__))), 'releases')
        timestamp = datetime.datetime.now().strftime("%Y%m%d%H")
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
