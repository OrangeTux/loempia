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

// A `Vector` represents a movement in x and y direction.
type Vector = (i32, i32);

impl From<Vector> for Command {
    fn from(value: Vector) -> Self {
        fn movement_on_x_axis(delta_x: i32) -> (i32, i32) {
            (delta_x, -delta_x)
        }

        fn movement_on_y_axis(delta_y: i32) -> (i32, i32) {
            (delta_y, delta_y)
        }
        let (delta_x, delta_y) = value;

        let (x1, y1) = movement_on_x_axis(delta_x);
        let (x2, y2) = movement_on_y_axis(delta_y);

        Command::SM {
            duration: 1000,
            axis_step_1: x1 + x2,
            axis_step_2: Some(y1 + y2),
        }
    }
}

struct Vectors(pub Vec<Vector>);

/// A series of connected `Point`s form a `Path`.
pub type Path = Vec<Point>;

/// A `Layer` is a group of paths that are plotted using the same head.
/// Multiple layers allow one to change head to plot in mulitple colors.
pub type Layer = Vec<Path>;

/// A `Plot` is a collection of `Layer`s
pub struct Plot {
    pub layers: Vec<Layer>,
}

pub struct Driver {
    file: serial_unix::TTYPort,
}

impl TryFrom<Path> for Vectors {
    type Error = Error;

    fn try_from(path: Path) -> Result<Self, Self::Error> {
        if path.len() <= 1 {
            panic!("Failed to calculate movements. The given track has not enought points.");
        }

        let vectors: Vec<Vector> = path
            .windows(2)
            .map(|points| match points {
                [(x1, y1), (x2, y2)] => (x2 - x1, y2 - y1),
                _ => panic!("This shouldn't happen."),
            })
            .collect();

        Ok(Vectors(vectors))
    }
}

impl Driver {
    pub fn open(path: &path::Path) -> Result<Self, Error> {
        let mut port = serial_unix::TTYPort::open(path)?;
        port.set_timeout(Duration::from_millis(10000))?;

        Ok(Self { file: port })
    }

    pub fn plot(&mut self, plot: &Plot) -> Result<(), Error> {
        // TODO: Create `Vector` for all other layers, not just the first layer.
        let vectors: Vec<Vectors> = plot.layers[0]
            .iter()
            .map(|path| {
                // TODO: Understand why next line is required. Why can't i just pass `path` to
                // try_from()`?
                let path: Path = path.to_vec();
                Vectors::try_from(path).expect("FUU")
            })
            .collect();

        // TODO: Create `Vec<Commands>` for all Vectors.
        let commands: Vec<Command> = vectors[0]
            .0
            .iter()
            .map(|vector| {
                // TODO: Understand why next line is required. Why can't i just pass `vector` to
                // from()`?
                let vector: Vector = *vector;
                Command::from(vector)
            })
            .collect();

        self.execute_command(Command::Any("SP,0".to_string()))?;

        for command in commands {
            self.execute_command(command)?;
        }

        self.execute_command(Command::Any("SP,1".to_string()))?;

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
#[derive(Debug)]
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
