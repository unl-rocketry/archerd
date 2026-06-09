//! Components for interacting with the rotator for tracking purposes.
//!
//! A connection to the rotator should be made using an automatic selection algorithm,
//! or by using the web API to connect.

pub mod dummyport;
pub mod endpoints;

use core::fmt::Display;
use rocket::FromFormField;
use std::{io::{self, Write as _}, num::ParseFloatError, ops::Neg as _, str::ParseBoolError};
use std::string::ParseError;
use serialport::SerialPort;

use crate::response::Error;

/// Command that the rotator accepts.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
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
    GetErrors,

    Halt,
}

impl Display for Command {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cmd_text = match self {
            Self::DegreesVertical => "DVER",
            Self::DegreesHorizontal => "DHOR",
            Self::CalibrateVertical => "CALV",
            Self::CalibrateHorizontal => "CALH",
            Self::Movement => "MOVC",
            Self::MoveVerticalSteps => "MOVV",
            Self::MoveHorizontalSteps => "MOVH",
            Self::GetPosition => "GETP",
            Self::GetCalibrated => "GETC",
            Self::GetVersion => "VERS",
            Self::GetErrors => "GERR",
            Self::Halt => "HALT",
        };

        write!(f, "{cmd_text}")
    }
}

/// Direction accepted by [`Command::Movement`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, FromFormField)]
pub enum Direction {
    // Vertical
    Up,
    Down,
    StopVertical,

    // Horizontal
    Left,
    Right,
    StopHorizontal,
}

impl TryFrom<&str> for Direction {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "UP" => Self::Up,
            "DN" => Self::Down,
            "SV" => Self::StopVertical,
            "LT" => Self::Left,
            "RT" => Self::Right,
            "SH" => Self::StopHorizontal,
            _ => return Err(()),
        })
    }
}

impl Display for Direction {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cmd_text = match self {
            Self::Up => "UP",
            Self::Down => "DN",
            Self::StopVertical => "SV",
            Self::Left => "LT",
            Self::Right => "RT",
            Self::StopHorizontal => "SH",
        };

        write!(f, "{cmd_text}")
    }
}

/// A two-axis rotator, utilizing the
/// [protocol specified here](https://github.com/unl-rocketry/tracker-embedded/blob/main-rust/PROTOCOL.md).
pub struct Rotator {
    port: Box<dyn SerialPort>,
}

#[allow(clippy::missing_errors_doc)]
impl Rotator {
    pub const BAUD: u32 = 115_200;

    /// Create a new rotator based on a serial port.
    ///
    /// # Errors
    /// If the port does not initalize properly or cannot change to
    /// [`Self::BAUD`] then this function will error.
    pub fn new(mut port: Box<dyn SerialPort>) -> Result<Self, io::Error> {
        port.set_baud_rate(Self::BAUD)?;
        port.set_timeout(std::time::Duration::from_millis(25))?;

        Ok(Self { port })
    }

    pub fn port(&self) -> &Box<dyn SerialPort> {
        &self.port
    }

    /// Send a command followed by arguments. Returns either an error if sending failed, or the
    pub fn send_command(
        &mut self,
        command: Command,
        args: &[&str],
    ) -> Result<String, std::io::Error> {
        self.port.clear(serialport::ClearBuffer::All)?;

        let mut command_string = Vec::new();

        self.port.write_all(command.to_string().as_bytes())?;
        command_string.write_all(command.to_string().as_bytes())?;

        for arg in args {
            self.port.write_all(b" ")?;
            command_string.write_all(b" ")?;

            self.port.write_all(arg.as_bytes())?;
            command_string.write_all(arg.as_bytes())?;
        }

        self.port.write_all(b"\n")?;
        command_string.write_all(b"\n")?;

        let command_string = String::from_utf8_lossy(&command_string).to_string();

        Ok(command_string)
    }

    /// Send a raw message.
    fn _send_message(&mut self, message: &str) -> Result<(), std::io::Error> {
        dbg!(message);
        self.port.write_all(message.as_bytes())?;
        self.port.write_all(b"\n")?;

        Ok(())
    }

    /// Read the rotator response and determine errors or validation
    pub fn validate_parse(&mut self, command_string: &str) -> Result<Option<Vec<String>>, io::Error> {
        let mut response_string = String::new();

        // Fill up the result string with what the rotator spits out
        let mut buffer = [0; 2048];
        while let Ok(num_read) = self.port.read(&mut buffer)
            && num_read != 0
        {
            let Ok(read_buffer) = str::from_utf8(&buffer[..num_read]) else {
                return Err(io::Error::other("invalid response"));
            };

            response_string.push_str(read_buffer);
        }

        // Split the response into "lines" by the newline characters
        let response_lines: Vec<_> = response_string.split_terminator('\n').collect();

        dbg!(&response_lines);

        // The first line should be an echo of what was sent
        if *response_lines
            .first()
            .ok_or_else(|| io::Error::other("response empty"))?
            != command_string.trim()
        {
            return Err(io::Error::other("invalid response"));
        }

        // Split the second line into a status followed by the return values
        if response_lines.len() < 2 {
            return Err(io::Error::other("response not two lines"))
        }

        let response_list: Vec<&str> = response_lines[1].splitn(2, ' ').collect();
        dbg!(&response_list);
        match response_list[0] {
            "ERR" => return Err(io::Error::other(response_list[1].to_string())),
            "OK" => (),
            _ => return Err(io::Error::other("invalid response")),
        }

        // Split the return values further
        let response_list: Vec<_> = response_list[1]
            .split_ascii_whitespace()
            .map(std::string::ToString::to_string)
            .collect();

        if response_list.is_empty() {
            Ok(None)
        } else {
            Ok(Some(response_list))
        }
    }

    pub async fn set_position_vertical(&mut self, degrees: f32) -> Result<(), Error> {
        let cmd_string = self.send_command(Command::DegreesVertical, &[&format!("{degrees:0.3}")])?;
        self.validate_parse(&cmd_string)?;

        Ok(())
    }

    /// Set a defined position for the rotator in the horizontal axis.
    pub async fn set_position_horizontal(&mut self, degrees: f32) -> Result<(), Error> {
        let cmd_string = self.send_command(
            Command::DegreesHorizontal,
            &[&format!("{:0.3}", degrees.neg())],
        )?;
        self.validate_parse(&cmd_string)?;

        Ok(())
    }

    /// Calibrates the vertical axis.
    pub async fn calibrate_vertical(&mut self, set: bool) -> Result<(), Error> {
        let cmd_string = if set {
            self.send_command(Command::CalibrateVertical, &["SET"])?
        } else {
            self.send_command(Command::CalibrateVertical, &[])?
        };

        self.validate_parse(&cmd_string)?;

        Ok(())
    }

    /// Calibrates the horizontal axis.
    pub async fn calibrate_horizontal(&mut self) -> Result<(), Error> {
        let cmd_string = self.send_command(Command::CalibrateHorizontal, &[])?;
        self.validate_parse(&cmd_string)?;

        Ok(())
    }

    /// Moves in a direction indefinitely specified by the command, or stops, if the command is to stop.
    pub async fn move_direction(&mut self, direction: Direction) -> Result<(), Error> {
        let cmd_string =
            self.send_command(Command::CalibrateHorizontal, &[&direction.to_string()])?;
        self.validate_parse(&cmd_string)?;

        Ok(())
    }

    /// Moves by the specified number of steps in the vertical axis.
    pub async fn move_vertical_steps(&mut self, steps: i32) -> Result<(), Error> {
        let cmd_string = self.send_command(Command::MoveVerticalSteps, &[&steps.to_string()])?;
        self.validate_parse(&cmd_string)?;

        Ok(())
    }

    /// Moves by the specified number of steps in the horizontal axis.
    pub async fn move_horizontal_steps(&mut self, steps: i32) -> Result<(), Error> {
        let cmd_string = self.send_command(Command::MoveHorizontalSteps, &[&steps.to_string()])?;
        self.validate_parse(&cmd_string)?;

        Ok(())
    }

    /// Gets the current position for both the vertical and horizontal axes.
    pub async fn position(&mut self) -> Result<(f32, f32), Error> {
        let cmd_string = self.send_command(Command::GetPosition, &[])?;
        let value_list = self
            .validate_parse(&cmd_string)?
            .ok_or_else(|| io::Error::other("ExpectedValue"))?;

        if value_list.len() != 2 {
            Err(io::Error::other("InvalidResponse"))?
        }

        let (v, h) = (
            value_list[0]
                .parse::<f32>()
                .map_err(|e: ParseFloatError| io::Error::other(e.to_string()))?,
            value_list[1]
                .parse::<f32>()
                .map_err(|e: ParseFloatError| io::Error::other(e.to_string()))?,
        );

        Ok((v, h))
    }

    /// Gets the calibration status of the rotator. This must be true to use
    /// `set_position_vertical` and `set_position_horizontal`.
    pub async fn calibrated(&mut self) -> Result<bool, Error> {
        let cmd_string = self.send_command(Command::GetCalibrated, &[])?;

        let value_list = self
            .validate_parse(&cmd_string)?
            .ok_or_else(|| io::Error::other("ExpectedValue"))?;


        let calibrated = value_list[0]
            .parse::<bool>()
            .map_err(|e: ParseBoolError| io::Error::other(e.to_string()))?;

        Ok(calibrated)
    }

    /// Immediately stops both motors by locking them to perform an emergency stop.
    pub async fn halt(&mut self) -> Result<(), Error> {
        let cmd_string = self.send_command(Command::Halt, &[])?;
        self.validate_parse(&cmd_string)?;

        Ok(())
    }

    /// Gets the current version of the software on the rotator.
    pub async fn version(&mut self) -> Result<String, Error> {
        let cmd_string = self.send_command(Command::GetVersion, &[])?;

        Ok(self
            .validate_parse(&cmd_string)?
            .ok_or_else(|| io::Error::other("ExpectedValue"))?[0].clone())
    }

    pub async fn errors(&mut self) -> Result<String, Error> {
        let cmd_string = self.send_command(Command::GetErrors, &[])?;

        let value_list = self
            .validate_parse(&cmd_string)?
            .ok_or_else(|| io::Error::other("ExpectedValue"))?;


        let error = value_list
            .join(" ");

        Ok(error)
    }
}
