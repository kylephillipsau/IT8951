//! IT8951 communication protocol implementation.
//!
//! This module implements the low-level IT8951 protocol for SPI communication.
//! The IT8951 uses a preamble-based protocol where each command/data transfer
//! is preceded by a 16-bit preamble indicating the operation type.
//!
//! # Protocol Overview
//!
//! ## Preambles
//! - `0x6000`: Write command code
//! - `0x0000`: Write data
//! - `0x1000`: Read data
//!
//! ## Communication Flow
//! 1. Wait for HRDY (hardware ready) pin to be high
//! 2. Assert CS (chip select) low
//! 3. Send preamble (2 bytes)
//! 4. Wait for HRDY again
//! 5. Transfer command/data (2 bytes)
//! 6. De-assert CS high
//!
//! For read operations, two dummy bytes are sent before reading the actual data.

pub mod commands;
pub mod registers;
pub mod transport;

pub use commands::{Command, UserCommand};
pub use registers::Register;
pub use transport::Transport;
