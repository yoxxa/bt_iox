use std::thread;
use tracing::{info, instrument};
use tracing_subscriber;

mod parani;
mod uconnect;
mod networking;
mod config;

#[instrument]
fn main() {
    // Configure a custom event formatter
    let format = tracing_subscriber::fmt::format()
        .with_level(true) // do include levels in formatted output
        .with_target(false) // do not include targets
        .with_thread_names(true) // include the name of the current thread
        .without_time() // do not include timestamps
        .json();
    tracing_subscriber::fmt()
        .event_format(format)
        .init();
    info!("Initialised logging");
    
    // TODO - use a single Configuration and pass to all threads?
    let config = config::Configuration::new();
    let config_parani = config.clone();
    let config_uconnect = config.clone();
    info!("Gathered configuration");

    let heartbeat = thread::Builder::new().name("heartbeat".to_string()).spawn( || {
        info!("Started heartbeat thread");
        let heartbeat = networking::Heartbeat::new(config);
        heartbeat.run();
    });
    let parani = thread::Builder::new().name("parani".to_string()).spawn( || {
        info!("Started parani thread");
        let mut parani = parani::ParaniSD1000::new(config_parani);
        parani.run();
    });
    let uconnect = thread::Builder::new().name("uconnect".to_string()).spawn( || {
        info!("Started uconnect thread");
        let mut uconnect = uconnect::UConnectS2B5232R::new(config_uconnect);
        uconnect.run();
    });

    heartbeat.unwrap().join().unwrap();
    parani.unwrap().join().unwrap();
    uconnect.unwrap().join().unwrap();
}