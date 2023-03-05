use loempia::roland_dxy::{Driver, default_port_settings};
use loempia::{point::Coordinate, Error, Plot};
use std::path::Path;

fn main() -> Result<(), Error> {
    let serial_path = Path::new("/dev/ttyUSB0");
    let mut driver = Driver::open(serial_path, default_port_settings())?;

    let path = vec![
        Coordinate::new(0, 0),
        Coordinate::new(1000, 0),
        Coordinate::new(1000, 1000),
        Coordinate::new(0, 1000),
        Coordinate::new(0, 0),
    ];
    let plot = Plot::from_path(path)?;

    driver.plot(&plot)?;
    Ok(())
}
