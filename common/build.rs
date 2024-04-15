use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use scrypto::prelude::*;

const DEFAULT_RESOURCE: &str = "resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3";

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

    writeln!(f, "pub const _KEEPER_REWARD_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic({:?});", 
        env::var("KEEPER_REWARD_RESOURCE").map(|var| {
            ResourceAddress::try_from_bech32(&decoder, &var)
        }).unwrap_or_else(|_| {
            ResourceAddress::try_from_bech32(&decoder, &DEFAULT_RESOURCE)
        }).unwrap().into_node_id().to_vec()
    ).unwrap();
}