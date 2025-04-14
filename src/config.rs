use configparser::ini::Ini;

pub struct Configuration {
    pub server_ip_address: String,
    pub server_port: u16,
    pub parani_asset_number: u16,
    pub logging_type: String,
    pub syslog_server_ip_address: String 
}

impl Configuration {
    pub fn new() -> Configuration {
        let mut config = Ini::new();
        config.load("/iox_data/package_config.ini").unwrap();
    
        Configuration {
            server_ip_address: config.get("networking", "server_ip_address").unwrap(),
            server_port: config.get("networking", "server_port").unwrap().parse::<u16>().unwrap(),
            parani_asset_number: config.get("parani", "asset_number").unwrap().parse::<u16>().unwrap(),
            logging_type: config.get("logging", "logging_type").unwrap(),
            syslog_server_ip_address: config.get("logging", "syslog_server_ip_address").unwrap()
        }
    }
}