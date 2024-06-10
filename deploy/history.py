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

# def parse_field(field):
#     match field['kind']:
#         case 'Enum':
#             temp = []
#             for item in field['fields']:
#                 temp.append(parse_field(item['value']))
#             return temp
#         case 'Array':
#             temp = []
#             for item in field['elements']:
#                 temp.append(item['value'])
#             return temp
#         case 'Tuple':
#             temp = []
#             for item in field['fields']:
#                 temp.append(parse_field(item['value']))
#             return temp
#         case 'Map':
#             temp = {}
#             for item in field['entries']:
#                 temp[item['key']] = parse_field(item['value'])
#             return temp
#         case _:
#             return field['value']

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
        account_component = "component_tdx_2_1cpvc34pvpwcl9j984p53zr3s0neh0lxqml8cjrkwlr3n0ak2aks6zv"
        # account_component = "component_tdx_2_1cr3l32aq6cy7ee8kuz7fxjqe6xvagwmcaek4zpaef7j4dahyg35p3k"
        # account_component = "component_tdx_2_1cqsdhl3jnx63zvdk9ltxw7vvr5hx74dvr2k7rwmmfm074p04u0ld3e"

        result = await gateway.get_component_history(account_component)

        trade_history = []
        liquidation_history = []
        
        for transaction in result['items']:
            txid = transaction['intent_hash']
            for event in transaction['receipt']['events']:
                try:
                    name = event['name']
                    fields = event['data']['fields']
                    match name:
                        case 'EventMarginOrder':
                            account = fields[0]['value']
                            if account != account_component:
                                continue

                            pair = fields[1]['value']
                            limit_variant = int(fields[2]['variant_id'])
                            if limit_variant == 0 or limit_variant == 1:
                                limit_price = float(fields[2]['fields'][0]['value'])
                            else:
                                limit_price = None
                            size_open = float(fields[3]['value'])
                            size_close = float(fields[4]['value'])
                            fee_pool = float(fields[7]['value'])
                            fee_protocol = float(fields[8]['value'])
                            fee_treasury = float(fields[9]['value'])
                            fee_referral = float(fields[10]['value'])
                            index_price = float(fields[11]['value'])

                            size = size_open + size_close
                            fee = fee_pool + fee_protocol + fee_treasury + fee_referral

                            if limit_variant == 0 and size > 0:
                                type = 'Stop Long'
                            elif limit_variant == 0 and size <= 0:
                                type = 'Limit Short'
                            elif limit_variant == 1 and size >= 0:
                                type = 'Limit Long'
                            elif limit_variant == 1 and size < 0:
                                type = 'Stop Short'    
                            elif limit_variant == 2 and size >= 0:
                                type = 'Market Long'
                            elif limit_variant == 2 and size < 0:
                                type = 'Market Short'
                            else:
                                type = 'Unknown'

                            trade_history.append({
                                    'type': type, 
                                    'pair': pair, 
                                    'size': size, 
                                    'fee': fee, 
                                    'limit_price': limit_price, 
                                    'index_price': index_price,
                                    'txid': txid
                                }
                            )
                        case 'EventAutoDeleverage':
                            account = fields[0]['value']
                            if account != account_component:
                                continue
                            
                            pair = fields[1]['value']
                            size = float(fields[2]['value'])
                            fee_pool = float(fields[5]['value'])
                            fee_protocol = float(fields[6]['value'])
                            fee_treasury = float(fields[7]['value'])
                            fee_referral = float(fields[8]['value'])
                            index_price = float(fields[9]['value'])
                    
                            limit_price = None
                            fee = fee_pool + fee_protocol + fee_treasury + fee_referral

                            trade_history.append({
                                'type': 'Auto Deleverage',
                                'pair': pair,
                                'size': size,
                                'fee': fee,
                                'limit_price': limit_price,
                                'index_price': index_price,
                                'txid': txid
                            })
                        case 'EventLiquidation':
                            account = fields[0]['value']
                            if account != account_component:
                                continue

                            liquidation_history.append({
                                'txid': txid
                            })
                except:
                    continue

        for event in trade_history:
            print(json.dumps(event, indent=2))

if __name__ == '__main__':
    asyncio.run(main())

