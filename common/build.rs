use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use scrypto::prelude::*;

const DEFAULT_RESOURCE: &str = "resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3";
const DEFAULT_COMPONENT: &str = "account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma";

fn main() {
    // Determine the network to use
    let network = match env::var("NETWORK_ID").unwrap_or_default().as_str() {
        "1" => NetworkDefinition::mainnet(),
        "2" => NetworkDefinition::stokenet(),
        _ => NetworkDefinition::simulator(),
    };
    let decoder = AddressBech32Decoder::new(&network);

    // Specify the path for the output file
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("env_constants.rs");
    let mut f = File::create(&dest_path).unwrap();

    // Write constants to the output file
    writeln!(f, "pub const _AUTHORITY_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("AUTHORITY_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ResourceAddress::try_from_bech32(&decoder, &DEFAULT_RESOURCE)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _BASE_AUTHORITY_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("BASE_AUTHORITY_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ResourceAddress::try_from_bech32(&decoder, &DEFAULT_RESOURCE)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _BASE_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("BASE_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ResourceAddress::try_from_bech32(&decoder, &DEFAULT_RESOURCE)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _LP_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("LP_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ResourceAddress::try_from_bech32(&decoder, &DEFAULT_RESOURCE)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _REFERRAL_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("REFERRAL_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ResourceAddress::try_from_bech32(&decoder, &DEFAULT_RESOURCE)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _PROTOCOL_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("PROTOCOL_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ResourceAddress::try_from_bech32(&decoder, &DEFAULT_RESOURCE)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _KEEPER_REWARD_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("KEEPER_REWARD_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ResourceAddress::try_from_bech32(&decoder, &DEFAULT_RESOURCE)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _FEE_OATH_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("_FEE_OATH_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ResourceAddress::try_from_bech32(&decoder, &DEFAULT_RESOURCE)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _ORACLE_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic({:?});", 
        env::var("ORACLE_COMPONENT").map(|var| {
            ComponentAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ComponentAddress::try_from_bech32(&decoder, &DEFAULT_COMPONENT)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _CONFIG_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic({:?});", 
        env::var("CONFIG_COMPONENT").map(|var| {
            ComponentAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ComponentAddress::try_from_bech32(&decoder, &DEFAULT_COMPONENT)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _POOL_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic({:?});", 
        env::var("POOL_COMPONENT").map(|var| {
            ComponentAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ComponentAddress::try_from_bech32(&decoder, &DEFAULT_COMPONENT)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _REFERRAL_GENERATOR_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic({:?});", 
        env::var("REFERRAL_GENERATOR_COMPONENT").map(|var| {
            ComponentAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ComponentAddress::try_from_bech32(&decoder, &DEFAULT_COMPONENT)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _FEE_DISTRIBUTOR_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic({:?});", 
        env::var("FEE_DISTRIBUTOR_COMPONENT").map(|var| {
            ComponentAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ComponentAddress::try_from_bech32(&decoder, &DEFAULT_COMPONENT)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _FEE_DELEGATOR_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic({:?});", 
        env::var("FEE_DELEGATOR_COMPONENT").map(|var| {
            ComponentAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ComponentAddress::try_from_bech32(&decoder, &DEFAULT_COMPONENT)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _PERMISSION_REGISTRY_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic({:?});", 
        env::var("PERMISSION_REGISTRY_COMPONENT").map(|var| {
            ComponentAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ComponentAddress::try_from_bech32(&decoder, &DEFAULT_COMPONENT)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();
}
