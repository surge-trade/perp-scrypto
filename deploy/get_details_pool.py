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
                "get_pool_details"
            ;
        '''

        result = await gateway.preview_transaction(manifest)
        result = result['receipt']['output'][0]['programmatic_json']['fields']

        token_amount = result[0]['value']
        balance = result[1]['value']
        unrealized_pool_funding = result[2]['value']
        pnl_snap = result[3]['value']
        skew_ratio = result[4]['value']
        skew_ratio_cap = result[5]['value']
        lp_supply = result[6]['value']
        lp_price = result[7]['value']

        pool_details = {
            'token_amount': token_amount,
            'balance': balance,
            'unrealized_pool_funding': unrealized_pool_funding,
            'pnl_snap': pnl_snap,
            'skew_ratio': skew_ratio,
            'skew_ratio_cap': skew_ratio_cap,
            'lp_supply': lp_supply,
            'lp_price': lp_price,
        }

        print(json.dumps(pool_details, indent=2))

if __name__ == '__main__':
    asyncio.run(main())

