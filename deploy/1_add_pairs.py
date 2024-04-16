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
        print('Config loaded:', config_data)

        owner_resource = config_data['OWNER_RESOURCE']
        exchange_component = config_data['EXCHANGE_COMPONENT']

        balance = await gateway.get_xrd_balance(account)
        if balance < 1000:
            print('FUND ACCOUNT:', account.as_str())
        while balance < 1000:
            await asyncio.sleep(5)
            balance = await gateway.get_xrd_balance(account)

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
                    ret.ManifestBuilderValue.U16_VALUE(0),  # pub pair_id: PairId,
                    ret.ManifestBuilderValue.BOOL_VALUE(False), # pub disabled: bool,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(120), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0'))  # pub fee_1: Decimal,
                ])
            ])]
        )
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(exchange_component)),
            'update_pair_configs',
            [ret.ManifestBuilderValue.ARRAY_VALUE(ret.ManifestBuilderValueKind.TUPLE_VALUE, [
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.U16_VALUE(0),  # pub pair_id: PairId,
                    ret.ManifestBuilderValue.BOOL_VALUE(False), # pub disabled: bool,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(120), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0'))  # pub fee_1: Decimal,
                ])
            ])]
        )
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(exchange_component)),
            'update_pair_configs',
            [ret.ManifestBuilderValue.ARRAY_VALUE(ret.ManifestBuilderValueKind.TUPLE_VALUE, [
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.U16_VALUE(0),  # pub pair_id: PairId,
                    ret.ManifestBuilderValue.BOOL_VALUE(False), # pub disabled: bool,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')), # update_price_delta_ratio
                    ret.ManifestBuilderValue.I64_VALUE(120), # update_period_seconds
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # pub margin_initial: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.005')), # pub margin_maintenance: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # pub funding_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_2: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_2_delta: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_pool_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_pool_1: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')),  # pub funding_share: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')),  # pub fee_0: Decimal,
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0'))  # pub fee_1: Decimal,
                ])
            ])]
        )

        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Transaction status:', status)

if __name__ == '__main__':
    asyncio.run(main())

