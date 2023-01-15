use loempia::{Command, Driver, Error};
use std::path::Path;

fn main() -> Result<(), Error> {
    let path = Path::new("/dev/ttyACM0");

    let mut driver = Driver::open(path)?;

    driver.execute_command(Command::V)?;
    Ok(())
}
