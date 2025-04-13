use std::time::Duration;
use std::env;

use time::OffsetDateTime;
use serialport;

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
}