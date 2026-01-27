//! SPI interface abstraction.

use crate::error::Result;

/// Trait for SPI data transfer operations.
pub trait SpiTransfer {
    /// Transfers a single byte over SPI.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to transfer
    ///
    /// # Returns
    ///
    /// The byte received during the transfer.
    fn transfer_byte(&mut self, byte: u8) -> Result<u8>;

    /// Transfers multiple bytes over SPI.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Buffer containing bytes to transfer
    ///
    /// # Returns
    ///
    /// Vector of bytes received during the transfer.
    fn transfer(&mut self, buffer: &[u8]) -> Result<Vec<u8>>;

    /// Sets the SPI clock speed in Hz.
    ///
    /// Used to switch between slower command speed and faster data transfer speed.
    fn set_speed(&mut self, _speed_hz: u32) -> Result<()> {
        Ok(()) // Default no-op
    }
}

/// Trait for SPI interface configuration and control.
pub trait SpiInterface: SpiTransfer {
    /// Sets the SPI clock frequency in Hz.
    fn set_clock_hz(&mut self, hz: u32) -> Result<()>;

    /// Gets the current SPI clock frequency in Hz.
    fn clock_hz(&self) -> u32;

    /// Sets the SPI mode (CPOL and CPHA).
    ///
    /// IT8951 uses Mode 0 (CPOL=0, CPHA=0).
    fn set_mode(&mut self, mode: SpiMode) -> Result<()>;

    /// Sets the bit order (MSB or LSB first).
    ///
    /// IT8951 uses MSB first.
    fn set_bit_order(&mut self, order: BitOrder) -> Result<()>;
}

/// SPI mode configuration (CPOL and CPHA).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiMode {
    /// Mode 0: CPOL=0, CPHA=0
    Mode0,
    /// Mode 1: CPOL=0, CPHA=1
    Mode1,
    /// Mode 2: CPOL=1, CPHA=0
    Mode2,
    /// Mode 3: CPOL=1, CPHA=1
    Mode3,
}

impl SpiMode {
    /// Returns the mode number (0-3).
    pub fn mode_number(&self) -> u8 {
        match self {
            SpiMode::Mode0 => 0,
            SpiMode::Mode1 => 1,
            SpiMode::Mode2 => 2,
            SpiMode::Mode3 => 3,
        }
    }
}

/// Bit order for SPI transfers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitOrder {
    /// Most significant bit first
    MsbFirst,
    /// Least significant bit first
    LsbFirst,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spi_mode_number() {
        assert_eq!(SpiMode::Mode0.mode_number(), 0);
        assert_eq!(SpiMode::Mode1.mode_number(), 1);
        assert_eq!(SpiMode::Mode2.mode_number(), 2);
        assert_eq!(SpiMode::Mode3.mode_number(), 3);
    }
}
