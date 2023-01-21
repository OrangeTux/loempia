use loempia::{Driver, Error, Plot};
use std::path::Path;

fn main() -> Result<(), Error> {
    let serial_path = Path::new("/dev/ttyACM0");
    let mut driver = Driver::open(serial_path)?;

    let path = vec![
        (1000, 1000),
        (2000, 0000),
        (3000, 1000),
        (4000, 0000),
        (0000, 0000),
        (2000, 2000),
        (3000, 1000),
        (1000, 1000),
    ];
    let plot = Plot::from_path(path);

    driver.plot(&plot)?;
    Ok(())
}
