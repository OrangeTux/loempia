use std::path::Path;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use loempia::roland_dxy::{default_port_settings, Driver};
use loempia::{point::Coordinate, Error, Plot};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Plot
    Plot {
        /// Path to serial device.
        #[arg(short, long, default_value = "/dev/ttyUSB0")]
        device: PathBuf,
    },
    Preview {
        /// Location where SVG is written to.
        #[arg(short, long, default_value = "/tmp/square.svg")]
        output: PathBuf,
    },
}

fn get_plot() -> Plot {
    let path = vec![
        Coordinate::new(0, 0),
        Coordinate::new(1000, 0),
        Coordinate::new(1000, 1000),
        Coordinate::new(0, 1000),
        Coordinate::new(0, 0),
    ];
    Plot::from_path(path).expect("Failed to create Plot.")
}

fn main() -> Result<(), Error> {
    let plot = get_plot();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Plot { device } => {
            let serial_path = Path::new(device);
            let mut driver = Driver::open(serial_path, default_port_settings())?;
            driver.plot(&plot)?;
        }
        Commands::Preview { output } => {
            let document = plot.preview();
            svg::save(output, &document).expect(&format!("Failed to save preview at {}.", output.display()));
            println!("Preview written to {}.", output.display());
        }
    }

    Ok(())
}
