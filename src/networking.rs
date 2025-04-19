use time::OffsetDateTime;
use deku::prelude::*;
use std::{
    net::UdpSocket,
    thread, 
    time::Duration
};
use tracing::{error, debug, info};

use crate::config::Configuration;

// Used for setting packet parameters
fn tenths_time(time: u8) -> u8 { (time / 10) % 10 }
fn oneths_time(time: u8) -> u8 { time % 10 }

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct IncomingMessageProtocol {
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
    pub fn new(asset_number: u16, mac_address: [u8; 12], timestamp: OffsetDateTime) -> IncomingMessageProtocol {
        debug!("Construct IncomingMessageProtocol");
        // Creates array with [0] holding high byte, [1] holding low byte
        let asset_number= asset_number.to_be_bytes();
        // Use these defaults for now, update later
        Self {
            signature: 0xEE,
            source_identifier_type: 60,
            has_ibeacon: 0,
            padding1: 0,
            low_byte_asset_number: asset_number[1],
            high_byte_asset_number: asset_number[0],
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

    pub fn send_imp_v1(&self, socket: &UdpSocket) {
        match socket.send(&self.to_bytes().unwrap()) {
            Ok(size) => {
                let peer = &socket.peer_addr().unwrap();
                debug!("Ok() received for .send() on &UdpSocket - sent {size} bytes to {peer}");
                info!("Sent IncomingMessageProtocol packet to server");
            },
            Err(error) => {
                error!("Err() received for .send() on &UdpSocket - {error}");
            }
        };
    }
}

pub struct Heartbeat {
    config: Configuration
}

impl Heartbeat {
    pub fn new(config: Configuration) -> Heartbeat {
        Self {
            config
        }
    }

    fn heartbeat(&self, socket: &UdpSocket) {
        let packet = IncomingMessageProtocol::new(
            self.config.asset_number,
            *b"            ", 
            OffsetDateTime::now_utc()
        );
        packet.send_imp_v1(&socket);
    }

    pub fn run(&self) {
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
            self.heartbeat(&socket);
            thread::sleep(Duration::from_secs(15));
        }
    }
}