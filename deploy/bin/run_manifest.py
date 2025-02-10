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
#     Address("account_rdx12yn43ckkkre9un54424nvck48vf70cgyq8np4ajsrwkc9q3m20ndmd")
#     "lock_fee"
#     Decimal("10")
# ;
# CALL_METHOD
#     Address("account_rdx12yn43ckkkre9un54424nvck48vf70cgyq8np4ajsrwkc9q3m20ndmd")
#     "create_proof_of_amount"
#     Address("resource_rdx1t5av9jksz5a2952qmhv5h7t2k0xt4vkv4wj7ekdchjkq435ujudss5")
#     Decimal("4")
# ;
# SET_ROLE
#     Address("component_rdx1cqrfmpkp96hvlykahmhmu2w48kk2w7w35396vkrze9jwufxtvdzlkk")
#     Enum<0u8>()
#     "keeper_process"
#     Enum<2u8>(
#         Enum<1u8>(
#             Array<Enum>(
#                 Enum<0u8>(
#                     Enum<0u8>(
#                         Enum<0u8>(
#                             NonFungibleGlobalId("resource_rdx1nfxxxxxxxxxxed25sgxxxxxxxxx002236757237xxxxxxxxxed25sg:[38482c29c0a3e1ad84ee5ad45e09501ae7133fe94e916ec38c7af7fa8f]")
#                         )
#                     )
#                 ),
#                 Enum<0u8>(
#                     Enum<0u8>(
#                         Enum<0u8>(
#                             NonFungibleGlobalId("resource_rdx1nfxxxxxxxxxxed25sgxxxxxxxxx002236757237xxxxxxxxxed25sg:[d3ad5455ba9cb61b1bd41a932c285168428c4a4ab47dadc40941dab32b]")
#                         )
#                     )
#                 ),
#                 Enum<0u8>(
#                     Enum<0u8>(
#                         Enum<0u8>(
#                             NonFungibleGlobalId("resource_rdx1nfxxxxxxxxxxed25sgxxxxxxxxx002236757237xxxxxxxxxed25sg:[6f1d138cd61baa02faaca78a9e3759ba65b99e7c672979bb2fd365008c]")
#                         )
#                     )
#                 )
#             )
#         )
#     )
# ;

# CALL_METHOD
#     Address("account_rdx12yn43ckkkre9un54424nvck48vf70cgyq8np4ajsrwkc9q3m20ndmd")
#     "lock_fee"
#     Decimal("10")
# ;
# CALL_METHOD
#     Address("account_rdx12yn43ckkkre9un54424nvck48vf70cgyq8np4ajsrwkc9q3m20ndmd")
#     "create_proof_of_amount"
#     Address("resource_rdx1t5av9jksz5a2952qmhv5h7t2k0xt4vkv4wj7ekdchjkq435ujudss5")
#     Decimal("4")
# ;
# CALL_METHOD
#     Address("component_rdx1cpeszqnemrmdmmftznvg6rwv75jxlr5cnrvwypuag0mfcnesu8ls95")
#     "add_key"
#     0u64
#     Bytes("891fa1c6410621dfb3a693fcbb5663025d564f2a6584d8446b608fc74a4083ab7ee0e767e3a8e987b95bc18df6090536")
# ;

# CALL_METHOD
#     Address("account_rdx12yn43ckkkre9un54424nvck48vf70cgyq8np4ajsrwkc9q3m20ndmd")
#     "lock_fee"
#     Decimal("10")
# ;
# CALL_METHOD
#     Address("account_rdx12yn43ckkkre9un54424nvck48vf70cgyq8np4ajsrwkc9q3m20ndmd")
#     "create_proof_of_amount"
#     Address("resource_rdx1t5av9jksz5a2952qmhv5h7t2k0xt4vkv4wj7ekdchjkq435ujudss5")
#     Decimal("4")
# ;
# SET_ROLE
#     Address("component_rdx1cqrpn76ra5qf9xz5374mqduaw6r55dwzqeum85lm3ul2ct3eu2xc4n")
#     Enum<0u8>()
#     "account_management_user"
#     Enum<1u8>()
# ;

# CALL_METHOD
#     Address("account_tdx_2_12yxkxw7zez00czzfdk2hp8f0pfxcqez4jgrj0x608h39470eyqd965")
#     "lock_fee"
#     Decimal("100")
# ;
# CALL_METHOD
#     Address("account_tdx_2_12yxkxw7zez00czzfdk2hp8f0pfxcqez4jgrj0x608h39470eyqd965")
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

# CALL_METHOD
#     Address("account_tdx_2_12yxkxw7zez00czzfdk2hp8f0pfxcqez4jgrj0x608h39470eyqd965")
#     "lock_fee"
#     Decimal("100")
# ;
# CALL_METHOD
#     Address("account_tdx_2_12yxkxw7zez00czzfdk2hp8f0pfxcqez4jgrj0x608h39470eyqd965")
#     "create_proof_of_amount"
#     Address("resource_tdx_2_1t5g9tl3mk3a0dvlkvy0l9lpynse3ftv9thz050wmqrj273mplzz88v")
#     Decimal("1")
# ;
# CALL_METHOD
#     Address("component_tdx_2_1crtp3jrth56qnas5lhyxjy0qjxmfl4wf8q57fw6mrhf6pdcxh4ptcr")
#     "remove_collateral_config"
#     Address("resource_tdx_2_1tkr5t0zhag4w5evpcnsrya7tljeevqgmvd7l6r35ueqaca3amkl3wr")
# ;

# CALL_METHOD
#     Address("account_tdx_2_12yxkxw7zez00czzfdk2hp8f0pfxcqez4jgrj0x608h39470eyqd965")
#     "lock_fee"
#     Decimal("100")
# ;
# CALL_METHOD
#     Address("account_tdx_2_12yxkxw7zez00czzfdk2hp8f0pfxcqez4jgrj0x608h39470eyqd965")
#     "create_proof_of_amount"
#     Address("resource_tdx_2_1t48h0pc5rupvjy2q09cvunprk6lyxmuxvr543zy6qxk07fjqk8eg35")
#     Decimal("1")
# ;
# CALL_METHOD
#     Address("component_tdx_2_1crp59cff90nhg32gesghh55gh44qut8ahuu5ayh2dkzjgt2f4esgrf")
#     "add_key"
#     1u64
#     Bytes("b60f1610a172790598c1b4cd4a4dc34daa4e972bb7a9fa77245357eab78471c15927a28af8779e021236ab68ff0c8bd3")
# ;
