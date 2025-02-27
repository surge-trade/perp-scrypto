use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use scrypto::prelude::*;

fn write_resource(var_name: &str, const_name: &str, decoder: &AddressBech32Decoder, file: &mut File) {
    let (var, resource) = env::var(var_name).map(|var| {
        let resource = ResourceAddress::try_from_bech32(&decoder, &var).unwrap();
        (Some(var), resource)
    }).unwrap_or_else(|_| {
        let resource = ResourceAddress::try_from_hex("5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6").unwrap();
        (None, resource)
    });
    writeln!(file, "pub const {}: ResourceAddress = ResourceAddress::new_or_panic({:?});", const_name, resource.into_node_id().to_vec()).unwrap();
    
    if let Some(var) = var {
        println!("cargo:warning={}: {}", var_name, var);
    }
}

fn write_package(var_name: &str, const_name: &str, decoder: &AddressBech32Decoder, file: &mut File) {
    let (var, package) = env::var(var_name).map(|var| {
        let package = PackageAddress::try_from_bech32(&decoder, &var).unwrap();
        (Some(var), package)
    }).unwrap_or_else(|_| {
        let package = PackageAddress::try_from_hex("0d89f83cb8550e3ec06cee7272b67d0855beff3a66bfbbfb33a127b5cfac").unwrap();
        (None, package)
    });
    writeln!(file, "pub const {}: PackageAddress = PackageAddress::new_or_panic({:?});", const_name, package.into_node_id().to_vec()).unwrap();
    
    if let Some(var) = var {
        println!("cargo:warning={}: {}", var_name, var);
    }
}

fn write_component(var_name: &str, const_name: &str, decoder: &AddressBech32Decoder, file: &mut File) {
    let (var, component) = env::var(var_name).map(|var| {
        let component = ComponentAddress::try_from_bech32(&decoder, &var).unwrap();
        (Some(var), component)
    }).unwrap_or_else(|_| {
        let component = ComponentAddress::try_from_hex("c169a00e3637d04099d059cb22912aaae58f08cdaf03139a3e10f40ac8cd").unwrap();
        (None, component)
    });
    writeln!(file, "pub const {}: ComponentAddress = ComponentAddress::new_or_panic({:?});", const_name, component.into_node_id().to_vec()).unwrap();
    
    if let Some(var) = var {
        println!("cargo:warning={}: {}", var_name, var);
    }
}

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
    write_resource("AUTHORITY_RESOURCE", "_AUTHORITY_RESOURCE", &decoder, &mut f);
    write_resource("BASE_AUTHORITY_RESOURCE", "_BASE_AUTHORITY_RESOURCE", &decoder, &mut f);
    write_resource("BASE_RESOURCE", "_BASE_RESOURCE", &decoder, &mut f);
    write_resource("LP_RESOURCE", "_LP_RESOURCE", &decoder, &mut f);
    write_resource("REFERRAL_RESOURCE", "_REFERRAL_RESOURCE", &decoder, &mut f);
    write_resource("RECOVERY_KEY_RESOURCE", "_RECOVERY_KEY_RESOURCE", &decoder, &mut f);
    write_resource("PROTOCOL_RESOURCE", "_PROTOCOL_RESOURCE", &decoder, &mut f);
    write_resource("KEEPER_REWARD_RESOURCE", "_KEEPER_REWARD_RESOURCE", &decoder, &mut f);
    write_resource("FEE_OATH_RESOURCE", "_FEE_OATH_RESOURCE", &decoder, &mut f);

    write_package("ORACLE_PACKAGE", "ORACLE_PACKAGE", &decoder, &mut f);
    write_package("CONFIG_PACKAGE", "CONFIG_PACKAGE", &decoder, &mut f);
    write_package("ACCOUNT_PACKAGE", "MARGIN_ACCOUNT_PACKAGE", &decoder, &mut f);
    write_package("POOL_PACKAGE", "MARGIN_POOL_PACKAGE", &decoder, &mut f);
    write_package("REFERRAL_GENERATOR_PACKAGE", "REFERRAL_GENERATOR_PACKAGE", &decoder, &mut f);
    write_package("FEE_DISTRIBUTOR_PACKAGE", "FEE_DISTRIBUTOR_PACKAGE", &decoder, &mut f);
    write_package("FEE_DELEGATOR_PACKAGE", "FEE_DELEGATOR_PACKAGE", &decoder, &mut f);
    write_package("PERMISSION_REGISTRY_PACKAGE", "PERMISSION_REGISTRY_PACKAGE", &decoder, &mut f);
    write_package("TOKEN_WRAPPER_PACKAGE", "TOKEN_WRAPPER_PACKAGE", &decoder, &mut f);

    write_component("ORACLE_COMPONENT", "_ORACLE_COMPONENT", &decoder, &mut f);
    write_component("CONFIG_COMPONENT", "_CONFIG_COMPONENT", &decoder, &mut f);
    write_component("POOL_COMPONENT", "_POOL_COMPONENT", &decoder, &mut f);
    write_component("REFERRAL_GENERATOR_COMPONENT", "_REFERRAL_GENERATOR_COMPONENT", &decoder, &mut f);
    write_component("FEE_DISTRIBUTOR_COMPONENT", "_FEE_DISTRIBUTOR_COMPONENT", &decoder, &mut f);
    write_component("FEE_DELEGATOR_COMPONENT", "_FEE_DELEGATOR_COMPONENT", &decoder, &mut f);
    write_component("PERMISSION_REGISTRY_COMPONENT", "_PERMISSION_REGISTRY_COMPONENT", &decoder, &mut f);
    write_component("TOKEN_WRAPPER_COMPONENT", "_TOKEN_WRAPPER_COMPONENT", &decoder, &mut f);
}
