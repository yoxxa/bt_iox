use std::thread;

mod parani;
mod networking;
mod config;

fn main() {
    let config = config::Configuration::new();

    let parani = thread::spawn( || {
        let mut parani = parani::ParaniSD1000::new(config);
        loop {
            parani.run();
        }
    });
    parani.join().unwrap();
}