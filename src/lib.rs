use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Yolo")]
    IOError(#[from] io::Error),

    #[error("Failed to write command {0}: {1}.")]
    CommandError(String, io::Error),

    #[error("Failed to read response to command {0}: {1}.")]
    ResponseError(String, io::Error),
}
