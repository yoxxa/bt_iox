use time::OffsetDateTime;
use deku::prelude::*;

// Used for setting packet parameters
fn tenths_time(time: u8) -> u8 { (time / 10) % 10 }
fn oneths_time(time: u8) -> u8 { time % 10 }

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
struct IncomingMessageProtocol {
    // Header fields
    signature: u8,
    #[deku(bits = 6)]
    source_identifier_type: u8,
    // Payload fields
    #[deku(bits = 1)]
    has_ibeacon: u8,
    #[deku(bits = 1)]
    padding1: u8,
    low_byte_asset_number: u8,
    high_byte_asset_number: u8,
    source_identifier: [u8; 12],
    #[deku(bits = 1)]
    padding2: u8,
    // Timestamp fields - ten_x is tenths of that value
    #[deku(bits = 3)]    
    ten_seconds: u8,
    #[deku(bits = 4)]
    seconds: u8,
    #[deku(bits = 1)]
    padding3: u8,
    #[deku(bits = 3)]
    ten_minutes: u8,
    #[deku(bits = 4)]
    minutes: u8,
    #[deku(bits = 2)]
    padding4: u8,
    #[deku(bits = 2)]
    ten_hour: u8,
    #[deku(bits = 4)]
    hours: u8,
    #[deku(bits = 2)]
    padding5: u8,
    #[deku(bits = 2)]
    ten_date: u8,
    #[deku(bits = 4)]
    date: u8,
    #[deku(bits = 1)]
    is_utc: u8,
    #[deku(bits = 2)]
    padding6: u8,
    #[deku(bits = 1)]
    ten_month: u8,
    #[deku(bits = 4)]
    month: u8,
    #[deku(bits = 4)]
    ten_year: u8,
    #[deku(bits = 4)]
    year: u8,
}

impl IncomingMessageProtocol {
    pub fn new(mac_address: [u8; 12], timestamp: OffsetDateTime) -> IncomingMessageProtocol {
        // Use these defaults for now, update later
        Self {
            signature: 0xEE,
            source_identifier_type: 60,
            has_ibeacon: 0,
            padding1: 0,
            low_byte_asset_number: 0xFF,
            high_byte_asset_number: 0x00,
            source_identifier: mac_address,
            padding2: 0,
            ten_seconds: tenths_time(timestamp.second()),
            seconds: oneths_time(timestamp.second()),
            padding3: 0,
            ten_minutes: tenths_time(timestamp.minute()),
            minutes: oneths_time(timestamp.minute()),
            padding4: 0,
            ten_hour: tenths_time(timestamp.hour()),
            hours: oneths_time(timestamp.hour()),
            padding5: 0,
            ten_date: tenths_time(timestamp.day()),
            date: oneths_time(timestamp.day()),
            is_utc: 1,
            padding6: 0,
            ten_month: tenths_time(timestamp.month().into()),
            month: oneths_time(timestamp.month().into()),
            ten_year: tenths_time((timestamp.year() - 2000).try_into().unwrap()),
            year: oneths_time((timestamp.year() - 2000).try_into().unwrap())
        }
    }
}