import radix_engine_toolkit as ret
from typing import Tuple
import aiohttp
import os
import random

class Gateway:
    def __init__(self, session: aiohttp.ClientSession) -> None:
        self.gateway_url = os.getenv('GATEWAY_URL')
        self.session = session

    def random_nonce(self) -> int:
        return random.randint(0, 0xFFFFFFFF)
    
    async def network_configuration(self) -> str:
        headers = {
            'Content-Type': 'application/json',
        }
        body = {}
        async with self.session.post(
            f'{self.gateway_url}/status/network-configuration',
            json=body,
            headers=headers) as response:
        
            data = await response.json()
            return {
                'network_id': data['network_id'],
                'network_name': data['network_name'],
                'xrd': data['well_known_addresses']['xrd'],
                'ed25519_virtual_badge': data['well_known_addresses']['ed25519_signature_virtual_badge'],
            }

    async def get_current_epoch(self) -> int:
        headers = {
            'Content-Type': 'application/json',
        }
        async with self.session.post(
            f'{self.gateway_url}/transaction/construction',
            json={},
            headers=headers) as response:
        
            data = await response.json()
            return data['ledger_state']['epoch']

    async def get_xrd_balance(self, account: ret.Address) -> float:
        network_config = await self.network_configuration()
        xrd = network_config['xrd']

        headers = {
            'Content-Type': 'application/json',
        }
        body = {
            'address': account.as_str(),
        }
        async with self.session.post(
            f'{self.gateway_url}/state/entity/page/fungibles/',
            json=body,
            headers=headers) as response:
        
            data = await response.json()
            amount = 0
            for item in data['items']:
                if item['resource_address'] == xrd:
                    amount = float(item['amount'])
                    break
            return amount

    async def submit_transaction(self, transaction: str) -> dict:
        headers = {
            'Content-Type': 'application/json',
        }
        body = {
            "notarized_transaction_hex": transaction
        }
        async with self.session.post(
            f'{self.gateway_url}/transaction/submit',
            json=body,
            headers=headers) as response:
        
            data = await response.json()
            return data

    async def get_transaction_details(self, intent: str) -> dict:
        headers = {
            'Content-Type': 'application/json',
        }
        body = {
            'intent_hash': intent,
            "opt_ins": {
                "receipt_state_changes": True,
            },
        }
        async with self.session.post(
            f'{self.gateway_url}/transaction/committed-details',
            json=body,
            headers=headers) as response:

            if response.status == 404:
                return None
            else:
                data = await response.json()
                return data
            
    async def get_new_addresses(self, intent: str) -> list:
        details = None
        while details is None:
            details = await self.get_transaction_details(intent)
        addresses = []
        for e in details['transaction']['receipt']['state_updates']['new_global_entities']:
            addresses.append(e['entity_address'])
        return addresses
    
    async def get_transaction_status(self, intent: str) -> dict:
        details = None
        while details is None:
            details = await self.get_transaction_details(intent)
        status = details['transaction']['transaction_status']
        return status

    async def preview_transaction(self, manifest: str) -> dict:
        headers = {
            'Content-Type': 'application/json',
        }
        body = {
            "manifest": manifest,
            "start_epoch_inclusive": 0,
            "end_epoch_exclusive": 1,
            "tip_percentage": 0,
            "nonce": 4294967295,
            "signer_public_keys": [],
            "flags": {
                "use_free_credit": False,
                "assume_all_signature_proofs": True,
                "skip_epoch_check": True
            }
        }
        async with self.session.post(
            f'{self.gateway_url}/transaction/preview',
            json=body,
            headers=headers) as response:
        
            data = await response.json()
            return data

    async def build_transaction(
            self,
            builder: ret.ManifestBuilder, 
            public_key: ret.PublicKey, 
            private_key: ret.PrivateKey,
            blobs: list = [],
            epochs_valid: int = 10
        ) -> Tuple[str, str]:
        
        epoch = await self.get_current_epoch()
        network_config = await self.network_configuration()
        network_id = network_config['network_id']

        manifest: ret.TransactionManifest = builder.build(network_id)
        manifest.statically_validate()
        header: ret.TransactionHeader = ret.TransactionHeader(
            network_id=network_id,
            start_epoch_inclusive=epoch,
            end_epoch_exclusive=epoch + epochs_valid,
            nonce=self.random_nonce(),
            notary_public_key=public_key,
            notary_is_signatory=False,
            tip_percentage=0,
        )
        transaction: ret.NotarizedTransaction = (
            ret.TransactionBuilder()
                .header(header)
                .manifest(manifest)
                .sign_with_private_key(private_key)
                .notarize_with_private_key(private_key)
        )
        intent = transaction.intent_hash().as_str()
        payload = bytearray(transaction.compile()).hex()
        return payload, intent
    
    async def build_transaction_str(
        self,
        manifest: str, 
        public_key: ret.PublicKey, 
        private_key: ret.PrivateKey,
        blobs: list = [],
        epochs_valid: int = 10
    ) -> Tuple[str, str]:
    
        epoch = await self.get_current_epoch()
        network_config = await self.network_configuration()
        network_id = network_config['network_id']

        manifest: ret.TransactionManifest = ret.TransactionManifest(ret.Instructions.from_string(manifest, network_id), blobs)
        manifest.statically_validate()
        header: ret.TransactionHeader = ret.TransactionHeader(
            network_id=network_id,
            start_epoch_inclusive=epoch,
            end_epoch_exclusive=epoch + epochs_valid,
            nonce=self.random_nonce(),
            notary_public_key=public_key,
            notary_is_signatory=False,
            tip_percentage=0,
        )
        transaction: ret.NotarizedTransaction = (
            ret.TransactionBuilder()
                .header(header)
                .manifest(manifest)
                .sign_with_private_key(private_key)
                .notarize_with_private_key(private_key)
        )
        intent = transaction.intent_hash().as_str()
        payload = bytearray(transaction.compile()).hex()
        return payload, intent
    
    async def build_publish_transaction(
        self,
        account: str,
        code: bytes,
        definition: bytes,
        owner_role: ret.OwnerRole,
        public_key: ret.PublicKey,
        private_key: ret.PrivateKey,
        metadata: dict = {},
        epochs_valid: int = 10
    ) -> Tuple[str, str]:
        
        epoch = await self.get_current_epoch()
        network_config = await self.network_configuration()
        network_id = network_config['network_id']

        manifest: ret.TransactionManifest = (
            ret.ManifestBuilder().account_lock_fee(account, ret.Decimal('300'))
            .package_publish_advanced(
                owner_role=owner_role,
                code=code,
                definition=definition,
                metadata=metadata,
                package_address=None,
            )
            .build(network_id)
        )
        manifest.statically_validate()
        header: ret.TransactionHeader = ret.TransactionHeader(
            network_id=network_id,
            start_epoch_inclusive=epoch,
            end_epoch_exclusive=epoch + epochs_valid,
            nonce=self.random_nonce(),
            notary_public_key=public_key,
            notary_is_signatory=False,
            tip_percentage=0,
        )
        transaction: ret.NotarizedTransaction = (
            ret.TransactionBuilder()
                .header(header)
                .manifest(manifest)
                .sign_with_private_key(private_key)
                .notarize_with_private_key(private_key)
        )
        intent = transaction.intent_hash().as_str()
        payload = bytearray(transaction.compile()).hex()
        return payload, intent
