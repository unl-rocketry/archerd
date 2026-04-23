pub mod endpoints;

use std::io::{Error, Write as _};
use core::fmt::Display;
use rocket::FromFormField;

use serialport::SerialPort;

/// Command that the rotator accepts.
#[non_exhaustive]
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
            _ => return Err(())
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
    const BAUD: u32 = 115_200;

    /// Create a new rotator based on a serial port.
    ///
    /// # Errors
    /// If the port does not initalize properly or cannot change to
    /// [`Self::BAUD`] then this function will error.
    pub fn new(mut port: Box<dyn SerialPort>) -> Result<Self, Error> {
        port.set_baud_rate(Self::BAUD)?;
        port.set_timeout(std::time::Duration::from_millis(500))?;

        Ok(Self {
            port
        })
    }


    /// Moves by the specified number of steps in the horizontal axis.
    pub fn move_horizontal_steps(&mut self, steps: i32) -> Result<(), Error> {
        let cmd_string = self.send_command(Command::MoveHorizontalSteps, &[&steps.to_string()])?;
        self.validate_parse(&cmd_string)?;
        Ok(())
    }

    /// Send a command followed by arguments. Returns either an error if sending failed, or the
    pub fn send_command(&mut self, command: Command, args: &[&str]) -> Result<String, std::io::Error> {
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
        self.port.write_all(message.as_bytes())?;
        self.port.write_all(b"\n")?;

        Ok(())
    }

    /// Read the rotator response and determine errors or validation
    pub fn validate_parse(&mut self, command_string: &str) -> Result<Option<Vec<String>>, Error> {
        let mut response_string = String::new();

        // Fill up the result string with what the rotator spits out
        let mut buffer = [0; 2048];
        while let Ok(num_read) = self.port.read(&mut buffer) && num_read != 0 {
            let Ok(read_buffer) = str::from_utf8(&buffer[..num_read]) else {
                return Err(Error::other("invalid response"))
            };

            response_string.push_str(read_buffer);
        }

        // Split the response into "lines" by the newline characters
        let response_lines: Vec<_> = response_string.split_terminator('\n').collect();

        // The first line should be an echo of what was sent
        if response_lines[0] != command_string.trim() {
            return Err(Error::other("invalid response"))
        }

        // Split the second line into a status followed by the return values
        let response_list: Vec<&str> = response_lines[1].splitn(2, ' ').collect();
        match response_list[0] {
            "ERR" => return Err(Error::other(response_list[1].to_string())),
            "OK" => (),
            _ => return Err(Error::other("invalid response"))
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
}
