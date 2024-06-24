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
        if limit_variant == 0 or limit_variant == 1:
            limit_price = float(request_inner[3]['fields'][0]['value'])
        else:
            limit_price = None

        activate_requests = []
        for i in request_inner[4]['elements']:
            activate_requests.append(i['value'])

        cancel_requests = []
        for i in request_inner[5]['elements']:
            cancel_requests.append(i['value'])

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

        request_details = {
            'pair': pair_id,
            'size': size,
            'reduce_only': reduce_only,
            'limit_price': limit_price,
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

        config_path = join(path, 'config.json')
        with open(config_path, 'r') as config_file:
            config_data = json.load(config_file)

        account = 'account_tdx_2_12xm33jq02wztgwshhunnuzs3e6lzh3ce3hagg98llxhyx9khqr730c'
        rank_resource = 'resource_tdx_2_1n2jcct0nnyd03ant5zjl2wza40yuw9kh4vqacwrc5vytvpk0a049un'
        rank_id = '#1#'
        phase1_component = 'component_tdx_2_1cp9xzzeg58rg5ut9k64m6nmmp73c7af5uh3ckcrvsfnkwauwvyv0ex'
        phase2_component = 'component_tdx_2_1crc6l8fswy5eqkwpz729kmg3hgklu56q4damz7pq2dlqafv6pwltjz'

        manifest = f'''
            CALL_METHOD
                Address("{account}")
                "create_proof_of_non_fungibles"
                Address("{rank_resource}")
                Array<NonFungibleLocalId>(
                    NonFungibleLocalId("{rank_id}")
                )
            ;
            CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES
                Address("{rank_resource}")
                Array<NonFungibleLocalId>(
                    NonFungibleLocalId("{rank_id}")
                )
                Proof("rank1")
            ;
            CLONE_PROOF
                Proof("rank1")
                Proof("rank2")
            ;
            CALL_METHOD
                Address("{phase1_component}")
                "burn"
                Proof("rank1")
            ;
            CALL_METHOD
                Address("{phase2_component}")
                "vest"
                Proof("rank2")
                1764234000i64
            ;
        '''

        # manifest = f'''
        #     CALL_METHOD
        #         Address("{account}")
        #         "create_proof_of_non_fungibles"
        #         Address("{rank_resource}")
        #         Array<NonFungibleLocalId>(
        #             NonFungibleLocalId("{rank_id}")
        #         )
        #     ;
        #     CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES
        #         Address("{rank_resource}")
        #         Array<NonFungibleLocalId>(
        #             NonFungibleLocalId("{rank_id}")
        #         )
        #         Proof("rank1")
        #     ;
        #     CALL_METHOD
        #         Address("{phase1_component}")
        #         "unlock"
        #         Proof("rank1")
        #     ;
        #     CALL_METHOD
        #         Address("{account}")
        #         "deposit_batch"
        #         Expression("ENTIRE_WORKTOP")
        #     ;
        # '''

        # manifest = f'''
        #     CALL_METHOD
        #         Address("{phase2_component}")
        #         "get_info"
        #     ;
        # '''

        result = await gateway.preview_transaction(manifest)
        if result['receipt']['status'] == 'Failed':
            print("#### FAILED ####")
            print(result['receipt']['error_message'])
            return 
        
        result = result['receipt']['output'][4]
        print(result)

if __name__ == '__main__':
    asyncio.run(main())

