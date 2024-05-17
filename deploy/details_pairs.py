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
                "get_pair_details"
                Array<Tuple>(
                    Tuple(
                        "BTC/USD",
                        Decimal("60000"),
                    ),
                )
            ;
        '''

        result = await gateway.preview_transaction(manifest)

        pairs = []
        for pair in result['receipt']['output'][1]['programmatic_json']['elements']:
            pair = pair['fields']
            pairs.append({
                'pair_id': pair[0]['value'],
                'oi_long': pair[1]['value'],
                'oi_short': pair[2]['value'],
                'funding_1': pair[3]['value'],
                'funding_2': pair[4]['value'],
                'funding_long': pair[5]['value'],
                'funding_short': pair[6]['value'],
                'funding_share': pair[7]['value'],
                'pair_config': pair[8],
            })

        print(pairs)

if __name__ == '__main__':
    asyncio.run(main())

