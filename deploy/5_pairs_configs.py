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

        # manifest = f'''
        #         CALL_METHOD
        #             Address("{account.as_str()}")
        #             "lock_fee"
        #             Decimal("10")
        #         ;
        #         CALL_METHOD
        #             Address("{account.as_str()}")
        #             "create_proof_of_amount"
        #             Address("{owner_resource}")
        #             Decimal("1")
        #         ;
        #         CALL_METHOD
        #             Address("{oracle_component}")
        #             "update_pairs"
        #             Bytes("5c202114030c074254432f555344a0000072fa3b722504db0d000000000000000000000000000005d8c2736600000000030c074554482f555344a00020d89f237caecac100000000000000000000000000000005d8c2736600000000030c075852442f555344a00040db87a7bc75000000000000000000000000000000000005d8c2736600000000030c07424e422f555344a000c0fb137d72b49d2000000000000000000000000000000005d8c2736600000000030c07534f4c2f555344a000405897e0bfcc5b0700000000000000000000000000000005d8c2736600000000030c075852502f555344a000343d11b7b9dd060000000000000000000000000000000005d8c2736600000000030c08444f47452f555344a000345c7f8fcfb7010000000000000000000000000000000005d8c2736600000000030c074144412f555344a0001818d76c1f6c050000000000000000000000000000000005d8c2736600000000030c08415641582f555344a0d0d324e2ab291e7e0100000000000000000000000000000005d8c2736600000000030c084c494e4b2f555344a00018280c3a429fc90000000000000000000000000000000005d8c2736600000000030c07444f542f555344a0e81f1646f8ec87510000000000000000000000000000000005d8c2736600000000030c084e4541522f555344a000c0af17cc62e8470000000000000000000000000000000005d8c2736600000000030c094d415449432f555344a00024308549a917080000000000000000000000000000000005d8c2736600000000030c074c54432f555344a000446411794d3e060400000000000000000000000000000005d8c2736600000000030c0841544f4d2f555344a000e441a40f47dd600000000000000000000000000000000005d8c2736600000000030c075355492f555344a00070dad8035a2c0c0000000000000000000000000000000005d8c2736600000000030c074150542f555344a0e83fc2a8190d17610000000000000000000000000000000005d8c2736600000000030c074152422f555344a00068dd4c1a99720b0000000000000000000000000000000005d8c2736600000000030c07494e4a2f555344a000a8372c2820642d0100000000000000000000000000000005d8c2736600000000030c075345492f555344a000b079abe30519050000000000000000000000000000000005d8c2736600000000")
        #             Bytes("81712c4d0bda15de30006d571853e5ec41a06363bc23a594e24a9d90b46de97cac920d2ca9704ee80acb944675be94b7158b7b7c08f38291636fc55124319890baa21431d12964365228ba41c3c685f6155355b8b8a451999785f719f49c1535")
        #         ;
        #     '''

        # payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
        # print('Transaction id:', intent)
        # await gateway.submit_transaction(payload)
        # status = await gateway.get_transaction_status(intent)
        # print('Transaction status:', status)

        # pairs = [
        #     'BTC/USD', 
        #     'ETH/USD', 
        #     'XRD/USD', 
        #     # 'BNB/USD', 
        #     'SOL/USD', 
        #     # 'XRP/USD', 
        #     # 'DOGE/USD', 
        #     # 'ADA/USD', 
        #     # 'AVAX/USD', 
        #     # 'LINK/USD', 
        #     # 'DOT/USD', 
        #     # 'NEAR/USD', 
        #     # 'LTC/USD', 
        #     # 'ATOM/USD', 
        #     # 'SUI/USD', 
        #     # 'APT/USD', 
        #     # 'ARB/USD', 
        #     # 'INJ/USD', 
        #     # 'SEI/USD',
        # ]

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.account_create_proof_of_amount(
            account,
            ret.Address(owner_resource),
            ret.Decimal('4')
        )
        # builder = builder.call_method(
        #     ret.ManifestBuilderAddress.STATIC(ret.Address(exchange_component)),
        #     'update_pair_configs',
        #     [ret.ManifestBuilderValue.ARRAY_VALUE(ret.ManifestBuilderValueKind.TUPLE_VALUE, [
        #         ret.ManifestBuilderValue.TUPLE_VALUE([
        #             ret.ManifestBuilderValue.STRING_VALUE(pair), # pub pair_id: PairId,
        #             ret.ManifestBuilderValue.I64_VALUE(5), # price_age_max
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('10000000')), # pub oi_max: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # update_price_delta_ratio
        #             ret.ManifestBuilderValue.I64_VALUE(3600), # update_period_seconds
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # pub margin_initial: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # pub margin_maintenance: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1')), # pub funding_1: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1')),  # pub funding_2: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('100')),  # pub funding_2_delta: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('100')),  # pub funding_2_decay: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.02')),  # pub funding_pool_0: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.25')),  # pub funding_pool_1: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.05')),  # pub funding_share: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.0005')),  # pub fee_0: Decimal,
        #             ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.00000002'))  # pub fee_1: Decimal,
        #         ])
        #         for pair in pairs
        #     ])]
        # )
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(exchange_component)),
            'update_pair_configs',
            [ret.ManifestBuilderValue.ARRAY_VALUE(ret.ManifestBuilderValueKind.TUPLE_VALUE, [
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('BTC/USD'), # pub pair_id: PairId,
                    ret.ManifestBuilderValue.I64_VALUE(5), # price_age_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('50')), # pub oi_max: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(3600), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.1')), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1')), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1')),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('50')),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('100')),  # pub funding_2_decay: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.02')),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.25')),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.05')),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.00000001'))  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('ETH/USD'), # pub pair_id: PairId,
                    ret.ManifestBuilderValue.I64_VALUE(5), # price_age_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('700')), # pub oi_max: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(3600), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.1')), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1')), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1')),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('50')),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('100')),  # pub funding_2_decay: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.02')),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.25')),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.05')),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.00000001'))  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('SOL/USD'), # pub pair_id: PairId,
                    ret.ManifestBuilderValue.I64_VALUE(5), # price_age_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('10000')), # pub oi_max: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(3600), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.1')), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1')), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1')),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('50')),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('100')),  # pub funding_2_decay: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.02')),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.25')),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.05')),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.00000001'))  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('XRD/USD'), # pub pair_id: PairId,
                    ret.ManifestBuilderValue.I64_VALUE(5), # price_age_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('4000000')), # pub oi_max: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(3600), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.2')), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1')), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1')),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('50')),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('100')),  # pub funding_2_decay: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.02')),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.25')),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.05')),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.00000008'))  # pub fee_1: Decimal,
                ]),
            ])]
        )

        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        print('Transaction id:', intent)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Transaction status:', status)

if __name__ == '__main__':
    asyncio.run(main())

