use loempia::Error;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;

struct Driver {
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
        Ok(())
    }
}

/// Command supported by the device.
pub enum Command {
    /// Analog value get - Read all analog (ADC) input values.
    A,
    /// Analog Configure - Configure an analog input channel.
    AC {
        channel: u8,
        enable: bool,
    },
    // Query EBB nickname tag.
    QT,
    // Set EBB nickname tag - This command sets the EBB's "nickname".
    ST {
        name: String,
    },
    // Toggle Pen -  This command toggles the state of the pen (up->down and down->up).
    TP {
        duration: Option<u16>,
    },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cmd: String = match self {
            Command::A => "A".into(),
            Command::AC { channel, enable } => {
                format!("AC,{},{}", channel, enable.clone() as u8)
            }
            Command::QT => "QT".into(),
            Command::ST { name } => {
                format!("ST,{}", name)
            }
            Command::TP { duration } => match duration {
                None => "TP".into(),
                Some(duration) => format!("TP,{}", duration),
            },
        };

        write!(f, "{}", cmd)
    }
}

fn main() {
    let path = Path::new("/dev/ttyACM0");

    let mut driver = Driver::open(path).expect("Failed to open driver");
    driver
        .execute_command(Command::TP { duration: Some(15) })
        .expect("Failed to execute command");
    driver.execute_command(Command::QT).unwrap();
    driver
        .execute_command(Command::ST {
            name: "Loempia".to_string(),
        })
        .unwrap();
    driver.execute_command(Command::QT).unwrap();
}
