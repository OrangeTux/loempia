mod how_to;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use thiserror::Error;

pub struct Driver {
    file: File,
}

impl Driver {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = match OpenOptions::new().read(true).write(true).open(&path) {
            Ok(file) => file,
            Err(err) => return Err(err.into()),
        };

        Ok(Self { file })
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
            if buffer[0] as char == '\n' {
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
        step_frequency: Option<u16>,
        position_1: Option<u32>,
        position_2: Option<u32>,
    },
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

            Command::HM { step_frequency, .. } => match step_frequency {
                Some(_) => {
                    unimplemented!("Arguments to command HM are not supported yet.");
                }
                None => "HM".to_string(),
            },
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
    #[error("Yolo")]
    IOError(#[from] io::Error),

    #[error("Failed to write command {0}: {1}.")]
    CommandError(String, io::Error),

    #[error("Failed to read response to command {0}: {1}.")]
    ResponseError(String, io::Error),

    #[error("Command {0} failed with error: {1}.")]
    ErrorResponse(Command, String),
}
