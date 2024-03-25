import radix_engine_toolkit as ret
from typing import Tuple
import secrets
import json
import os

import logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s"
)

def new_account(network_id) -> Tuple[ret.PrivateKey, ret.PublicKey, ret.Address]:
    private_key_bytes: bytes = secrets.randbits(256).to_bytes(32, 'little')
    private_key: ret.PrivateKey = ret.PrivateKey.new_ed25519(private_key_bytes)
    public_key: ret.PublicKey = private_key.public_key()
    account: ret.Address = ret.derive_virtual_account_address_from_public_key(
        public_key, network_id
    )

    try:
        with open('accounts.json', 'r') as f:
            data = json.load(f)
    except:
        logging.info('No accounts found. New file will be created.')
        data = {'accounts': []}

    data['accounts'].append(private_key_bytes.hex())

    with open('accounts.json', 'w') as f:
        json.dump(data, f)

    return private_key, public_key, account

def load_account(network_id, account_index: int = 0,) -> Tuple[ret.PrivateKey, ret.PublicKey, ret.Address]:
    try:
        with open('accounts.json', 'r') as f:
            data = json.load(f)
    except:
        logging.error('No accounts found.')
        return None

    if account_index < 0 or account_index >= len(data['accounts']):
        logging.error('Account not found.')
        return None

    private_key_bytes = bytes.fromhex(data['accounts'][account_index])
    private_key: ret.PrivateKey = ret.PrivateKey.new_ed25519(private_key_bytes)
    public_key: ret.PublicKey = private_key.public_key()
    account: ret.Address = ret.derive_virtual_account_address_from_public_key(
        public_key, network_id
    )

    # Return account details
    return private_key, public_key, account