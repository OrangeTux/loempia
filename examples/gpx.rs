use std::path::PathBuf;
use std::{fs, path};

use clap::{Parser, Subcommand};
use roxmltree::{Document, Node};

use loempia::point::Coordinate;
use loempia::roland_dxy::{default_port_settings, Driver};
use loempia::{get_boundaries, Error, Plot};

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

/// Adjust every point by the given latitude and longitude.
fn adjust(paths: loempia::Paths, adjustment: (i32, i32)) -> loempia::Paths {
    let (lat_adjustment, lon_adjustment) = adjustment;
    let x = paths
        .paths
        .iter()
        .map(|path| {
            path.iter()
                .map(|Coordinate { x, y, .. }| {
                    Coordinate::new(x + lat_adjustment, y + lon_adjustment)
                })
                .collect()
        })
        .collect();

    loempia::Paths::new(x).unwrap()
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
                .map(|(lat, lon)| Coordinate::new(*lat as i32, *lon as i32))
                .collect()
        })
        .collect();

    loempia::Paths::new(paths).unwrap()
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

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Plot
    Plot {
        input: PathBuf,

        /// Path to serial device.
        #[arg(short, long, default_value = "/dev/ttyUSB0")]
        device: PathBuf,
    },
    Preview {
        input: PathBuf,

        /// Location where SVG is written to.
        #[arg(short, long, default_value = "/tmp/gpx.svg")]
        output: PathBuf,
    },
}

fn get_plot(path: &PathBuf) -> Result<Plot, Error> {
    let text = fs::read_to_string(path).expect(&format!("Failed to read {}.", path.display()));
    let doc = Document::parse(&text).expect(&format!("Failed to parse {} as GPX.", path.display()));

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

    let paths = down_size(paths, 1);

    let paths = scale(paths, 500_000.0);
    let paths: loempia::Paths = to_paths(paths);
    let (min_lat, min_lon, max_lat, max_lon) = get_boundaries(&paths);

    let lat_adjustment: i32 = -{
        if min_lat < max_lat {
            min_lat
        } else {
            max_lat
        }
    };

    let lon_adjustment: i32 = -{
        if min_lon < max_lon {
            min_lon
        } else {
            max_lon
        }
    };

    let paths = adjust(paths, (lat_adjustment, lon_adjustment));

    Ok(Plot::new(paths))
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Plot { input, device } => {
            let plot = get_plot(input)?;
            let serial_path = path::Path::new(device);
            let mut driver = Driver::open(serial_path, default_port_settings())?;
            driver.plot(&plot)?;
        }
        Commands::Preview { input, output } => {
            let plot = get_plot(input)?;
            let document = plot.preview();
            svg::save(output, &document).expect(&format!("Failed to save preview at {}.", output.display()));
            println!("Preview written to {}.", output.display());
        }
    }

    Ok(())
}
