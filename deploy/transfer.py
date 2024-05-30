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

        balance = await gateway.get_xrd_balance(account)
        if balance < 1000:
            print('FUND ACCOUNT:', account.as_str())
        while balance < 1000:
            await asyncio.sleep(5)
            balance = await gateway.get_xrd_balance(account)

        print('ACCOUNT:', account.as_str())

        transfer_account = input('Input account to transfer to: ')
        transfer_resource = config_data['BASE_RESOURCE']
        transfer_amount = input('Input amount to transfer: ')

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(account),
            'withdraw',
            [
                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(transfer_resource))),
                ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal(transfer_amount))
            ]
        )
        builder = builder.take_all_from_worktop(ret.Address(transfer_resource), ret.ManifestBuilderBucket("token"))
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(transfer_account)),
            'try_deposit_or_abort',
            [
                ret.ManifestBuilderValue.BUCKET_VALUE(ret.ManifestBuilderBucket("token")),
                ret.ManifestBuilderValue.ENUM_VALUE(0, [])
            ]
        )

        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        print('Transaction id:', intent)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Transaction status:', status)

if __name__ == '__main__':
    asyncio.run(main())

