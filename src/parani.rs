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
        Self {
            config: config,
            port: serialport::new(
                env::var("IR_PARANI_SERIAL").unwrap_or("/dev/ttySerial".into()), 
                57600
                )
                .timeout(Duration::from_secs(16))
                .open().expect("Failed to open port"),
            data: Vec::new()
        }
    }

    fn bt_cancel(&mut self) {
        self.port.write_all(BT_CANCEL).expect("BT_CANCEL write failure");
    }

    fn bt_inq(&mut self) {
        self.port.write_all(BT_INQ).expect("BT_INQ write error");
    }

    fn collect_data(&mut self) {
        // Remove all previous elements from Vec<BtInqData> so doesn't compound over time
        self.data.clear();
        let mut port = BufReader::new(&mut *self.port);
        loop {
            let mut line = String::new();
            // TODO - Handle timeout error, will occur once .timeout() duration is reached. Use match.
            port.read_line(&mut line).expect("Error reading");
    
            // TODO - Use match statements in future iteration?
            if line != "OK\r\n" {
                if line != "\r\n"
                && line != "\\r\\n"
                && line != ""
                && line != "ERROR\\r\\n"
                && line != "ERROR\r\n" {
                    let parts = line.split(",");
                    let collection: Vec<&str> = parts.collect();
                    
                    self.data.push(BtInqData {
                        mac_address: collection[0].to_string(),
                        timestamp: OffsetDateTime::now_utc()
                    });
                }
            } else {
                break;
            }
        }
    }

    fn ats_s4(&mut self) {
        self.port.write_all(ATS_S4).expect("ATS_S4 write error");
    }

    fn ats_s24(&mut self) {
        self.port.write_all(ATS_S24).expect("ATS_S24 write error");
    }

    fn ats_s33(&mut self) {
        self.port.write_all(ATS_S33).expect("ATS_S33 write error");
    }

    fn set_s_registers(&mut self) {
        self.ats_s4();
        self.ats_s24();
        self.ats_s33();
        self.port.clear(serialport::ClearBuffer::All)
        .expect("Failed to clear buffer");
    }

    fn send_data_to_server(&self, socket: &UdpSocket) {
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
        let socket = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind to address");
        socket.connect(format!("{}:{}", self.config.server_ip_address, self.config.server_port))
            .expect("connect function failed");
        self.set_s_registers();
        loop {
            self.bt_cancel();
            self.bt_inq();
            self.collect_data();
            self.send_data_to_server(&socket);
        }
    }
}