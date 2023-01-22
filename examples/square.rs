use loempia::{Driver, Error, Plot};
use std::path::Path;

fn main() -> Result<(), Error> {
    let serial_path = Path::new("/dev/ttyACM0");
    let mut driver = Driver::open(serial_path)?;

    let path = vec![(0, 0), (1000, 0), (1000, 1000), (0, 1000), (0, 0)];
    let plot = Plot::from_path(path);

    driver.plot(&plot)?;
    Ok(())
}
