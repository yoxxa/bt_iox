use configparser::ini::Ini;

pub struct Configuration {
    server_ip_address: String,
    server_port: u64,
    logging_type: String,
    syslog_server_ip_address: String 
}

impl Configuration {
    pub fn new() -> Configuration {
        let mut config = Ini::new();
        config.load("/iox_data/package_config.ini").unwrap();
    
        Configuration {
            server_ip_address: config.get("networking", "server_ip_address").unwrap(),
            server_port: config.getuint("networking", "server_port").unwrap().unwrap(),
            logging_type: config.get("logging", "logging_type").unwrap(),
            syslog_server_ip_address: config.get("logging", "syslog_server_ip_address").unwrap()
        }
    }
}