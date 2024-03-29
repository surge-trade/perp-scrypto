import radix_engine_toolkit as ret
import asyncio
import httpx
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
from tools.manifests import lock_fee, deposit_all

pairs = [
    'SUPRA.PROD.EVM.BTC_USDT',
    'SUPRA.PROD.EVM.ETH_USDT',
    'SUPRA.PROD.EVM.SOL_USDT',
]

class Oracle:
    def __init__(self, account: ret.Address, public_key: ret.PublicKey, private_key: ret.PrivateKey, gateway: Gateway,  oracle_component: ret.Address):
        self.account = account
        self.public_key = public_key
        self.private_key = private_key
        self.gateway = gateway
        self.oracle_component = oracle_component

    async def submit_price(self, pair_id: int, price: ret.Decimal):
        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, self.account, 2)
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(self.oracle_component),
            'push_price',
            [
                ret.ManifestBuilderValue.U16_VALUE(pair_id),
                ret.ManifestBuilderValue.DECIMAL_VALUE(price),
            ]
        )
        builder = deposit_all(builder, self.account)

        payload, intent = await self.gateway.build_transaction(builder, self.public_key, self.private_key)
        await self.gateway.submit_transaction(payload)
        print(pair_id, price.as_str())

async def stream_prices(oracle: Oracle, pair_id: int, pair_name: str):
    while True:
        async with httpx.AsyncClient(http2=True) as client:
            async with client.stream("GET", "https://datahub01.surge.trade/charts/updates", params={'pair_name': pair_name}) as response:
                if response.status_code == 200:
                    tick = datetime.datetime.now()
                    async for chunk in response.aiter_bytes():
                        if b'data:' in chunk:
                            try:
                                data = json.loads(chunk.decode().split('data: ')[1])
                                print(data)
                                price = data['price']
                                tock = datetime.datetime.now()
                                if tock - tick > datetime.timedelta(seconds=10):
                                    asyncio.create_task(oracle.submit_price(pair_id, ret.Decimal(str(price))))
                                    tick = datetime.datetime.now()
                            except Exception as e:
                                print(f"Error processing chunk: {e}")
                    end_time = datetime.datetime.now()
                else:
                    print(f"Failed to fetch data for {pair_name}, status code: {response.status_code}")

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

        oracle_component = config_data['ORACLE_COMPONENT']
        oracle = Oracle(account, public_key, private_key, gateway, ret.Address(oracle_component))

        balance = await gateway.get_xrd_balance(account)
        if balance < 1000:
            print('FUND ACCOUNT:', account.as_str())
        while balance < 1000:
            await asyncio.sleep(5)
            balance = await gateway.get_xrd_balance(account)

        tasks = []
        for pair_id, pair_name in enumerate(pairs):
            tasks.append(stream_prices(oracle, pair_id, pair_name))
        await asyncio.gather(*tasks)

if __name__ == '__main__':
    asyncio.run(main())
