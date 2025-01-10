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
        referral_id = '{46da0ac91fd11880-369096d5479a1f9a-cdb4e4feecb5e9c9-3e29c398020e67b2}'

        manifest = f'''
            CALL_METHOD
                Address("{exchange_component}")
                "get_referral_details"
                NonFungibleLocalId("{referral_id}")
            ;
        '''

        result = await gateway.preview_transaction(manifest)
        if result['receipt']['status'] == 'Failed':
            print("#### FAILED ####")
            print(result['receipt']['error_message'])
            return 
        
        result = result['receipt']['output'][0]['programmatic_json']['fields']

        referral = {
            'name': result[0]['fields'][0]['value'],
            'description': result[0]['fields'][1]['value'],
            'key_image_url': result[0]['fields'][2]['value'],
            'fee_referral': result[0]['fields'][3]['value'],
            'fee_rebate': result[0]['fields'][4]['value'],
            'referrals': result[0]['fields'][5]['value'],
            'max_referrals': result[0]['fields'][6]['value'],
            'balance': result[0]['fields'][7]['value'],
            'total_rewarded': result[0]['fields'][8]['value'],
        }

        allocations = []
        for allocation in result[1]['elements']:
            claims = []
            for claim in allocation['fields'][0]['elements']:
                claims.append({
                    'resource': claim['fields'][0]['value'],
                    'amount': claim['fields'][1]['value'],
                })
            allocations.append({
                'claims': claims,
                'count': allocation['fields'][1]['value'],
                'max_count': allocation['fields'][2]['value'],
            })

        referral_details = {
            'referral': referral,
            'allocations': allocations,
        }

        print(json.dumps(referral_details, indent=2))

if __name__ == '__main__':
    asyncio.run(main())
