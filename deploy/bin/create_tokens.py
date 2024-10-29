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
from tools.manifests import create_referral_str, create_recovery_key_str

timestamp = datetime.datetime.now().strftime("%Y%m%d%H")

async def main():
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

        owner_amount = '4'
        owner_resource = config_data['OWNER_RESOURCE']
        authority_resource = config_data['AUTHORITY_RESOURCE']
        env_registry_component = config_data['ENV_REGISTRY_COMPONENT']

        balance = await gateway.get_xrd_balance(account)
        if balance < 1000:
            print('FUND ACCOUNT:', account.as_str())
        while balance < 1000:
            await asyncio.sleep(5)
            balance = await gateway.get_xrd_balance(account)

        if 'REFERRAL_RESOURCE' not in config_data:
            manifest = create_referral_str(account, owner_amount, owner_resource, authority_resource)
            payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
            await gateway.submit_transaction(payload)
            addresses = await gateway.get_new_addresses(intent)
            config_data['REFERRAL_RESOURCE'] = addresses[0]

        referral_resource = config_data['REFERRAL_RESOURCE']
        print('REFERRAL_RESOURCE:', referral_resource)

        if 'RECOVERY_KEY_RESOURCE' not in config_data:
            manifest = create_recovery_key_str(account, owner_amount, owner_resource, authority_resource)
            payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
            await gateway.submit_transaction(payload)
            addresses = await gateway.get_new_addresses(intent)
            config_data['RECOVERY_KEY_RESOURCE'] = addresses[0]

        recovery_key_resource = config_data['RECOVERY_KEY_RESOURCE']
        print('RECOVERY_KEY_RESOURCE:', recovery_key_resource)

        manifest = f'''
            CALL_METHOD
                Address("{account.as_str()}")
                "lock_fee"
                Decimal("10")
            ;
            CALL_METHOD
                Address("{account.as_str()}")
                "create_proof_of_amount"
                Address("{owner_resource}")
                Decimal("4")
            ;
            CALL_METHOD
                Address("{env_registry_component}")
                "set_variables"
                Array<Tuple>(
                    Tuple(
                        "referral_resource",
                        "{referral_resource}"
                    ),
                    Tuple(
                        "recovery_key_resource",
                        "{recovery_key_resource}"
                    ),
                )
            ;
        '''

        payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
        await gateway.submit_transaction(payload)
        print('Transaction id:', intent)
        status = await gateway.get_transaction_status(intent)
        print('Register variables:', status)

        print('---------- COMPLETE ----------')

        print(f'REFERRAL_RESOURCE={referral_resource}')
        print(f'RECOVERY_KEY_RESOURCE={recovery_key_resource}')

        config_data['REFERRAL_RESOURCE'] = referral_resource
        config_data['RECOVERY_KEY_RESOURCE'] = recovery_key_resource

        with open(config_path, 'w') as config_file:
            json.dump(config_data, config_file, indent=4)
        print(f'Config saved')

        print('-------------------------------------')

if __name__ == '__main__':
    asyncio.run(main())

