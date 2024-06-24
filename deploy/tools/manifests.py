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
            'icon_url': MetadataInitEntry(MetadataValue.STRING_VALUE('https://surge.trade/images/owner_token.png'), True),
            'info_url': MetadataInitEntry(MetadataValue.STRING_VALUE('https://surge.trade'), True),
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
            'icon_url': MetadataInitEntry(MetadataValue.STRING_VALUE('https://surge.trade/images/authority_token.png'), True),
            'info_url': MetadataInitEntry(MetadataValue.STRING_VALUE('https://surge.trade'), True),
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

def mint_base_authority(builder: ManifestBuilder) -> ManifestBuilder:
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
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('Base Authority'), True),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('BAUTH'), True),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('A single attos holds exceptional power.'), True),
            'icon_url': MetadataInitEntry(MetadataValue.STRING_VALUE('https://surge.trade/images/base_authority_token.png'), True),
            'info_url': MetadataInitEntry(MetadataValue.STRING_VALUE('https://surge.trade'), True),
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
            role=AccessRule.allow_all, 
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
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge wrapped USD.'), True),
            'icon_url': MetadataInitEntry(MetadataValue.STRING_VALUE('https://surge.trade/images/susd_token.png'), True),
            'info_url': MetadataInitEntry(MetadataValue.STRING_VALUE('https://surge.trade'), True),
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

def create_lp(builder: ManifestBuilder, owner_role: OwnerRole, authority_resource: str) -> ManifestBuilder:
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
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge LP'), True),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('SLP'), True),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge liquidity pool LP token.'), True),
            'icon_url': MetadataInitEntry(MetadataValue.STRING_VALUE('https://surge.trade/images/lp_token.png'), True),
            'info_url': MetadataInitEntry(MetadataValue.STRING_VALUE('https://surge.trade'), True),
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

def create_referral_str(account: Address, owner_resource: str, authority_resource: str) -> str:
    return f'''
CALL_METHOD
    Address("{account.as_str()}")
    "lock_fee"
    Decimal("10")
;
CREATE_NON_FUNGIBLE_RESOURCE
    Enum<2u8>(
        Enum<2u8>(
            Enum<0u8>(
                Enum<0u8>(
                    Enum<1u8>(
                        Address("{owner_resource}")
                    )
                )
            )
        )
    )
    Enum<3u8>()
    true
    Enum<0u8>(
        Enum<0u8>(
            Tuple(
                Array<Enum>(
                    Enum<14u8>(
                        Array<Enum>(
                            Enum<0u8>(
                                192u8
                            ),
                            Enum<0u8>(
                                192u8
                            ),
                            Enum<0u8>(
                                10u8
                            ),
                            Enum<0u8>(
                                10u8
                            ),
                            Enum<0u8>(
                                192u8
                            ),
                            Enum<0u8>(
                                192u8
                            )
                        )
                    )
                ),
                Array<Tuple>(
                    Tuple(
                        Enum<1u8>(
                            "ReferralData"
                        ),
                        Enum<1u8>(
                            Enum<0u8>(
                                Array<String>(
                                    "fee_referral",
                                    "fee_rebate",
                                    "referrals",
                                    "max_referrals",
                                    "balance",
                                    "total_rewarded"
                                )
                            )
                        )
                    )
                ),
                Array<Enum>(
                    Enum<0u8>()
                )
            )
        ),
        Enum<1u8>(
            0u64
        ),
        Array<String>(
            "fee_referral",
            "fee_rebate",
            "referrals",
            "max_referrals",
            "balance",
            "total_rewarded"
        )
    )
    Tuple(
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<2u8>(
                        Enum<0u8>(
                            Enum<0u8>(
                                Enum<1u8>(
                                    Address("{authority_resource}")
                                )
                            )
                        )
                    )
                ),
                Enum<1u8>(
                    Enum<1u8>()
                )
            )
        ),
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<1u8>()
                ),
                Enum<0u8>()
            )
        ),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<1u8>()
                ),
                Enum<0u8>()
            )
        ),
        Enum<0u8>(),
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<2u8>(
                        Enum<0u8>(
                            Enum<0u8>(
                                Enum<1u8>(
                                    Address("{authority_resource}")
                                )
                            )
                        )
                    )
                ),
                Enum<1u8>(
                    Enum<1u8>()
                )
            )
        )
    )
    Tuple(
        Map<String, Tuple>(
            "name" => Tuple(
                Enum<1u8>(
                    Enum<0u8>(
                        "Surge Referral"
                    )
                ),
                false
            ),
            "description" => Tuple(
                Enum<1u8>(
                    Enum<0u8>(
                        "Surge referral badge that can grant reduced fees and earn rewards."
                    )
                ),
                false
            ),
            "icon_url" => Tuple(
                Enum<1u8>(
                    Enum<13u8>(
                        "https://surge.trade/images/referral_badge.png"
                    )
                ),
                false
            ),
            "info_url" => Tuple(
                Enum<1u8>(
                    Enum<13u8>(
                        "https://surge.trade"
                    )
                ),
                false
            )
        ),
        Map<String, Enum>()
    )
    Enum<0u8>()
;
'''

def create_protocol_resource(builder: ManifestBuilder, owner_role: OwnerRole, authority_resource: str) -> ManifestBuilder:
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
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge'), True),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('SRG'), True),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge protocol utility token.'), True),
            'icon_url': MetadataInitEntry(MetadataValue.STRING_VALUE('https://surge.trade/images/surge_token.png'), True),
            'info_url': MetadataInitEntry(MetadataValue.STRING_VALUE('https://surge.trade'), True),
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
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge Keeper Reward'), True),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('SRWD'), True),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge keeper reward token.'), True),
            'icon_url': MetadataInitEntry(MetadataValue.STRING_VALUE('https://surge.trade/images/keeper_reward_token.png'), True),
            'info_url': MetadataInitEntry(MetadataValue.STRING_VALUE('https://surge.trade'), True),
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

def mint_test_btc(builder: ManifestBuilder) -> ManifestBuilder:
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
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('Bitcoin'), True),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('BTC'), True),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('The original cryptocurrency.'), True),
        },
        roles={},
    )

    return builder.create_fungible_resource_manager(
        owner_role=OwnerRole.NONE(),
        track_total_supply=True,
        divisibility=18,
        initial_supply=Decimal('21000000'),
        resource_roles=resource_roles,
        metadata=metadata,
        address_reservation=None,
    )

def mint_test_usd(builder: ManifestBuilder) -> ManifestBuilder:
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
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('USD'), True),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('USD'), True),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('The greatest tool of a powerful nation.'), True),
        },
        roles={},
    )

    return builder.create_fungible_resource_manager(
        owner_role=OwnerRole.NONE(),
        track_total_supply=True,
        divisibility=18,
        initial_supply=Decimal('1000000000'),
        resource_roles=resource_roles,
        metadata=metadata,
        address_reservation=None,
    )
