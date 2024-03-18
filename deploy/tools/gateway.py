from radix_engine_toolkit import *
from typing import Tuple
from aiohttp import ClientSession
import asyncio
import json
import os
import random

class Gateway:
    def __init__(self, session: ClientSession) -> None:
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

    async def get_xrd_balance(self, account: str) -> float:
        network_config = await self.network_configuration()
        xrd = network_config['xrd']

        headers = {
            'Content-Type': 'application/json',
        }
        body = {
            'address': account,
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
            builder: ManifestBuilder, 
            public_key: PublicKey, 
            private_key: PrivateKey,
            blobs: list = [],
            epochs_valid: int = 10
        ) -> Tuple[str, str]:
        
        epoch = await self.get_current_epoch()
        network_config = await self.network_configuration()
        network_id = network_config['network_id']

        manifest: TransactionManifest = builder.build(network_id)
        manifest.statically_validate()
        header: TransactionHeader = TransactionHeader(
            network_id=network_id,
            start_epoch_inclusive=epoch,
            end_epoch_exclusive=epoch + epochs_valid,
            nonce=self.random_nonce(),
            notary_public_key=public_key,
            notary_is_signatory=False,
            tip_percentage=0,
        )
        transaction: NotarizedTransaction = (
            TransactionBuilder()
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
        owner_role: OwnerRole,
        public_key: PublicKey,
        private_key: PrivateKey,
        metadata: dict = {},
        epochs_valid: int = 10
    ) -> Tuple[str, str]:
        
        epoch = await self.get_current_epoch()
        network_config = await self.network_configuration()
        network_id = network_config['network_id']

        manifest: TransactionManifest = (
            ManifestBuilder().account_lock_fee(account, Decimal('300'))
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
        header: TransactionHeader = TransactionHeader(
            network_id=network_id,
            start_epoch_inclusive=epoch,
            end_epoch_exclusive=epoch + epochs_valid,
            nonce=self.random_nonce(),
            notary_public_key=public_key,
            notary_is_signatory=False,
            tip_percentage=0,
        )
        transaction: NotarizedTransaction = (
            TransactionBuilder()
                .header(header)
                .manifest(manifest)
                .sign_with_private_key(private_key)
                .notarize_with_private_key(private_key)
        )
        intent = transaction.intent_hash().as_str()
        payload = bytearray(transaction.compile()).hex()
        return payload, intent
