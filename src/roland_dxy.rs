use std::io::{Cursor, Write};
use std::path;
use std::time::Duration;

use serial_core::{BaudRate, CharSize, FlowControl, Parity, PortSettings, SerialPort, StopBits};

use crate::point::{Absolute, Coordinate, Relative};
use crate::{Error, Plot, Strokes};

#[derive(Debug)]
pub enum Command {
    // Scale
    SC(i32, i32, i32, i32),

    EA(usize, usize),
    /// Plotter is changed into initial state.
    IN,
    //
    IP(usize, usize, usize, usize),

    /// Plot characters.
    //LB(Vec<u8>),

    /// Plot absolute.
    PA(Option<Coordinate<Absolute>>),

    /// Lower the pen and move to given absolute coordinate.
    PD(Option<Coordinate<Absolute>>),

    /// Plot re
    PR(Option<Coordinate<Relative>>),
    /// Raise the pen and move to given absolute coordinate.
    PU(Option<Coordinate<Absolute>>),

    /// Select the pen from the given slot.
    SP(u8),
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Command::SC(x_min, x_max, y_min, y_max) => {
                format!("SC{},{},{},{}", x_min, x_max, y_min, y_max)
            }
            Command::EA(x, y) => {
                format!("SC{},{}", x, y)
            }

            Command::IN => String::from("IN;"),
            Command::IP(x_min, x_max, y_min, y_max) => {
                format!("IP{},{},{},{}", x_min, x_max, y_min, y_max)
            }

            Command::PA(None) => String::from("PA;"),
            Command::PA(Some(coordinate)) => {
                format!("PA{},{};", coordinate.x, coordinate.y)
            }
            Command::PD(None) => String::from("PD;"),
            Command::PD(Some(coordinate)) => {
                format!("PD{},{};", coordinate.x, coordinate.y)
            }
            Command::PR(None) => String::from("PR;"),
            Command::PR(Some(coordinate)) => {
                format!("PR{},{};", coordinate.x, coordinate.y)
            }
            Command::PU(None) => String::from("PU;"),
            Command::PU(Some(coordinate)) => {
                format!("PU{},{};", coordinate.x, coordinate.y)
            }
            Command::SP(number) => format!("SP{};", number),
            //_ => panic!("{}", format!("{:?}", self)),
        };

        write!(f, "{}", string)
    }
}

fn to_hp_gl(strokes: &Strokes) -> Cursor<Vec<u8>> {
    let mut hpgl = Cursor::new(Vec::new());

    hpgl.write(Command::SP(1).to_string().as_bytes());

    strokes.0.iter().for_each(|stroke| {
        //Raise pen, just to be sure.
        hpgl.write(&Command::PU(None).to_string().as_bytes());

        //Move to to absolute start of the stroke.
        hpgl.write(&Command::PA(Some(stroke.start)).to_string().as_bytes());

        // Lower the pen.
        hpgl.write(&Command::PD(None).to_string().as_bytes());

        stroke.path.iter().for_each(|point| {
            // Move to each coordinate relative to current position.
            hpgl.write(&Command::PR(Some(*point)).to_string().as_bytes());
        });
    });

    // Raise pen and move to home
    hpgl.write(
        &Command::PU(Some(Coordinate::new(0, 0)))
            .to_string()
            .as_bytes(),
    );

    // Return pen to slot and go home.
    hpgl.write(&Command::SP(0).to_string().as_bytes());

    hpgl
}

pub struct Driver {
    file: serial_unix::TTYPort,
}

pub fn default_port_settings() -> PortSettings {
    PortSettings {
        baud_rate: BaudRate::Baud9600,
        char_size: CharSize::Bits7,
        parity: Parity::ParityEven,
        stop_bits: StopBits::Stop1,
        flow_control: FlowControl::FlowNone,
    }
}

impl Driver {
    pub fn open(path: &path::Path, settings: PortSettings) -> Result<Self, Error> {
        let mut port = serial::open(path)?;
        port.set_timeout(Duration::from_millis(10000))?;
        port.configure(&settings)
            .expect("Failed to configure serial port.");

        Ok(Self { file: port })
    }

    pub fn plot(&mut self, plot: &Plot) -> Result<(), Error> {
        let strokes: Strokes = Strokes::try_from(&plot.paths)?;
        let (length, height) = plot.dimensions();

        let aspect_ratio = 10_000.0 / 7_000.0;
        let x_ratio = 10_000.0 / length as f32;
        let y_ratio = 7_000.0 / height as f32;

        self.file.write(Command::IN.to_string().as_bytes());
        self.file
            .write(Command::IP(0, 0, 10_000, 7000).to_string().as_bytes());
        if x_ratio < y_ratio {
            self.file.write(
                Command::SC(0, length, 0, (height as f32 * aspect_ratio) as i32)
                    .to_string()
                    .as_bytes(),
            );
        } else {
            self.file.write(
                Command::SC(0, (length as f32 * aspect_ratio) as i32, 0, height)
                    .to_string()
                    .as_bytes(),
            );
        }

        let hpgl = to_hp_gl(&strokes);
        self.file.write(&hpgl.into_inner());

        Ok(())
    }
}
