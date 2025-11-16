//! Hardware Abstraction Layer (HAL) for IT8951 controller.
//!
//! This module provides traits for SPI and GPIO interfaces, allowing
//! the IT8951 driver to work with different hardware implementations.

pub mod spi;
pub mod gpio;

#[cfg(test)]
pub mod mock;

pub use self::spi::{SpiInterface, SpiTransfer, SpiMode, BitOrder};
pub use self::gpio::{InputPin, OutputPin, PinState};
