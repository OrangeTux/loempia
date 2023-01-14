use loempia::{Command, Driver};
use std::path::Path;

fn main() {
    let path = Path::new("/dev/ttyACM0");

    let mut driver = Driver::open(path).expect("Failed to open driver");

    driver
        .execute_command(Command::SM {
            duration: 1000,
            axis_step_1: 1000,
            axis_step_2: Some(-1000),
        })
        .unwrap();
    driver
        .execute_command(Command::SM {
            duration: 1000,
            axis_step_1: 1000,
            axis_step_2: Some(1000),
        })
        .unwrap();
    driver
        .execute_command(Command::SM {
            duration: 1000,
            axis_step_1: -1000,
            axis_step_2: Some(1000),
        })
        .unwrap();

    driver
        .execute_command(Command::SM {
            duration: 1000,
            axis_step_1: -1000,
            axis_step_2: Some(-1000),
        })
        .unwrap();
}
