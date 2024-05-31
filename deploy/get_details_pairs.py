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
from tools.price_feeds import get_prices

async def main():
    path = dirname(realpath(__file__))
    chdir(path)

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
        for pair in result['receipt']['output'][0]['programmatic_json']['elements']:
            pair = pair['fields']
            pair_id = pair[0]['value']
            oi_long = float(pair[1]['value'])
            oi_short = float(pair[2]['value'])
            funding_2 = float(pair[3]['value'])
            pair_config = pair[4]['fields']
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

            ref_price = prices[pair_id]
            skew = (oi_long - oi_short) * ref_price
            funding_1 = skew * pair_config['funding_1']
            funding_2 = funding_2 * pair_config['funding_2']
            if oi_long == 0 or oi_short == 0:
                funding_long = 0
                funding_short = 0
                funding_share = 0
            else:
                funding = funding_1 + funding_2
                if funding > 0:
                    funding_long = funding
                    funding_long_index = funding_long / oi_long
                    funding_share = funding_long * pair_config['funding_share']
                    funding_short_index = -(funding_long - funding_share) / oi_short
                    funding_long = funding_long_index
                    funding_short = funding_short_index
                    funding_share = funding_share
                else:
                    funding_short = -funding
                    funding_short_index = funding_short / pair['oi_short']
                    funding_share = funding_short * pair['pair_config']['funding_share']
                    funding_long_index = -(funding_short - funding_share) / oi_long
                    funding_long = funding_long_index
                    funding_short = funding_short_index
                    funding_share = funding_share

            pairs.append({
                'pair_id': pair_id,
                'oi_long': oi_long,
                'oi_short': oi_short,
                'skew': skew,
                'funding_1': funding_1,
                'funding_2': funding_2,
                'funding_long': funding_long,
                'funding_short': funding_short,
                'funding_share': funding_share,
                'pair_config': pair_config,
            })

        # // let (funding_long_index, funding_short_index, funding_share) = if !oi_long.is_zero() && !oi_short.is_zero() {
        # //     if funding_rate.is_positive() {
        # //         let funding_long = funding_rate;
        # //         let funding_long_index = funding_long / oi_long;

        # //         let funding_share = funding_long * pair_config.funding_share;
        # //         let funding_short_index = -(funding_long - funding_share) / oi_short;

        # //         (funding_long_index, funding_short_index, funding_share)
        # //     } else {
        # //         let funding_short = -funding_rate;
        # //         let funding_short_index = funding_short / oi_short;

        # //         let funding_share = funding_short * pair_config.funding_share;
        # //         let funding_long_index = -(funding_short - funding_share) / oi_long;

        # //         (funding_long_index, funding_short_index, funding_share)
        # //     }
        # // } else {
        # //     (dec!(0), dec!(0), dec!(0))
        # // };            

        print(json.dumps(pairs, indent=2))

if __name__ == '__main__':
    asyncio.run(main())

