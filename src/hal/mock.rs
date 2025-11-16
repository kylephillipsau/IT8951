//! Mock HAL implementations for testing.

use crate::error::Result;
use crate::hal::{BitOrder, InputPin, OutputPin, PinState, SpiInterface, SpiMode, SpiTransfer};
use std::sync::{Arc, Mutex};

/// Mock SPI interface for testing.
#[derive(Debug, Clone)]
pub struct MockSpi {
    clock_hz: u32,
    mode: SpiMode,
    bit_order: BitOrder,
    /// Recorded transfers for verification
    pub transfers: Arc<Mutex<Vec<Vec<u8>>>>,
    /// Responses to return for transfers
    pub responses: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl MockSpi {
    /// Creates a new mock SPI interface.
    pub fn new() -> Self {
        Self {
            clock_hz: 24_000_000,
            mode: SpiMode::Mode0,
            bit_order: BitOrder::MsbFirst,
            transfers: Arc::new(Mutex::new(Vec::new())),
            responses: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Adds a response to be returned by the next transfer.
    pub fn add_response(&mut self, response: Vec<u8>) {
        self.responses.lock().unwrap().push(response);
    }

    /// Returns all recorded transfers.
    pub fn get_transfers(&self) -> Vec<Vec<u8>> {
        self.transfers.lock().unwrap().clone()
    }

    /// Clears all recorded transfers and responses.
    pub fn clear(&mut self) {
        self.transfers.lock().unwrap().clear();
        self.responses.lock().unwrap().clear();
    }
}

impl Default for MockSpi {
    fn default() -> Self {
        Self::new()
    }
}

impl SpiTransfer for MockSpi {
    fn transfer_byte(&mut self, byte: u8) -> Result<u8> {
        let mut transfers = self.transfers.lock().unwrap();
        transfers.push(vec![byte]);

        let mut responses = self.responses.lock().unwrap();
        if let Some(response) = responses.first_mut() {
            if !response.is_empty() {
                return Ok(response.remove(0));
            }
        }

        Ok(0x00) // Default response
    }

    fn transfer(&mut self, buffer: &[u8]) -> Result<Vec<u8>> {
        let mut transfers = self.transfers.lock().unwrap();
        transfers.push(buffer.to_vec());

        let mut responses = self.responses.lock().unwrap();
        if let Some(response) = responses.first() {
            if response.len() >= buffer.len() {
                return Ok(responses.remove(0));
            }
        }

        Ok(vec![0x00; buffer.len()]) // Default response
    }
}

impl SpiInterface for MockSpi {
    fn set_clock_hz(&mut self, hz: u32) -> Result<()> {
        self.clock_hz = hz;
        Ok(())
    }

    fn clock_hz(&self) -> u32 {
        self.clock_hz
    }

    fn set_mode(&mut self, mode: SpiMode) -> Result<()> {
        self.mode = mode;
        Ok(())
    }

    fn set_bit_order(&mut self, order: BitOrder) -> Result<()> {
        self.bit_order = order;
        Ok(())
    }
}

/// Mock GPIO input pin for testing.
#[derive(Debug, Clone)]
pub struct MockInputPin {
    state: Arc<Mutex<PinState>>,
}

impl MockInputPin {
    /// Creates a new mock input pin with the specified initial state.
    pub fn new(initial_state: PinState) -> Self {
        Self {
            state: Arc::new(Mutex::new(initial_state)),
        }
    }

    /// Sets the pin state (simulates external hardware).
    pub fn set_state(&mut self, state: PinState) {
        *self.state.lock().unwrap() = state;
    }
}

impl Default for MockInputPin {
    fn default() -> Self {
        Self::new(PinState::Low)
    }
}

impl InputPin for MockInputPin {
    fn is_high(&self) -> Result<bool> {
        Ok(matches!(*self.state.lock().unwrap(), PinState::High))
    }
}

/// Mock GPIO output pin for testing.
#[derive(Debug, Clone)]
pub struct MockOutputPin {
    state: Arc<Mutex<PinState>>,
    /// History of state changes for verification
    pub history: Arc<Mutex<Vec<PinState>>>,
}

impl MockOutputPin {
    /// Creates a new mock output pin with the specified initial state.
    pub fn new(initial_state: PinState) -> Self {
        let pin = Self {
            state: Arc::new(Mutex::new(initial_state)),
            history: Arc::new(Mutex::new(Vec::new())),
        };
        pin.history.lock().unwrap().push(initial_state);
        pin
    }

    /// Gets the current pin state.
    pub fn get_state(&self) -> PinState {
        *self.state.lock().unwrap()
    }

    /// Gets the state change history.
    pub fn get_history(&self) -> Vec<PinState> {
        self.history.lock().unwrap().clone()
    }

    /// Clears the state change history.
    pub fn clear_history(&mut self) {
        self.history.lock().unwrap().clear();
    }
}

impl Default for MockOutputPin {
    fn default() -> Self {
        Self::new(PinState::Low)
    }
}

impl OutputPin for MockOutputPin {
    fn set_high(&mut self) -> Result<()> {
        *self.state.lock().unwrap() = PinState::High;
        self.history.lock().unwrap().push(PinState::High);
        Ok(())
    }

    fn set_low(&mut self) -> Result<()> {
        *self.state.lock().unwrap() = PinState::Low;
        self.history.lock().unwrap().push(PinState::Low);
        Ok(())
    }

    fn toggle(&mut self) -> Result<()> {
        let new_state = match *self.state.lock().unwrap() {
            PinState::High => PinState::Low,
            PinState::Low => PinState::High,
        };
        *self.state.lock().unwrap() = new_state;
        self.history.lock().unwrap().push(new_state);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_spi_transfer() {
        let mut spi = MockSpi::new();
        spi.add_response(vec![0x12, 0x34]);

        let result = spi.transfer(&[0xAB, 0xCD]).unwrap();
        assert_eq!(result, vec![0x12, 0x34]);

        let transfers = spi.get_transfers();
        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0], vec![0xAB, 0xCD]);
    }

    #[test]
    fn test_mock_spi_config() {
        let mut spi = MockSpi::new();
        assert_eq!(spi.clock_hz(), 24_000_000);

        spi.set_clock_hz(12_000_000).unwrap();
        assert_eq!(spi.clock_hz(), 12_000_000);
    }

    #[test]
    fn test_mock_input_pin() {
        let mut pin = MockInputPin::new(PinState::Low);
        assert!(!pin.is_high().unwrap());
        assert!(pin.is_low().unwrap());

        pin.set_state(PinState::High);
        assert!(pin.is_high().unwrap());
        assert!(!pin.is_low().unwrap());
    }

    #[test]
    fn test_mock_output_pin() {
        let mut pin = MockOutputPin::new(PinState::Low);
        assert_eq!(pin.get_state(), PinState::Low);

        pin.set_high().unwrap();
        assert_eq!(pin.get_state(), PinState::High);

        pin.set_low().unwrap();
        assert_eq!(pin.get_state(), PinState::Low);

        let history = pin.get_history();
        assert_eq!(history, vec![PinState::Low, PinState::High, PinState::Low]);
    }

    #[test]
    fn test_mock_output_pin_toggle() {
        let mut pin = MockOutputPin::new(PinState::Low);

        pin.toggle().unwrap();
        assert_eq!(pin.get_state(), PinState::High);

        pin.toggle().unwrap();
        assert_eq!(pin.get_state(), PinState::Low);
    }
}
