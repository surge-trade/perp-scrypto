#![allow(dead_code)]

use referral_generator::ReferralData;
use scrypto::prelude::Url;
use scrypto_test::prelude::*;

#[derive(Clone)]
pub struct Resources {
    pub owner_resource: ResourceAddress,
    pub owner_role: OwnerRole,
    pub authority_resource: ResourceAddress,
    pub base_authority_resource: ResourceAddress,
    pub base_resource: ResourceAddress,
    pub lp_resource: ResourceAddress,
    pub protocol_resource: ResourceAddress,
    pub referral_resource: ResourceAddress,
    pub keeper_reward_resource: ResourceAddress,
}

pub fn create_resources(
    account: ComponentAddress,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> Resources {
    let owner_resource = ledger.create_fungible_resource(dec!(9), 0, account);
    let owner_role = OwnerRole::Fixed(rule!(allow_all)); // OwnerRole::Updatable(rule!(require_amount(dec!(4), owner_resource)));
    let authority_resource = ledger.create_fungible_resource(dec!(1), 18, account);
    let base_authority_resource = ledger.create_fungible_resource(dec!(1), 18, account);

    let base_resource = create_base_resource(account, owner_role.clone(), base_authority_resource, ledger);
    let lp_resource = create_lp_resource(owner_role.clone(), authority_resource, ledger);
    let protocol_resource = mint_protocol_resource(account, owner_role.clone(), ledger);
    let referral_resource = create_referral_resource(owner_role.clone(), authority_resource, ledger);
    let keeper_reward_resource = create_keeper_reward_resource(owner_role.clone(), authority_resource, ledger);

    Resources {
        owner_resource,
        owner_role,
        authority_resource,
        base_authority_resource,
        base_resource,
        lp_resource,
        protocol_resource,
        referral_resource,
        keeper_reward_resource,
    }
}

fn mint_protocol_resource(
    account: ComponentAddress,
    owner_role: OwnerRole,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> ResourceAddress {
    let metadata = metadata!(
        init {
            "name" => "Surge", updatable;
            "symbol" => "SRG", updatable;
            "description" => "Surge protocol utility token.", updatable;
            "icon_url" => Url::of("https://surge.trade/images/surge_token.png"), updatable;
            "info_url" => Url::of("https://surge.trade"), updatable;
        }
    );

    let resource_roles = FungibleResourceRoles {
        burn_roles: burn_roles! {
            burner => rule!(allow_all);
            burner_updater => rule!(deny_all);
        },
        ..Default::default()
    };

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_fungible_resource(
            owner_role, 
            true, 
            DIVISIBILITY_MAXIMUM, 
            resource_roles, 
            metadata, 
            Some(dec!(100000000))
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success().new_resource_addresses()[0]
}

fn create_base_resource(
    account: ComponentAddress,
    owner_role: OwnerRole, 
    base_authority_resource: ResourceAddress, 
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> ResourceAddress {
    let metadata = metadata!(
        init {
            "name" => "Surge USD", updatable;
            "symbol" => "sUSD", updatable;
            "description" => "Surge wrapped USD.", updatable;
            "icon_url" => Url::of("https://surge.trade/images/susd_token.png"), updatable;
            "info_url" => Url::of("https://surge.trade"), updatable;
        }
    );
    let resource_roles = FungibleResourceRoles {
        mint_roles: mint_roles! {
            minter => rule!(require(base_authority_resource));
            minter_updater => rule!(deny_all);
        },
        burn_roles: burn_roles! {
            burner => rule!(allow_all);
            burner_updater => rule!(deny_all);
        },
        ..Default::default()
    };
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_fungible_resource(
            owner_role, 
            true,
            DIVISIBILITY_MAXIMUM,
            resource_roles, 
            metadata, 
            Some(dec!(100000000000)) // None
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success().new_resource_addresses()[0]
}

fn create_lp_resource(
    owner_role: OwnerRole, 
    authority_resource: ResourceAddress, 
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> ResourceAddress {
    let metadata = metadata!(
        init {
            "name" => "Surge LP", updatable;
            "symbol" => "SLP", updatable;
            "description" => "Surge liquidity pool LP token.", updatable;
            "icon_url" => Url::of("https://surge.trade/images/surge_lp_token.png"), updatable;
            "info_url" => Url::of("https://surge.trade"), updatable;
        }
    );
    let resource_roles = FungibleResourceRoles {
        mint_roles: mint_roles! {
            minter => rule!(require(authority_resource));
            minter_updater => rule!(deny_all);
        },
        burn_roles: burn_roles! {
            burner => rule!(require(authority_resource));
            burner_updater => rule!(deny_all);
        },
        ..Default::default()
    };
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_fungible_resource(
            owner_role, 
            true,
            DIVISIBILITY_MAXIMUM,
            resource_roles, 
            metadata, 
            None
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success().new_resource_addresses()[0]
}

fn create_referral_resource(
    owner_role: OwnerRole, 
    authority_resource: ResourceAddress, 
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> ResourceAddress {
    let metadata = metadata!(
        init {
            "name" => "Surge Referral", updatable;
            "description" => "Surge referral badge that can grant reduced fees and earn rewards.", updatable;
            "icon_url" => Url::of("https://surge.trade/images/referral_badge.png"), updatable;
            "info_url" => Url::of("https://surge.trade"), updatable;
        }
    );
    let resource_roles = NonFungibleResourceRoles {
        mint_roles: mint_roles! {
            minter => rule!(require(authority_resource));
            minter_updater => rule!(deny_all);
        },
        burn_roles: burn_roles! {
            burner => rule!(deny_all);
            burner_updater => OWNER;
        },
        freeze_roles: None,
        recall_roles: None,
        withdraw_roles: withdraw_roles! {
            withdrawer => rule!(deny_all);
            withdrawer_updater => OWNER;
        },
        deposit_roles: None,
        non_fungible_data_update_roles: non_fungible_data_update_roles! {
            non_fungible_data_updater => rule!(require(authority_resource));
            non_fungible_data_updater_updater => rule!(deny_all);
        },
    };

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_non_fungible_resource::<Vec<(NonFungibleLocalId, ReferralData)>, ReferralData>(
            owner_role, 
            NonFungibleIdType::RUID, 
            true, 
            resource_roles, 
            metadata, 
            None
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success().new_resource_addresses()[0]
}

fn create_keeper_reward_resource(
    owner_role: OwnerRole,
    authority_resource: ResourceAddress,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> ResourceAddress {
    let metadata = metadata!(
        init {
            "name" => "Surge Keeper Reward", updatable;
            "symbol" => "SKR", updatable;
            "description" => "Surge keeper reward token.", updatable;
            "icon_url" => Url::of("https://surge.trade/images/surge_keeper_reward_token.png"), updatable;
            "info_url" => Url::of("https://surge.trade"), updatable;
        }
    );

    let resource_roles = FungibleResourceRoles {
        mint_roles: mint_roles! {
            minter => rule!(require(authority_resource));
            minter_updater => rule!(deny_all);
        },
        burn_roles: burn_roles! {
            burner => rule!(allow_all);
            burner_updater => rule!(deny_all);
        },
        ..Default::default()
    };

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_fungible_resource(
            owner_role, 
            true, 
            DIVISIBILITY_MAXIMUM, 
            resource_roles, 
            metadata, 
            None
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success().new_resource_addresses()[0]
}


