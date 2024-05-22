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
                'amount': elem[1]['value'],
                'margin_initial': elem[2]['value'],
                'margin_maintenance': elem[3]['value'],
                'cost': elem[4]['value'],
                'funding': elem[5]['value'],
            })

        collaterals = []
        for elem in result[2]['elements']:
            elem = elem['fields']
            collaterals.append({
                'resource': elem[0]['value'],
                'amount': elem[1]['value'],
                'amount_discounted': elem[2]['value'],
                'margin': elem[3]['value'],
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

        print('Virtual Balance:', virtual_balance)
        print('Positions:', positions)
        print('Collaterals:', collaterals)
        print('Valid Requests Start:', valid_requests_start)
        print('Active Requests:', active_requests)
        print('Requests History:', requests_history)

if __name__ == '__main__':
    asyncio.run(main())

