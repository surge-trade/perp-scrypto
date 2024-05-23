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

        manifest = f'''
            CALL_METHOD
                Address("{exchange_component}")
                "get_account_details"
                Address("{account_component}")
                10u64
                Enum<0u8>()
            ;
        '''

        result = await gateway.preview_transaction(manifest)
        result = result['receipt']['output'][0]['programmatic_json']['fields']

        virtual_balance = result[0]['value']

        positions = result[1]['elements']
        for elem in positions:
            elem = elem['fields']
            positions.append({
                'pair_id': elem[0]['value'],
                'amount': int(elem[1]['value']),
                'margin_initial': int(elem[2]['value']),
                'margin_maintenance': int(elem[3]['value']),
                'cost': int(elem[4]['value']),
                'funding': int(elem[5]['value']),
            })

        collaterals = []
        for elem in result[2]['elements']:
            elem = elem['fields']
            collaterals.append({
                'pair_id': elem[0]['value'],
                'resource': elem[1]['value'],
                'amount': int(elem[2]['value']),
                'amount_discounted': int(elem[3]['value']),
                'margin': int(elem[4]['value']),
            })

        valid_requests_start = result[3]['value']

        active_requests = []
        for elem in result[4]['elements']:
            elem = elem['fields']
            active_requests.append({
                'index': elem[0]['value'],
                'request': elem[1]['value'],
                'submission': elem[2]['value'],
                'expiry': elem[3]['value'],
                'status': elem[4]['value'],
            })

        requests_history = []
        for elem in result[5]['elements']:
            elem = elem['fields']
            requests_history.append({
                'index': elem[0]['value'],
                'request': elem[1]['value'],
                'submission': elem[2]['value'],
                'expiry': elem[3]['value'],
                'status': elem[4]['value'],
            })

        pair_ids = [pos['pair_id'] for pos in positions]
        prices = await get_prices(session, pair_ids)

        for position in positions:
            position['price'] = prices[position['pair_id']]
            position['value'] = position['amount'] * position['price']
            position['margin_maintenance'] = position['margin_maintenance'] * position['price']
            position['margin_initial'] = position['margin_initial'] * position['price']
            position['pnl'] = position['value'] - position['cost'] - position['funding']

        for collateral in collaterals:
            collateral['price'] = prices[collateral['pair_id']]
            collateral['value'] = collateral['amount'] * collateral['price']
            collateral['value_discounted'] = collateral['amount_discounted'] * collateral['price']
            collateral['margin'] = collateral['margin'] * collateral['price']

        print('Virtual Balance:', virtual_balance)
        print('Positions:', positions)
        print('Collaterals:', collaterals)
        print('Valid Requests Start:', valid_requests_start)
        print('Active Requests:', active_requests)
        print('Requests History:', requests_history)

if __name__ == '__main__':
    asyncio.run(main())

