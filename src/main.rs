use std::thread;

mod parani;
mod networking;
mod config;

fn main() {
    // TODO - use a single Configuration and pass to all threads?
    let config = config::Configuration::new();
    let config_parani = config.clone();

    let heartbeat = thread::spawn( || {
        let heartbeat = networking::Heartbeat::new(config);
        heartbeat.run();
    });
    let parani = thread::spawn( || {
        let mut parani = parani::ParaniSD1000::new(config_parani);
        loop {
            parani.run();
        }
    });

    heartbeat.join().unwrap();
    parani.join().unwrap();
}