import qrcode
import asyncio
import io
import sys
from os.path import dirname, join, realpath
from os import chdir
from aiohttp import ClientSession, TCPConnector
from dotenv import load_dotenv
load_dotenv()

path = dirname(dirname(realpath(__file__)))
sys.path.append(path)
chdir(path)
load_dotenv()

from tools.gateway import Gateway
from tools.accounts import new_account, load_account

async def main():
    async with ClientSession(connector=TCPConnector(ssl=False)) as session:
        gateway = Gateway(session)
        network_config = await gateway.network_configuration()
        account_details = load_account(network_config['network_id'])
        if account_details is None:
            account_details = new_account(network_config['network_id'])
        _, _, account = account_details

        print('ACCOUNT:', account.as_str())
        qr = qrcode.QRCode()
        qr.add_data(account.as_str())
        f = io.StringIO()
        qr.print_ascii(out=f)
        f.seek(0)
        print(f.read())

if __name__ == '__main__':
    asyncio.run(main())