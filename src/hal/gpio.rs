//! GPIO interface abstraction.

use crate::error::Result;

/// Pin state (high or low).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinState {
    /// Low (0V)
    Low,
    /// High (3.3V or 5V depending on system)
    High,
}

impl From<bool> for PinState {
    fn from(value: bool) -> Self {
        if value {
            PinState::High
        } else {
            PinState::Low
        }
    }
}

impl From<PinState> for bool {
    fn from(state: PinState) -> bool {
        matches!(state, PinState::High)
    }
}

/// Trait for GPIO input pins.
pub trait InputPin {
    /// Reads the current state of the pin.
    fn is_high(&self) -> Result<bool>;

    /// Reads the current state of the pin.
    fn is_low(&self) -> Result<bool> {
        self.is_high().map(|high| !high)
    }

    /// Reads the pin state.
    fn read(&self) -> Result<PinState> {
        self.is_high()
            .map(|high| if high { PinState::High } else { PinState::Low })
    }
}

/// Trait for GPIO output pins.
pub trait OutputPin {
    /// Sets the pin to high.
    fn set_high(&mut self) -> Result<()>;

    /// Sets the pin to low.
    fn set_low(&mut self) -> Result<()>;

    /// Sets the pin to the specified state.
    fn set_state(&mut self, state: PinState) -> Result<()> {
        match state {
            PinState::High => self.set_high(),
            PinState::Low => self.set_low(),
        }
    }

    /// Toggles the pin state.
    fn toggle(&mut self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_state_from_bool() {
        assert_eq!(PinState::from(true), PinState::High);
        assert_eq!(PinState::from(false), PinState::Low);
    }

    #[test]
    fn test_bool_from_pin_state() {
        assert!(bool::from(PinState::High));
        assert!(!bool::from(PinState::Low));
    }
}
