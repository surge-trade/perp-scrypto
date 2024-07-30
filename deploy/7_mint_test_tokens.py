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
from tools.manifests import lock_fee, deposit_all, mint_test_btc, mint_test_usd

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

        print('ACCOUNT:', account.as_str())

        config_path = join(path, 'config.json')
        with open(config_path, 'r') as config_file:
            config_data = json.load(config_file)

        faucet_component = config_data['FAUCET_COMPONENT']

        deposit_account = account
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
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(faucet_component)),
            'free_tokens',
            []
        )
        builder = builder.account_try_deposit_entire_worktop_or_abort(deposit_account, None)
        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        print('Transaction id:', intent)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Transaction status:', status)

        # builder = ret.ManifestBuilder()
        # builder = lock_fee(builder, account, 100)
        # builder = mint_test_btc(builder)
        # builder = deposit_all(builder, account)

        # payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        # await gateway.submit_transaction(payload)
        # addresses = await gateway.get_new_addresses(intent)
        # btc_resource = addresses[0]
        # config_data['BTC_RESOURCE'] = btc_resource
        # print(f'BTC_RESOURCE: {btc_resource}')

        # builder = ret.ManifestBuilder()
        # builder = lock_fee(builder, account, 100)
        # builder = mint_test_usd(builder)
        # builder = deposit_all(builder, account)

        # payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        # await gateway.submit_transaction(payload)
        # addresses = await gateway.get_new_addresses(intent)
        # usd_resource = addresses[0]
        # config_data['USD_RESOURCE'] = usd_resource
        # print(f'USD_RESOURCE: {usd_resource}')

if __name__ == '__main__':
    asyncio.run(main())

