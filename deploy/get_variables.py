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
from tools.manifests import lock_fee, deposit_all, withdraw_to_bucket

async def main():
    path = dirname(realpath(__file__))
    chdir(path)

    async with ClientSession(connector=TCPConnector(ssl=False)) as session:
        gateway = Gateway(session)

        config_path = join(path, 'config.json')
        with open(config_path, 'r') as config_file:
            config_data = json.load(config_file)

        env_registry_component = config_data['ENV_REGISTRY_COMPONENT']

        manifest = f'''
            CALL_METHOD
                Address("{env_registry_component}")
                "get_variables"
                Array<String>(
                    "exchange_component",
                )
            ;
        '''

        result = await gateway.preview_transaction(manifest)
        for elem in result['receipt']['output'][0]['programmatic_json']['entries']:
            print(elem)
            # print(elem['key']['value'], elem['value']['fields'][0]['value'])

if __name__ == '__main__':
    asyncio.run(main())

