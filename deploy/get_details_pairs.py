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
                Array<Tuple>(
                    "BTC/USD",
                    "ETH/USD",
                    "XRD/USD",
                )
            ;
        '''

        result = await gateway.preview_transaction(manifest)

        # pub struct PairConfig {
        #     /// Price feed id
        #     pub pair_id: PairId,
        #     /// If the pair is disabled
        #     pub disabled: bool,
        #     /// Price delta ratio before updating a pair will be rewarded
        #     pub update_price_delta_ratio: Decimal,
        #     /// Time before updating a pair will be rewarded
        #     pub update_period_seconds: i64,
        #     /// Initial margin required
        #     pub margin_initial: Decimal,
        #     /// Maintenance margin required
        #     pub margin_maintenance: Decimal,
        #     /// Skew based funding 
        #     pub funding_1: Decimal,
        #     /// Integral of skew based funding
        #     pub funding_2: Decimal,
        #     /// Rate of change of funding 2 integral
        #     pub funding_2_delta: Decimal,
        #     /// Constant pool funding
        #     pub funding_pool_0: Decimal,
        #     /// Skew based pool funding
        #     pub funding_pool_1: Decimal,
        #     /// Share of regular funding taken as pool funding
        #     pub funding_share: Decimal,
        #     /// Constant fee
        #     pub fee_0: Decimal,
        #     /// Skew based fee
        #     pub fee_1: Decimal,
        # }

        pairs = []
        for pair in result['receipt']['output'][0]['programmatic_json']['elements']:
            pair = pair['fields']
            pair_config = pair[4]['fields']
            pairs.append({
                'pair_id': pair[0]['value'],
                'oi_long': int(pair[1]['value']),
                'oi_short': int(pair[2]['value']),
                'funding_2': int(pair[3]['value']),
                'pair_config': {
                    'pair_id': pair_config[0]['value'],
                    'disabled': bool(pair_config[1]['value']),
                    'update_price_delta_ratio': int(pair_config[2]['value']),
                    'update_period_seconds': int(pair_config[3]['value']),
                    'margin_initial': int(pair_config[4]['value']),
                    'margin_maintenance': int(pair_config[5]['value']),
                    'funding_1': int(pair_config[6]['value']),
                    'funding_2': int(pair_config[7]['value']),
                    'funding_2_delta': int(pair_config[8]['value']),
                    'funding_pool_0': int(pair_config[9]['value']),
                    'funding_pool_1': int(pair_config[10]['value']),
                    'funding_share': int(pair_config[11]['value']),
                    'fee_0': int(pair_config[12]['value']),
                    'fee_1': int(pair_config[13]['value']),
                },
            })

        pair_ids = [pair['pair_id'] for pair in pairs]
        prices = await get_prices(session, pair_ids)

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

        for pair in pairs:
            pair['price'] = prices[pair['pair_id']]
            pair['skew'] = (pair['oi_long'] - pair['oi_short']) * pair['price']
            pair['funding_1'] = pair['skew'] * pair['pair_config']['funding_1']
            pair['funding_2'] = pair['funding_2'] * pair['pair_config']['funding_2']
            if pair['oi_long'] == 0 or pair['oi_short'] == 0:
                pair['funding_long'] = 0
                pair['funding_short_index'] = 0
                pair['funding_share'] = 0
            else:
                funding = pair['funding_1'] + pair['funding_2']
                if funding > 0:
                    funding_long = funding
                    funding_long_index = funding_long / pair['oi_long']
                    funding_share = funding_long * pair['pair_config']['funding_share']
                    funding_short_index = -(funding_long - funding_share) / pair['oi_short']
                    pair['funding_long'] = funding_long_index
                    pair['funding_short'] = funding_short_index
                    pair['funding_share'] = funding_share
                else:
                    funding_short = -funding
                    funding_short_index = funding_short / pair['oi_short']
                    funding_share = funding_short * pair['pair_config']['funding_share']
                    funding_long_index = -(funding_short - funding_share) / pair['oi_long']
                    pair['funding_long'] = funding_long_index
                    pair['funding_short'] = funding_short_index
                    pair['funding_share'] = funding_share

        print(pairs)

if __name__ == '__main__':
    asyncio.run(main())

