use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use scrypto::prelude::*;

fn main() {
    let default_resource: ResourceAddress = ResourceAddress::try_from_hex("5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6").unwrap();
    let default_component: ComponentAddress = ComponentAddress::try_from_hex("c169a00e3637d04099d059cb22912aaae58f08cdaf03139a3e10f40ac8cd").unwrap();

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
            ResourceAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_resource
        }).into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _BASE_AUTHORITY_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("BASE_AUTHORITY_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_resource
        }).into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _BASE_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("BASE_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_resource
        }).into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _LP_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("LP_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_resource
        }).into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _REFERRAL_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("REFERRAL_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_resource
        }).into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _PROTOCOL_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("PROTOCOL_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_resource
        }).into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _KEEPER_REWARD_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("KEEPER_REWARD_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_resource
        }).into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _FEE_OATH_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("_FEE_OATH_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_resource
        }).into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _ORACLE_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic({:?});", 
        env::var("ORACLE_COMPONENT").map(|var| {
            ComponentAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_component
        }).into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _CONFIG_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic({:?});", 
        env::var("CONFIG_COMPONENT").map(|var| {
            ComponentAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_component
        }).into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _POOL_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic({:?});", 
        env::var("POOL_COMPONENT").map(|var| {
            ComponentAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_component
        }).into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _REFERRAL_GENERATOR_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic({:?});", 
        env::var("REFERRAL_GENERATOR_COMPONENT").map(|var| {
            ComponentAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_component
        }).into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _FEE_DISTRIBUTOR_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic({:?});", 
        env::var("FEE_DISTRIBUTOR_COMPONENT").map(|var| {
            ComponentAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_component
        }).into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _FEE_DELEGATOR_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic({:?});", 
        env::var("FEE_DELEGATOR_COMPONENT").map(|var| {
            ComponentAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_component
        }).into_node_id().to_vec()
    ).unwrap();

    writeln!(f, "pub const _PERMISSION_REGISTRY_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic({:?});", 
        env::var("PERMISSION_REGISTRY_COMPONENT").map(|var| {
            ComponentAddress::try_from_bech32(&decoder, &var).unwrap()
        }).unwrap_or_else(|_| {
            default_component
        }).into_node_id().to_vec()
    ).unwrap();
}
