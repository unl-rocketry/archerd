use std::{io::{Read, Write}, time::Duration};

use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};

#[derive(Clone, Copy)]
pub struct DummyPort {
    baud: u32,
    data_bits: DataBits,
    flow_control: FlowControl,
    parity: Parity,
    stop_bits: StopBits,
    timeout: Duration,
}

impl Default for DummyPort {
    fn default() -> Self {
        Self {
            baud: 115_200,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: Duration::from_secs(1)
        }
    }
}

impl Write for DummyPort {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Read for DummyPort {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(0)
    }
}

impl SerialPort for DummyPort {
    fn name(&self) -> Option<String> {
        Some("dummy".to_string())
    }

    fn baud_rate(&self) -> serialport::Result<u32> {
        Ok(self.baud)
    }

    fn data_bits(&self) -> serialport::Result<serialport::DataBits> {
        Ok(self.data_bits)
    }

    fn flow_control(&self) -> serialport::Result<serialport::FlowControl> {
        Ok(self.flow_control)
    }

    fn parity(&self) -> serialport::Result<serialport::Parity> {
        Ok(self.parity)
    }

    fn stop_bits(&self) -> serialport::Result<serialport::StopBits> {
        Ok(self.stop_bits)
    }

    fn timeout(&self) -> std::time::Duration {
        self.timeout
    }

    fn set_baud_rate(&mut self, baud_rate: u32) -> serialport::Result<()> {
        self.baud = baud_rate;
        Ok(())
    }

    fn set_data_bits(&mut self, data_bits: serialport::DataBits) -> serialport::Result<()> {
        self.data_bits = data_bits;
        Ok(())
    }

    fn set_flow_control(&mut self, flow_control: serialport::FlowControl) -> serialport::Result<()> {
        self.flow_control = flow_control;
        Ok(())
    }

    fn set_parity(&mut self, parity: serialport::Parity) -> serialport::Result<()> {
        self.parity = parity;
        Ok(())
    }

    fn set_stop_bits(&mut self, stop_bits: serialport::StopBits) -> serialport::Result<()> {
        self.stop_bits = stop_bits;
        Ok(())
    }

    fn set_timeout(&mut self, timeout: std::time::Duration) -> serialport::Result<()> {
        self.timeout = timeout;
        Ok(())
    }

    fn write_request_to_send(&mut self, _level: bool) -> serialport::Result<()> {
        Ok(())
    }

    fn write_data_terminal_ready(&mut self, _level: bool) -> serialport::Result<()> {
        Ok(())
    }

    fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn bytes_to_read(&self) -> serialport::Result<u32> {
        Ok(0)
    }

    fn bytes_to_write(&self) -> serialport::Result<u32> {
        Ok(0)
    }

    fn clear(&self, _buffer_to_clear: serialport::ClearBuffer) -> serialport::Result<()> {
        Ok(())
    }

    fn try_clone(&self) -> serialport::Result<Box<dyn SerialPort>> {
        Ok(Box::new(*self))
    }

    fn set_break(&self) -> serialport::Result<()> {
        Ok(())
    }

    fn clear_break(&self) -> serialport::Result<()> {
        Ok(())
    }
}
