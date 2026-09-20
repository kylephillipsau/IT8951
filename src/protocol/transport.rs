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
    command_speed_hz: u32,
    data_speed_hz: u32,
    /// Reusable transmit buffer for bulk pixel writes.
    tx_buf: Vec<u8>,
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
            command_speed_hz: 0,
            data_speed_hz: 0,
            tx_buf: Vec::new(),
        }
    }

    /// Sets the SPI speeds for command and data transfers.
    ///
    /// When both are non-zero, the transport will switch to `data_speed_hz`
    /// for bulk data transfers and back to `command_speed_hz` afterward.
    pub fn set_speeds(&mut self, command_speed_hz: u32, data_speed_hz: u32) {
        self.command_speed_hz = command_speed_hz;
        self.data_speed_hz = data_speed_hz;
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
        let mut polls: u32 = 0;

        while !self.hrdy.is_high()? {
            if start.elapsed() > self.timeout {
                return Err(Error::Timeout(self.timeout.as_millis() as u64));
            }
            // Command acks arrive within tens of microseconds, so spin briefly first.
            // The IT8951 holds HRDY low for the whole waveform (200-500 ms) while a
            // display update runs, so back off to sleeping rather than pinning a core.
            polls += 1;
            if polls < 64 {
                std::thread::yield_now();
            } else if polls < 256 {
                std::thread::sleep(Duration::from_micros(50));
            } else {
                std::thread::sleep(Duration::from_micros(500));
            }
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
    /// # Protocol
    /// 1. Wait for ready
    /// 2. Send preamble (0x6000) + command in one transfer
    pub fn write_command(&mut self, cmd: Command) -> Result<()> {
        self.wait_ready()?;
        self.write_words(&[PREAMBLE_WRITE_CMD, cmd.as_u16()])?;
        Ok(())
    }

    /// Writes a user command code to the device.
    pub fn write_user_command(&mut self, cmd: UserCommand) -> Result<()> {
        self.wait_ready()?;
        self.write_words(&[PREAMBLE_WRITE_CMD, cmd.as_u16()])?;
        Ok(())
    }

    /// Writes a 16-bit data value to the device.
    ///
    /// # Protocol
    /// 1. Wait for ready
    /// 2. Send preamble (0x0000) + data in one transfer
    pub fn write_data(&mut self, data: u16) -> Result<()> {
        self.wait_ready()?;
        self.write_words(&[PREAMBLE_WRITE_DATA, data])?;
        Ok(())
    }

    /// Writes multiple 16-bit data values to the device.
    ///
    /// Sends preamble + data in chunks, keeping each chunk in a single CS session.
    pub fn write_data_batch(&mut self, data: &[u16]) -> Result<()> {
        let use_fast_speed = self.data_speed_hz > 0 && self.command_speed_hz > 0;
        if use_fast_speed {
            self.spi.set_speed(self.data_speed_hz)?;
        }

        let result = self.write_data_batch_inner(data);

        if use_fast_speed {
            self.spi.set_speed(self.command_speed_hz)?;
        }

        result
    }

    /// Writes packed pixel bytes as the IT8951 expects them in little-endian mode.
    ///
    /// Each pair of bytes forms one 16-bit word with the first byte in the low half,
    /// so the SPI stream (MSB first) is the input with every byte pair swapped. The
    /// data is copied once into a reusable transmit buffer, chunked to the spidev
    /// buffer size, with a data preamble at the start of every chunk.
    pub fn write_pixel_bytes(&mut self, data: &[u8]) -> Result<()> {
        let use_fast_speed = self.data_speed_hz > 0 && self.command_speed_hz > 0;
        if use_fast_speed {
            self.spi.set_speed(self.data_speed_hz)?;
        }

        let result = self.write_pixel_bytes_inner(data);

        if use_fast_speed {
            self.spi.set_speed(self.command_speed_hz)?;
        }

        result
    }

    fn write_pixel_bytes_inner(&mut self, data: &[u8]) -> Result<()> {
        // Preamble (2 bytes) + payload must fit in one spidev transfer (65536 bytes).
        const MAX_CHUNK_BYTES: usize = 65534;

        let mut tx = std::mem::take(&mut self.tx_buf);
        let mut offset = 0;
        let mut result = Ok(());
        while offset < data.len() {
            let chunk = &data[offset..(offset + MAX_CHUNK_BYTES).min(data.len())];
            tx.clear();
            tx.extend_from_slice(&PREAMBLE_WRITE_DATA.to_be_bytes());
            for pair in chunk.chunks(2) {
                if pair.len() == 2 {
                    tx.push(pair[1]);
                    tx.push(pair[0]);
                } else {
                    tx.push(0);
                    tx.push(pair[0]);
                }
            }
            result = self.wait_ready().and_then(|_| self.spi.transfer(&tx).map(|_| ()));
            if result.is_err() {
                break;
            }
            offset += chunk.len();
        }
        self.tx_buf = tx;
        result
    }

    fn write_data_batch_inner(&mut self, data: &[u16]) -> Result<()> {
        const MAX_CHUNK_WORDS: usize = 32767;

        self.wait_ready()?;

        // First chunk includes preamble
        let first_chunk_size = data.len().min(MAX_CHUNK_WORDS);
        let mut words = Vec::with_capacity(first_chunk_size + 1);
        words.push(PREAMBLE_WRITE_DATA);
        words.extend_from_slice(&data[..first_chunk_size]);
        self.write_words(&words)?;

        // Remaining chunks also need preamble for each new CS session
        let mut offset = first_chunk_size;
        while offset < data.len() {
            self.wait_ready()?;
            let chunk_size = (data.len() - offset).min(MAX_CHUNK_WORDS);
            let mut chunk = Vec::with_capacity(chunk_size + 1);
            chunk.push(PREAMBLE_WRITE_DATA);
            chunk.extend_from_slice(&data[offset..offset + chunk_size]);
            self.write_words(&chunk)?;
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

        // Verify SPI transfer contains preamble + command in one transfer
        let transfers = transport.spi.get_transfers();
        assert_eq!(transfers.len(), 1);
        // Should be 4 bytes: preamble (0x6000) + command
        assert_eq!(transfers[0].len(), 4);
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
    fn test_write_pixel_bytes_swaps_pairs_and_prefixes_preamble() {
        let mut transport = setup_transport();

        transport.write_pixel_bytes(&[0x11, 0x22, 0x33]).unwrap();

        let transfers = transport.spi.get_transfers();
        assert_eq!(transfers.len(), 1);
        // preamble 0x0000, then (0x22,0x11), then odd trailing byte as (0x00,0x33)
        assert_eq!(transfers[0], vec![0x00, 0x00, 0x22, 0x11, 0x00, 0x33]);
    }

    #[test]
    fn test_write_pixel_bytes_chunks_with_preamble_each() {
        let mut transport = setup_transport();

        let data = vec![0xABu8; 65534 + 10];
        transport.write_pixel_bytes(&data).unwrap();

        let transfers = transport.spi.get_transfers();
        assert_eq!(transfers.len(), 2);
        assert_eq!(transfers[0].len(), 65536);
        assert_eq!(transfers[1].len(), 12);
        assert_eq!(&transfers[1][..2], &[0x00, 0x00]);
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

        // Each operation is a separate transfer:
        // 1. Command (preamble + RegRead) - 4 bytes
        // 2. Address (preamble + addr) - 4 bytes
        // 3. Read (preamble + dummy + data) - 6 bytes
        transport.spi.add_response(vec![0x00; 4]); // command response
        transport.spi.add_response(vec![0x00; 4]); // address response
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
