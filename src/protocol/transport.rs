//! Low-level IT8951 transport layer.
//!
//! This module implements the IT8951 SPI protocol with preambles,
//! hardware ready checks, and chip select control.

use crate::error::{Error, Result};
use crate::hal::{InputPin, OutputPin, SpiTransfer};
use crate::protocol::{Command, Register, UserCommand};
use byteorder::{BigEndian, ByteOrder};
use std::time::{Duration, Instant};

/// Preamble for writing command code (0x6000)
const PREAMBLE_WRITE_CMD: u16 = 0x6000;

/// Preamble for writing data (0x0000)
const PREAMBLE_WRITE_DATA: u16 = 0x0000;

/// Preamble for reading data (0x1000)
const PREAMBLE_READ_DATA: u16 = 0x1000;

/// Default timeout for waiting for hardware ready (5 seconds)
const DEFAULT_TIMEOUT_MS: u64 = 5000;

/// IT8951 transport layer.
///
/// Handles low-level SPI communication with proper preambles,
/// timing, and hardware control.
#[derive(Debug)]
pub struct Transport<SPI, HRDY, CS> {
    spi: SPI,
    hrdy: HRDY,
    cs: CS,
    timeout: Duration,
}

impl<SPI, HRDY, CS> Transport<SPI, HRDY, CS>
where
    SPI: SpiTransfer,
    HRDY: InputPin,
    CS: OutputPin,
{
    /// Creates a new transport with the given SPI and GPIO interfaces.
    pub fn new(spi: SPI, hrdy: HRDY, cs: CS) -> Self {
        Self {
            spi,
            hrdy,
            cs,
            timeout: Duration::from_millis(DEFAULT_TIMEOUT_MS),
        }
    }

    /// Sets the timeout for hardware ready waits.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Waits for the hardware ready pin to go high.
    ///
    /// Returns an error if the timeout is exceeded.
    fn wait_ready(&self) -> Result<()> {
        let start = Instant::now();

        while !self.hrdy.is_high()? {
            if start.elapsed() > self.timeout {
                return Err(Error::Timeout(self.timeout.as_millis() as u64));
            }
            // Small yield to prevent busy-waiting
            std::thread::yield_now();
        }

        Ok(())
    }

    /// Writes a 16-bit value as two bytes (big-endian).
    fn write_u16(&mut self, value: u16) -> Result<()> {
        let mut buf = [0u8; 2];
        BigEndian::write_u16(&mut buf, value);

        self.spi.transfer_byte(buf[0])?;
        self.spi.transfer_byte(buf[1])?;

        Ok(())
    }

    /// Reads a 16-bit value as two bytes (big-endian).
    fn read_u16(&mut self) -> Result<u16> {
        let high = self.spi.transfer_byte(0x00)?;
        let low = self.spi.transfer_byte(0x00)?;

        Ok(((high as u16) << 8) | (low as u16))
    }

    /// Writes a command code to the device.
    ///
    /// # Protocol
    /// 1. Wait for ready
    /// 2. Assert CS low
    /// 3. Send write command preamble (0x6000)
    /// 4. Wait for ready
    /// 5. Send command code
    /// 6. De-assert CS high
    pub fn write_command(&mut self, cmd: Command) -> Result<()> {
        self.wait_ready()?;

        self.cs.set_low()?;

        // Send preamble for write command
        self.write_u16(PREAMBLE_WRITE_CMD)?;

        self.wait_ready()?;

        // Send command code
        self.write_u16(cmd.as_u16())?;

        self.cs.set_high()?;

        Ok(())
    }

    /// Writes a user command code to the device.
    pub fn write_user_command(&mut self, cmd: UserCommand) -> Result<()> {
        self.wait_ready()?;

        self.cs.set_low()?;

        self.write_u16(PREAMBLE_WRITE_CMD)?;

        self.wait_ready()?;

        self.write_u16(cmd.as_u16())?;

        self.cs.set_high()?;

        Ok(())
    }

    /// Writes a 16-bit data value to the device.
    ///
    /// # Protocol
    /// 1. Wait for ready
    /// 2. Assert CS low
    /// 3. Send write data preamble (0x0000)
    /// 4. Wait for ready
    /// 5. Send data
    /// 6. De-assert CS high
    pub fn write_data(&mut self, data: u16) -> Result<()> {
        self.wait_ready()?;

        self.cs.set_low()?;

        // Send preamble for write data
        self.write_u16(PREAMBLE_WRITE_DATA)?;

        self.wait_ready()?;

        // Send data
        self.write_u16(data)?;

        self.cs.set_high()?;

        Ok(())
    }

    /// Writes multiple 16-bit data values to the device.
    ///
    /// More efficient than calling write_data repeatedly.
    pub fn write_data_batch(&mut self, data: &[u16]) -> Result<()> {
        self.wait_ready()?;

        self.cs.set_low()?;

        // Send preamble for write data
        self.write_u16(PREAMBLE_WRITE_DATA)?;

        self.wait_ready()?;

        // Send all data values
        for &value in data {
            self.write_u16(value)?;
        }

        self.cs.set_high()?;

        Ok(())
    }

    /// Reads a 16-bit data value from the device.
    ///
    /// # Protocol
    /// 1. Wait for ready
    /// 2. Assert CS low
    /// 3. Send read data preamble (0x1000)
    /// 4. Wait for ready
    /// 5. Send two dummy bytes
    /// 6. Wait for ready
    /// 7. Read data (2 bytes)
    /// 8. De-assert CS high
    pub fn read_data(&mut self) -> Result<u16> {
        self.wait_ready()?;

        self.cs.set_low()?;

        // Send preamble for read data
        self.write_u16(PREAMBLE_READ_DATA)?;

        self.wait_ready()?;

        // Send dummy bytes
        self.spi.transfer_byte(0x00)?;
        self.spi.transfer_byte(0x00)?;

        self.wait_ready()?;

        // Read data
        let data = self.read_u16()?;

        self.cs.set_high()?;

        Ok(data)
    }

    /// Reads multiple 16-bit data values from the device.
    ///
    /// More efficient than calling read_data repeatedly.
    pub fn read_data_batch(&mut self, count: usize) -> Result<Vec<u16>> {
        let mut result = Vec::with_capacity(count);

        self.wait_ready()?;

        self.cs.set_low()?;

        // Send preamble for read data
        self.write_u16(PREAMBLE_READ_DATA)?;

        self.wait_ready()?;

        // Send dummy bytes
        self.spi.transfer_byte(0x00)?;
        self.spi.transfer_byte(0x00)?;

        self.wait_ready()?;

        // Read all data values
        for _ in 0..count {
            result.push(self.read_u16()?);
        }

        self.cs.set_high()?;

        Ok(result)
    }

    /// Writes a command with arguments.
    ///
    /// Sends the command code followed by the argument data words.
    pub fn write_command_with_args(&mut self, cmd: Command, args: &[u16]) -> Result<()> {
        self.write_command(cmd)?;

        for &arg in args {
            self.write_data(arg)?;
        }

        Ok(())
    }

    /// Writes a user command with arguments.
    pub fn write_user_command_with_args(
        &mut self,
        cmd: UserCommand,
        args: &[u16],
    ) -> Result<()> {
        self.write_user_command(cmd)?;

        for &arg in args {
            self.write_data(arg)?;
        }

        Ok(())
    }

    /// Reads a register value.
    ///
    /// Sends a RegRead command with the register address, then reads the value.
    pub fn read_register(&mut self, reg: Register) -> Result<u16> {
        self.write_command(Command::RegRead)?;
        self.write_data(reg.addr())?;
        self.read_data()
    }

    /// Writes a register value.
    ///
    /// Sends a RegWrite command with the register address and value.
    pub fn write_register(&mut self, reg: Register, value: u16) -> Result<()> {
        self.write_command(Command::RegWrite)?;
        self.write_data(reg.addr())?;
        self.write_data(value)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal::mock::{MockInputPin, MockOutputPin, MockSpi};
    use crate::hal::PinState;

    fn setup_transport() -> Transport<MockSpi, MockInputPin, MockOutputPin> {
        let spi = MockSpi::new();
        let hrdy = MockInputPin::new(PinState::High); // Always ready for tests
        let cs = MockOutputPin::new(PinState::High);

        Transport::new(spi, hrdy, cs)
    }

    #[test]
    fn test_write_command() {
        let mut transport = setup_transport();

        transport.write_command(Command::SysRun).unwrap();

        // Verify CS was toggled
        let history = transport.cs.get_history();
        assert!(history.contains(&PinState::Low));
        assert!(history.contains(&PinState::High));

        // Verify SPI transfers
        let transfers = transport.spi.get_transfers();
        assert!(!transfers.is_empty());
    }

    #[test]
    fn test_write_data() {
        let mut transport = setup_transport();

        transport.write_data(0x1234).unwrap();

        let transfers = transport.spi.get_transfers();
        assert!(!transfers.is_empty());
    }

    #[test]
    fn test_read_data() {
        let mut transport = setup_transport();

        // Set up response: preamble (2 bytes) + dummy (2 bytes) + data (2 bytes)
        transport
            .spi
            .add_response(vec![0x00, 0x00, 0x00, 0x00, 0x12, 0x34]);

        let result = transport.read_data().unwrap();

        // Should read the last 2 bytes as 0x1234
        assert_eq!(result, 0x1234);
    }

    #[test]
    fn test_timeout() {
        let spi = MockSpi::new();
        let hrdy = MockInputPin::new(PinState::Low); // Never ready
        let cs = MockOutputPin::new(PinState::High);

        let mut transport = Transport::new(spi, hrdy, cs);
        transport.set_timeout(Duration::from_millis(10));

        // This should timeout
        let result = transport.wait_ready();
        assert!(matches!(result, Err(Error::Timeout(_))));
    }

    #[test]
    fn test_write_data_batch() {
        let mut transport = setup_transport();

        let data = vec![0x1111, 0x2222, 0x3333];
        transport.write_data_batch(&data).unwrap();

        let transfers = transport.spi.get_transfers();
        assert!(!transfers.is_empty());
    }

    #[test]
    fn test_write_command_with_args() {
        let mut transport = setup_transport();

        let args = vec![0x1234, 0x5678];
        transport
            .write_command_with_args(Command::RegWrite, &args)
            .unwrap();

        let transfers = transport.spi.get_transfers();
        // Should have command + args transfers
        assert!(transfers.len() > 1);
    }

    #[test]
    fn test_read_register() {
        let mut transport = setup_transport();

        // RegRead command: cmd preamble (2) + cmd (2) + addr preamble (2) + addr (2)
        // + read preamble (2) + dummy (2) + data (2) = 14 bytes total
        transport.spi.add_response(vec![
            0x00, 0x00, 0x00, 0x00, // command
            0x00, 0x00, 0x00, 0x00, // address
            0x00, 0x00, 0x00, 0x00, // read preamble + dummy
            0x12, 0x34, // actual data
        ]);

        let result = transport.read_register(Register::I80CPCR).unwrap();
        assert_eq!(result, 0x1234);
    }

    #[test]
    fn test_write_register() {
        let mut transport = setup_transport();

        transport
            .write_register(Register::I80CPCR, 0xABCD)
            .unwrap();

        let transfers = transport.spi.get_transfers();
        assert!(transfers.len() >= 2); // Command + address + value
    }
}
