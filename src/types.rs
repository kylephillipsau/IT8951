//! Core data types for the IT8951 driver.

use crate::error::{Error, Result};

/// Information about the connected IT8951 device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// Panel width in pixels
    pub panel_width: u16,

    /// Panel height in pixels
    pub panel_height: u16,

    /// Image buffer base address in device memory
    pub img_buf_addr: u32,

    /// Firmware version string
    pub fw_version: String,

    /// LUT (Look-Up Table) version string
    pub lut_version: String,
}

impl DeviceInfo {
    /// Creates a new DeviceInfo from raw register data.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw 16-bit words from device info query
    pub fn from_raw(data: &[u16]) -> Result<Self> {
        if data.len() < 20 {
            return Err(Error::Protocol("Device info data too short".to_string()));
        }

        let panel_width = data[0];
        let panel_height = data[1];
        let img_buf_addr_l = data[2];
        let img_buf_addr_h = data[3];
        let img_buf_addr = (img_buf_addr_h as u32) << 16 | (img_buf_addr_l as u32);

        // FW version is 8 words (16 bytes) starting at offset 4
        let fw_version = Self::parse_version_string(&data[4..12]);

        // LUT version is 8 words (16 bytes) starting at offset 12
        let lut_version = Self::parse_version_string(&data[12..20]);

        Ok(Self {
            panel_width,
            panel_height,
            img_buf_addr,
            fw_version,
            lut_version,
        })
    }

    /// Parses a version string from raw 16-bit words.
    fn parse_version_string(words: &[u16]) -> String {
        let bytes: Vec<u8> = words
            .iter()
            .flat_map(|&word| vec![(word & 0xFF) as u8, (word >> 8) as u8])
            .take_while(|&b| b != 0) // Stop at null terminator
            .collect();

        String::from_utf8(bytes).unwrap_or_else(|_| "Unknown".to_string())
    }

    /// Returns the total number of pixels.
    pub fn pixel_count(&self) -> usize {
        self.panel_width as usize * self.panel_height as usize
    }
}

/// A rectangular area on the display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Area {
    /// X coordinate (left edge)
    pub x: u16,

    /// Y coordinate (top edge)
    pub y: u16,

    /// Width in pixels
    pub width: u16,

    /// Height in pixels
    pub height: u16,
}

impl Area {
    /// Creates a new area.
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns true if the area is valid for the given display dimensions.
    pub fn is_valid(&self, display_width: u16, display_height: u16) -> bool {
        self.width > 0
            && self.height > 0
            && self.x < display_width
            && self.y < display_height
            && self.x.saturating_add(self.width) <= display_width
            && self.y.saturating_add(self.height) <= display_height
    }

    /// Returns the intersection of two areas, if any.
    pub fn intersect(&self, other: &Area) -> Option<Area> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);

        if x < x2 && y < y2 {
            Some(Area::new(x, y, x2 - x, y2 - y))
        } else {
            None
        }
    }

    /// Returns the area encompassing both areas.
    pub fn union(&self, other: &Area) -> Area {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let x2 = (self.x + self.width).max(other.x + other.width);
        let y2 = (self.y + self.height).max(other.y + other.height);

        Area::new(x, y, x2 - x, y2 - y)
    }

    /// Returns the number of pixels in the area.
    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    /// Returns the right edge coordinate (x + width).
    pub fn right(&self) -> u16 {
        self.x + self.width
    }

    /// Returns the bottom edge coordinate (y + height).
    pub fn bottom(&self) -> u16 {
        self.y + self.height
    }
}

impl From<(u16, u16, u16, u16)> for Area {
    fn from((x, y, width, height): (u16, u16, u16, u16)) -> Self {
        Self::new(x, y, width, height)
    }
}

/// Display refresh modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum DisplayMode {
    /// Initialization mode (clears ghosting)
    Init = 0,

    /// Direct Update (fast monochrome)
    Du = 1,

    /// Grayscale Clearing (16-level high quality)
    Gc16 = 2,

    /// Grayscale Level (16-level faster)
    Gl16 = 3,

    /// Animation mode (very fast)
    A2 = 4,
}

impl DisplayMode {
    /// Returns the mode value as a u16.
    pub fn as_u16(&self) -> u16 {
        *self as u16
    }
}

/// Pixel formats supported by the IT8951.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PixelFormat {
    /// 2 bits per pixel (4 gray levels)
    Bpp2 = 0,

    /// 3 bits per pixel (8 gray levels)
    Bpp3 = 1,

    /// 4 bits per pixel (16 gray levels)
    Bpp4 = 2,

    /// 8 bits per pixel (256 gray levels)
    Bpp8 = 3,
}

impl PixelFormat {
    /// Returns the number of bits per pixel.
    pub fn bits(&self) -> u8 {
        match self {
            PixelFormat::Bpp2 => 2,
            PixelFormat::Bpp3 => 3,
            PixelFormat::Bpp4 => 4,
            PixelFormat::Bpp8 => 8,
        }
    }

    /// Returns the number of gray levels.
    pub fn gray_levels(&self) -> u16 {
        1 << self.bits()
    }

    /// Returns the format value as a u16.
    pub fn as_u16(&self) -> u16 {
        *self as u16
    }
}

/// Display rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Rotation {
    /// No rotation
    Rotate0 = 0,

    /// 90 degrees clockwise
    Rotate90 = 1,

    /// 180 degrees
    Rotate180 = 2,

    /// 270 degrees clockwise (90 counter-clockwise)
    Rotate270 = 3,
}

impl Rotation {
    /// Returns the rotation value as a u16.
    pub fn as_u16(&self) -> u16 {
        *self as u16
    }
}

/// Byte order for pixel data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Endian {
    /// Little endian
    Little = 0,

    /// Big endian
    Big = 1,
}

impl Endian {
    /// Returns the endian value as a u16.
    pub fn as_u16(&self) -> u16 {
        *self as u16
    }
}

/// Image loading information.
#[derive(Debug, Clone, Copy)]
pub struct LoadImageInfo {
    /// Endian type
    pub endian: Endian,

    /// Pixel format
    pub pixel_format: PixelFormat,

    /// Rotation
    pub rotate: Rotation,

    /// Start address of source frame buffer
    pub start_fb_addr: u32,

    /// Base address of target image buffer
    pub img_buf_base_addr: u32,
}

impl LoadImageInfo {
    /// Creates a new LoadImageInfo with default values.
    pub fn new(start_fb_addr: u32, img_buf_base_addr: u32) -> Self {
        Self {
            endian: Endian::Little,
            pixel_format: PixelFormat::Bpp8,
            rotate: Rotation::Rotate0,
            start_fb_addr,
            img_buf_base_addr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info_from_raw() {
        let mut data = vec![0u16; 20];
        data[0] = 800; // width
        data[1] = 600; // height
        data[2] = 0x1234; // addr low
        data[3] = 0x5678; // addr high

        let info = DeviceInfo::from_raw(&data).unwrap();
        assert_eq!(info.panel_width, 800);
        assert_eq!(info.panel_height, 600);
        assert_eq!(info.img_buf_addr, 0x56781234);
        assert_eq!(info.pixel_count(), 480000);
    }

    #[test]
    fn test_area_validity() {
        let area = Area::new(0, 0, 100, 100);
        assert!(area.is_valid(800, 600));

        let area = Area::new(750, 550, 100, 100);
        assert!(!area.is_valid(800, 600));
    }

    #[test]
    fn test_area_intersection() {
        let area1 = Area::new(0, 0, 100, 100);
        let area2 = Area::new(50, 50, 100, 100);

        let intersection = area1.intersect(&area2);
        assert_eq!(intersection, Some(Area::new(50, 50, 50, 50)));

        let area3 = Area::new(200, 200, 100, 100);
        assert_eq!(area1.intersect(&area3), None);
    }

    #[test]
    fn test_area_union() {
        let area1 = Area::new(0, 0, 100, 100);
        let area2 = Area::new(50, 50, 100, 100);

        let union = area1.union(&area2);
        assert_eq!(union, Area::new(0, 0, 150, 150));
    }

    #[test]
    fn test_pixel_format_bits() {
        assert_eq!(PixelFormat::Bpp2.bits(), 2);
        assert_eq!(PixelFormat::Bpp3.bits(), 3);
        assert_eq!(PixelFormat::Bpp4.bits(), 4);
        assert_eq!(PixelFormat::Bpp8.bits(), 8);
    }

    #[test]
    fn test_pixel_format_gray_levels() {
        assert_eq!(PixelFormat::Bpp2.gray_levels(), 4);
        assert_eq!(PixelFormat::Bpp3.gray_levels(), 8);
        assert_eq!(PixelFormat::Bpp4.gray_levels(), 16);
        assert_eq!(PixelFormat::Bpp8.gray_levels(), 256);
    }

    #[test]
    fn test_display_mode_values() {
        assert_eq!(DisplayMode::Init.as_u16(), 0);
        assert_eq!(DisplayMode::Du.as_u16(), 1);
        assert_eq!(DisplayMode::Gc16.as_u16(), 2);
        assert_eq!(DisplayMode::Gl16.as_u16(), 3);
        assert_eq!(DisplayMode::A2.as_u16(), 4);
    }

    #[test]
    fn test_area_from_tuple() {
        let area: Area = (10, 20, 100, 200).into();
        assert_eq!(area.x, 10);
        assert_eq!(area.y, 20);
        assert_eq!(area.width, 100);
        assert_eq!(area.height, 200);
    }
}
