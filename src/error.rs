use std::fmt::{Display, Formatter, Result};

/// An enum representing all possible errors that the PSG may encounter during operation.
#[derive(Debug)]
pub enum Error {
    /// The clock rate is too high for the requested sample rate.
    ClockRateTooHigh
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Error::ClockRateTooHigh => f.write_str("the clock rate is too high for the requested sample rate")
        }
    }
}

impl std::error::Error for Error {}
