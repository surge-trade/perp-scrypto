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
from tools.manifests import lock_fee, deposit_all, withdraw_to_bucket

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
        token_wrapper_component = config_data['TOKEN_WRAPPER_COMPONENT']
        xrd = network_config['xrd']

        balance = await gateway.get_xrd_balance(account)
        if balance < 1000:
            print('FUND ACCOUNT:', account.as_str())
        while balance < 1000:
            await asyncio.sleep(5)
            balance = await gateway.get_xrd_balance(account)

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = withdraw_to_bucket(
            builder, 
            account, 
            ret.Address(xrd), 
            ret.Decimal('1000'), 
            'bucket1'
        )
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(token_wrapper_component)),
            'wrap',
            [ret.ManifestBuilderValue.BUCKET_VALUE(ret.ManifestBuilderBucket('bucket1'))]
        )
        builder = deposit_all(builder, account)

        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Transaction status:', status)

if __name__ == '__main__':
    asyncio.run(main())
