//! Display operations for IT8951.
//!
//! This module implements display-related operations including clearing,
//! refreshing, and loading image data to the e-paper display.

use crate::device::IT8951;
use crate::error::{Error, Result};
use crate::hal::{InputPin, OutputPin, SpiTransfer};
use crate::protocol::{Command, UserCommand};
use crate::types::{Area, DisplayMode, Endian, LoadImageInfo, PixelFormat, Rotation};

impl<SPI, HRDY, CS, RESET> IT8951<SPI, HRDY, CS, RESET>
where
    SPI: SpiTransfer,
    HRDY: InputPin,
    CS: OutputPin,
    RESET: OutputPin,
{
    /// Clears the entire display to the specified grayscale value.
    ///
    /// This fills the frame buffer with the given value and optionally
    /// refreshes the display.
    ///
    /// # Arguments
    ///
    /// * `value` - Grayscale value (0x00 = black, 0xFF = white)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// display.clear(0xFF)?; // Clear to white
    /// display.refresh(DisplayMode::Init)?;
    /// ```
    pub fn clear(&mut self, value: u8) -> Result<()> {
        let device_info = self
            .device_info
            .as_ref()
            .ok_or_else(|| Error::Init("Device not initialized".to_string()))?;

        let area = Area::new(0, 0, device_info.panel_width, device_info.panel_height);

        self.fill_area(&area, value)
    }

    /// Fills a rectangular area with a solid grayscale value.
    ///
    /// # Arguments
    ///
    /// * `area` - The area to fill
    /// * `value` - Grayscale value to fill with
    pub fn fill_area(&mut self, area: &Area, value: u8) -> Result<()> {
        let device_info = self
            .device_info
            .as_ref()
            .ok_or_else(|| Error::Init("Device not initialized".to_string()))?;

        // Validate area
        if !area.is_valid(device_info.panel_width, device_info.panel_height) {
            return Err(Error::InvalidArea(*area));
        }

        // Create load image info
        let load_info = LoadImageInfo {
            endian: Endian::Little,
            pixel_format: PixelFormat::Bpp8,
            rotate: Rotation::Rotate0,
            start_fb_addr: 0, // Not used for fill
            img_buf_base_addr: device_info.img_buf_addr,
        };

        // Load image area command
        self.load_image_area_start(&load_info, area)?;

        // Wait for device to be ready before sending pixel data
        self.wait_display_ready()?;

        // Write the fill data (packed pixels)
        let pixels_per_word = 2; // 8bpp = 2 pixels per 16-bit word
        let total_pixels = area.pixel_count();
        let num_words = (total_pixels + pixels_per_word - 1) / pixels_per_word;

        // Pack two pixels into each 16-bit word
        let packed_value = ((value as u16) << 8) | (value as u16);

        let mut words = vec![packed_value; num_words];

        // Handle odd pixel count
        if total_pixels % 2 == 1 {
            if let Some(last) = words.last_mut() {
                *last = value as u16; // Only one pixel in last word
            }
        }

        self.transport.write_data_batch(&words)?;

        // End load image
        self.transport.write_command(Command::LoadImageEnd)?;

        Ok(())
    }

    /// Starts loading an image area.
    fn load_image_area_start(&mut self, load_info: &LoadImageInfo, area: &Area) -> Result<()> {
        // Set the image buffer base address (LISAR register)
        // Split 32-bit address into two 16-bit words
        let addr = load_info.img_buf_base_addr;
        let addr_high = (addr >> 16) as u16;
        let addr_low = (addr & 0xFFFF) as u16;

        // Write to LISAR+2 (high word) then LISAR (low word)
        // This order matches the C implementation
        self.transport.write_register(crate::protocol::Register::new(0x020A), addr_high)?;
        self.transport.write_register(crate::protocol::Register::new(0x0208), addr_low)?;

        // Build argument word
        let arg = ((load_info.endian.as_u16()) << 8)
            | ((load_info.pixel_format.as_u16()) << 4)
            | (load_info.rotate.as_u16());

        let args = [arg, area.x, area.y, area.width, area.height];

        self.transport
            .write_command_with_args(Command::LoadImageArea, &args)?;

        Ok(())
    }

    /// Refreshes the entire display with the specified mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - Display refresh mode
    ///
    /// # Examples
    ///
    /// ```ignore
    /// display.refresh(DisplayMode::Gc16)?; // High quality grayscale
    /// display.refresh(DisplayMode::Du)?;   // Fast monochrome
    /// ```
    pub fn refresh(&mut self, mode: DisplayMode) -> Result<()> {
        let device_info = self
            .device_info
            .as_ref()
            .ok_or_else(|| Error::Init("Device not initialized".to_string()))?;

        let area = Area::new(0, 0, device_info.panel_width, device_info.panel_height);

        self.refresh_area(&area, mode)
    }

    /// Refreshes a specific area of the display.
    ///
    /// # Arguments
    ///
    /// * `area` - The area to refresh
    /// * `mode` - Display refresh mode
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let area = Area::new(100, 100, 200, 200);
    /// display.refresh_area(&area, DisplayMode::Du)?;
    /// ```
    pub fn refresh_area(&mut self, area: &Area, mode: DisplayMode) -> Result<()> {
        let device_info = self
            .device_info
            .as_ref()
            .ok_or_else(|| Error::Init("Device not initialized".to_string()))?;

        // Validate area
        if !area.is_valid(device_info.panel_width, device_info.panel_height) {
            return Err(Error::InvalidArea(*area));
        }

        // Send display area command
        let args = [area.x, area.y, area.width, area.height, mode.as_u16()];

        self.transport
            .write_user_command_with_args(UserCommand::DisplayArea, &args)?;

        Ok(())
    }

    /// Loads image data into a specific area of the display buffer.
    ///
    /// The image data should be in the specified pixel format, with pixels
    /// packed according to the format (2 pixels per word for 8bpp).
    ///
    /// # Arguments
    ///
    /// * `data` - Image pixel data (grayscale, packed)
    /// * `area` - Destination area on display
    /// * `format` - Pixel format of the image data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut image_data = vec![0x80; 400]; // 20x20 image
    /// let area = Area::new(0, 0, 20, 20);
    /// display.load_image(&image_data, &area, PixelFormat::Bpp8)?;
    /// ```
    pub fn load_image(&mut self, data: &[u8], area: &Area, format: PixelFormat) -> Result<()> {
        let device_info = self
            .device_info
            .as_ref()
            .ok_or_else(|| Error::Init("Device not initialized".to_string()))?;

        // Validate area
        if !area.is_valid(device_info.panel_width, device_info.panel_height) {
            return Err(Error::InvalidArea(*area));
        }

        // Validate data size
        let expected_size = match format {
            PixelFormat::Bpp8 => area.pixel_count(),
            PixelFormat::Bpp4 => (area.pixel_count() + 1) / 2,
            PixelFormat::Bpp3 => (area.pixel_count() * 3 + 7) / 8,
            PixelFormat::Bpp2 => (area.pixel_count() + 3) / 4,
        };

        if data.len() < expected_size {
            return Err(Error::InvalidDimensions(
                data.len() as u16,
                expected_size as u16,
            ));
        }

        // Create load image info
        let load_info = LoadImageInfo {
            endian: Endian::Little,
            pixel_format: format,
            rotate: Rotation::Rotate0,
            start_fb_addr: 0,
            img_buf_base_addr: device_info.img_buf_addr,
        };

        // Start load image area
        self.load_image_area_start(&load_info, area)?;

        // Convert bytes to 16-bit words for transfer
        let mut words = Vec::with_capacity((data.len() + 1) / 2);
        for chunk in data.chunks(2) {
            let word = if chunk.len() == 2 {
                ((chunk[1] as u16) << 8) | (chunk[0] as u16)
            } else {
                chunk[0] as u16
            };
            words.push(word);
        }

        self.transport.write_data_batch(&words)?;

        // End load image
        self.transport.write_command(Command::LoadImageEnd)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal::mock::{MockInputPin, MockOutputPin, MockSpi};
    use crate::hal::PinState;

    fn setup_initialized_device() -> IT8951<MockSpi, MockInputPin, MockOutputPin, MockOutputPin> {
        let spi = MockSpi::new();
        let hrdy = MockInputPin::new(PinState::High);
        let cs = MockOutputPin::new(PinState::High);
        let reset = MockOutputPin::new(PinState::High);

        let mut device = IT8951::new(spi, hrdy, cs, reset, 1500);

        // Manually set device info to simulate initialization
        device.device_info = Some(crate::types::DeviceInfo {
            panel_width: 800,
            panel_height: 600,
            img_buf_addr: 0x001236E0,
            fw_version: "test".to_string(),
            lut_version: "test".to_string(),
        });

        device
    }

    #[test]
    fn test_clear() {
        let mut device = setup_initialized_device();
        assert!(device.clear(0xFF).is_ok());
    }

    #[test]
    fn test_clear_without_init() {
        let spi = MockSpi::new();
        let hrdy = MockInputPin::new(PinState::High);
        let cs = MockOutputPin::new(PinState::High);
        let reset = MockOutputPin::new(PinState::High);

        let mut device = IT8951::new(spi, hrdy, cs, reset, 1500);

        // Should fail because not initialized
        assert!(matches!(device.clear(0xFF), Err(Error::Init(_))));
    }

    #[test]
    fn test_fill_area() {
        let mut device = setup_initialized_device();
        let area = Area::new(0, 0, 100, 100);

        assert!(device.fill_area(&area, 0x80).is_ok());
    }

    #[test]
    fn test_fill_area_invalid() {
        let mut device = setup_initialized_device();
        let area = Area::new(700, 500, 200, 200); // Out of bounds

        assert!(matches!(
            device.fill_area(&area, 0x80),
            Err(Error::InvalidArea(_))
        ));
    }

    #[test]
    fn test_refresh() {
        let mut device = setup_initialized_device();
        assert!(device.refresh(DisplayMode::Gc16).is_ok());
    }

    #[test]
    fn test_refresh_area() {
        let mut device = setup_initialized_device();
        let area = Area::new(100, 100, 200, 200);

        assert!(device.refresh_area(&area, DisplayMode::Du).is_ok());
    }

    #[test]
    fn test_load_image() {
        let mut device = setup_initialized_device();
        let area = Area::new(0, 0, 20, 20);
        let data = vec![0x80; 400]; // 20x20 pixels

        assert!(device.load_image(&data, &area, PixelFormat::Bpp8).is_ok());
    }

    #[test]
    fn test_load_image_wrong_size() {
        let mut device = setup_initialized_device();
        let area = Area::new(0, 0, 20, 20);
        let data = vec![0x80; 100]; // Too small

        assert!(matches!(
            device.load_image(&data, &area, PixelFormat::Bpp8),
            Err(Error::InvalidDimensions(_, _))
        ));
    }
}
