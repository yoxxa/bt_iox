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

const BT_CANCEL: &[u8; 12] = b"AT+BTCANCEL\r";
const BT_INQ: &[u8; 10] = b"AT+BTINQ?\r";
// S4=0 sets BT_INQ to anonymise device name, saving operations
const ATS_S4: &[u8; 7] = b"ATS4=0\r";
// S24=1000 sets max device scan amount to 1000
const ATS_S24: &[u8; 11] = b"ATS24=1000\r";
// S33=15 sets scan time to 15 seconds
const ATS_S33: &[u8; 9] = b"ATS33=15\r";

#[derive(Debug)]
struct BtInqData {
    mac_address: String,
    timestamp: OffsetDateTime
}

pub struct ParaniSD1000 {
    config: Configuration,
    port: Box<dyn serialport::SerialPort>,
    data: Vec<BtInqData>
}

impl ParaniSD1000 {
    pub fn new(config: Configuration) -> ParaniSD1000 {
        let port = match serialport::new(
            env::var("IR_PARANI_SERIAL").unwrap_or("/dev/ttySerial".into()), 
            57600
            )
            .timeout(Duration::from_secs(16))
            .open() {
                Ok(port) => {
                    debug!("Ok() received for .open() on SerialPortBuilder");
                    info!("Success opening ParaniSD1000 port");
                    port
                },
                Err(error) => {
                    debug!("Err() received for .open() on SerialPortBuilder - {error}");
                    error!("Error opening ParaniSD1000 port");
                    panic!("Cannot open port to ParaniSD1000");
                }
            };
        Self {
            config,
            port,
            data: Vec::new()
        }
    }

    fn bt_cancel(&mut self) {
        match self.port.write_all(BT_CANCEL) {
            Ok(_) => {
                debug!("Success writing BT_CANCEL");
            },
            Err(error) => {
                error!("BT_CANCEL write failure - {error}");
            }
        }
    }

    fn bt_inq(&mut self) {
        match self.port.write_all(BT_INQ) {
            Ok(_) => {
                debug!("Success writing BT_INQ");
            },
            Err(error) => {
                error!("BT_INQ write failure - {error}");
            }
        }
    }

    fn collect_data(&mut self) {
        // Remove all previous elements from Vec<BtInqData> so doesn't compound over time
        self.data.clear();
        let mut port = BufReader::new(&mut *self.port);
        loop {
            let mut line = String::new();
            match port.read_line(&mut line) {
                Ok(_) => {
                // TODO - Use match statements in future iteration?
                    if line != "OK\r\n" {
                        if line != "\r\n"
                        && line != "\\r\\n"
                        && line != ""
                        && line != "ERROR\\r\\n"
                        && line != "ERROR\r\n" {
                            let collection: Vec<&str> = line.split(",").collect();
                            self.data.push(BtInqData {
                                mac_address: collection[0].to_string(),
                                timestamp: OffsetDateTime::now_utc()
                            });
                            debug!("Data received for ParaniSD1000");
                        }
                    } else {
                        break;
                    }
                },
                // this should only be reached upon device or cable failure.
                Err(error) => {
                    debug!("Err() received for .read_line() on BufReader<&mut dyn SerialPort> - {error}");
                    error!("Error reading data from ParaniSD1000");
                    panic!("Error during data collection operation of ParaniSD1000, ensure ParaniSD1000 and cables are functional");
                }
            }
        }
    }

    fn ats_s4(&mut self) {
        match self.port.write_all(ATS_S4) {
            Ok(_) => {
                info!("Success writing S4 register");
            },
            Err(error) => {
                error!("ATS_S4 write error - {error}");
            }
        }
    }

    fn ats_s24(&mut self) {
        match self.port.write_all(ATS_S24) {
            Ok(_) => {
                info!("Success writing S24 register");
            },
            Err(error) => {
                error!("ATS_S24 write error - {error}");
            }
        }
    }

    fn ats_s33(&mut self) {
        match self.port.write_all(ATS_S33) {
            Ok(_) => {
                info!("Success writing S33 register");
            },
            Err(error) => {
                error!("ATS_S33 write error - {error}");
            }  
        }
    }

    fn set_s_registers(&mut self) {
        self.ats_s4();
        self.ats_s24();
        self.ats_s33();
        match self.port.clear(serialport::ClearBuffer::All) {
            Ok(_) => {},
            Err(error) => {
                error!("Failed to clear buffer - {error}")
            }
        }
    }

    fn send_data_to_server(&self, socket: &UdpSocket) {
        if self.data.is_empty() {
            debug!("No data to send");
        }
        // if there is no data in &self.data, for loop is skipped
        for data in &self.data {
            let packet: IncomingMessageProtocol = IncomingMessageProtocol::new(
                self.config.asset_number,
                data.mac_address.as_bytes().try_into().unwrap(),
                data.timestamp
            );
            packet.send_imp_v1(&socket);
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
        self.set_s_registers();
        loop {
            self.bt_cancel();
            self.bt_inq();
            self.collect_data();
            self.send_data_to_server(&socket);
        }
    }
}