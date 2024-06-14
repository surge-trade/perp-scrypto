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
    path = dirname(dirname(realpath(__file__)))
    chdir(path)

    async with ClientSession(connector=TCPConnector(ssl=False)) as session:
        pair_ids = ['AAPL/USD', 'TSLA/USD']
        prices = await get_prices(session, pair_ids)

        print(prices)

if __name__ == '__main__':
    asyncio.run(main())

