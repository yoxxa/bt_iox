use std::collections::HashMap;
use std::hash::RandomState;
use tracing::{instrument, error, debug};
use configparser::ini::Ini;

#[derive(Clone)]
pub struct Configuration {
    pub server_ip_address: String,
    pub server_port: u16,
    pub asset_number: u16,
}

impl Configuration {
    #[instrument]
    pub fn new() -> Configuration {
        let mut config = Ini::new();
        match config.load("/iox_data/package_config.ini") {
            Ok(config) => {
                debug!("Ok() received for .load() on type: Ini");
                config
            },
            Err(error) => { 
                error!("Err() received for .load() on type: Ini - {error}");
                panic!("Failed to load package_config.ini, check if fields are valid");
            }
        };
        // TODO - implement checking for these values instead of implicit .unwrap()
        Configuration {
            server_ip_address: config.get("networking", "server_ip_address").unwrap(),
            server_port: config.get("networking", "server_port").unwrap().parse::<u16>().unwrap(),
            asset_number: config.get("asset", "asset_number").unwrap().parse::<u16>().unwrap(),
        }
    }
}