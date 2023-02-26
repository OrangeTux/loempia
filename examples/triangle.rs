use loempia::point::Coordinate;
use loempia::{Driver, Error, Plot};
use std::path::Path;

fn main() -> Result<(), Error> {
    let serial_path = Path::new("/dev/ttyACM0");
    let mut driver = Driver::open(serial_path)?;

    let path = vec![
        Coordinate::new(1000, 1000),
        Coordinate::new(2000, 0000),
        Coordinate::new(3000, 1000),
        Coordinate::new(4000, 0000),
        Coordinate::new(0000, 0000),
        Coordinate::new(2000, 2000),
        Coordinate::new(3000, 1000),
        Coordinate::new(1000, 1000),
    ];
    let plot = {
        let path = path;
        let paths = loempia::Paths::new(vec![path])?;
        Plot::new(paths)
    };

    driver.plot(&plot)?;
    Ok(())
}
