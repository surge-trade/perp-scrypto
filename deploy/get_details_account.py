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
from tools.price_feeds import get_feeds, get_prices

async def main():
    path = dirname(realpath(__file__))
    chdir(path)

    async with ClientSession(connector=TCPConnector(ssl=False)) as session:
        gateway = Gateway(session)

        config_path = join(path, 'config.json')
        with open(config_path, 'r') as config_file:
            config_data = json.load(config_file)

        exchange_component = config_data['EXCHANGE_COMPONENT']
        account_component = config_data['ACCOUNT_COMPONENT']
        # account_component = "component_tdx_2_1cqj8s9qydafchjlfuafayal2hwq7rhwev5j82uc00lcf5d5eszc0wz"
        account_component = "component_tdx_2_1cr3l32aq6cy7ee8kuz7fxjqe6xvagwmcaek4zpaef7j4dahyg35p3k"

        manifest = f'''
            CALL_METHOD
                Address("{exchange_component}")
                "get_account_details"
                Address("{account_component}")
                30u64
                Enum<0u8>()
            ;
        '''

        result = await gateway.preview_transaction(manifest)
        if result['receipt']['status'] == 'Failed':
            print("#### FAILED ####")
            print(result['receipt']['error_message'])
            return 
        
        result = result['receipt']['output'][0]['programmatic_json']['fields']
        pair_ids = set()
        for elem in result[1]['elements']:
            pair_ids.add(elem['fields'][0]['value'])
        for elem in result[2]['elements']:
            pair_ids.add(elem['fields'][0]['value'])
        prices = await get_prices(session, pair_ids)

        balance = result[0]['value']

        positions = []
        for elem in result[1]['elements']:            
            elem = elem['fields']
            pair = elem[0]['value']
            size = float(elem[1]['value'])
            margin = float(elem[2]['value'])
            margin_maintenance = float(elem[3]['value'])
            cost = float(elem[4]['value'])
            funding = float(elem[5]['value'])

            ref_price = prices[pair]
            entry_price = cost / size
            value = size * ref_price
            margin = margin * ref_price
            margin_maintenance = margin_maintenance * ref_price
            pnl = value - cost - funding
            roi = pnl / cost * 100

            positions.append({
                'pair': pair,
                'size': size,
                'value': value,
                'entry_price': entry_price,
                'ref_price': ref_price,
                'margin': margin,
                'margin_maintenance': margin_maintenance,
                'pnl': pnl,
                'roi': roi,
            })

        collaterals = []
        for elem in result[2]['elements']:
            elem = elem['fields']

            pair = elem[0]['value']
            resource = elem[1]['value']
            amount = float(elem[2]['value'])
            discount = float(elem[3]['value'])
            margin = float(elem[4]['value'])

            ref_price = prices[pair]
            value = amount * ref_price
            value_discounted = value * discount
            margin = margin * ref_price

            collaterals.append({
                'pair': pair,
                'resource': resource,
                'ref_price': ref_price,
                'amount': amount,
                'value': value,
                'discount': discount,
                'value_discounted': value_discounted,
                'margin': margin,
            })

        valid_requests_start = result[3]['value']

        active_requests = []
        for elem in result[4]['elements']:
            elem = elem['fields']
            active_requests.append({
                'index': elem[0]['value'],
                'request': elem[1],
                'submission': elem[2]['value'],
                'expiry': elem[3]['value'],
                'status': elem[4]['value'],
            })

        requests_history = []
        for elem in result[5]['elements']:
            elem = elem['fields']
            requests_history.append({
                'index': elem[0]['value'],
                'request': elem[1],
                'submission': elem[2]['value'],
                'expiry': elem[3]['value'],
                'status': elem[4]['value'],
            })

        print('Balance:', balance)
        print('Positions:', positions)
        print('Collaterals:', collaterals)
        print('Valid Requests Start:', valid_requests_start)
        print('Active Requests:', active_requests)
        print('Requests History:', requests_history)

if __name__ == '__main__':
    asyncio.run(main())

