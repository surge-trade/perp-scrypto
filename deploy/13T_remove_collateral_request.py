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
from tools.manifests import lock_fee, deposit_all

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

        if network_config['network_name'] == 'stokenet':
            config_path = join(path, 'stokenet.config.json')
        elif network_config['network_name'] == 'mainnet':
            config_path = join(path, 'mainnet.config.json')
        else:
            raise ValueError(f'Unsupported network: {network_config["network_name"]}')
        
        with open(config_path, 'r') as config_file:
            config_data = json.load(config_file)

        base_resource = config_data['BASE_RESOURCE']
        exchange_component = config_data['EXCHANGE_COMPONENT']
        account_component = config_data['ACCOUNT_COMPONENT']

        balance = await gateway.get_xrd_balance(account)
        if balance < 1000:
            print('FUND ACCOUNT:', account.as_str())
        while balance < 1000:
            await asyncio.sleep(5)
            balance = await gateway.get_xrd_balance(account)

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(exchange_component)),
            'remove_collateral_request',
            [
                ret.ManifestBuilderValue.ENUM_VALUE(0, []), # Fee oath
                ret.ManifestBuilderValue.U64_VALUE(10000000000), # Expiry seconds
                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(account_component))), # Margin account
                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(account)), # Target account
                ret.ManifestBuilderValue.ARRAY_VALUE(ret.ManifestBuilderValueKind.TUPLE_VALUE, [ # Claims
                    ret.ManifestBuilderValue.TUPLE_VALUE([
                        ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(base_resource))),
                        ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('100')),
                    ]),
                ])
            ]
        )

        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        print('Transaction id:', intent)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Transaction status:', status)

if __name__ == '__main__':
    asyncio.run(main())

