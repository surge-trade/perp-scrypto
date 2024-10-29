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

        if network_config['network_name'] == 'stokenet':
            config_path = join(path, 'stokenet.config.json')
        elif network_config['network_name'] == 'mainnet':
            config_path = join(path, 'mainnet.config.json')
        else:
            raise ValueError(f'Unsupported network: {network_config["network_name"]}')
        
        with open(config_path, 'r') as config_file:
            config_data = json.load(config_file)

        faucet_owner_resource = config_data['FAUCET_OWNER_RESOURCE']
        faucet_component = config_data['FAUCET_COMPONENT']
        token_wrapper_component = config_data['TOKEN_WRAPPER_COMPONENT']
        usdc_resource = config_data['USDC_RESOURCE']

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

        amount = '10000000';

        builder = ret.ManifestBuilder()
        builder = lock_fee(builder, account, 100)
        builder = builder.account_create_proof_of_amount(
            account,
            ret.Address(faucet_owner_resource),
            ret.Decimal('1')
        )
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(faucet_component)),
            'admin_mint_token',
            [
                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(usdc_resource))),
                ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal(amount)),
            ]
        )
        builder = builder.take_all_from_worktop(
            ret.Address(usdc_resource),
            ret.ManifestBuilderBucket('bucket1')
        )
        builder = builder.call_method(
            ret.ManifestBuilderAddress.STATIC(ret.Address(token_wrapper_component)),
            'wrap',
            [ret.ManifestBuilderValue.BUCKET_VALUE(ret.ManifestBuilderBucket('bucket1'))]
        )
        builder = deposit_all(builder, account)

        payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        print('Transaction id:', intent)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Transaction status:', status)

if __name__ == '__main__':
    asyncio.run(main())

