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

        exchange_component = config_data['EXCHANGE_COMPONENT']

        manifest = f'''
            CALL_METHOD
                Address("{exchange_component}")
                "get_pair_details"
                Array<Tuple>(
                    Tuple(
                        "BTC/USD",
                        Decimal("60000"),
                    ),
                )
            ;
        '''

        result = await gateway.preview_transaction(manifest)

        pairs = []
        for pair in result['receipt']['output'][0]['programmatic_json']['elements']:
            pair = pair['fields']
            pairs.append({
                'pair_id': pair[0]['value'],
                'oi_long': pair[1]['value'],
                'oi_short': pair[2]['value'],
                'funding_1': pair[3]['value'],
                'funding_2': pair[4]['value'],
                'funding_long': pair[5]['value'],
                'funding_short': pair[6]['value'],
                'funding_share': pair[7]['value'],
                'pair_config': pair[8],
            })

        print(pairs)

if __name__ == '__main__':
    asyncio.run(main())

