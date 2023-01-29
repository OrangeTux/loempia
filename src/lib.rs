use std::convert::TryFrom;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::ops;
use std::path;
use std::time::Duration;
use thiserror::Error;

use svg::node::element::path::Data;
use svg::node::element::Path as SVG_Path;
use svg::Document;

use serial_core::SerialDevice;

/// A `Point` is a coordinate on a 2D cartesian plane. Multi
pub type Point = (i32, i32);

/// A series of connected `Point`s form a `Path`.
pub type Path = Vec<Point>;

pub struct Paths {
    paths: Vec<Path>,
}

impl Paths {
    /// Create new `Paths`. Returns `Err` when Vector doesn't contain a path with minimum length of
    /// 2 points.
    pub fn new(paths: Vec<Path>) -> Result<Self, Error> {
        // Filter all paths with 0 or 1 points. They can't be plotted.
        let paths: Vec<Path> = paths.into_iter().filter(|path| path.len() > 1).collect();

        if paths.is_empty() {
            return Err(Error::InvalidPathError(
                "`paths` doesn't contain a single path with lenght of 2 points or more."
                    .to_string(),
            ));
        }

        //let start = start.ok_or(Error::InvalidPathError("Path is empty".to_string()))?;
        Ok(Self { paths })
    }
}

/// Get the boundaries of the boundaries of a series of paths. The function is generic
/// over the type coordinate type
pub fn get_boundaries<T: PartialOrd + Copy>(paths: &Vec<Vec<(T, T)>>) -> (T, T, T, T) {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (None, None, None, None);

    paths.iter().for_each(|path| {
        path.iter().for_each(|(x, y)| {
            _ = min_x.get_or_insert(x);
            _ = max_x.get_or_insert(x);

            // `unwrap()` is safe here.
            if x < min_x.unwrap() {
                min_x.replace(x);
            }

            if x > max_x.unwrap() {
                max_x.replace(x);
            }

            _ = min_y.get_or_insert(y);
            _ = max_y.get_or_insert(y);

            if y < min_y.unwrap() {
                min_y.replace(y);
            }

            if y > max_y.unwrap() {
                max_y.replace(y);
            }
        });
    });

    (
        *min_x.unwrap(),
        *min_y.unwrap(),
        *max_x.unwrap(),
        *max_y.unwrap(),
    )
}

// A `Vector` represents a movement in an x and y direction.
type Vector = (i32, i32);

impl From<&Vector> for Command {
    fn from(value: &Vector) -> Self {
        fn movement_on_x_axis(delta_x: i32) -> (i32, i32) {
            (delta_x, -delta_x)
        }

        fn movement_on_y_axis(delta_y: i32) -> (i32, i32) {
            (-delta_y, -delta_y)
        }
        let (delta_x, delta_y) = value;

        let (x1, y1) = movement_on_x_axis(*delta_x);
        let (x2, y2) = movement_on_y_axis(*delta_y);

        Command::SM {
            duration: 1000,
            axis_step_1: x1 + x2,
            axis_step_2: Some(y1 + y2),
        }
    }
}

/// A `Stroke` is a collection of `Vector`s.
#[derive(PartialEq, Debug)]
struct Stroke(pub Vec<Vector>);

impl ops::Deref for Stroke {
    type Target = Vec<Vector>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&Path> for Stroke {
    type Error = Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let vectors: Result<Vec<Vector>, Self::Error>  = path
            .windows(2)
            .map(|points| match points {
                [(x1, y1), (x2, y2)] => Ok((x2 - x1, y2 - y1)),
                _ => Err(Error::ConversionError(format!("Failed to convert `Path` to `Stroke`. Given `Path` contains {:?} `Point`s, but requires at least 2 `Point`s.", path.len())))
            })
            .collect();

        let vectors = vectors?;
        Ok(Stroke(vectors))
    }
}

#[derive(PartialEq, Debug)]
struct Strokes(pub Vec<Stroke>);

impl ops::Deref for Strokes {
    type Target = Vec<Stroke>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&Paths> for Strokes {
    type Error = Error;

    fn try_from(paths: &Paths) -> Result<Self, Self::Error> {
        let strokes: Result<Vec<Stroke>, Self::Error> =
            paths.paths.iter().map(Stroke::try_from).collect();

        Ok(Strokes(strokes?))
    }
}

fn convert_to_series_of_commands(strokes: Strokes) -> Vec<Vec<Command>> {
    strokes
        .iter()
        .map(|stroke| stroke.iter().map(Command::from).collect())
        .collect()
}

pub struct Plot {
    paths: Paths,
    start: Point,
}

impl Plot {
    /// Create a new `Plot`.
    pub fn new(paths: Paths) -> Self {
        let start = paths.paths[0][0];
        Plot { paths, start }
    }

    /// Create a new `Plot` using a single `Path`.
    pub fn from_path(path: Path) -> Result<Self, Error> {
        let paths = Paths::new(vec![path])?;
        Ok(Plot::new(paths))
    }

    /// Create SVG preview `Plot` as SVG.
    pub fn preview(&self) -> Document {
        let (min_x, min_y, max_x, max_y) = get_boundaries(&self.paths.paths);
        let mut data = Data::new().move_to(self.start);

        let strokes: Strokes = (&self.paths).try_into().unwrap();

        for stroke in strokes.0 {
            for point in &stroke.0 {
                data = data.line_by(*point);
            }
        }

        data = data.close();

        let path = SVG_Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 10)
            .set("d", data);

        Document::new()
            .set(
                "viewBox",
                (min_x, min_y, max_x - min_x, max_y - min_y),
            )
            .add(path)
    }
}

pub struct Driver {
    file: serial_unix::TTYPort,
}

impl Driver {
    pub fn open(path: &path::Path) -> Result<Self, Error> {
        let mut port = serial_unix::TTYPort::open(path)?;
        port.set_timeout(Duration::from_millis(10000))?;

        Ok(Self { file: port })
    }

    pub fn plot(&mut self, plot: &Plot) -> Result<(), Error> {
        let strokes: Strokes = Strokes::try_from(&plot.paths)?;
        let commands = convert_to_series_of_commands(strokes);

        for stroke in commands {
            self.execute_command(Command::Any("SP,0".to_string()))?;

            for command in stroke {
                self.execute_command(command)?;
            }
            self.execute_command(Command::Any("SP,1".to_string()))?;
        }

        Ok(())
    }

    pub fn execute_command(&mut self, cmd: Command) -> Result<(), Error> {
        let mut _cmd = cmd.to_string();
        _cmd.push('\r');
        println!("Writing command: {:?}", _cmd.to_string());

        self.file
            .write_all(_cmd.as_bytes())
            .map_err(|err| Error::CommandError(cmd.to_string(), err))?;

        let mut response = String::new();

        loop {
            let mut buffer = [0; 1];
            self.file
                .read_exact(&mut buffer)
                .map_err(|err| Error::ResponseError(cmd.to_string(), err))?;

            response.push(buffer[0] as char);
            if response.ends_with("\r\n") {
                break;
            }
        }

        println!("Response {:?}", response);
        if response.starts_with('!') {
            return Err(Error::ErrorResponse(cmd, response));
        }

        Ok(())
    }
}

/// Command supported by the device.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// Analog value get - Read all analog (ADC) input values.
    A,
    /// Analog Configure - Configure an analog input channel.
    AC {
        channel: u8,
        enable: bool,
    },
    Any(String),
    EM {
        enable_1: u8,
        enable_2: u8,
    },
    // Query EBB nickname tag.
    HM {
        step_frequency: u16,
        position_1: Option<u32>,
        position_2: Option<u32>,
    },
    QC,
    QT,
    /// Query Step position
    QS,
    R,
    // Set EBB nickname tag - This command sets the EBB's "nickname".
    ST {
        name: String,
    },
    // Stepper Move
    SM {
        // TODO: Create custom type that prevents values over 16777215.
        duration: u32,

        // TODO: Create custom type that prevents underflow and overflow.
        axis_step_1: i32,

        // TODO: Create custom type that prevents underflow and overflow.
        axis_step_2: Option<i32>,
    },
    // Toggle Pen -  This command toggles the state of the pen (up->down and down->up).
    TP {
        duration: Option<u16>,
    },
    V,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cmd: String = match self {
            Command::A => "A".into(),
            Command::AC { channel, enable } => {
                format!("AC,{},{}", channel, enable.clone() as u8)
            }
            Command::EM { enable_1, enable_2 } => {
                format!("EM,{},{}", enable_1, enable_2)
            }

            Command::HM { step_frequency, .. } => {
                format!("HM,{}", step_frequency)
            }
            Command::QC => "QC".into(),
            Command::QT => "QT".into(),
            Command::QS => "QS".into(),
            Command::R => "R".into(),
            Command::ST { name } => {
                format!("ST,{}", name)
            }
            Command::SM {
                duration,
                axis_step_1,
                axis_step_2,
            } => {
                let mut cmd = format!("SM,{},{}", duration, axis_step_1);
                if let Some(axis_step_2) = axis_step_2 {
                    cmd = format!("{},{}", cmd, axis_step_2)
                }
                cmd
            }

            Command::TP { duration } => match duration {
                None => "TP".into(),
                Some(duration) => format!("TP,{}", duration),
            },
            Command::V => "V".into(),
            Command::Any(cmd) => cmd.into(),
        };

        write!(f, "{}", cmd)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Something went wrong with the serial port.")]
    SerialPortError(#[from] serial_core::Error),

    #[error("Yolo")]
    IOError(#[from] io::Error),

    #[error("Failed to write command {0}: {1}.")]
    CommandError(String, io::Error),

    #[error("Failed to read response to command {0}: {1}.")]
    ResponseError(String, io::Error),

    #[error("Command {0} failed with error: {1}.")]
    ErrorResponse(Command, String),

    #[error("{0}")]
    ConversionError(String),

    #[error("{0}")]
    InvalidPathError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_valid_path_to_stroke() {
        let path: Path = vec![(0, 0), (1, 0), (1, 1), (0, 1), (0, 0)];

        let vectors = Stroke::try_from(&path).unwrap();
        let expected = Stroke(vec![(1, 0), (0, 1), (-1, 0), (0, -1)]);

        assert_eq!(vectors, expected);
    }

    #[test]
    fn convert_valid_stroke_to_series_of_commands() {
        let stroke = Stroke(vec![(1, 0), (0, 1), (-1, 0), (0, -1)]);

        let commands = convert_to_series_of_commands(Strokes(vec![stroke]));
        let expected = vec![vec![
            Command::SM {
                duration: 1000,
                axis_step_1: 1,
                axis_step_2: Some(-1),
            },
            Command::SM {
                duration: 1000,
                axis_step_1: -1,
                axis_step_2: Some(-1),
            },
            Command::SM {
                duration: 1000,
                axis_step_1: -1,
                axis_step_2: Some(1),
            },
            Command::SM {
                duration: 1000,
                axis_step_1: 1,
                axis_step_2: Some(1),
            },
        ]];

        assert_eq!(commands, expected);
    }
}
