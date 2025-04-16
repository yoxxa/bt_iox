use std::thread;

mod parani;
mod uconnect;
mod networking;
mod config;

fn main() {
    // TODO - use a single Configuration and pass to all threads?
    let config = config::Configuration::new();
    let config_parani = config.clone();
    let config_uconnect = config.clone();

    let heartbeat = thread::spawn( || {
        let heartbeat = networking::Heartbeat::new(config);
        loop {
            heartbeat.run();
        }
    });
    let parani = thread::spawn( || {
        let mut parani = parani::ParaniSD1000::new(config_parani);
        loop {
            parani.run();
        }
    });
    let uconnect = thread::spawn( || {
        let mut uconnect = uconnect::UConnectS2B5232R::new(config_uconnect);
        loop {
            uconnect.run();
        }
    });

    heartbeat.join().unwrap();
    parani.join().unwrap();
    uconnect.join().unwrap();
}