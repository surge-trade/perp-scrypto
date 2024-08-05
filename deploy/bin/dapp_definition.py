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
from tools.manifests import lock_fee, deposit_all

async def main():
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

        owner_resource = config_data['OWNER_RESOURCE']
        faucet_owner_resource = config_data['FAUCET_OWNER_RESOURCE']

        balance = await gateway.get_xrd_balance(account)
        if balance < 1000:
            print('FUND ACCOUNT:', account.as_str())
        while balance < 1000:
            await asyncio.sleep(5)
            balance = await gateway.get_xrd_balance(account)

        data = {
            "BASE_RESOURCE": "resource_tdx_2_1t5cdu0q026l37k6mfz7phvdrrc2xd5sd4fwh8t0dja65plss8uy9rj",
            "LP_RESOURCE": "resource_tdx_2_1thc3frgceyfa2kqy5kzf9su6jsr5ravj2gngsyxzz9d4uclcssfnxe",
            "REFERRAL_RESOURCE": "resource_tdx_2_1ngr79yy7f09nxe50wsuk2hh60wgtmjmk695c2nee2pyhz8ln8a5a7l",
            "PROTOCOL_RESOURCE": "resource_tdx_2_1thyvdxccuj7hhgw0543ct0d53g4ft2rtszl77a85wxm74du2zhs4vw",
            "KEEPER_REWARD_RESOURCE": "resource_tdx_2_1t4w5rg4549lq5me5zqzng3wgyk8uxw8zj8nyf3ry4jx9v8wyrx0ctx",
            "FAUCET_COMPONENT": "component_tdx_2_1cp6pevyk6jrv573v0z38f0tt7282a4c9dfpnf3gscgt9gq0v0e0667",
            "TOKEN_WRAPPER_COMPONENT": "component_tdx_2_1cr44cp5s3mv0smr0hwj2er8agw9q9mvqjw7u0e3uqn8auyf4pq987g",
            "ORACLE_COMPONENT": "component_tdx_2_1crp59cff90nhg32gesghh55gh44qut8ahuu5ayh2dkzjgt2f4esgrf",
            "CONFIG_COMPONENT": "component_tdx_2_1crkfxkke5pm3xenp5dvz8yxw54mhalc43dtrm3rerkl8r283f3puhn",
            "POOL_COMPONENT": "component_tdx_2_1cpcg8uc4naqq88dkf5pyy3m92ewedawmq49nsf7hxmqa3jjv7e600l",
            "REFERRAL_GENERATOR_COMPONENT": "component_tdx_2_1cr4h0860su7xcgqtpg3qfsenll4nj4aypz4z3j7rk49fr2z440v303",
            "FEE_DISTRIBUTOR_COMPONENT": "component_tdx_2_1czzxkuseat8p7x4pmfnajx4jsjp3ghlz70jn393th05qjhd6av006p",
            "FEE_DELEGATOR_COMPONENT": "component_tdx_2_1cpg0tam0hxgd2r0zqzh4qupav6spjaj59r76vwelqvrmlgv7n4rrk9",
            "FEE_OATH_RESOURCE": "resource_tdx_2_1t4qca6n25lpa34ufqha73hclfwy5j44nvk5rrsu2v4r0rshdecvact",
            "PERMISSION_REGISTRY_COMPONENT": "component_tdx_2_1cq7kcg0zqs37z9jvkg9cjqhxv34g65mt4pa448glwf20xrrudet83d",
            "ENV_REGISTRY_COMPONENT": "component_tdx_2_1cp8mmzmwjhzcph82l3n0eet3pka75mk246qpsf05s4dqmvq2tgeq42",
            "EXCHANGE_COMPONENT": "component_tdx_2_1czlx83r6w7mkd9j03k9djshr8qmuj2kxt2ymhc806yngpd3hwexvp7"
        }

        dapp_definition = account.as_str()
        entities = [f'Address("{entity}")' for entity in data.values()]

        name = 'Surge'
        description = 'Feel the Surge!'
        icon_url = 'https://surge.trade/images/icon_dapp.png'
        claimed_entities = ', '.join(entities)
        websites = ', '.join([f'"https://surge.trade"'])

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
                Decimal("1")
            ;
            CALL_METHOD
                Address("{account.as_str()}")
                "create_proof_of_amount"
                Address("{faucet_owner_resource}")
                Decimal("1")
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
        for entity in entities:
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

