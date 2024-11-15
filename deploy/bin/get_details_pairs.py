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
        network_config = await gateway.network_configuration()

        if network_config['network_name'] == 'stokenet':
            config_path = join(path, 'stokenet.config.json')
        elif network_config['network_name'] == 'mainnet':
            config_path = join(path, 'mainnet.config.json')
        else:
            raise ValueError(f'Unsupported network: {network_config["network_name"]}')
        
        with open(config_path, 'r') as config_file:
            config_data = json.load(config_file)

        exchange_component = config_data['EXCHANGE_COMPONENT']

        pair_ids = ['BTC/USD', 'ETH/USD', 'SOL/USD', 'XRD/USD', 'SUI/USD', 'DOGE/USD', 'ADA/USD']
        prices = await get_prices(session, pair_ids)

        manifest = f'''
            CALL_METHOD
                Address("{exchange_component}")
                "get_pair_details"
                Array<String>(
                    "BTC/USD",
                    "ETH/USD",
                    "SOL/USD",
                    "XRD/USD",
                    "SUI/USD",
                    "DOGE/USD",
                    "ADA/USD",
                )
            ;
        '''

        result = await gateway.preview_transaction(manifest)

        pairs = []
        for elem in result['receipt']['output'][0]['programmatic_json']['elements']:
            elem = elem['fields']
            pair = elem[0]['value']
            
            pool_position = elem[1]['fields']
            oi_long = float(pool_position[0]['value'])
            oi_short = float(pool_position[1]['value'])
            cost = float(pool_position[2]['value'])
            funding_2_raw = float(pool_position[5]['value'])

            pair_config = elem[2]['fields']
            pair_config = {
                'pair_id': pair_config[0]['value'],
                'price_max_age': int(pair_config[1]['value']),
                'oi_max': float(pair_config[2]['value']),
                'trade_size_min': float(pair_config[3]['value']),
                'update_price_delta_ratio': float(pair_config[4]['value']),
                'update_period_seconds': float(pair_config[5]['value']),
                'margin': float(pair_config[6]['value']),
                'margin_maintenance': float(pair_config[7]['value']),
                'funding_1': float(pair_config[8]['value']),
                'funding_2': float(pair_config[9]['value']),
                'funding_2_delta': float(pair_config[10]['value']),
                'funding_2_decay': float(pair_config[11]['value']),
                'funding_pool_0': float(pair_config[12]['value']),
                'funding_pool_1': float(pair_config[13]['value']),
                'funding_share': float(pair_config[14]['value']),
                'fee_0': float(pair_config[15]['value']),
                'fee_1': float(pair_config[16]['value']),
            }

            price = prices[pair]
            oi_net = oi_long + oi_short
            skew = (oi_long - oi_short) * price

            funding_1 = skew * pair_config['funding_1']
            funding_2_max = oi_long * price
            funding_2_min = -oi_short * price
            funding_2 = min(max(funding_2_raw, funding_2_min), funding_2_max) * pair_config['funding_2']
            
            if oi_long == 0 or oi_short == 0:
                funding_long = 0
                funding_short = 0
                funding_share = 0
                funding_pool = 0
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

                funding_long = (funding_long_index + funding_pool_index) / price
                funding_short = (funding_short_index + funding_pool_index) / price
                funding_pool += funding_share

            pairs.append({
                'pair': pair,
                'oi_long': oi_long,
                'oi_short': oi_short,
                'oi_net': oi_net,
                'cost': cost,
                'skew': skew,
                'funding_1': funding_1,
                'funding_2': funding_2,
                'funding_2_raw': funding_2_raw,
                'funding_2_max': funding_2_max,
                'funding_2_min': funding_2_min,
                'funding_long_apr': funding_long,
                'funding_long_24h': funding_long / 365,
                'funding_short_apr': funding_short,
                'funding_short_24h': funding_short / 365,
                'funding_pool_24h': funding_pool / 365,
                'pair_config': pair_config,
            })     

        print(json.dumps(pairs, indent=2))

if __name__ == '__main__':
    asyncio.run(main())

