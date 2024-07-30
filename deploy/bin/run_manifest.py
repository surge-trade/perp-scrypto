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

        balance = await gateway.get_xrd_balance(account)
        if balance < 1000:
            print('FUND ACCOUNT:', account.as_str())
        while balance < 1000:
            await asyncio.sleep(5)
            balance = await gateway.get_xrd_balance(account)

        print('ACCOUNT:', account.as_str())

        manifest = input('Input manifest: ')

        payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
        print('Transaction id:', intent)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Transaction status:', status)

if __name__ == '__main__':
    asyncio.run(main())

# CALL_METHOD
#     Address("account_tdx_2_12ylzgawnp489cv07k69fvgtwy9xv9y4v43smvwk0mpp0qps3dsnsnn")
#     "lock_fee"
#     Decimal("100")
# ;
# CALL_METHOD
#     Address("account_tdx_2_12ylzgawnp489cv07k69fvgtwy9xv9y4v43smvwk0mpp0qps3dsnsnn")
#     "create_proof_of_amount"
#     Address("resource_tdx_2_1t4d0yjchyssrj0q49swvp9sgl37qrg9z25th48sk0xltdatmdzvvzk")
#     Decimal("1")
# ;
# CALL_METHOD
#     Address("component_tdx_2_1cpvhd5k8nqmv03mtg0c0a4t2gvrgkauqg0u7srv96zjw5ejd7ucnf9")
#     "update_mint_amount"
#     Address("resource_tdx_2_1tkr5t0zhag4w5evpcnsrya7tljeevqgmvd7l6r35ueqaca3amkl3wr")
#     Decimal("0")
# ;