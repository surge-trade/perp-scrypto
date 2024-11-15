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
        
        data = result['receipt']['output'][4]['programmatic_json']['fields']
        lock_amount = data[0]['fields'][4]['value']
        bonus_amount = data[1]['value']
        unlock_time = data[0]['fields'][9]['fields'][0]['value']
        print(lock_amount)
        print(bonus_amount)
        print(unlock_time)

if __name__ == '__main__':
    asyncio.run(main())

