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

    let heartbeat = thread::Builder::new().name("heartbeat".to_string()).spawn( || {
        let heartbeat = networking::Heartbeat::new(config);
        loop {
            heartbeat.run();
        }
    });
    let parani = thread::Builder::new().name("parani".to_string()).spawn( || {
        let mut parani = parani::ParaniSD1000::new(config_parani);
        loop {
            parani.run();
        }
    });
    let uconnect = thread::Builder::new().name("uconnect".to_string()).spawn( || {
        let mut uconnect = uconnect::UConnectS2B5232R::new(config_uconnect);
        loop {
            uconnect.run();
        }
    });

    heartbeat.unwrap().join().unwrap();
    parani.unwrap().join().unwrap();
    uconnect.unwrap().join().unwrap();
}