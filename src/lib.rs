use std::convert::TryFrom;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::path;
use std::time::Duration;
use thiserror::Error;

use serial_core::SerialDevice;

/// A `Point` represent a coordinate on a 2D cartesian plane.
pub type Point = (i32, i32);

// A `Vector` represents a movement in an x and y direction.
type Vector = (i32, i32);

impl From<&Vector> for Command {
    fn from(value: &Vector) -> Self {
        fn movement_on_x_axis(delta_x: i32) -> (i32, i32) {
            (delta_x, -delta_x)
        }

        fn movement_on_y_axis(delta_y: i32) -> (i32, i32) {
            (delta_y, delta_y)
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

#[derive(PartialEq, Debug)]
struct Stroke(pub Vec<Vector>);

impl TryFrom<&Path> for Stroke {
    type Error = Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        if path.len() <= 1 {
            panic!("Failed to calculate stroke. The given path has not enough points.");
        }

        let vectors: Vec<Vector> = path
            .windows(2)
            .map(|points| match points {
                [(x1, y1), (x2, y2)] => (x2 - x1, y2 - y1),
                _ => panic!("This shouldn't happen."),
            })
            .collect();

        Ok(Stroke(vectors))
    }
}

#[derive(PartialEq, Debug)]
struct Strokes(pub Vec<Stroke>);

impl TryFrom<&Paths> for Strokes {
    type Error = Error;

    fn try_from(paths: &Paths) -> Result<Self, Self::Error> {
        let strokes: Vec<Stroke> = paths
            .0
            .iter()
            .map(|path| Stroke::try_from(path).expect("yolo"))
            .collect();

        Ok(Strokes(strokes))
    }
}

/// A series of connected `Point`s form a `Path`.
pub type Path = Vec<Point>;

pub struct Paths(pub Vec<Path>);

fn convert_to_series_of_commands(strokes: Strokes) -> Vec<Vec<Command>> {
    strokes
        .0
        .iter()
        .map(|stroke| {
            stroke
                .0
                .iter()
                .map(|vector| Command::from(vector))
                .collect()
        })
        .collect()
}

/// A `Plot` is a collection of `Layer`s
pub struct Plot {
    pub paths: Paths,
}

impl Plot {
    pub fn from_path(path: Path) -> Self {
        Plot {
            paths: Paths(vec![path]),
        }
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
        let strokes: Strokes = Strokes::try_from(&plot.paths).unwrap();
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
        //file.flush().unwrap();
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
                axis_step_1: 1,
                axis_step_2: Some(1),
            },
            Command::SM {
                duration: 1000,
                axis_step_1: -1,
                axis_step_2: Some(1),
            },
            Command::SM {
                duration: 1000,
                axis_step_1: -1,
                axis_step_2: Some(-1),
            },
        ]];

        assert_eq!(commands, expected);
    }
}
