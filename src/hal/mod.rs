//! Hardware Abstraction Layer (HAL) for IT8951 controller.
//!
//! This module provides traits for SPI and GPIO interfaces, allowing
//! the IT8951 driver to work with different hardware implementations.

pub mod gpio;
pub mod spi;

#[cfg(test)]
pub mod mock;

pub use self::gpio::{InputPin, OutputPin, PinState};
pub use self::spi::{BitOrder, SpiInterface, SpiMode, SpiTransfer};
