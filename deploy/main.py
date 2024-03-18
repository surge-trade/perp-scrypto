from radix_engine_toolkit import *
import asyncio
from os.path import dirname, join, realpath
from aiohttp import ClientSession, TCPConnector
from subprocess import run
from dotenv import load_dotenv
load_dotenv()

from tools.gateway import Gateway
from tools.accounts import new_account, load_account
from tools.manifests import lock_fee, deposit_all, mint_owner_badge, mint_authority, create_base

def clean(name: str) -> None:
    path = join(dirname(dirname(realpath(__file__))), name)
    print(f'Clean: {path}')
    run(['cargo', 'clean'], cwd=path, check=True)

def build(name: str, vars: list) -> (bytes, bytes):
    path = join(dirname(dirname(realpath(__file__))), name)
    print(f'Build: {path}')
    run(['scrypto', 'build'] + [f'{key}={value}' for key, value in vars], cwd=path, check=True)

    code, definition = None, None
    with open(join(path, f'target/wasm32-unknown-unknown/release/{name}.wasm'), 'rb') as f:
        code = f.read()
    with open(join(path, f'target/wasm32-unknown-unknown/release/{name}.rpd'), 'rb') as f:
        definition = f.read()
    return code, definition

async def main():
    async with ClientSession(connector=TCPConnector(ssl=False)) as session:
        # clean('utils')
        # clean('account')
        # clean('pool')
        # clean('referrals')
        # clean('token_wrapper')
        # clean('exchange')

        gateway = Gateway(session)
        network_config = await gateway.network_configuration()
        account_details = load_account(network_config['network_id'])
        if account_details is None:
            account_details = new_account(network_config['network_id'])
        private_key, public_key, account = account_details

        # balance = await core_node.get_account_balance(account, XRD)
        # if balance < 100:
        #     print('FUND ACCOUNT:', public_key)
        # while balance < 100:
        #     await asyncio.sleep(5)
        #     balance = await core_node.get_account_balance(account, XRD)

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = mint_owner_badge(builder)
        builder = deposit_all(builder, account)

        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        owner_resource = await gateway.get_new_addresses(intent)[0]
        owner_role = OwnerRole.UPDATABLE(AccessRule.require(ResourceOrNonFungible.RESOURCE(Address(owner_resource)))),
        manifest_owner_role = ManifestBuilderValue.ENUM_VALUE(2, 
            [ManifestBuilderValue.ENUM_VALUE(2, 
                [ManifestBuilderValue.ENUM_VALUE(0, 
                    [ManifestBuilderValue.ENUM_VALUE(0, 
                        [ManifestBuilderValue.ENUM_VALUE(1, 
                            Address(owner_resource)
                        )]
                    )]
                )]
            )]
        )
        none = ManifestBuilderValue.ENUM_VALUE(0, [])

        # Enum<2u8>(
        #     Enum<2u8>(
        #         Enum<0u8>(
        #             Enum<0u8>(
        #                 Enum<1u8>(
        #                     Address("resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3")
        #                 )
        #             )
        #         )
        #     )
        # )

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = mint_authority(builder)
        builder = deposit_all(builder, account)
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        authority_resource = await gateway.get_new_addresses(intent)[0]

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = create_base(builder, owner_role, authority_resource)
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        base_resource = await gateway.get_new_addresses(intent)[0]

        vars = [
            ('NETWORK_ID', network_config['network_id']),
            ('AUTHORITY_RESOURCE', authority_resource),
            ('BASE_RESOURCE', base_resource),
        ]

        code, definition = build('token_wrapper', vars)
        payload, intent = await gateway.build_publish_transaction(
            account,
            code,
            definition,
            owner_role,
            public_key,
            private_key,
        )
        await gateway.submit_transaction(payload)
        token_wrapper_package = await gateway.get_new_addresses(intent)[0]

        vars.append(('TOKEN_WRAPPER_PACKAGE', token_wrapper_package))

        code, definition = build('account', vars)
        payload, intent = await gateway.build_publish_transaction(
            account,
            code,
            definition,
            owner_role,
            public_key,
            private_key,
        )
        await gateway.submit_transaction(payload)
        account_package = await gateway.get_new_addresses(intent)[0]

        vars.append(('ACCOUNT_PACKAGE', account_package))

        code, definition = build('pool', vars)
        payload, intent = await gateway.build_publish_transaction(
            account,
            code,
            definition,
            owner_role,
            public_key,
            private_key,
        )
        await gateway.submit_transaction(payload)
        pool_package = await gateway.get_new_addresses(intent)[0]

        vars.append(('POOL_PACKAGE', pool_package))

        code, definition = build('oracle', vars)
        payload, intent = await gateway.build_publish_transaction(
            account,
            code,
            definition,
            owner_role,
            public_key,
            private_key,
        )
        await gateway.submit_transaction(payload)
        oracle_package = await gateway.get_new_addresses(intent)[0]

        vars.append(('ORACLE_PACKAGE', oracle_package))

        code, definition = build('referrals', vars)
        payload, intent = await gateway.build_publish_transaction(
            account,
            code,
            definition,
            owner_role,
            public_key,
            private_key,
        )
        await gateway.submit_transaction(payload)
        referrals_package = await gateway.get_new_addresses(intent)[0]

        vars.append(('REFERRALS_PACKAGE', referrals_package))

        code, definition = build('exchange', vars)
        payload, intent = await gateway.build_publish_transaction(
            account,
            code,
            definition,
            owner_role,
            public_key,
            private_key,
        )
        await gateway.submit_transaction(payload)
        exchange_package = await gateway.get_new_addresses(intent)[0]

        vars.append(('EXCHANGE_PACKAGE', exchange_package))

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_function(
            pool_package,
            'TokenWrapper',
            'new',
            [manifest_owner_role]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        token_wrapper_component = await gateway.get_new_addresses(intent)[0]

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_function(
            pool_package,
            'MarginPool',
            'new',
            [manifest_owner_role]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        temp = await gateway.get_new_addresses(intent)
        pool_component, lp_resource = temp[0], temp[1]

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_function(
            pool_package,
            'Oracle',
            'new',
            [manifest_owner_role]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        oracle_component = await gateway.get_new_addresses(intent)[0]

        builder = ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_function(
            pool_package,
            'Referrals',
            'new',
            [manifest_owner_role]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        referrals_component = await gateway.get_new_addresses(intent)[0]

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
            pool_package,
            'Exchange',
            'new',
            [
                manifest_owner_role, 
                ManifestBuilderValue.BUCKET_VALUE(ManifestBuilderBucket("authority")),
                ManifestBuilderValue.ADDRESS_VALUE(Address(pool_component)),
                ManifestBuilderValue.ADDRESS_VALUE(Address(oracle_component)),
                ManifestBuilderValue.ADDRESS_VALUE(Address(referrals_component)),
            ]
        )
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        exchange_component = await gateway.get_new_addresses(intent)[0]

        
if __name__ == '__main__':
    asyncio.run(main())
