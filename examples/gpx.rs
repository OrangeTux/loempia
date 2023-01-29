use loempia::{Driver, Error, Plot};
use roxmltree::{Document, Node};
use std::{env, fs, path};

type Point = (f32, f32);
type Path = Vec<Point>;

/// Build a collection of `Path`s from a "trk" element.
/// A `Path` is build for every "trkseg" child.
///
/// <trk>
///     <trkseg>
///         ...
///     </trkseg>
///     <trkseg>
///         ...
///     </trkseg>
///     ..
/// </trk>
fn track_to_paths(node: &Node) -> Vec<Path> {
    node.children()
        .filter_map(|child| {
            if !child.has_tag_name("trkseg") {
                return None;
            }
            Some(track_segment_to_path(&child))
        })
        .collect()
}

/// Build a Path from the "trkpt" elements inside an "trgseg" element.
///
/// <trksg>
///     <trkpt lat="1" lon="2"></trkpt>
///     <trkpt lat="2" lon="3"></trkpt>
///     ..
/// </trksg>
fn track_segment_to_path(node: &Node) -> Path {
    return node
        .children()
        .filter_map(|child| {
            if !child.has_tag_name("trkpt") {
                return None;
            }

            let lat: f32 = child
                .attribute("lat")
                .unwrap_or_else(|| {
                    panic!(
                        "Element <trkpt> at line {} is missing attribute \"lat\".",
                        child.document().text_pos_at(child.position())
                    )
                })
                .parse()
                .unwrap_or_else(|_| {
                    panic!(
                        "Failed to parse attribute \"lat\" at line {} as float.",
                        child.document().text_pos_at(child.position())
                    )
                });
            let lon: f32 = child
                .attribute("lon")
                .unwrap_or_else(|| {
                    panic!(
                        "Element <trkseg> at line {} is missing attribute \"lon\".",
                        child.document().text_pos_at(child.position())
                    )
                })
                .parse()
                .unwrap_or_else(|_| {
                    panic!(
                        "Failed to parse attribute \"lon\" at line {} as float.",
                        child.document().text_pos_at(child.position())
                    )
                });

            Some((lat, lon))
        })
        .collect();
}

/// Get the four boundaries of the `Path`s, meaning the minimum and maximum latitude and longitude.
fn get_boundaries(paths: &Vec<Path>) -> (f32, f32, f32, f32) {
    let (mut min_lat, mut min_lon, mut max_lat, mut max_lon) = (None, None, None, None);

    paths.iter().for_each(|path| {
        path.iter().for_each(|(lat, lon)| {
            _ = min_lat.get_or_insert(lat);
            _ = max_lat.get_or_insert(lat);

            // `unwrap()` is safe here.
            if lat < min_lat.unwrap() {
                min_lat.replace(lat);
            }

            if lat > max_lat.unwrap() {
                max_lat.replace(lat);
            }

            _ = min_lon.get_or_insert(lon);
            _ = max_lon.get_or_insert(lon);

            if lon < min_lon.unwrap() {
                min_lon.replace(lon);
            }

            if lon > max_lon.unwrap() {
                max_lon.replace(lon);
            }
        });
    });

    (
        *min_lat.unwrap(),
        *min_lon.unwrap(),
        *max_lat.unwrap(),
        *max_lon.unwrap(),
    )
}

/// Adjust every point by the given latitude and longitude.
fn adjust(paths: Vec<Path>, adjustment: (f32, f32)) -> Vec<Path> {
    let (lat_adjustment, lon_adjustment) = adjustment;
    paths
        .iter()
        .map(|path| {
            path.iter()
                .map(|(lat, lon)| (lat + lat_adjustment, lon + lon_adjustment))
                .collect()
        })
        .collect()
}

/// Multiply every point by the given scale.
fn scale(paths: Vec<Path>, factor: f32) -> Vec<Path> {
    paths
        .iter()
        .map(|path| {
            path.iter()
                .map(|(lat, lon)| (lat * factor, lon * factor))
                .collect()
        })
        .collect()
}

/// Build `loempia::Path` from `Path`s build from `f32.
fn to_paths(paths: Vec<Path>) -> loempia::Paths {
    let paths = paths
        .iter()
        .map(|path| {
            path.iter()
                // This conversion panic when `lat` or `lon` are out of the bounds for `i32`.
                // However, that seems unlikely, given valid values for latitude range from -90 to
                // 90. While longitude ranges from -180 to 180.
                .map(|(lat, lon)| (*lat as i32, *lon as i32))
                .collect()
        })
        .collect();

    loempia::Paths(paths)
}

fn down_size(paths: Vec<Path>, resolution: usize) -> Vec<Path> {
    paths
        .iter()
        .map(|path| {
            path.iter()
                .step_by(resolution)
                .map(|(lat, lon)| (*lat, *lon))
                .collect()
        })
        .collect()
}

fn main() -> Result<(), Error> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage:\n\tcargo run --example gpx -- input.gpx [resolution]");
        std::process::exit(1);
    }

    let text = fs::read_to_string(&args[1]).unwrap();
    let resolution: usize = args
        .get(2)
        .unwrap_or(&String::from("1"))
        .parse()
        .expect("Failed to parse resolution into a usize.");
    let doc = Document::parse(&text).expect("Failed to parse given file as GPX.");

    let paths: Vec<_> = doc
        .descendants()
        .filter_map(|node| {
            if !node.has_tag_name("trk") {
                return None;
            };
            Some(track_to_paths(&node))
        })
        .fold(vec![], |mut acc, x| {
            acc.extend(x);
            acc
        });

    let paths = down_size(paths, resolution);
    let (min_lat, min_lon, max_lat, max_lon) = get_boundaries(&paths);

    let lat_adjustment: f32 = -{
        if min_lat < max_lat {
            min_lat
        } else {
            max_lat
        }
    };

    let lon_adjustment: f32 = -{
        if min_lon < max_lon {
            min_lon
        } else {
            max_lon
        }
    };

    let paths = adjust(paths, (lat_adjustment, lon_adjustment));
    let paths = scale(paths, 500_000.0);
    let paths: loempia::Paths = to_paths(paths);

    println!("Found {} tracks", paths.len());

    let serial_path = path::Path::new("/dev/ttyACM0");
    let mut driver = Driver::open(serial_path)?;

    let plot = Plot { paths };

    driver.plot(&plot)?;
    Ok(())
}
