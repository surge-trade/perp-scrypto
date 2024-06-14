import qrcode
import io
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

path = dirname(dirname(realpath(__file__)))
sys.path.append(path)
chdir(path)
load_dotenv()

from tools.gateway import Gateway
from tools.accounts import new_account, load_account
from tools.manifests import lock_fee, deposit_all, withdraw_to_bucket
from tools.price_feeds import get_prices

async def main():
    async with ClientSession(connector=TCPConnector(ssl=False)) as session:
        gateway = Gateway(session)

        config_path = join(path, 'config.json')
        with open(config_path, 'r') as config_file:
            config_data = json.load(config_file)

        exchange_component = config_data['EXCHANGE_COMPONENT']

        pair_ids = ['BTC/USD', 'ETH/USD', 'XRD/USD']
        prices = await get_prices(session, pair_ids)

        manifest = f'''
            CALL_METHOD
                Address("{exchange_component}")
                "get_pair_details"
                Array<String>(
                    "BTC/USD",
                    "ETH/USD",
                    "XRD/USD",
                )
            ;
        '''

        result = await gateway.preview_transaction(manifest)

        pairs = []
        for elem in result['receipt']['output'][0]['programmatic_json']['elements']:
            elem = elem['fields']
            pair = elem[0]['value']
            oi_long = float(elem[1]['value'])
            oi_short = float(elem[2]['value'])
            funding_2 = float(elem[3]['value'])

            pair_config = elem[4]['fields']
            pair_config = {
                'pair_id': pair_config[0]['value'],
                'disabled': bool(pair_config[1]['value']),
                'update_price_delta_ratio': float(pair_config[2]['value']),
                'update_period_seconds': float(pair_config[3]['value']),
                'margin': float(pair_config[4]['value']),
                'margin_maintenance': float(pair_config[5]['value']),
                'funding_1': float(pair_config[6]['value']),
                'funding_2': float(pair_config[7]['value']),
                'funding_2_delta': float(pair_config[8]['value']),
                'funding_pool_0': float(pair_config[9]['value']),
                'funding_pool_1': float(pair_config[10]['value']),
                'funding_share': float(pair_config[11]['value']),
                'fee_0': float(pair_config[12]['value']),
                'fee_1': float(pair_config[13]['value']),
            }

            price = prices[pair]
            oi_net = oi_long + oi_short
            skew = (oi_long - oi_short) * price

            funding_1 = skew * pair_config['funding_1']
            funding_2 = funding_2 * pair_config['funding_2']

            if oi_long == 0 or oi_short == 0:
                funding_share = 0
                funding_long = 0
                funding_short = 0
            else:
                funding = funding_1 + funding_2
                if funding > 0:
                    funding_long = funding
                    funding_share = funding_long * pair_config['funding_share']
                    funding_long_index = funding_long / oi_long
                    funding_short_index = -(funding_long - funding_share) / oi_short
                else:
                    funding_short = -funding
                    funding_share = funding_short * pair_config['funding_share']
                    funding_long_index = -(funding_short - funding_share) / oi_long
                    funding_short_index = funding_short / oi_short

            funding_pool_0 = oi_net * price * pair_config['funding_pool_0']
            funding_pool_1 = abs(skew) * pair_config['funding_pool_1']
            funding_pool = funding_pool_0 + funding_pool_1
            funding_pool_index = funding_pool / oi_net

            funding_long = funding_long_index + funding_pool_index
            funding_short = funding_short_index + funding_pool_index
            funding_pool += funding_share

            pairs.append({
                'pair': pair,
                'oi_long': oi_long,
                'oi_short': oi_short,
                'oi_net': oi_net,
                'skew': skew,
                'funding_1': funding_1,
                'funding_2': funding_2,
                'funding_long': funding_long,
                'funding_short': funding_short,
                'funding_pool': funding_pool,
                'pair_config': pair_config,
            })     

        print(json.dumps(pairs, indent=2))

if __name__ == '__main__':
    asyncio.run(main())

