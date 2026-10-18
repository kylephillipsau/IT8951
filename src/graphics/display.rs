//! Display integration for framebuffer graphics
//!
//! This module provides methods to transfer framebuffer content to the
//! IT8951 display hardware.

use crate::device::IT8951;
use crate::error::Result;
use crate::graphics::Framebuffer;
use crate::hal::{InputPin, OutputPin, SpiTransfer};
use crate::types::{Area, DisplayMode, PixelFormat};

/// Pack 8bpp pixel data into 4bpp format.
///
/// Each 8bpp pixel's top 4 bits become a nibble. Two pixels are packed per byte:
/// high nibble = first pixel, low nibble = second pixel.
fn pack_8bpp_to_4bpp(data: &[u8]) -> Vec<u8> {
    let mut packed = Vec::with_capacity((data.len() + 1) / 2);
    for pair in data.chunks(2) {
        let hi = pair[0] >> 4;
        let lo = if pair.len() == 2 { pair[1] >> 4 } else { 0 };
        packed.push((hi << 4) | lo);
    }
    packed
}

impl<SPI, HRDY, CS, RESET> IT8951<SPI, HRDY, CS, RESET>
where
    SPI: SpiTransfer,
    HRDY: InputPin,
    CS: OutputPin,
    RESET: OutputPin,
{
    /// Draw a framebuffer to the display
    ///
    /// Loads the framebuffer content to the specified area on the display
    /// and optionally refreshes it with the given display mode.
    ///
    /// # Arguments
    ///
    /// * `framebuffer` - The framebuffer to draw
    /// * `area` - The target area on the display (must match framebuffer dimensions)
    /// * `refresh` - Whether to refresh the display after loading
    /// * `mode` - The display mode to use for refresh (if refresh is true)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use it8951::{IT8951, DisplayMode};
    /// use it8951::graphics::Framebuffer;
    /// use it8951::Area;
    ///
    /// let mut display = IT8951::builder().build_mock().unwrap();
    /// display.init().unwrap();
    ///
    /// let mut fb = Framebuffer::new(800, 600);
    /// fb.draw_rect(100, 100, 200, 150, 0x00, false);
    ///
    /// let area = Area::new(0, 0, 800, 600);
    /// display.draw_framebuffer(&fb, &area, true, DisplayMode::Gc16).unwrap();
    /// ```
    pub fn draw_framebuffer(
        &mut self,
        framebuffer: &Framebuffer,
        area: &Area,
        refresh: bool,
        mode: DisplayMode,
    ) -> Result<()> {
        // Verify framebuffer dimensions match area
        if framebuffer.width() != area.width || framebuffer.height() != area.height {
            return Err(crate::error::Error::Device(format!(
                "Framebuffer dimensions ({}x{}) don't match area dimensions ({}x{})",
                framebuffer.width(),
                framebuffer.height(),
                area.width,
                area.height
            )));
        }

        // Pack 8bpp framebuffer to 4bpp (IT8951 only uses top 4 bits anyway)
        let packed = pack_8bpp_to_4bpp(framebuffer.data());
        self.load_image(&packed, area, PixelFormat::Bpp4)?;

        // Optionally refresh the display
        if refresh {
            self.refresh_area(area, mode)?;
        }

        Ok(())
    }

    /// Draw a framebuffer to the entire display
    ///
    /// Convenience method that draws a framebuffer to fill the entire panel
    /// and refreshes it with the specified mode.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use it8951::{IT8951, DisplayMode};
    /// use it8951::graphics::Framebuffer;
    ///
    /// let mut display = IT8951::builder().build_mock().unwrap();
    /// display.init().unwrap();
    ///
    /// let mut fb = Framebuffer::new(800, 600);
    /// fb.clear(0xFF);
    /// fb.draw_circle(400, 300, 100, 0x00, false);
    ///
    /// display.draw_framebuffer_full(&fb, DisplayMode::Gc16).unwrap();
    /// ```
    pub fn draw_framebuffer_full(
        &mut self,
        framebuffer: &Framebuffer,
        mode: DisplayMode,
    ) -> Result<()> {
        let device_info = self
            .device_info
            .as_ref()
            .ok_or_else(|| crate::error::Error::Init("Device not initialized".to_string()))?;

        // Verify framebuffer matches panel size
        if framebuffer.width() != device_info.panel_width
            || framebuffer.height() != device_info.panel_height
        {
            return Err(crate::error::Error::Device(format!(
                "Framebuffer size ({}x{}) doesn't match panel size ({}x{})",
                framebuffer.width(),
                framebuffer.height(),
                device_info.panel_width,
                device_info.panel_height
            )));
        }

        let area = Area::new(0, 0, device_info.panel_width, device_info.panel_height);
        self.draw_framebuffer(framebuffer, &area, true, mode)
    }

    /// Create a framebuffer matching the display panel size
    ///
    /// Convenience method to create a framebuffer with dimensions
    /// matching the connected display panel.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use it8951::IT8951;
    ///
    /// let mut display = IT8951::builder().build_mock().unwrap();
    /// display.init().unwrap();
    ///
    /// let fb = display.create_framebuffer().unwrap();
    /// assert_eq!(fb.width(), 800);
    /// assert_eq!(fb.height(), 600);
    /// ```
    pub fn create_framebuffer(&self) -> Result<Framebuffer> {
        let device_info = self
            .device_info
            .as_ref()
            .ok_or_else(|| crate::error::Error::Init("Device not initialized".to_string()))?;

        Ok(Framebuffer::new(
            device_info.panel_width,
            device_info.panel_height,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::IT8951Builder;
    use crate::types::DeviceInfo;

    #[test]
    fn test_draw_framebuffer() {
        let mut device = IT8951Builder::new().build_mock().unwrap();

        // Mock device info
        device.device_info = Some(DeviceInfo {
            panel_width: 800,
            panel_height: 600,
            img_buf_addr: 0x00100000,
            fw_version: "1.0".to_string(),
            lut_version: "1.0".to_string(),
        });

        let mut fb = Framebuffer::new(100, 100);
        fb.clear(0xFF);

        let area = Area::new(0, 0, 100, 100);
        device
            .draw_framebuffer(&fb, &area, false, DisplayMode::Gc16)
            .unwrap();
    }

    #[test]
    fn test_draw_framebuffer_dimension_mismatch() {
        let mut device = IT8951Builder::new().build_mock().unwrap();

        device.device_info = Some(DeviceInfo {
            panel_width: 800,
            panel_height: 600,
            img_buf_addr: 0x00100000,
            fw_version: "1.0".to_string(),
            lut_version: "1.0".to_string(),
        });

        let fb = Framebuffer::new(100, 100);
        let area = Area::new(0, 0, 200, 100); // Different width

        assert!(device
            .draw_framebuffer(&fb, &area, false, DisplayMode::Gc16)
            .is_err());
    }

    #[test]
    fn test_create_framebuffer() {
        let mut device = IT8951Builder::new().build_mock().unwrap();

        device.device_info = Some(DeviceInfo {
            panel_width: 800,
            panel_height: 600,
            img_buf_addr: 0x00100000,
            fw_version: "1.0".to_string(),
            lut_version: "1.0".to_string(),
        });

        let fb = device.create_framebuffer().unwrap();
        assert_eq!(fb.width(), 800);
        assert_eq!(fb.height(), 600);
    }

    #[test]
    fn test_create_framebuffer_not_initialized() {
        let device = IT8951Builder::new().build_mock().unwrap();
        assert!(device.create_framebuffer().is_err());
    }

    #[test]
    fn test_draw_framebuffer_full() {
        let mut device = IT8951Builder::new().build_mock().unwrap();

        device.device_info = Some(DeviceInfo {
            panel_width: 100,
            panel_height: 100,
            img_buf_addr: 0x00100000,
            fw_version: "1.0".to_string(),
            lut_version: "1.0".to_string(),
        });

        let mut fb = Framebuffer::new(100, 100);
        fb.clear(0xFF);

        device.draw_framebuffer_full(&fb, DisplayMode::Gc16).unwrap();
    }

    #[test]
    fn test_draw_framebuffer_full_size_mismatch() {
        let mut device = IT8951Builder::new().build_mock().unwrap();

        device.device_info = Some(DeviceInfo {
            panel_width: 800,
            panel_height: 600,
            img_buf_addr: 0x00100000,
            fw_version: "1.0".to_string(),
            lut_version: "1.0".to_string(),
        });

        let fb = Framebuffer::new(100, 100);
        assert!(device.draw_framebuffer_full(&fb, DisplayMode::Gc16).is_err());
    }
}
