//! IT8951 E-Paper Display Controller Driver
//!
//! This crate provides a Rust driver for the IT8951 e-paper controller chip,
//! commonly used in e-paper displays such as the Waveshare 6-inch e-Paper HAT.
//!
//! # Features
//!
//! - Type-safe API with compile-time guarantees
//! - Hardware abstraction layer for portability
//! - Support for multiple display modes and pixel formats
//! - Comprehensive error handling
//! - Mock implementations for testing without hardware
//!
//! # Quick Start
//!
//! ```ignore
//! // The IT8951 main struct will be available in Phase 3
//! use it8951::{IT8951, DisplayMode};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut display = IT8951::builder()
//!         .spi_device("/dev/spidev0.0")?
//!         .spi_hz(24_000_000)
//!         .vcom(1500)
//!         .build()?;
//!
//!     display.init()?;
//!     display.clear(0xFF)?;
//!     display.refresh(DisplayMode::Gc16)?;
//!     Ok(())
//! }
//! ```
//!
//! # Architecture
//!
//! The driver is organized into several modules:
//!
//! - [`hal`] - Hardware abstraction layer (SPI, GPIO)
//! - [`error`] - Error types and Result aliases
//! - [`types`] - Core data structures
//! - [`protocol`] - IT8951 communication protocol (coming in Phase 2)
//! - [`device`] - Device management and initialization (coming in Phase 3)
//! - [`display`] - Display operations (coming in Phase 4)
//! - [`graphics`] - Drawing primitives (coming in Phase 5)
//!
//! # Phase 1 Status
//!
//! This is Phase 1 (Foundation) of the implementation. Currently available:
//!
//! - ✅ Error types and handling
//! - ✅ HAL traits for SPI and GPIO
//! - ✅ Mock HAL implementations for testing
//! - ✅ Core data structures (Area, DeviceInfo, DisplayMode, etc.)
//!
//! Coming in future phases:
//!
//! - 🔄 Protocol layer (Phase 2)
//! - 🔄 Device initialization and management (Phase 3)
//! - 🔄 Display operations (Phase 4)
//! - 🔄 Graphics primitives (Phase 5)

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod error;
pub mod hal;
pub mod types;

// Re-export commonly used types
pub use error::{Error, Result};
pub use hal::{BitOrder, InputPin, OutputPin, PinState, SpiInterface, SpiMode, SpiTransfer};
pub use types::{Area, DeviceInfo, DisplayMode, Endian, LoadImageInfo, PixelFormat, Rotation};

// Re-export mock implementations for testing
#[cfg(test)]
pub use hal::mock::{MockInputPin, MockOutputPin, MockSpi};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_types() {
        let area = Area::new(0, 0, 800, 600);
        assert_eq!(area.pixel_count(), 480000);

        let mode = DisplayMode::Gc16;
        assert_eq!(mode.as_u16(), 2);

        let format = PixelFormat::Bpp8;
        assert_eq!(format.bits(), 8);
        assert_eq!(format.gray_levels(), 256);
    }
}
