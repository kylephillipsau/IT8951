//! IT8951 device management.
//!
//! This module provides the main `IT8951` device struct and associated
//! management operations including initialization, VCOM configuration,
//! and power state control.

mod builder;

pub use builder::IT8951Builder;

use crate::error::{Error, Result};
use crate::hal::{InputPin, OutputPin, SpiTransfer};
use crate::protocol::{Command, Register, Transport, UserCommand};
use crate::types::DeviceInfo;
use std::time::Duration;

/// Main IT8951 e-paper display controller.
///
/// This struct manages the IT8951 device, providing high-level operations
/// for display control, VCOM management, and power state control.
///
/// # Examples
///
/// ```ignore
/// use it8951::IT8951;
///
/// // Create using builder pattern (Phase 3+)
/// let mut display = IT8951::builder()
///     .spi_device("/dev/spidev0.0")?
///     .build()?;
///
/// display.init()?;
/// println!("Panel: {}x{}", display.width(), display.height());
/// ```
#[derive(Debug)]
pub struct IT8951<SPI, HRDY, CS, RESET> {
    pub(crate) transport: Transport<SPI, HRDY, CS>,
    reset: RESET,
    pub(crate) device_info: Option<DeviceInfo>,
    vcom: u16,
}

impl<SPI, HRDY, CS, RESET> IT8951<SPI, HRDY, CS, RESET>
where
    SPI: SpiTransfer,
    HRDY: InputPin,
    CS: OutputPin,
    RESET: OutputPin,
{
    /// Creates a new IT8951 device.
    ///
    /// Use the builder pattern via `IT8951::builder()` for easier construction.
    pub fn new(spi: SPI, hrdy: HRDY, cs: CS, reset: RESET, vcom: u16) -> Self {
        Self {
            transport: Transport::new(spi, hrdy, cs),
            reset,
            device_info: None,
            vcom,
        }
    }

    /// Creates a new builder for configuring the IT8951 device.
    pub fn builder() -> IT8951Builder {
        IT8951Builder::new()
    }

    /// Initializes the IT8951 device.
    ///
    /// This performs a hardware reset, retrieves device information,
    /// and configures the VCOM voltage.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Hardware reset fails
    /// - Device info retrieval fails
    /// - VCOM configuration fails
    pub fn init(&mut self) -> Result<()> {
        // Perform hardware reset
        self.reset()?;

        // Wait for device to be ready after reset
        std::thread::sleep(Duration::from_millis(100));

        // Get device information
        let device_info = self.get_device_info()?;
        let img_buf_addr = device_info.img_buf_addr;
        self.device_info = Some(device_info);

        // Set image buffer base address (required before any image operations)
        let addr_high = (img_buf_addr >> 16) as u16;
        let addr_low = (img_buf_addr & 0xFFFF) as u16;
        self.transport.write_register(Register::new(0x020A), addr_high)?;
        self.transport.write_register(Register::new(0x0208), addr_low)?;

        // Enable I80 packed mode
        self.transport
            .write_register(Register::I80CPCR, 0x0001)?;

        // Configure VCOM if different from current value
        let current_vcom = self.read_vcom()?;
        if current_vcom != self.vcom {
            self.write_vcom(self.vcom)?;
        }

        Ok(())
    }

    /// Performs a hardware reset of the IT8951.
    ///
    /// Toggles the RESET pin low for 100ms, then high.
    pub fn reset(&mut self) -> Result<()> {
        self.reset.set_low()?;
        std::thread::sleep(Duration::from_millis(100));
        self.reset.set_high()?;
        Ok(())
    }

    /// Retrieves device information from the IT8951.
    ///
    /// Returns panel dimensions, firmware version, LUT version, etc.
    pub fn get_device_info(&mut self) -> Result<DeviceInfo> {
        // Send get device info command
        self.transport
            .write_user_command(UserCommand::GetDevInfo)?;

        // Read device info structure (20 words = 40 bytes)
        let data = self.transport.read_data_batch(20)?;

        DeviceInfo::from_raw(&data)
    }

    /// Returns the device information.
    ///
    /// Returns `None` if `init()` has not been called yet.
    pub fn device_info(&self) -> Option<&DeviceInfo> {
        self.device_info.as_ref()
    }

    /// Returns the panel width in pixels.
    ///
    /// # Panics
    ///
    /// Panics if called before `init()`.
    pub fn width(&self) -> u16 {
        self.device_info
            .as_ref()
            .expect("init() must be called first")
            .panel_width
    }

    /// Returns the panel height in pixels.
    ///
    /// # Panics
    ///
    /// Panics if called before `init()`.
    pub fn height(&self) -> u16 {
        self.device_info
            .as_ref()
            .expect("init() must be called first")
            .panel_height
    }

    /// Returns the image buffer base address.
    ///
    /// # Panics
    ///
    /// Panics if called before `init()`.
    pub fn img_buf_addr(&self) -> u32 {
        self.device_info
            .as_ref()
            .expect("init() must be called first")
            .img_buf_addr
    }

    /// Reads the current VCOM value from the device.
    pub fn read_vcom(&mut self) -> Result<u16> {
        self.transport.write_user_command(UserCommand::Vcom)?;
        self.transport.write_data(0)?; // 0 = read
        self.transport.read_data()
    }

    /// Writes the VCOM value to the device.
    ///
    /// # Arguments
    ///
    /// * `vcom` - VCOM value (e.g., 1500 for -1.50V)
    ///
    /// # Errors
    ///
    /// Returns an error if the VCOM value is out of range (0-5000).
    pub fn write_vcom(&mut self, vcom: u16) -> Result<()> {
        if vcom > 5000 {
            return Err(Error::InvalidVcom(vcom));
        }

        self.transport.write_user_command(UserCommand::Vcom)?;
        self.transport.write_data(1)?; // 1 = write
        self.transport.write_data(vcom)?;

        self.vcom = vcom;

        Ok(())
    }

    /// Gets the configured VCOM value.
    pub fn vcom(&self) -> u16 {
        self.vcom
    }

    /// Puts the device into system run mode.
    pub fn run(&mut self) -> Result<()> {
        self.transport.write_command(Command::SysRun)
    }

    /// Puts the device into standby mode (low power).
    pub fn standby(&mut self) -> Result<()> {
        self.transport.write_command(Command::Standby)
    }

    /// Puts the device into sleep mode (lowest power).
    pub fn sleep(&mut self) -> Result<()> {
        self.transport.write_command(Command::Sleep)
    }

    /// Waits for the display to be ready.
    ///
    /// Polls the LUTAFSR register until all LUT engines are free.
    pub fn wait_display_ready(&mut self) -> Result<()> {
        loop {
            let status = self.transport.read_register(Register::LUTAFSR)?;
            if status == 0 {
                break;
            }
            std::thread::yield_now();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal::mock::{MockInputPin, MockOutputPin, MockSpi};
    use crate::hal::PinState;

    fn setup_device() -> IT8951<MockSpi, MockInputPin, MockOutputPin, MockOutputPin> {
        let spi = MockSpi::new();
        let hrdy = MockInputPin::new(PinState::High);
        let cs = MockOutputPin::new(PinState::High);
        let reset = MockOutputPin::new(PinState::High);

        IT8951::new(spi, hrdy, cs, reset, 1500)
    }

    #[test]
    fn test_device_creation() {
        let device = setup_device();
        assert_eq!(device.vcom(), 1500);
        assert!(device.device_info().is_none());
    }

    #[test]
    fn test_reset() {
        let mut device = setup_device();
        device.reset().unwrap();

        let history = device.reset.get_history();
        assert!(history.contains(&PinState::Low));
        assert!(history.contains(&PinState::High));
    }

    #[test]
    fn test_vcom_validation() {
        let mut device = setup_device();

        // Valid VCOM
        assert!(device.write_vcom(1500).is_ok());

        // Invalid VCOM (too high)
        assert!(matches!(
            device.write_vcom(6000),
            Err(Error::InvalidVcom(6000))
        ));
    }

    #[test]
    fn test_power_state_commands() {
        let mut device = setup_device();

        assert!(device.run().is_ok());
        assert!(device.standby().is_ok());
        assert!(device.sleep().is_ok());
    }
}
