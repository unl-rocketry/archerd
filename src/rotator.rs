use std::{fmt::Display, ops::Neg};

use serialport::SerialPort;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    DegreesVertical,
    DegreesHorizontal,

    CalibrateVertical,
    CalibrateHorizontal,

    Movement,
    MoveVerticalSteps,
    MoveHorizontalSteps,

    GetPosition,
    GetCalibrated,
    GetVersion,

    Halt,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cmd_text = match self {
            Command::DegreesVertical => "DVER",
            Command::DegreesHorizontal => "DHOR",
            Command::CalibrateVertical => "CALV",
            Command::CalibrateHorizontal => "CALH",
            Command::Movement => "MOVC",
            Command::MoveVerticalSteps => "MOVV",
            Command::MoveHorizontalSteps => "MOVH",
            Command::GetPosition => "GETP",
            Command::GetCalibrated => "GETC",
            Command::GetVersion => "VERS",
            Command::Halt => "HALT",
        };

        write!(f, "{cmd_text}")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    // Vertical
    Up,
    Down,
    StopVertical,

    // Horizontal
    Left,
    Right,
    StopHoriztonal,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cmd_text = match self {
            Direction::Up => "UP",
            Direction::Down => "DN",
            Direction::StopVertical => "SV",
            Direction::Left => "LT",
            Direction::Right => "RT",
            Direction::StopHoriztonal => "SH",
        };

        write!(f, "{cmd_text}")
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("the command was rejected: {0}")]
    CommandRejection(String),

    #[error("the response from the rotator was invalid")]
    InvalidResponse,

    #[error("the underlying serial port had an error")]
    SerialError(#[from] std::io::Error),
}

/// A two-axis rotator, utilizing the protocol specified here:
/// https://github.com/unl-rocketry/tracker-embedded/blob/main-rust/PROTOCOL.md
pub struct Rotator {
    port: Box<dyn SerialPort>,
}

impl Rotator {
    pub fn new(port: Box<dyn SerialPort>) -> Self {
        Self {
            port
        }
    }

    pub fn set_position(&mut self, horizontal: f32, vertical: f32) -> Result<Option<String>, Error> {
        self.set_position_vertical(vertical)?;
        self.set_position_horizontal(horizontal)?;

        Ok(None)
    }

    pub fn set_position_vertical(&mut self, pos: f32) -> Result<Option<String>, Error> {
        self.port.write_all(format!("{} {:0.3}", Command::DegreesVertical, pos).as_bytes())?;
        self.validate_parse()
    }

    pub fn set_position_horizontal(&mut self, pos: f32) -> Result<Option<String>, Error> {
        self.port.write_all(format!("{} {:0.3}", Command::DegreesHorizontal, pos.neg()).as_bytes())?;
        self.validate_parse()
    }

    // Read the rotator response and determine errors or validation
    fn validate_parse(&mut self) -> Result<Option<String>, Error> {
        Ok(None)
    }
}
