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
from tools.price_feeds import get_feeds, get_prices

def parse_request(elem):
    request = elem['fields']

    index = request[0]['value']
    submission = request[2]['value']
    expiry = request[3]['value']
    status_id = int(request[4]['value'])
    request_variant_id = int(request[1]['variant_id'])
    request_inner = request[1]['fields'][0]['fields']

    if status_id == 0:
        status = 'Dormant'
    elif status_id == 1:
        status = 'Active'
    elif status_id == 2:
        status = 'Executed'
    elif status_id == 3:
        status = 'Canceled'
    elif status_id == 4:
        status = 'Expired'
    elif status_id == 5:
        status = 'Failed'
    else:
        status = 'Unknown'

    if request_variant_id == 0:
        type = 'Remove Collateral'
        target_account = request_inner[0]['value']

        claims = []
        for claim in request_inner[1]['elements']:
            claim = claim['fields']
            claims.append({
                'resource': claim[0]['value'],
                'size': claim[1]['value'],
            })

        request_details = {
            'target_account': target_account,
            'claims': claims,
        }
    elif request_variant_id == 1:
        pair_id = request_inner[0]['value']
        size = float(request_inner[1]['value'])
        reduce_only = bool(request_inner[2]['value'])
        limit_variant = int(request_inner[3]['variant_id'])
        if limit_variant == 0:
            limit_price = None
        else:
            limit_price = float(request_inner[3]['fields'][0]['value'])
        slippage_variant = int(request_inner[4]['variant_id'])
        if slippage_variant == 0:
            limit_slippage = None
        elif slippage_variant == 1:
            limit_slippage = request_inner[4]['fields'][0]['value'] + '%' # percent value
        else:
            limit_slippage = request_inner[4]['fields'][0]['value'] # usd value

        activate_requests = []
        for i in request_inner[5]['elements']:
            activate_requests.append(i['value'])

        cancel_requests = []
        for i in request_inner[6]['elements']:
            cancel_requests.append(i['value'])

        if limit_variant == 0 and size >= 0:
            type = 'Market Long'
        elif limit_variant == 0 and size < 0:
            type = 'Market Short'
        elif limit_variant == 1 and size > 0:
            type = 'Stop Long'
        elif limit_variant == 1 and size <= 0:
            type = 'Limit Short'
        elif limit_variant == 2 and size >= 0:
            type = 'Limit Long'
        elif limit_variant == 2 and size < 0:
            type = 'Stop Short'    
        else:
            type = 'Unknown'

        request_details = {
            'pair': pair_id,
            'size': size,
            'reduce_only': reduce_only,
            'limit_price': limit_price,
            'limit_slippage': limit_slippage,
            'activate_requests': activate_requests,
            'cancel_requests': cancel_requests,
        }
    else:
        type = 'Unknown'
        request_details = None

    return {
        'type': type,
        'index': index,
        'submission': submission,
        'expiry': expiry,
        'status': status,
        'request_details': request_details,
    }

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
        account_component = config_data['ACCOUNT_COMPONENT']

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

            mark_price = prices[pair]
            entry_price = cost / size
            value = size * mark_price
            margin = margin * mark_price
            margin_maintenance = margin_maintenance * mark_price
            pnl = value - cost - funding
            roi = pnl / abs(cost) * 100

            positions.append({
                'pair': pair,
                'size': size,
                'value': value,
                'entry_price': entry_price,
                'mark_price': mark_price,
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

            mark_price = prices[pair]
            value = amount * mark_price
            value_discounted = value * discount
            margin = margin * mark_price

            collaterals.append({
                'pair': pair,
                'resource': resource,
                'mark_price': mark_price,
                'amount': amount,
                'value': value,
                'discount': discount,
                'value_discounted': value_discounted,
                'margin': margin,
            })

        valid_requests_start = result[3]['value']

        active_requests = []
        for elem in result[4]['elements']:
            active_requests.append(parse_request(elem))

        requests_history = []
        for elem in result[5]['elements']:
            requests_history.append(parse_request(elem))

        balance = float(result[0]['value'])
        total_pnl = sum([x['pnl'] for x in positions])
        total_margin = sum([x['margin'] for x in positions]) + sum([x['margin'] for x in collaterals])
        total_margin_maintenance = sum([x['margin_maintenance'] for x in positions]) + sum([x['margin'] for x in collaterals])
        total_collateral_value = sum([x['value'] for x in collaterals])
        total_collateral_value_discounted = sum([x['value_discounted'] for x in collaterals])

        account_value = balance + total_pnl + total_collateral_value
        account_value_discounted = balance + total_pnl + total_collateral_value_discounted
        available_margin = account_value_discounted - total_margin
        available_margin_maintenance = account_value_discounted - total_margin_maintenance

        overview = {
            'account_value': account_value,
            'account_value_discounted': account_value_discounted,
            'available_margin': available_margin,
            'available_margin_maintenance': available_margin_maintenance,
            'balance': balance,
            'total_pnl': total_pnl,
            'total_margin': total_margin,
            'total_margin_maintenance': total_margin_maintenance,
            'total_collateral_value': total_collateral_value,
            'total_collateral_value_discounted': total_collateral_value_discounted,
        }

        account_details = {
            'balance': balance,
            'positions': positions,
            'collaterals': collaterals,
            'valid_requests_start': valid_requests_start,
            'active_requests': active_requests,
            'requests_history': requests_history,
            'overview': overview,
        }

        print(json.dumps(account_details, indent=2))

if __name__ == '__main__':
    asyncio.run(main())

