use std::io::{BufReader, BufRead};
use std::time::Duration;
use std::env;
use std::net::UdpSocket;
use tracing::{error, debug, info};
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
        let port = match serialport::new(
            env::var("IR_OTHER_SERIAL").unwrap_or("/dev/ttySerial1".into()), 
            115200
            )
            // TODO - figure out appropriate timeout
            .timeout(Duration::from_secs(600))
            .open() {
                Ok(port) => {
                    debug!("Ok() received for .open() on SerialPortBuilder");
                    info!("Success opening UConnectS2B5232R port");
                    port
                },
                Err(error) => {
                    debug!("Err() received for .open() on SerialPortBuilder - {error}");
                    error!("Error opening UConnectS2B5232R port");
                    panic!("Cannot open port to UConnectS2B5232R");
                }
            };
        Self {
            config,
            port,
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
                    debug!("Data received from UConnectS2B5232R");
                } 
            },
            // indicates timeout has been reached
            Err(_) => {
                debug!("No data received from UConnectS2B5232R within timeout period");
                self.data = None;
            }
        }
    }

    fn send_data_to_server(&self, socket: &UdpSocket) {
        match &self.data {
            Some(data) => {
                let mac_address = data.mac_address.as_bytes();
                // check for if is valid MAC address length
                if mac_address.len() == 12 {
                    let packet: IncomingMessageProtocol = IncomingMessageProtocol::new(
                        self.config.asset_number,
                        mac_address.try_into().unwrap(),
                        data.timestamp
                    );
                    packet.send_imp_v1(&socket);
                }
            },
            None => {
                debug!("No data to send");
            }
        }
    }
    
    pub fn run(&mut self) {
        let socket = match UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => {
                debug!("Ok() received for UdpSocket::bind");
                info!("Success binding UdpSocket");
                socket
            },
            Err(error) => {
                error!("Err() received for UdpSocket::bind - {error}");
                panic!("Failed to bind UdpSocket");
            }
        };
        match socket.connect(format!("{}:{}", self.config.server_ip_address, self.config.server_port)) {
            Ok(_) => {
                debug!("Ok() received for .connect() on UdpSocket");
                info!("Success connect UdpSocket to server");
            },
            Err(error) => { 
                error!("Err() received for .connect() on UdpSocket - {error}");
                panic!("Failed to connect UdpSocket to server"); 
            }
        };
        loop {
            self.collect_data();
            self.send_data_to_server(&socket);
        }
    }
}