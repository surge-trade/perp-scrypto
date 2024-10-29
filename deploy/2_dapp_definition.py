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

        balance = await gateway.get_xrd_balance(account)
        if balance < 1000:
            print('FUND ACCOUNT:', account.as_str())
        while balance < 1000:
            await asyncio.sleep(5)
            balance = await gateway.get_xrd_balance(account)

        data = {
            "BASE_RESOURCE": config_data['BASE_RESOURCE'],
            "LP_RESOURCE": config_data['LP_RESOURCE'],
            "PROTOCOL_RESOURCE": config_data['PROTOCOL_RESOURCE'],
            "REFERRAL_RESOURCE": config_data['REFERRAL_RESOURCE'],
            "RECOVERY_KEY_RESOURCE": config_data['RECOVERY_KEY_RESOURCE'],
            "KEEPER_REWARD_RESOURCE": config_data['KEEPER_REWARD_RESOURCE'],
            "FEE_OATH_RESOURCE": config_data['FEE_OATH_RESOURCE'],
            "TOKEN_WRAPPER_PACKAGE": config_data['TOKEN_WRAPPER_PACKAGE'],
            "TOKEN_WRAPPER_COMPONENT": config_data['TOKEN_WRAPPER_COMPONENT'],
            "ORACLE_PACKAGE": config_data['ORACLE_PACKAGE'],
            "ORACLE_COMPONENT": config_data['ORACLE_COMPONENT'],
            "CONFIG_PACKAGE": config_data['CONFIG_PACKAGE'],
            "CONFIG_COMPONENT": config_data['CONFIG_COMPONENT'],
            "ACCOUNT_PACKAGE": config_data['ACCOUNT_PACKAGE'],
            "POOL_PACKAGE": config_data['POOL_PACKAGE'],
            "POOL_COMPONENT": config_data['POOL_COMPONENT'],
            "REFERRAL_GENERATOR_PACKAGE": config_data['REFERRAL_GENERATOR_PACKAGE'],
            "REFERRAL_GENERATOR_COMPONENT": config_data['REFERRAL_GENERATOR_COMPONENT'],
            "FEE_DISTRIBUTOR_PACKAGE": config_data['FEE_DISTRIBUTOR_PACKAGE'],
            "FEE_DISTRIBUTOR_COMPONENT": config_data['FEE_DISTRIBUTOR_COMPONENT'],
            "FEE_DELEGATOR_PACKAGE": config_data['FEE_DELEGATOR_PACKAGE'],
            "FEE_DELEGATOR_COMPONENT": config_data['FEE_DELEGATOR_COMPONENT'],
            "PERMISSION_REGISTRY_PACKAGE": config_data['PERMISSION_REGISTRY_PACKAGE'],
            "PERMISSION_REGISTRY_COMPONENT": config_data['PERMISSION_REGISTRY_COMPONENT'],
            "ENV_REGISTRY_PACKAGE": config_data['ENV_REGISTRY_PACKAGE'],
            "ENV_REGISTRY_COMPONENT": config_data['ENV_REGISTRY_COMPONENT'],
            "EXCHANGE_PACKAGE": config_data['EXCHANGE_PACKAGE'],
            "EXCHANGE_COMPONENT": config_data['EXCHANGE_COMPONENT']
        }
        if network_config['network_name'] == 'stokenet':
            faucet_owner_resource = config_data['FAUCET_OWNER_RESOURCE']
            data['FAUCET_COMPONENT'] = config_data['FAUCET_COMPONENT']

        dapp_definition = account.as_str()
        entities = [f'Address("{entity}")' for entity in data.values()]

        name = 'Surge'
        description = 'Feel the Surge!'
        icon_url = 'https://surge.trade/images/icon_dapp.png'
        claimed_entities = ', '.join(entities)
        websites = ', '.join([f'"https://www.surge.trade"', '"https://power.surge.trade"', '"https://ilovetesting.surge.trade"'])

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
            SET_METADATA
                Address("{dapp_definition}")
                "account_type"
                Enum<Metadata::String>("dapp definition")
            ;
            SET_METADATA
                Address("{dapp_definition}")
                "name"
                Enum<Metadata::String>("{name}")
            ;
            SET_METADATA
                Address("{dapp_definition}")
                "description"
                Enum<Metadata::String>("{description}")
            ;
            SET_METADATA
                Address("{dapp_definition}")
                "icon_url"
                Enum<Metadata::Url>("{icon_url}")
            ;
            SET_METADATA
                Address("{dapp_definition}")
                "claimed_entities"
                Enum<Metadata::AddressArray>(
                    Array<Address>({claimed_entities})
                )
            ;
            SET_METADATA
                Address("{dapp_definition}")
                "claimed_websites"
                Enum<Metadata::OriginArray>(
                    Array<String>({websites})
                )
            ;
        '''
        if network_config['network_name'] == 'stokenet':
            manifest += f'''
                CALL_METHOD
                    Address("{account.as_str()}")
                    "create_proof_of_amount"
                    Address("{faucet_owner_resource}")
                    Decimal("1")
                ;
            '''
        for entity in entities:
            if 'component' in entity or 'package' in entity:
                manifest += f'''
                    SET_METADATA
                        {entity}
                        "dapp_definition"
                        Enum<Metadata::Address>(Address("{dapp_definition}"))
                    ;
                '''
            elif 'resource' in entity:
                manifest += f'''
                    SET_METADATA
                        {entity}
                        "dapp_definitions"
                        Enum<Metadata::AddressArray>(
                            Array<Address>(Address("{dapp_definition}"))
                        )
                    ;
                '''
        print(manifest)

        payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print('Update dapp definition:', status)
        print('Dapp definition:', dapp_definition)

if __name__ == '__main__':
    asyncio.run(main())

