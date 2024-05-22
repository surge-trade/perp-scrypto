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
from tools.manifests import lock_fee, deposit_all, mint_owner_badge, mint_authority, mint_base_authority, create_base, create_keeper_reward

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
        '-v', f'/root/surge-scrypto/config:/config', 
        '-v', f'/root/surge-scrypto/account:/account',
        '-v', f'/root/surge-scrypto/permission_registry:/permission_registry',
        '-v', f'/root/surge-scrypto/pool:/pool',
        '-v', f'/root/surge-scrypto/referral_generator:/referral_generator',
        ] + 
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
        clean('env_registry')
        clean('pool')
        clean('referral_generator')
        clean('permission_registry')
        clean('fee_distributor')
        clean('fee_delegator')
        clean('exchange')

        gateway = Gateway(session)
        network_config = await gateway.network_configuration()
        account_details = load_account(network_config['network_id'])
        if account_details is None:
            account_details = new_account(network_config['network_id'])
        private_key, public_key, account = account_details

        print('ACCOUNT:', account.as_str())
        balance = await gateway.get_xrd_balance(account)
        if balance < 10000:
            builder = ret.ManifestBuilder()
            builder = builder.call_method(
                ret.ManifestBuilderAddress.STATIC(ret.Address(network_config['faucet'])),
                'lock_fee',
                [ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('100'))]
            )
            builder = builder.call_method(
                ret.ManifestBuilderAddress.STATIC(ret.Address(network_config['faucet'])),
                'free',
                []
            )
            builder = deposit_all(builder, account)

            payload, intent = await gateway.build_transaction(builder, public_key, private_key)
            await gateway.submit_transaction(payload)

        #     print('FUND ACCOUNT:', account.as_str())
        #     qr = qrcode.QRCode()
        #     qr.add_data(account.as_str())
        #     f = io.StringIO()
        #     qr.print_ascii(out=f)
        #     f.seek(0)
        #     print(f.read())
        # while balance < 10000:
        #     await asyncio.sleep(5)
        #     balance = await gateway.get_xrd_balance(account)

        state_version = await gateway.get_state_version()
        print('STATE_VERSION:', state_version)

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = mint_owner_badge(builder)
        builder = deposit_all(builder, account)

        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        owner_resource = addresses[0]
        print('OWNER_RESOURCE:', owner_resource)

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

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = mint_authority(builder)
        builder = deposit_all(builder, account)
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        authority_resource = addresses[0]
        print('AUTHORITY_RESOURCE:', authority_resource)

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = mint_base_authority(builder)
        builder = deposit_all(builder, account)
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        base_authority_resource = addresses[0]
        print('BASE_AUTHORITY_RESOURCE:', base_authority_resource)

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = create_base(builder, owner_role, base_authority_resource)
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        base_resource = addresses[0]
        print('BASE_RESOURCE:', base_resource)

        builder = ret.ManifestBuilder()
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
            ('BASE_AUTHORITY_RESOURCE', base_authority_resource),
            ('BASE_RESOURCE', base_resource),
            ('KEEPER_REWARD_RESOURCE', keeper_reward_resource),
        ]

        oracle_key = 'b9dca0b122bc34356550c32beb31c726f993fcf1fb16aecdbe95b5181e8505b98c5f1286969664d69c4358dc16261640'
        oracle_key_array = [ret.ManifestBuilderValue.U8_VALUE(b) for b in bytes.fromhex(oracle_key)]
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

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_function(
            ret.ManifestBuilderAddress.STATIC(ret.Address(oracle_package)),
            'Oracle',
            'new',
            [manifest_owner_role, ret.ManifestBuilderValue.ARRAY_VALUE(ret.ManifestBuilderValueKind.U8_VALUE, oracle_key_array)]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        oracle_component = addresses[0]
        print('ORACLE_COMPONENT:', oracle_component)

        # code, definition = build('env_registry', envs, network_config['network_name'])
        # payload, intent = await gateway.build_publish_transaction(
        #     account,
        #     code,
        #     definition,
        #     owner_role,
        #     public_key,
        #     private_key,
        # )
        # await gateway.submit_transaction(payload)
        # addresses = await gateway.get_new_addresses(intent)
        # env_registry_package = addresses[0]
        # envs.append(('ENV_REGISTRY_PACKAGE', env_registry_package))
        # print('ENV_REGISTRY_PACKAGE:', env_registry_package)

        # builder = ret.ManifestBuilder()
        # builder = lock_fee(builder, account, 100)
        # builder = builder.call_function(
        #     ret.ManifestBuilderAddress.STATIC(ret.Address(env_registry_package)),
        #     'EnvRegistry',
        #     'new',
        #     [manifest_owner_role]
        # )
        # payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        # await gateway.submit_transaction(payload)
        # addresses = await gateway.get_new_addresses(intent)
        # env_registry_component = addresses[0]
        # print('ENV_REGISTRY_COMPONENT:', env_registry_component)

        temp_config_path = join(path, 'config.json')
        with open(temp_config_path, 'r') as config_file:
            temp_config_data = json.load(config_file)
        env_registry_package = temp_config_data['ENV_REGISTRY_PACKAGE']
        env_registry_component = temp_config_data['ENV_REGISTRY_COMPONENT']

        code, definition = build('permission_registry', envs, network_config['network_name'])
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
        permission_registry_package = addresses[0]
        envs.append(('PERMISSION_REGISTRY_PACKAGE', permission_registry_package))
        print('PERMISSION_REGISTRY_PACKAGE:', permission_registry_package)

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_function(
            ret.ManifestBuilderAddress.STATIC(ret.Address(permission_registry_package)),
            'PermissionRegistry',
            'new',
            [manifest_owner_role]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        permission_registry_component = addresses[0]
        print('PERMISSION_REGISTRY_COMPONENT:', permission_registry_component)

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

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.account_withdraw(
            account,
            ret.Address(base_authority_resource),
            ret.Decimal('1')
        )
        builder = builder.take_from_worktop(
            ret.Address(base_authority_resource),
            ret.Decimal('1'),
            ret.ManifestBuilderBucket("authority")
        )
        builder = builder.call_function(
            ret.ManifestBuilderAddress.STATIC(ret.Address(token_wrapper_package)),
            'TokenWrapper',
            'new',
            [manifest_owner_role, ret.ManifestBuilderValue.BUCKET_VALUE(ret.ManifestBuilderBucket("authority"))]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        token_wrapper_component = addresses[0]
        print('TOKEN_WRAPPER_COMPONENT:', token_wrapper_component)

        code, definition = build('config', envs, network_config['network_name'])
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
        config_package = addresses[0]
        envs.append(('CONFIG_PACKAGE', config_package))
        print('CONFIG_PACKAGE:', config_package)

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_function(
            ret.ManifestBuilderAddress.STATIC(ret.Address(config_package)),
            'Config',
            'new',
            [manifest_owner_role]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        config_component = addresses[0]
        print('CONFIG_COMPONENT:', config_component)

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

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_function(
            ret.ManifestBuilderAddress.STATIC(ret.Address(pool_package)),
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

        code, definition = build('referral_generator', envs, network_config['network_name'])
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
        referral_generator_package = addresses[0]
        envs.append(('REFERRAL_GENERATOR_PACKAGE', referral_generator_package))
        print('REFERRAL_GENERATOR_PACKAGE:', referral_generator_package)

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_function(
            ret.ManifestBuilderAddress.STATIC(ret.Address(referral_generator_package)),
            'ReferralGenerator',
            'new',
            [manifest_owner_role]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        referral_generator_component = addresses[0]
        print('REFERRAL_GENERATOR_COMPONENT:', referral_generator_component)

        code, definition = build('fee_distributor', envs, network_config['network_name'])
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
        fee_distributor_package = addresses[0]
        envs.append(('FEE_DISTRIBUTOR_PACKAGE', fee_distributor_package))
        print('FEE_DISTRIBUTOR_PACKAGE:', fee_distributor_package)

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_function(
            ret.ManifestBuilderAddress.STATIC(ret.Address(fee_distributor_package)),
            'FeeDistributor',
            'new',
            [manifest_owner_role]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        fee_distributor_component = addresses[0]
        print('FEE_DISTRIBUTOR_COMPONENT:', fee_distributor_component)

        code, definition = build('fee_delegator', envs, network_config['network_name'])
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
        fee_delegator_package = addresses[0]
        envs.append(('FEE_DELEGATOR_PACKAGE', fee_delegator_package))
        print('FEE_DELEGATOR_PACKAGE:', fee_delegator_package)

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_function(
            ret.ManifestBuilderAddress.STATIC(ret.Address(fee_delegator_package)),
            'FeeDelegator',
            'new',
            [manifest_owner_role]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        fee_delegator_component = addresses[0]
        print('FEE_DELEGATOR_COMPONENT:', fee_delegator_component)

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
        builder = builder.account_withdraw(
            account,
            ret.Address(authority_resource),
            ret.Decimal('1')
        )            
        builder = builder.take_from_worktop(
            ret.Address(authority_resource),
            ret.Decimal('1'),
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
                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(referral_generator_component))),
                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(permission_registry_component))),
                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(oracle_component))),
                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(fee_distributor_component))),
                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(fee_delegator_component))),
            ]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        exchange_component = addresses[0]
        print('EXCHANGE_COMPONENT:', exchange_component)

        manifest = f'''
            CALL_METHOD
                Address("{account.as_str()}")
                "lock_fee"
                Decimal("10")
            ;
            CALL_METHOD
                Address("{account.as_str()}")
                "create_proof_of_amount"
                Address("{owner_resource}")
                Decimal("1")
            ;
            CALL_METHOD
                Address("{env_registry_component}")
                "set_variables"
                Array<String>(
                    "{exchange_component}"
                )
            ;
        '''

        payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Register exchange:', status)

        print('---------- DEPLOY COMPLETE ----------')

        print(f'STATE_VERSION={state_version}')

        print(f'OWNER_RESOURCE={owner_resource}')
        print(f'AUTHORITY_RESOURCE={authority_resource}')
        print(f'BASE_AUTHORITY_RESOURCE={base_authority_resource}')
        print(f'BASE_RESOURCE={base_resource}')
        print(f'KEEPER_REWARD_RESOURCE={keeper_reward_resource}')

        print(f'TOKEN_WRAPPER_PACKAGE={token_wrapper_package}')
        print(f'CONFIG_PACKAGE={config_package}')
        print(f'ACCOUNT_PACKAGE={account_package}')
        print(f'POOL_PACKAGE={pool_package}')
        print(f'REFERRAL_GENERATOR_PACKAGE={referral_generator_package}')
        print(f'PERMISSION_REGISTRY_PACKAGE={permission_registry_package}')
        print(f'ORACLE_PACKAGE={oracle_package}')
        print(f'FEE_DISTRIBUTOR_PACKAGE={fee_distributor_package}')
        print(f'FEE_DELEGATOR_PACKAGE={fee_delegator_package}')
        print(f'ENV_REGISTRY_PACKAGE={env_registry_package}')
        print(f'EXCHANGE_PACKAGE={exchange_package}')

        print(f'TOKEN_WRAPPER_COMPONENT={token_wrapper_component}')
        print(f'CONFIG_COMPONENT={config_component}')
        print(f'POOL_COMPONENT={pool_component}')
        print(f'REFERRAL_GENERATOR_COMPONENT={referral_generator_component}')
        print(f'PERMISSION_REGISTRY_COMPONENT={permission_registry_component}')
        print(f'ORACLE_COMPONENT={oracle_component}')
        print(f'FEE_DISTRIBUTOR_COMPONENT={fee_distributor_component}')
        print(f'FEE_DELEGATOR_COMPONENT={fee_delegator_component}')
        print(f'ENV_REGISTRY_COMPONENT={env_registry_component}')
        print(f'EXCHANGE_COMPONENT={exchange_component}')

        config_data = {
            'OWNER_RESOURCE': owner_resource,
            'AUTHORITY_RESOURCE': authority_resource,
            'BASE_AUTHORITY_RESOURCE': base_authority_resource,
            'BASE_RESOURCE': base_resource,
            'KEEPER_REWARD_RESOURCE': keeper_reward_resource,
            'TOKEN_WRAPPER_PACKAGE': token_wrapper_package,
            'CONFIG_PACKAGE': config_package,
            'ACCOUNT_PACKAGE': account_package,
            'POOL_PACKAGE': pool_package,
            'REFERRAL_GENERATOR_PACKAGE': referral_generator_package,
            'PERMISSION_REGISTRY_PACKAGE': permission_registry_package,
            'ORACLE_PACKAGE': oracle_package,
            'FEE_DISTRIBUTOR_PACKAGE': fee_distributor_package,
            'FEE_DELEGATOR_PACKAGE': fee_delegator_package,
            'ENV_REGISTRY_PACKAGE': env_registry_package,
            'EXCHANGE_PACKAGE': exchange_package,
            'TOKEN_WRAPPER_COMPONENT': token_wrapper_component,
            'CONFIG_COMPONENT': config_component,
            'POOL_COMPONENT': pool_component,
            'REFERRAL_GENERATOR_COMPONENT': referral_generator_component,
            'PERMISSION_REGISTRY_COMPONENT': permission_registry_component,
            'ORACLE_COMPONENT': oracle_component,
            'FEE_DISTRIBUTOR_COMPONENT': fee_distributor_component,
            'FEE_DELEGATOR_COMPONENT': fee_delegator_component,
            'ENV_REGISTRY_COMPONENT': env_registry_component,
            'EXCHANGE_COMPONENT': exchange_component
        }

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
