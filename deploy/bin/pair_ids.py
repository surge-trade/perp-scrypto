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
from tools.price_feeds import get_feeds

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

        exchange_component = config_data['EXCHANGE_COMPONENT']

        pairs = [
            'BTC/USD', 
            'ETH/USD', 
            'XRD/USD', 
            'BNB/USD', 
            'SOL/USD', 
            'XRP/USD', 
            'DOGE/USD', 
            'ADA/USD', 
            'AVAX/USD', 
            'LINK/USD', 
            'DOT/USD', 
            'NEAR/USD', 
            'MATIC/USD', 
            'LTC/USD', 
            'ATOM/USD', 
            'SUI/USD', 
            'APT/USD', 
            'ARB/USD', 
            'INJ/USD', 
            'SEI/USD',
        ]
        feeds = await get_feeds(session, pairs)

        for feed in feeds.values():
            id = feed['id']
            symbol = feed['attributes']['symbol'][7:]
            print(f'map.insert("{id}".to_string(), "{symbol}".to_string());')


if __name__ == '__main__':
    asyncio.run(main())

