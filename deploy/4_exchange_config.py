import radix_engine_toolkit as ret
import asyncio
import datetime
import json
import sys
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
            ret.Decimal('4')
        )
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(exchange_component)),
            'update_exchange_config',
            [
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.U16_VALUE(30), # positions_max
                    ret.ManifestBuilderValue.U16_VALUE(5), # collaterals_max
                    ret.ManifestBuilderValue.U16_VALUE(100), # active_requests_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.15')), # skew_ratio_cap
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.2')), # adl_offset
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.07')), # adl_a
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.07')), # adl_b
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0')), # fee_liquidity_add
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.001')), # fee_liquidity_remove
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.2')), # fee_share_protocol
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.1')), # fee_share_treasury
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1')), # fee_share_referral
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # fee_max
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1000000')), # protocol_burn_amount
                    ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('1')), # reward_keeper
                ])
            ]
        )

        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        print('Transaction id:', intent)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Transaction status:', status)

if __name__ == '__main__':
    asyncio.run(main())

