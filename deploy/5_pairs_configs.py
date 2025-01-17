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

        if network_config['network_name'] == 'stokenet':
            config_path = join(path, 'stokenet.config.json')
        elif network_config['network_name'] == 'mainnet':
            config_path = join(path, 'mainnet.config.json')
        else:
            raise ValueError(f'Unsupported network: {network_config["network_name"]}')
        
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

        price_age_max = 5
        update_price_delta_ratio = ret.Decimal('0.005')
        update_period_seconds = 3600
        margin_initial_low_leverage = ret.Decimal('0.1')
        margin_initial_high_leverage = ret.Decimal('0.05')
        margin_maintenance = ret.Decimal('0.01')
        funding_1 = ret.Decimal('0.5')
        funding_2 = ret.Decimal('2')
        funding_2_delta = ret.Decimal('200')
        funding_2_decay = ret.Decimal('500')
        funding_pool_0 = ret.Decimal('0.02')
        funding_pool_1 = ret.Decimal('0.5')
        funding_share = ret.Decimal('0.1')
        fee_0 = ret.Decimal('0.001')
        fee_0_high = ret.Decimal('0.0015')
        fee_1_low = ret.Decimal('0.00000001')
        fee_1_mid = ret.Decimal('0.00000002')
        fee_1_high = ret.Decimal('0.00000008')

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.account_create_proof_of_amount(
            account,
            ret.Address(owner_resource),
            ret.Decimal('4')
        )
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(exchange_component)),
            'update_pair_configs',
            [ret.ManifestBuilderValue.ARRAY_VALUE(ret.ManifestBuilderValueKind.TUPLE_VALUE, [
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('BTC/USD'), # pub pair_id: PairId,
                    ret.ManifestBuilderValue.I64_VALUE(price_age_max), # price_age_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('10')), # pub oi_max: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(update_price_delta_ratio), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(update_period_seconds), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_initial_high_leverage), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_maintenance), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_1), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_delta),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_decay),  # pub funding_2_decay: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_0),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_1),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_share),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_0),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_1_low)  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('ETH/USD'), # pub pair_id: PairId,
                    ret.ManifestBuilderValue.I64_VALUE(price_age_max), # price_age_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('200')), # pub oi_max: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(update_price_delta_ratio), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(update_period_seconds), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_initial_high_leverage), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_maintenance), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_1), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_delta),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_decay),  # pub funding_2_decay: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_0),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_1),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_share),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_0),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_1_low)  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('SOL/USD'), # pub pair_id: PairId,
                    ret.ManifestBuilderValue.I64_VALUE(price_age_max), # price_age_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('2500')), # pub oi_max: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(update_price_delta_ratio), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(update_period_seconds), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_initial_high_leverage), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_maintenance), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_1), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_delta),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_decay),  # pub funding_2_decay: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_0),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_1),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_share),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_0),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_1_low)  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('XRD/USD'), # pub pair_id: PairId,
                    ret.ManifestBuilderValue.I64_VALUE(price_age_max), # price_age_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('5000000')), # pub oi_max: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(update_price_delta_ratio), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(update_period_seconds), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_initial_low_leverage), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_maintenance), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_1), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_delta),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_decay),  # pub funding_2_decay: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_0),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_1),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_share),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_0_high),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_1_high)  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('SUI/USD'), # pub pair_id: PairId,
                    ret.ManifestBuilderValue.I64_VALUE(price_age_max), # price_age_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('200000')), # pub oi_max: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(update_price_delta_ratio), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(update_period_seconds), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_initial_high_leverage), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_maintenance), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_1), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_delta),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_decay),  # pub funding_2_decay: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_0),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_1),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_share),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_0),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_1_mid)  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('DOGE/USD'), # pub pair_id: PairId,
                    ret.ManifestBuilderValue.I64_VALUE(price_age_max), # price_age_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1000000')), # pub oi_max: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(update_price_delta_ratio), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(update_period_seconds), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_initial_high_leverage), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_maintenance), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_1), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_delta),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_decay),  # pub funding_2_decay: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_0),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_1),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_share),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_0),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_1_mid)  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('ADA/USD'), # pub pair_id: PairId,
                    ret.ManifestBuilderValue.I64_VALUE(price_age_max), # price_age_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('750000')), # pub oi_max: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(update_price_delta_ratio), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(update_period_seconds), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_initial_high_leverage), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_maintenance), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_1), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_delta),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_decay),  # pub funding_2_decay: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_0),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_1),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_share),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_0),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_1_mid)  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('BNB/USD'), # pub pair_id: PairId,
                    ret.ManifestBuilderValue.I64_VALUE(price_age_max), # price_age_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1000')), # pub oi_max: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(update_price_delta_ratio), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(update_period_seconds), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_initial_high_leverage), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_maintenance), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_1), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_delta),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_decay),  # pub funding_2_decay: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_0),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_1),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_share),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_0),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_1_mid)  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('XRP/USD'), # pub pair_id: PairId,
                    ret.ManifestBuilderValue.I64_VALUE(price_age_max), # price_age_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('200000')), # pub oi_max: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(update_price_delta_ratio), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(update_period_seconds), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_initial_high_leverage), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_maintenance), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_1), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_delta),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_decay),  # pub funding_2_decay: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_0),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_1),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_share),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_0),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_1_mid)  # pub fee_1: Decimal,
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.STRING_VALUE('PEPE/USD'), # pub pair_id: PairId,
                    ret.ManifestBuilderValue.I64_VALUE(price_age_max), # price_age_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1000000')), # pub oi_max: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub trade_size_min: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(update_price_delta_ratio), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(update_period_seconds), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_initial_high_leverage), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(margin_maintenance), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_1), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_delta),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_2_decay),  # pub funding_2_decay: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_0),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_pool_1),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(funding_share),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_0),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(fee_1_mid)  # pub fee_1: Decimal,
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

