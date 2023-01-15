use loempia::{Command, Driver, Error};
use std::path::Path;

fn main() -> Result<(), Error> {
    let path = Path::new("/dev/ttyACM0");

    let mut driver = Driver::open(path)?;

    driver.execute_command(Command::Any("SP,0".to_string()))?;
    driver.execute_command(Command::SM {
        duration: 1000,
        axis_step_1: 1000,
        axis_step_2: Some(-1000),
    })?;
    driver.execute_command(Command::SM {
        duration: 1000,
        axis_step_1: 1000,
        axis_step_2: Some(1000),
    })?;
    driver.execute_command(Command::SM {
        duration: 1000,
        axis_step_1: -1000,
        axis_step_2: Some(1000),
    })?;

    driver.execute_command(Command::SM {
        duration: 1000,
        axis_step_1: -1000,
        axis_step_2: Some(-1000),
    })?;
    driver.execute_command(Command::Any("SP,1".to_string()))?;
    Ok(())
}
