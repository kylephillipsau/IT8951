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
//! - [`protocol`] - IT8951 communication protocol
//! - [`device`] - Device management and initialization
//! - [`display`] - Display operations
//! - [`graphics`] - Drawing primitives and framebuffer
//!
//! # Implementation Status
//!
//! ## Phase 1: Foundation ✅ COMPLETE
//!
//! - ✅ Error types and handling
//! - ✅ HAL traits for SPI and GPIO
//! - ✅ Mock HAL implementations for testing
//! - ✅ Core data structures (Area, DeviceInfo, DisplayMode, etc.)
//!
//! ## Phase 2: Protocol Layer ✅ COMPLETE
//!
//! - ✅ Command and register definitions
//! - ✅ Low-level SPI transport with preambles
//! - ✅ Hardware ready synchronization
//! - ✅ Register read/write operations
//! - ✅ Batch data transfer support
//!
//! ## Phase 3: Device Management ✅ COMPLETE
//!
//! - ✅ IT8951 device struct with builder pattern
//! - ✅ Device initialization and hardware reset
//! - ✅ Device information retrieval
//! - ✅ VCOM voltage configuration
//! - ✅ Power state management (run/standby/sleep)
//!
//! ## Phase 4: Display Operations ✅ COMPLETE
//!
//! - ✅ Clear and fill operations
//! - ✅ Full and partial area refresh
//! - ✅ Image loading with format validation
//! - ✅ Pixel packing for efficient transfer
//!
//! ## Phase 5: Graphics Layer ✅ COMPLETE
//!
//! - ✅ Framebuffer for in-memory drawing
//! - ✅ Line drawing (Bresenham's algorithm)
//! - ✅ Rectangle drawing (filled and outline)
//! - ✅ Circle drawing (midpoint algorithm)
//! - ✅ Display integration for framebuffer rendering
//!
//! ## Coming Soon
//!
//! - 🔄 Advanced features (Phase 6-7)

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod device;
pub mod display;
pub mod error;
pub mod graphics;
pub mod hal;
pub mod protocol;
pub mod types;

// Re-export commonly used types
pub use device::{IT8951, IT8951Builder};
pub use error::{Error, Result};
pub use graphics::Framebuffer;
pub use hal::{
    BitOrder, InputPin, LinuxInputPin, LinuxOutputPin, LinuxSpi, OutputPin, PinState, SpiInterface,
    SpiMode, SpiTransfer,
};
pub use protocol::{Command, Register, Transport, UserCommand};
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
