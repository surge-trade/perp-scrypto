import qrcode
import io
import radix_engine_toolkit as ret
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
from tools.manifests import lock_fee, deposit_all

async def main():
    path = dirname(realpath(__file__))
    chdir(path)

    async with ClientSession(connector=TCPConnector(ssl=False)) as session:
        gateway = Gateway(session)
        network_config = await gateway.network_configuration()
        account_details = load_account(network_config['network_id'])
        if account_details is None:
            account_details = new_account(network_config['network_id'])
        private_key, public_key, account = account_details

        config_path = join(path, 'config.json')
        with open(config_path, 'r') as config_file:
            config_data = json.load(config_file)

        owner_resource = config_data['OWNER_RESOURCE']
        oracle_component = config_data['ORACLE_COMPONENT']
        exchange_component = config_data['EXCHANGE_COMPONENT']

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
                    Address("{oracle_component}")
                    "update_pairs"
                    Bytes("5c202104030c074254432f555344a000b8e659da8d7e9d1f0e000000000000000000000000000005078c6b6600000000030c074554482f555344a0e093c65644365286bb00000000000000000000000000000005111c6b6600000000030c075852442f555344a000bc6c3d82037d000000000000000000000000000000000005111c6b6600000000030c07534f4c2f555344a00068b3d15b5ed9fe0700000000000000000000000000000005551a6b6600000000")
                    Bytes("a9bf994ab6f2861e40760cceca7b777f5f3877eb8ebffddff009ca7faa2c7d0c3d576fa3c1cb873f0e9ef3d4df6b54ff0e0bfc8f8fcac78b3bbb2fe2b9fbdea126b7512df2a252081652441b7e7871bfbae7a693e430e24bd5a50123e8b70e80")
                ;
            '''

        payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Transaction status:', status)

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.account_create_proof_of_amount(
            account,
            ret.Address(owner_resource),
            ret.Decimal('1')
        )
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(exchange_component)),
            'update_pair_configs',
            [ret.ManifestBuilderValue.ARRAY_VALUE(ret.ManifestBuilderValueKind.TUPLE_VALUE, [
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('BTC/USD'),  # pub pair_id: PairId,
                    ret.ManifestBuilderValue.BOOL_VALUE(False), # pub disabled: bool,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(600), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0'))  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('ETH/USD'),  # pub pair_id: PairId,
                    ret.ManifestBuilderValue.BOOL_VALUE(False), # pub disabled: bool,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(600), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0'))  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE("XRD/USD"),  # pub pair_id: PairId,
                    ret.ManifestBuilderValue.BOOL_VALUE(False), # pub disabled: bool,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(600), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0'))  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE("SOL/USD"),  # pub pair_id: PairId,
                    ret.ManifestBuilderValue.BOOL_VALUE(False), # pub disabled: bool,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(600), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0'))  # pub fee_1: Decimal,
                ]),
            ])]
        )

        # builder = builder.call_method(
        #     ret.ManifestBuilderAddress.STATIC(ret.Address(exchange_component)),
        #     'update_pair_configs',
        #     [ret.ManifestBuilderValue.ARRAY_VALUE(ret.ManifestBuilderValueKind.TUPLE_VALUE, [
        #         ret.ManifestBuilderValue.TUPLE_VALUE([
        #             ret.ManifestBuilderValue.STRING_VALUE('BTC/USD'),  # pub pair_id: PairId,
        #             ret.ManifestBuilderValue.BOOL_VALUE(False), # pub disabled: bool,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # update_price_delta_ratio
        #             ret.ManifestBuilderValue.I64_VALUE(3600), # update_period_seconds
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # pub margin_initial: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # pub margin_maintenance: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000317')), # pub funding_1: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000317')),  # pub funding_2: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.000000827')),  # pub funding_2_delta: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000159')),  # pub funding_pool_0: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000317')),  # pub funding_pool_1: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.1')),  # pub funding_share: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0005')),  # pub fee_0: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.1'))  # pub fee_1: Decimal,
        #         ]),
        #         ret.ManifestBuilderValue.TUPLE_VALUE([
        #             ret.ManifestBuilderValue.STRING_VALUE('ETH/USD'),  # pub pair_id: PairId,
        #             ret.ManifestBuilderValue.BOOL_VALUE(False), # pub disabled: bool,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # update_price_delta_ratio
        #             ret.ManifestBuilderValue.I64_VALUE(3600), # update_period_seconds
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # pub margin_initial: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # pub margin_maintenance: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000317')), # pub funding_1: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000317')),  # pub funding_2: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.000000827')),  # pub funding_2_delta: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000159')),  # pub funding_pool_0: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000317')),  # pub funding_pool_1: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.1')),  # pub funding_share: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0005')),  # pub fee_0: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.1'))  # pub fee_1: Decimal,
        #         ]),
        #         ret.ManifestBuilderValue.TUPLE_VALUE([
        #             ret.ManifestBuilderValue.STRING_VALUE("XRD/USD"),  # pub pair_id: PairId,
        #             ret.ManifestBuilderValue.BOOL_VALUE(False), # pub disabled: bool,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # update_price_delta_ratio
        #             ret.ManifestBuilderValue.I64_VALUE(3600), # update_period_seconds
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # pub margin_initial: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # pub margin_maintenance: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000317')), # pub funding_1: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000317')),  # pub funding_2: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.000000827')),  # pub funding_2_delta: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000159')),  # pub funding_pool_0: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000317')),  # pub funding_pool_1: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.1')),  # pub funding_share: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0005')),  # pub fee_0: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.1'))  # pub fee_1: Decimal,
        #         ]),
        #         ret.ManifestBuilderValue.TUPLE_VALUE([
        #             ret.ManifestBuilderValue.STRING_VALUE('SOL/USD'),  # pub pair_id: PairId,
        #             ret.ManifestBuilderValue.BOOL_VALUE(False), # pub disabled: bool,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # update_price_delta_ratio
        #             ret.ManifestBuilderValue.I64_VALUE(3600), # update_period_seconds
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # pub margin_initial: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # pub margin_maintenance: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000317')), # pub funding_1: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000317')),  # pub funding_2: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.000000827')),  # pub funding_2_delta: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000159')),  # pub funding_pool_0: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0000000317')),  # pub funding_pool_1: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.1')),  # pub funding_share: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0005')),  # pub fee_0: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.1'))  # pub fee_1: Decimal,
        #         ]),
        #     ])]
        # )

        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        print('Transaction id:', intent)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Transaction status:', status)

if __name__ == '__main__':
    asyncio.run(main())

