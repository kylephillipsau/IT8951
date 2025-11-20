//! Linux hardware implementations using spidev and gpio-cdev.

use crate::error::{Error, Result};
use crate::hal::{InputPin, OutputPin, PinState, SpiTransfer};
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};

/// Linux SPI device implementation.
#[derive(Debug)]
pub struct LinuxSpi {
    spi: Spidev,
}

impl LinuxSpi {
    /// Opens the SPI device at the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to SPI device (e.g., "/dev/spidev0.0")
    /// * `speed_hz` - SPI clock speed in Hz
    pub fn new(path: &str, speed_hz: u32) -> Result<Self> {
        let mut spi = Spidev::open(path).map_err(Error::Io)?;

        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(speed_hz)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build();

        spi.configure(&options).map_err(Error::Io)?;

        Ok(Self { spi })
    }

    /// Sets the SPI clock speed.
    pub fn set_speed(&mut self, speed_hz: u32) -> Result<()> {
        let options = SpidevOptions::new().max_speed_hz(speed_hz).build();
        self.spi.configure(&options).map_err(Error::Io)
    }
}

impl SpiTransfer for LinuxSpi {
    fn transfer_byte(&mut self, byte: u8) -> Result<u8> {
        let tx_buf = [byte];
        let mut rx_buf = [0u8; 1];
        {
            let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
            self.spi.transfer(&mut transfer).map_err(Error::Io)?;
        }
        Ok(rx_buf[0])
    }

    fn transfer(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let mut rx_buf = vec![0u8; data.len()];
        {
            let mut transfer = SpidevTransfer::read_write(data, &mut rx_buf);
            self.spi.transfer(&mut transfer).map_err(Error::Io)?;
        }
        Ok(rx_buf)
    }
}

/// Linux GPIO output pin implementation.
#[derive(Debug)]
pub struct LinuxOutputPin {
    handle: LineHandle,
}

impl LinuxOutputPin {
    /// Opens a GPIO pin as output.
    ///
    /// # Arguments
    ///
    /// * `chip` - GPIO chip path (e.g., "/dev/gpiochip0")
    /// * `pin` - GPIO pin number
    /// * `initial_state` - Initial pin state
    pub fn new(chip: &str, pin: u32, initial_state: PinState) -> Result<Self> {
        let mut gpio_chip = Chip::new(chip).map_err(|e| Error::Gpio(e.to_string()))?;

        let initial_value = match initial_state {
            PinState::High => 1,
            PinState::Low => 0,
        };

        let handle = gpio_chip
            .get_line(pin)
            .map_err(|e| Error::Gpio(e.to_string()))?
            .request(LineRequestFlags::OUTPUT, initial_value, "it8951")
            .map_err(|e| Error::Gpio(e.to_string()))?;

        Ok(Self { handle })
    }
}

impl OutputPin for LinuxOutputPin {
    fn set_high(&mut self) -> Result<()> {
        self.handle
            .set_value(1)
            .map_err(|e| Error::Gpio(e.to_string()))
    }

    fn set_low(&mut self) -> Result<()> {
        self.handle
            .set_value(0)
            .map_err(|e| Error::Gpio(e.to_string()))
    }

    fn toggle(&mut self) -> Result<()> {
        let current = self
            .handle
            .get_value()
            .map_err(|e| Error::Gpio(e.to_string()))?;
        self.handle
            .set_value(if current == 0 { 1 } else { 0 })
            .map_err(|e| Error::Gpio(e.to_string()))
    }
}

/// Linux GPIO input pin implementation.
#[derive(Debug)]
pub struct LinuxInputPin {
    handle: LineHandle,
}

impl LinuxInputPin {
    /// Opens a GPIO pin as input.
    ///
    /// # Arguments
    ///
    /// * `chip` - GPIO chip path (e.g., "/dev/gpiochip0")
    /// * `pin` - GPIO pin number
    pub fn new(chip: &str, pin: u32) -> Result<Self> {
        let mut gpio_chip = Chip::new(chip).map_err(|e| Error::Gpio(e.to_string()))?;

        let handle = gpio_chip
            .get_line(pin)
            .map_err(|e| Error::Gpio(e.to_string()))?
            .request(LineRequestFlags::INPUT, 0, "it8951")
            .map_err(|e| Error::Gpio(e.to_string()))?;

        Ok(Self { handle })
    }
}

impl InputPin for LinuxInputPin {
    fn is_high(&self) -> Result<bool> {
        self.handle
            .get_value()
            .map(|v| v == 1)
            .map_err(|e| Error::Gpio(e.to_string()))
    }

    fn is_low(&self) -> Result<bool> {
        self.handle
            .get_value()
            .map(|v| v == 0)
            .map_err(|e| Error::Gpio(e.to_string()))
    }
}

/// Default GPIO pin numbers for Waveshare e-Paper HAT.
pub mod pins {
    /// Chip Select pin (GPIO 8, active low)
    pub const CS: u32 = 8;

    /// Host Ready pin (GPIO 24, active high when ready)
    pub const HRDY: u32 = 24;

    /// Reset pin (GPIO 17, active low)
    pub const RST: u32 = 17;
}

/// Default SPI speeds.
pub mod speed {
    /// Speed for commands (1 MHz)
    pub const COMMAND_HZ: u32 = 1_000_000;

    /// Speed for data transfers (24 MHz)
    pub const DATA_HZ: u32 = 24_000_000;
}
