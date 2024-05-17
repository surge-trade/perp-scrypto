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

async def main():
    path = dirname(realpath(__file__))
    chdir(path)

    async with ClientSession(connector=TCPConnector(ssl=False)) as session:
        gateway = Gateway(session)
        network_config = await gateway.network_configuration()
        account_details = load_account(network_config['network_id'])
        if account_details is None:
            account_details = new_account(network_config['network_id'])
        private_key, public_key, account = account_details

        config_path = join(path, 'config.json')
        with open(config_path, 'r') as config_file:
            config_data = json.load(config_file)
        print('Config loaded:', config_data)

        exchange_component = config_data['EXCHANGE_COMPONENT']
        account_component = config_data['ACCOUNT_COMPONENT']

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

        manifest = f'''
            CALL_METHOD
                Address("{account.as_str()}")
                "lock_fee"
                Decimal("10")
            ;
            CALL_METHOD
                Address("{exchange_component}")
                "get_pool_details"
            ;
        '''

        result = await gateway.preview_transaction(manifest)
        result = result['receipt']['output'][1]['programmatic_json']['fields']

        base_tokens_amount = result[0]['value']
        virtual_balance = result[1]['value']
        unrealized_pool_funding = result[2]['value']
        pnl_snap = result[3]['value']
        skew_ratio = result[4]['value']
        skew_ratio_cap = result[5]['value']
        lp_supply = result[6]['value']
        lp_price = result[7]['value']

        print('Base Tokens Amount:', base_tokens_amount)
        print('Virtual Balance:', virtual_balance)
        print('Unrealized Pool Funding:', unrealized_pool_funding)
        print('PnL Snap:', pnl_snap)
        print('Skew Ratio:', skew_ratio)
        print('Skew Ratio Cap:', skew_ratio_cap)
        print('LP Supply:', lp_supply)
        print('LP Price:', lp_price)

if __name__ == '__main__':
    asyncio.run(main())

