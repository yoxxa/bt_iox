use std::time::Duration;
use std::env;

use time::OffsetDateTime;
use serialport;

const BT_CANCEL: &[u8; 12] = b"AT+BTCANCEL\r";
const BT_INQ: &[u8; 10] = b"AT+BTINQ?\r";

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
}