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
    SerialError(#[from] serialport::Error),

    #[error("there was an io error")]
    IOError(#[from] std::io::Error),
}

/// A two-axis rotator, utilizing the protocol specified here:
/// https://github.com/unl-rocketry/tracker-embedded/blob/main-rust/PROTOCOL.md
pub struct Rotator {
    port: Box<dyn SerialPort>,
}

impl Rotator {
    const BAUD: u32 = 115_200;

    pub fn new(mut port: Box<dyn SerialPort>) -> Result<Self, Error> {
        port.set_baud_rate(Self::BAUD)?;
        port.set_timeout(std::time::Duration::from_millis(500))?;

        Ok(Self {
            port
        })
    }

    /// Position in degrees to move to in the vertical axis.
    pub fn set_position_vertical(&mut self, pos: f32) -> Result<Option<String>, Error> {
        self.send_command(Command::DegreesVertical, &[&format!("{:0.3}", pos)])?;
        self.validate_parse()
    }

    /// Position in degrees to move to in the horizontal axis.
    pub fn set_position_horizontal(&mut self, pos: f32) -> Result<Option<String>, Error> {
        self.send_command(Command::DegreesHorizontal, &[&format!("{:0.3}", pos.neg())])?;
        self.validate_parse()
    }

    /// Calibrate vertical axis.
    pub fn calibrate_vertical(&mut self, set: bool) -> Result<Option<String>, Error> {
        if set {
            self.send_command(Command::CalibrateVertical, &["SET"])?;
        } else {
            self.send_command(Command::CalibrateVertical, &[])?;
        }
        self.validate_parse()
    }

    /// Calibrate horizontal axis.
    pub fn calibrate_horizontal(&mut self) -> Result<Option<String>, Error> {
        self.send_command(Command::CalibrateHorizontal, &[])?;
        self.validate_parse()
    }

    /// Moves in a direction indefinitely specified by the command, or stops, if the command is to stop.
    pub fn move_direction(&mut self, direction: Direction) -> Result<Option<String>, Error> {
        self.send_command(Command::CalibrateHorizontal, &[&direction.to_string()])?;
        self.validate_parse()
    }

    /// Moves by the specified number of steps in the vertical axis.
    pub fn move_vertical_steps(&mut self, steps: i32) -> Result<Option<String>, Error> {
        self.send_command(Command::MoveVerticalSteps, &[&steps.to_string()])?;
        self.validate_parse()
    }

    /// Moves by the specified number of steps in the horizontal axis.
    pub fn move_horizontal_steps(&mut self, steps: i32) -> Result<Option<String>, Error> {
        self.send_command(Command::MoveHorizontalSteps, &[&steps.to_string()])?;
        self.validate_parse()
    }

    /// Gets the current position for both the vertical and horizontal axes.
    pub fn position(&mut self) -> Result<Option<String>, Error> {
        self.send_command(Command::GetPosition, &[])?;
        self.validate_parse()
    }

    /// Gets the calibration status of the rotator. This must be true to use
    /// `set_position_vertical` and `set_position_horizontal`.
    pub fn calibrated(&mut self) -> Result<Option<String>, Error> {
        self.send_command(Command::GetCalibrated, &[])?;
        self.validate_parse()
    }

    /// Gets the current version of the software on the rotator.
    pub fn version(&mut self) -> Result<Option<String>, Error> {
        self.send_command(Command::GetVersion, &[])?;
        self.validate_parse()
    }

    /// Immediately stops both motors by locking them to perform an emergency stop.
    pub fn halt(&mut self) -> Result<Option<String>, Error> {
        self.send_command(Command::Halt, &[])?;
        self.validate_parse()
    }

    fn send_command(&mut self, command: Command, args: &[&str]) -> Result<(), std::io::Error> {
        self.port.write_all(command.to_string().as_bytes())?;

        for arg in args {
            self.port.write_all(b" ")?;
            self.port.write_all(arg.as_bytes())?;
        }

        self.port.write_all(b"\n")?;

        Ok(())
    }

    fn send_message(&mut self, message: &str) -> Result<(), std::io::Error> {
        self.port.write_all(message.as_bytes())?;
        self.port.write_all(b"\n")?;

        Ok(())
    }

    // Read the rotator response and determine errors or validation
    fn validate_parse(&mut self) -> Result<Option<String>, Error> {

        Ok(None)
    }
}
