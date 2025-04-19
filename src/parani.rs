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
                Ok(port) => port,
                Err(_) => panic!("Cannot open Parani device")
            };
        Self {
            config,
            port,
            data: Vec::new()
        }
    }

    fn bt_cancel(&mut self) {
        match self.port.write_all(BT_CANCEL) {
            Ok(_) => {},
            Err(error) => {
                eprintln!("BT_CANCEL write failure");
            }
        }
    }

    fn bt_inq(&mut self) {
        match self.port.write_all(BT_INQ) {
            Ok(_) => {},
            Err(error) => {
                eprintln!("BT_INQ write error");
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
                        }
                    } else {
                        break;
                    }
                },
                // this should really only be reached upon device or cable failure.
                // TODO - write handler for this error case
                Err(error) => {}
            }
        }
    }

    fn ats_s4(&mut self) {
        match self.port.write_all(ATS_S4) {
            Ok(_) => {},
            Err(error) => {
                eprintln!("ATS_S4 write error");
            }
        }
    }

    fn ats_s24(&mut self) {
        match self.port.write_all(ATS_S24) {
            Ok(_) => {},
            Err(error) => {
                eprintln!("ATS_S24 write error");
            }
        }
    }

    fn ats_s33(&mut self) {
        match self.port.write_all(ATS_S33) {
            Ok(_) => {},
            Err(error) => {
                eprintln!("ATS_S33 write error");
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
                eprintln!("Failed to clear buffer")
            }
        }
    }

    fn send_data_to_server(&self, socket: &UdpSocket) {
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
            Ok(socket) => socket,
            Err(error) => todo!("error handling for failed socket bind")
        };
        match socket.connect(format!("{}:{}", self.config.server_ip_address, self.config.server_port)) {
            Ok(_) => {},
            Err(error) => { eprintln!("connect function failed"); }
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