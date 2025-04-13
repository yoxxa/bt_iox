use std::thread;

mod parani;

fn main() {

    let mut parani = parani::ParaniSD1000::new();
    loop {
        parani.run();
    }

}