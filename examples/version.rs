use loempia::{Command, Driver};
use std::path::Path;

fn main() {
    let path = Path::new("/dev/ttyACM0");

    let mut driver = Driver::open(path).expect("Failed to open driver");

    driver.execute_command(Command::V).unwrap();
}
