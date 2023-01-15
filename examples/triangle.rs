use loempia::{plot_points, Driver, Error, Point};
use std::path::Path;

fn main() -> Result<(), Error> {
    let scale = 1000;
    let track: Vec<Point> = vec![
        (1, 1),
        (2, 0),
        (3, 1),
        (4, 0),
        (0, 0),
        (2, 2),
        (3, 1),
        (1, 1),
    ];

    let track: Vec<Point> = track.iter().map(|(x, y)| (x * scale, y * scale)).collect();

    let path = Path::new("/dev/ttyACM0");

    let mut driver = Driver::open(path)?;
    plot_points(&mut driver, track)?;
    Ok(())
}
