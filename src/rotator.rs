use std::{fmt::Display, io::{BufWriter, Write}, ops::Neg};

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
    #[error("the rotator returned an error: {0}")]
    ResponseError(String),

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
    pub fn set_position_vertical(&mut self, pos: f32) -> Result<Option<Vec<String>>, Error> {
        let cmd_string = self.send_command(Command::DegreesVertical, &[&format!("{:0.3}", pos)])?;
        self.validate_parse(&cmd_string)
    }

    /// Position in degrees to move to in the horizontal axis.
    pub fn set_position_horizontal(&mut self, pos: f32) -> Result<Option<Vec<String>>, Error> {
        let cmd_string = self.send_command(Command::DegreesHorizontal, &[&format!("{:0.3}", pos.neg())])?;
        self.validate_parse(&cmd_string)
    }

    /// Calibrate vertical axis.
    pub fn calibrate_vertical(&mut self, set: bool) -> Result<Option<Vec<String>>, Error> {
        let cmd_string = if set {
            self.send_command(Command::CalibrateVertical, &["SET"])?
        } else {
            self.send_command(Command::CalibrateVertical, &[])?
        };
        self.validate_parse(&cmd_string)
    }

    /// Calibrate horizontal axis.
    pub fn calibrate_horizontal(&mut self) -> Result<Option<Vec<String>>, Error> {
        let cmd_string = self.send_command(Command::CalibrateHorizontal, &[])?;
        self.validate_parse(&cmd_string)
    }

    /// Moves in a direction indefinitely specified by the command, or stops, if the command is to stop.
    pub fn move_direction(&mut self, direction: Direction) -> Result<Option<Vec<String>>, Error> {
        let cmd_string = self.send_command(Command::CalibrateHorizontal, &[&direction.to_string()])?;
        self.validate_parse(&cmd_string)
    }

    /// Moves by the specified number of steps in the vertical axis.
    pub fn move_vertical_steps(&mut self, steps: i32) -> Result<Option<Vec<String>>, Error> {
        let cmd_string = self.send_command(Command::MoveVerticalSteps, &[&steps.to_string()])?;
        self.validate_parse(&cmd_string)
    }

    /// Moves by the specified number of steps in the horizontal axis.
    pub fn move_horizontal_steps(&mut self, steps: i32) -> Result<Option<Vec<String>>, Error> {
        let cmd_string = self.send_command(Command::MoveHorizontalSteps, &[&steps.to_string()])?;
        self.validate_parse(&cmd_string)
    }

    /// Gets the current position for both the vertical and horizontal axes.
    pub fn position(&mut self) -> Result<Option<Vec<String>>, Error> {
        let cmd_string = self.send_command(Command::GetPosition, &[])?;
        self.validate_parse(&cmd_string)
    }

    /// Gets the calibration status of the rotator. This must be true to use
    /// `set_position_vertical` and `set_position_horizontal`.
    pub fn calibrated(&mut self) -> Result<Option<Vec<String>>, Error> {
        let cmd_string = self.send_command(Command::GetCalibrated, &[])?;
        self.validate_parse(&cmd_string)
    }

    /// Gets the current version of the software on the rotator.
    pub fn version(&mut self) -> Result<Option<Vec<String>>, Error> {
        let cmd_string = self.send_command(Command::GetVersion, &[])?;
        self.validate_parse(&cmd_string)
    }

    /// Immediately stops both motors by locking them to perform an emergency stop.
    pub fn halt(&mut self) -> Result<Option<Vec<String>>, Error> {
        let cmd_string = self.send_command(Command::Halt, &[])?;
        self.validate_parse(&cmd_string)
    }

    /// Send a command followed by arguments. Returns either an error if sending failed, or the
    fn send_command(&mut self, command: Command, args: &[&str]) -> Result<String, std::io::Error> {
        let mut command_string = BufWriter::new(Vec::new());

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

        let command_string = String::from_utf8(command_string.into_inner().unwrap()).unwrap();

        Ok(command_string)
    }

    /// Send a raw message.
    fn send_message(&mut self, message: &str) -> Result<(), std::io::Error> {
        self.port.write_all(message.as_bytes())?;
        self.port.write_all(b"\n")?;

        Ok(())
    }

    /// Read the rotator response and determine errors or validation
    fn validate_parse(&mut self, command_string: &str) -> Result<Option<Vec<String>>, Error> {
        let mut response_string = String::new();

        // Fill up the result string with what the rotator spits out
        let mut buffer = [0; 2048];
        while let Ok(num_read) = self.port.read(&mut buffer) && num_read != 0 {
            let Ok(read_buffer) = str::from_utf8(&buffer[..num_read]) else {
                return Err(Error::InvalidResponse)
            };

            response_string.push_str(read_buffer);
        }

        // Split the response into "lines" by the newline characters
        let response_lines: Vec<_> = response_string.split_terminator("\n").collect();

        // The first line should be an echo of what was sent
        if response_lines[0] != command_string {
            return Err(Error::InvalidResponse)
        }

        // Split the second line into a status followed by the return values
        let response_list: Vec<&str> = response_lines[1].splitn(2, ' ').collect();
        match response_list[0] {
            "ERR" => return Err(Error::ResponseError(response_list[1].to_string())),
            "OK" => (),
            _ => return Err(Error::InvalidResponse)
        }

        // Split the return values further
        let response_list: Vec<_> = response_list[1]
            .split_ascii_whitespace()
            .map(|r| r.to_string())
            .collect();

        if !response_list.is_empty() {
            Ok(Some(response_list))
        } else {
            Ok(None)
        }
    }
}
