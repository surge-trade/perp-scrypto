from radix_engine_toolkit import *

def lock_fee(builder: ManifestBuilder, account: Address, fee: int) -> ManifestBuilder:
    return builder.account_lock_fee(account, Decimal(str(fee)))

def deposit_all(builder: ManifestBuilder, account: Address) -> ManifestBuilder:
    return builder.account_deposit_entire_worktop(account)

def withdraw_to_bucket(builder: ManifestBuilder, account: Address, resource: Address, amount: Decimal, name: str) -> ManifestBuilder:
    builder = builder.account_withdraw(account, resource, amount)
    builder = builder.take_from_worktop(resource, amount, ManifestBuilderBucket(name))
    return builder

def mint_owner_badge(builder: ManifestBuilder) -> ManifestBuilder:
    resource_roles = FungibleResourceRoles(
        mint_roles=None,
        burn_roles=None,
        freeze_roles=None,
        recall_roles=None,
        withdraw_roles=None,
        deposit_roles=None,
    )
    metadata: MetadataModuleConfig = MetadataModuleConfig(
        init={
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('Glyph of Ownership'), True),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('OWN'), True),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('With power comes responsibility.'), True),
        },
        roles={},
    )

    return builder.create_fungible_resource_manager(
        owner_role=OwnerRole.NONE(),
        track_total_supply=True,
        divisibility=0,
        initial_supply=Decimal('9'),
        resource_roles=resource_roles,
        metadata=metadata,
        address_reservation=None,
    )

def mint_authority(builder: ManifestBuilder) -> ManifestBuilder:
    resource_roles = FungibleResourceRoles(
        mint_roles=None,
        burn_roles=None,
        freeze_roles=None,
        recall_roles=None,
        withdraw_roles=None,
        deposit_roles=None,
    )
    metadata: MetadataModuleConfig = MetadataModuleConfig(
        init={
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('Authority'), True),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('AUTH'), True),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('A single attos holds exceptional power.'), True),
        },
        roles={},
    )

    return builder.create_fungible_resource_manager(
        owner_role=OwnerRole.NONE(),
        track_total_supply=True,
        divisibility=18,
        initial_supply=Decimal('1'),
        resource_roles=resource_roles,
        metadata=metadata,
        address_reservation=None,
    )

def create_base(builder: ManifestBuilder, owner_role: OwnerRole, authority_resource: str) -> ManifestBuilder:
    resource_roles: FungibleResourceRoles = FungibleResourceRoles(
        mint_roles=ResourceManagerRole(
            role=AccessRule.require(ResourceOrNonFungible.RESOURCE(Address(authority_resource))), 
            role_updater=None
        ),
        burn_roles=ResourceManagerRole(
            role=AccessRule.require(ResourceOrNonFungible.RESOURCE(Address(authority_resource))), 
            role_updater=None
        ),
        freeze_roles=None,
        recall_roles=None,
        withdraw_roles=None,
        deposit_roles=None,
    )
    metadata = MetadataModuleConfig(
        init={
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('sUSD'), True),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('sUSD'), True),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('Wrapped USD stablecoin.'), True),
        },
        roles={},
    )

    return builder.create_fungible_resource_manager(
        owner_role=owner_role,
        track_total_supply=True,
        divisibility=18,
        initial_supply=None,
        resource_roles=resource_roles,
        metadata=metadata,
        address_reservation=None,
    )

def create_keeper_reward(builder: ManifestBuilder, owner_role: OwnerRole, authority_resource: str) -> ManifestBuilder:
    resource_roles: FungibleResourceRoles = FungibleResourceRoles(
        mint_roles=ResourceManagerRole(
            role=AccessRule.require(ResourceOrNonFungible.RESOURCE(Address(authority_resource))), 
            role_updater=None
        ),
        burn_roles=ResourceManagerRole(
            role=AccessRule.require(ResourceOrNonFungible.RESOURCE(Address(authority_resource))), 
            role_updater=None
        ),
        freeze_roles=None,
        recall_roles=None,
        withdraw_roles=None,
        deposit_roles=None,
    )
    metadata = MetadataModuleConfig(
        init={
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('Reward'), True),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('RWD'), True),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('Reward for performing keeper actions'), True),
        },
        roles={},
    )

    return builder.create_fungible_resource_manager(
        owner_role=owner_role,
        track_total_supply=True,
        divisibility=18,
        initial_supply=None,
        resource_roles=resource_roles,
        metadata=metadata,
        address_reservation=None,
    )
    