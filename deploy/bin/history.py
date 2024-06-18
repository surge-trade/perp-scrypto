import qrcode
import io
import radix_engine_toolkit as ret
import asyncio
import json
import sys
from os.path import dirname, join, realpath
from os import makedirs, chdir
from aiohttp import ClientSession, TCPConnector
from subprocess import run
from dotenv import load_dotenv
from datetime import datetime

path = dirname(dirname(realpath(__file__)))
sys.path.append(path)
chdir(path)
load_dotenv()

from tools.gateway import Gateway
from tools.accounts import new_account, load_account
from tools.manifests import lock_fee, deposit_all, withdraw_to_bucket
from tools.price_feeds import get_feeds, get_prices

async def main():
    async with ClientSession(connector=TCPConnector(ssl=False)) as session:
        gateway = Gateway(session)

        config_path = join(path, 'config.json')
        with open(config_path, 'r') as config_file:
            config_data = json.load(config_file)

        exchange_component = config_data['EXCHANGE_COMPONENT']
        # account_component = config_data['ACCOUNT_COMPONENT']
        account_component = "component_tdx_2_1cqlkqptfy6zx63fpw2wfs60dtha7hcldn6a7ksxrxakafrcp5d2htu"
        print(account_component)

        result = await gateway.get_component_history(account_component)

        trade_history = []
        liquidation_history = []
        
        for transaction in result['items']:
            txid = transaction['intent_hash']
            timestamp = transaction['confirmed_at'].split('.')[0] + '+00:00'
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
                            price = float(fields[2]['value'])
                            limit_variant = int(fields[3]['variant_id'])
                            size_open = float(fields[4]['value'])
                            size_close = float(fields[5]['value'])
                            pnl = float(fields[6]['value'])
                            funding = float(fields[7]['value'])
                            fee_pool = float(fields[8]['value'])
                            fee_protocol = float(fields[9]['value'])
                            fee_treasury = float(fields[10]['value'])
                            fee_referral = float(fields[11]['value'])

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
                                    'timestamp': timestamp,
                                    'pair': pair,
                                    'type': type, 
                                    'price': price,
                                    'size': size,
                                    'pnl': pnl,
                                    'funding': funding,
                                    'fee': fee, 
                                    'txid': txid
                                }
                            )
                        case 'EventAutoDeleverage':
                            account = fields[0]['value']
                            if account != account_component:
                                continue
                            
                            pair = fields[1]['value']
                            price = float(fields[2]['value'])
                            size = float(fields[3]['value'])
                            pnl = float(fields[4]['value'])
                            funding = float(fields[5]['value'])
                            fee_pool = float(fields[6]['value'])
                            fee_protocol = float(fields[7]['value'])
                            fee_treasury = float(fields[8]['value'])
                            fee_referral = float(fields[9]['value'])
                    
                            fee = fee_pool + fee_protocol + fee_treasury + fee_referral

                            trade_history.append({
                                'timestamp': timestamp,
                                'pair': pair,
                                'type': 'Auto Deleverage',
                                'price': price,
                                'size': size,
                                'pnl': pnl,
                                'funding': funding,
                                'fee': fee,
                                'txid': txid
                            })
                        case 'EventLiquidation':
                            account = fields[0]['value']
                            if account != account_component:
                                continue

                            liquidation_history.append({
                                'timestamp': timestamp,
                                'txid': txid
                            })
                except:
                    continue

        for event in trade_history:
            print(json.dumps(event, indent=2))

if __name__ == '__main__':
    asyncio.run(main())

