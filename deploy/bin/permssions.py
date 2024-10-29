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
from tools.manifests import lock_fee, deposit_all

async def main():
    path = dirname(dirname(realpath(__file__)))
    chdir(path)

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
        # print('Config loaded:', config_data)

        exchange_component = config_data['EXCHANGE_COMPONENT']

        balance = await gateway.get_xrd_balance(account)
        if balance < 1000:
            print('FUND ACCOUNT:', account.as_str())
            qr = qrcode.QRCode()
            qr.add_data(account.as_str())
            f = io.StringIO()
            qr.print_ascii(out=f)
            f.seek(0)
            print(f.read())
        while balance < 1000:
            await asyncio.sleep(5)
            balance = await gateway.get_xrd_balance(account)


        public_key_hash = ret.hash(public_key.value).as_str()[-58:]
        rule = f'{network_config["ed25519_virtual_badge"]}:[{public_key_hash}]'
        rule = 'resource_tdx_2_1nfxxxxxxxxxxed25sgxxxxxxxxx002236757237xxxxxxxxx3e2cpa:[f9945f0fef7b7ea8e769bbb073b3e362155e5df6d2ea7eb4efd028f1ec]'

        manifest = f'''
            CALL_METHOD
                Address("{account.as_str()}")
                "lock_fee"
                Decimal("10")
            ;
            CALL_METHOD
                Address("{exchange_component}")
                "get_permissions"
                Enum<2u8>(
                    Enum<0u8>(
                        Enum<0u8>(
                            Enum<0u8>(
                                NonFungibleGlobalId("{rule}")
                            )
                        )
                    )
                )
            ;
        '''

        result = await gateway.preview_transaction(manifest)
        result = result['receipt']['output'][1]['programmatic_json']['fields']
        print(result)

if __name__ == '__main__':
    asyncio.run(main())

