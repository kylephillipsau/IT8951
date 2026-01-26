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
    #[allow(dead_code)]
    cs: CS, // Kept for future manual CS support; currently SPI driver handles CS
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

    /// Writes a buffer of 16-bit values in a single SPI transfer.
    /// The SPI driver handles CS automatically per transfer.
    fn write_words(&mut self, words: &[u16]) -> Result<()> {
        let mut buf = vec![0u8; words.len() * 2];
        for (i, &word) in words.iter().enumerate() {
            BigEndian::write_u16(&mut buf[i * 2..], word);
        }
        self.spi.transfer(&buf)?;
        Ok(())
    }

    /// Writes a command code to the device.
    ///
    /// # Protocol (matching Waveshare C reference)
    /// 1. Wait for ready
    /// 2. Send preamble (0x6000)
    /// 3. Wait for ready again
    /// 4. Send command code
    pub fn write_command(&mut self, cmd: Command) -> Result<()> {
        self.wait_ready()?;
        self.write_words(&[PREAMBLE_WRITE_CMD])?;
        self.wait_ready()?;
        self.write_words(&[cmd.as_u16()])?;
        Ok(())
    }

    /// Writes a user command code to the device.
    pub fn write_user_command(&mut self, cmd: UserCommand) -> Result<()> {
        self.wait_ready()?;
        self.write_words(&[PREAMBLE_WRITE_CMD])?;
        self.wait_ready()?;
        self.write_words(&[cmd.as_u16()])?;
        Ok(())
    }

    /// Writes a 16-bit data value to the device.
    ///
    /// # Protocol (matching Waveshare C reference)
    /// 1. Wait for ready
    /// 2. Send preamble (0x0000)
    /// 3. Wait for ready again
    /// 4. Send data
    pub fn write_data(&mut self, data: u16) -> Result<()> {
        self.wait_ready()?;
        self.write_words(&[PREAMBLE_WRITE_DATA])?;
        self.wait_ready()?;
        self.write_words(&[data])?;
        Ok(())
    }

    /// Writes multiple 16-bit data values to the device.
    ///
    /// Following the IT8951 protocol: send preamble, wait for ready, then send data.
    /// This matches the Waveshare C reference implementation which waits after
    /// the preamble before sending pixel data.
    pub fn write_data_batch(&mut self, data: &[u16]) -> Result<()> {
        const MAX_CHUNK_WORDS: usize = 2048; // Max words per transfer

        // Send preamble first
        self.wait_ready()?;
        self.write_words(&[PREAMBLE_WRITE_DATA])?;

        // Wait for device to be ready after preamble (critical for IT8951)
        self.wait_ready()?;

        // Send data in chunks
        let mut offset = 0;
        while offset < data.len() {
            let chunk_size = (data.len() - offset).min(MAX_CHUNK_WORDS);
            self.write_words(&data[offset..offset + chunk_size])?;
            offset += chunk_size;
        }

        Ok(())
    }

    /// Reads a 16-bit data value from the device.
    ///
    /// Sends preamble + dummy bytes and reads response in one transfer.
    pub fn read_data(&mut self) -> Result<u16> {
        self.wait_ready()?;

        // Send preamble + dummy bytes, receive data
        // Format: [preamble_hi, preamble_lo, dummy, dummy, data_hi, data_lo]
        let tx = [
            (PREAMBLE_READ_DATA >> 8) as u8,
            (PREAMBLE_READ_DATA & 0xFF) as u8,
            0x00, 0x00, // dummy bytes
            0x00, 0x00, // will be read
        ];
        let rx = self.spi.transfer(&tx)?;

        // Data is in last 2 bytes
        let data = ((rx[4] as u16) << 8) | (rx[5] as u16);
        Ok(data)
    }

    /// Reads multiple 16-bit data values from the device.
    ///
    /// Sends preamble + dummy bytes and reads all data in one transfer.
    pub fn read_data_batch(&mut self, count: usize) -> Result<Vec<u16>> {
        self.wait_ready()?;

        // Build transmit buffer: preamble + dummy + space for data
        let tx_len = 2 + 2 + count * 2; // preamble + dummy + data
        let mut tx = vec![0u8; tx_len];
        tx[0] = (PREAMBLE_READ_DATA >> 8) as u8;
        tx[1] = (PREAMBLE_READ_DATA & 0xFF) as u8;

        let rx = self.spi.transfer(&tx)?;

        // Parse data from response (starts at byte 4)
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let offset = 4 + i * 2;
            let word = ((rx[offset] as u16) << 8) | (rx[offset + 1] as u16);
            result.push(word);
        }

        Ok(result)
    }

    /// Writes a command with arguments.
    ///
    /// Sends the command code followed by each argument with its own preamble.
    pub fn write_command_with_args(&mut self, cmd: Command, args: &[u16]) -> Result<()> {
        self.write_command(cmd)?;
        for &arg in args {
            self.write_data(arg)?;
        }
        Ok(())
    }

    /// Writes a user command with arguments.
    ///
    /// Sends the command code followed by each argument with its own preamble.
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

        // Verify SPI transfers: preamble (2 bytes) then command (2 bytes)
        // Protocol: wait -> preamble -> wait -> command
        let transfers = transport.spi.get_transfers();
        assert_eq!(transfers.len(), 2);
        assert_eq!(transfers[0].len(), 2); // Preamble
        assert_eq!(transfers[1].len(), 2); // Command
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

        // With new protocol (wait after preamble):
        // 1. write_command(RegRead): preamble (2 bytes), then command (2 bytes)
        // 2. write_data(addr): preamble (2 bytes), then data (2 bytes)
        // 3. read_data(): preamble + dummy + data (6 bytes)
        transport.spi.add_response(vec![0x00; 2]); // command preamble
        transport.spi.add_response(vec![0x00; 2]); // command code
        transport.spi.add_response(vec![0x00; 2]); // data preamble
        transport.spi.add_response(vec![0x00; 2]); // address value
        transport.spi.add_response(vec![0x00, 0x00, 0x00, 0x00, 0x12, 0x34]); // read response

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
