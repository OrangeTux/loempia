use std::io::{Cursor, Write};
use std::path;
use std::time::Duration;

use serial_core::SerialDevice;

use crate::point::{Absolute, Coordinate, Relative};
use crate::{Error, Plot, Strokes};

#[derive(Debug)]
pub enum Command {
    /// Plotter is changed into initial state.
    IN,

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

impl Command {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Command::IN => "IN;".as_bytes(),
            Command::PA(None) => "PA;".as_bytes(),
            //Command::PA(Some(coordinate)) => {
                //format!("PA{},{};", coordinate.x, coordinate.y).as_bytes()
            //}
            Command::PD(None) => "PD;".as_bytes(),
            //Command::PD(Some(coordinate)) => {
                //format!("PD{},{};", coordinate.x, coordinate.y).as_bytes()
            //}
            Command::PR(None) => "PR;".as_bytes(),
            //Command::PR(Some(coordinate)) => {
                //format!("PR{},{};", coordinate.x, coordinate.y).as_bytes()
            //}
            Command::PU(None) => "PU;".as_bytes(),
            //Command::PU(Some(coordinate)) => {
                //format!("PU{},{};", coordinate.x, coordinate.y).as_bytes()
            //}
            //Command::SP(number) => format!("SP{};", number).as_bytes(),
            _ => panic!("YOLO")
        }
    }
}

fn to_hp_gl(strokes: &Strokes) -> Cursor<Vec<u8>> {
    let mut hpgl = Cursor::new(Vec::new());
    hpgl.write(Command::IN.as_bytes());
    hpgl.write(Command::SP(1).as_bytes());

    strokes.0.iter().for_each(|stroke| {
        //Raise pen, just to be sure.
        hpgl.write(Command::PU(None).as_bytes());

        //Move to to absolute start.
        hpgl.write(Command::PU(Some(stroke.start)).as_bytes());

        stroke.path.iter().for_each(|point| {
            hpgl.write(Command::PR(Some(*point)).as_bytes());
        });
    });

    //Raise pen and move to home
    hpgl.write(Command::PU(Some(Coordinate::new(0, 0))).as_bytes());

    return hpgl;
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
        let hpgl = to_hp_gl(&strokes);
        dbg!(hpgl);


        return Ok(())

    }
}
