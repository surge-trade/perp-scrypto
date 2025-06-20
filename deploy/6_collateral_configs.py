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

        owner_resource = config_data['OWNER_RESOURCE']
        exchange_component = config_data['EXCHANGE_COMPONENT']
        btc_resource = config_data['BTC_RESOURCE']
        eth_resource = config_data['ETH_RESOURCE']
        lsulp_resource = config_data['LSULP_RESOURCE']
        link_resource = config_data['LINK_RESOURCE']

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


        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.account_create_proof_of_amount(
            account,
            ret.Address(owner_resource),
            ret.Decimal('4')
        )
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(exchange_component)),
            'update_collateral_configs',
            [ret.ManifestBuilderValue.ARRAY_VALUE(ret.ManifestBuilderValueKind.TUPLE_VALUE, [
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(network_config['xrd']))),
                    ret.ManifestBuilderValue.TUPLE_VALUE([
                        ret.ManifestBuilderValue.STRING_VALUE("XRD/USD"), # pair_id
                        ret.ManifestBuilderValue.I64_VALUE(5), # price_age_max
                        ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.90')), # discount
                        ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.02')), # margin
                    ])
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(lsulp_resource))),
                    ret.ManifestBuilderValue.TUPLE_VALUE([
                        ret.ManifestBuilderValue.STRING_VALUE("LSULP/USD"), # pair_id
                        ret.ManifestBuilderValue.I64_VALUE(5), # price_age_max
                        ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.88')), # discount
                        ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.02')), # margin
                    ])
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(btc_resource))),
                    ret.ManifestBuilderValue.TUPLE_VALUE([
                        ret.ManifestBuilderValue.STRING_VALUE("BTC/USD"), # pair_id
                        ret.ManifestBuilderValue.I64_VALUE(5), # price_age_max
                        ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.95')), # discount
                        ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # margin
                    ])
                ]),
                ret.ManifestBuilderValue.TUPLE_VALUE([
                    ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(eth_resource))),
                    ret.ManifestBuilderValue.TUPLE_VALUE([
                        ret.ManifestBuilderValue.STRING_VALUE("ETH/USD"), # pair_id
                        ret.ManifestBuilderValue.I64_VALUE(5), # price_age_max
                        ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.95')), # discount
                        ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('0.01')), # margin
                    ])
                ]),
            ])]
        )

        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        print('Transaction id:', intent)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Transaction status:', status)

if __name__ == '__main__':
    asyncio.run(main())

