use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

struct Driver {
    file: File,
}

impl Driver {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let mut file = match OpenOptions::new().read(true).write(true).open(&path) {
            Ok(file) => file,
            Err(err) => panic!("Failed to open file: {}", err),
        };

        Self { file }
    }

    pub fn execute_command(&mut self, cmd: Command) {
        let mut _cmd = cmd.to_string();
        _cmd.push('\r');
        println!("Writing command: {:?}", _cmd.to_string());

        self.file
            .write_all(_cmd.as_bytes())
            .expect("Failed to write command.");

        let mut response = String::new();

        loop {
            let mut buffer = [0; 1];
            self.file.read_exact(&mut buffer).expect("Failted to read");
            response.push(buffer[0] as char);
            if buffer[0] == 10 {
                break;
            }
        }

        println!("Response {:?}", response);
    }
}

pub enum Command {
    A,
    QT,
    TP(Option<u16>),
    ST(String),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cmd: String = match self {
            Command::A => "A".into(),
            Command::QT => "QT".into(),
            Command::ST(name) => {
                format!("ST,{}", name)
            }
            Command::TP(duration) => match duration {
                None => "TP".into(),
                Some(duration) => format!("TP,{}", duration),
            },
        };

        write!(f, "{}", cmd)
    }
}

fn main() {
    let path = Path::new("/dev/ttyACM0");

    let mut driver = Driver::new(path);
    //driver.execute_command(Command::TP(Some(15)));
    driver.execute_command(Command::QT);
    driver.execute_command(Command::ST("Loempia".to_string()));
    driver.execute_command(Command::QT);
}
