use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use regex::Regex;
use scrypto::prelude::*;

pub const DEFAULT_PACKAGE: &str = "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9";

fn main() {
    // Load the file
    let file_path = Path::new("src/exchange.rs");
    let mut data = fs::read_to_string(file_path).unwrap();
    let package_regex = Regex::new(r"package[_a-z0-9]+").unwrap();

    // Fetch the addresses from the environment
    let packages = vec![
        env::var("CONFIG_PACKAGE").unwrap_or(DEFAULT_PACKAGE.to_string()),
        env::var("ACCOUNT_PACKAGE").unwrap_or(DEFAULT_PACKAGE.to_string()),
        env::var("POOL_PACKAGE").unwrap_or(DEFAULT_PACKAGE.to_string()),
        env::var("ORACLE_PACKAGE").unwrap_or(DEFAULT_PACKAGE.to_string()),
        env::var("REFERRALS_PACKAGE").unwrap_or(DEFAULT_PACKAGE.to_string()),
        env::var("FEE_DELEGATOR_PACKAGE").unwrap_or(DEFAULT_PACKAGE.to_string()),
    ];

    // Find all matches and replace them sequentially with the new addresses
    let matches: Vec<_> = package_regex.find_iter(&data).collect();
    let mut new_data = String::new();
    let mut last_end = 0;
    for (i, mat) in matches.iter().enumerate() {
        if i < packages.len() {
            new_data.push_str(&data[last_end..mat.start()]);
            new_data.push_str(&packages[i]);
            last_end = mat.end();
        } else {
            break;
        }
    }
    new_data.push_str(&data[last_end..]);
    data = new_data;

    // Write the changes back to the file
    let mut file = File::create(file_path).unwrap();
    file.write_all(data.as_bytes()).unwrap();
}