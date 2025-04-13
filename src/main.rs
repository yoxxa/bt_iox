use std::thread;

mod parani;
mod networking;
mod config;

fn main() {
    let config = config::Configuration::new();

    let mut parani = parani::ParaniSD1000::new(config);
    loop {
        parani.run();
    }
}