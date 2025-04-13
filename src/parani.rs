use std::time::Duration;
use std::env;

use time::OffsetDateTime;
use serialport;

const BT_CANCEL: &[u8; 12] = b"AT+BTCANCEL\r";
const BT_INQ: &[u8; 10] = b"AT+BTINQ?\r";

#[derive(Debug)]
struct BtInqData {
    mac_address: String,
    timestamp: OffsetDateTime
}

struct ParaniSD1000 {
    port: Box<dyn serialport::SerialPort>,
    data: Vec<BtInqData>
}

impl ParaniSD1000 {
    fn new() -> ParaniSD1000 {
        Self {
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
        let mut port = BufReader::new(&mut *self.port);
        loop {
            let mut serial_buf = String::new();
            // TODO - Handle timeout error, will occur once .timeout() duration is reached. Use match.
            port.read_line(&mut serial_buf).expect("Error reading");
    
            // TODO - Use match statements in future iteration?
            if serial_buf != "OK\r\n" {
                if serial_buf != "\r\n"
                && serial_buf != "\\r\\n"
                && serial_buf != ""
                && serial_buf != "ERROR\\r\\n"
                && serial_buf != "ERROR\r\n" {
                    let parts = serial_buf.split(",");
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
}