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
from tools.manifests import lock_fee, deposit_all

async def main():
    async with ClientSession(connector=TCPConnector(ssl=False)) as session:
        gateway = Gateway(session)
        network_config = await gateway.network_configuration()
        account_details = load_account(network_config['network_id'])
        if account_details is None:
            account_details = new_account(network_config['network_id'])
        private_key, public_key, account = account_details

        if network_config['network_name'] == 'stokenet':
            config_path = join(path, 'stokenet.config.json')
        elif network_config['network_name'] == 'mainnet':
            config_path = join(path, 'mainnet.config.json')
        else:
            raise ValueError(f'Unsupported network: {network_config["network_name"]}')
        
        with open(config_path, 'r') as config_file:
            config_data = json.load(config_file)

        owner_resource = config_data['OWNER_RESOURCE']
        exchange_component = config_data['EXCHANGE_COMPONENT']
        base_resource = config_data['BASE_RESOURCE']
        receiver_account = ''

        balance = await gateway.get_xrd_balance(account)
        if balance < 1000:
            print('FUND ACCOUNT:', account.as_str())
        while balance < 1000:
            await asyncio.sleep(5)
            balance = await gateway.get_xrd_balance(account)

        manifest = f'''
            CALL_METHOD
                Address("{account.as_str()}")
                "lock_fee"
                Decimal("10")
            ;
            CALL_METHOD
                Address("{account.as_str()}")
                "create_proof_of_amount"
                Address("{owner_resource}")
                Decimal("4")
            ;
            CALL_METHOD
                Address("{account.as_str()}")
                "withdraw"
                Address("{base_resource}")
                Decimal("100")
            ;
            TAKE_ALL_FROM_WORKTOP
                Address("{base_resource}")
                Bucket("tokens")
            ;
            CALL_METHOD
                Address("{exchange_component}")
                "mint_referral_with_allocation"
                "Jack of Hearts"
                "Can be used to invite your friends to the Surge!"
                "https://www.surge.trade/images/card_H_J.png"
                Decimal("0.035")
                Decimal("0.14")
                100u64
                Array<Bucket>(
                    Bucket("tokens")
                )
                Array<Tuple>(
                    Tuple(
                        Address("{base_resource}"),
                        Decimal("20")
                    )
                )
                5u64
            ;
            CALL_METHOD
                Address("{receiver_account}")
                "try_deposit_batch_or_abort"
                Expression("ENTIRE_WORKTOP")
                Enum<0u8>()
            ;
        '''

        payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
        await gateway.submit_transaction(payload)
        print('Transaction id:', intent)
        status = await gateway.get_transaction_status(intent)
        print('Status:', status)

if __name__ == '__main__':
    asyncio.run(main())

