use std::io::{BufReader, BufRead};
use std::time::Duration;
use std::env;
use std::net::UdpSocket;

use time::OffsetDateTime;
use serialport;

use crate::{
    networking::IncomingMessageProtocol,
    config::Configuration
};

#[derive(Debug)]
struct Data {
    mac_address: String,
    timestamp: OffsetDateTime
}

pub struct UConnectS2B5232R {
    config: Configuration,
    port: Box<dyn serialport::SerialPort>,
    data: Option<Data>
}

impl UConnectS2B5232R {
    pub fn new(config: Configuration) -> UConnectS2B5232R {
        Self {
            config: config,
            port: serialport::new(
                env::var("IR_OTHER_SERIAL").unwrap_or("/dev/ttySerial1".into()), 
                115200
                )
                // TODO - figure out appropriate timeout
                .timeout(Duration::from_secs(600))
                .open().expect("Failed to open port"),
            data: None
        }
    }

    fn collect_data(&mut self) {
        let mut port = BufReader::new(&mut *self.port);
        let mut line = String::new();
        match port.read_line(&mut line) {
            Ok(_) => {
                // TODO - Use match statements in future iteration?
                if line != "\r\n"
                && line != "\\r\\n"
                && line != "" {
                    let collection: Vec<&str> = line.split(",").collect();
                    self.data = Some(Data {
                        mac_address: collection[3].to_string(),
                        timestamp: OffsetDateTime::now_utc()
                    });
                } 
            },
            Err(_) => {
                self.data = None;
            }
        }
    }

    fn send_data_to_server(&self, socket: &UdpSocket) {
        match &self.data {
            Some(data) => {
                let packet: IncomingMessageProtocol = IncomingMessageProtocol::new(
                    self.config.asset_number,
                data.mac_address.as_bytes().try_into().unwrap(),
                    data.timestamp
                );
                packet.send_imp_v1(&socket);
            },
            None => {}
        }
    }
    
    pub fn run(&mut self) {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind to address");
        socket.connect(format!("{}:{}", self.config.server_ip_address, self.config.server_port))
            .expect("connect function failed");
        loop {
            self.collect_data();
            self.send_data_to_server(&socket);
        }
    }
}