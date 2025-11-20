//! Builder pattern for IT8951 device construction.

#[cfg(test)]
use crate::device::IT8951;
#[cfg(test)]
use crate::error::{Error, Result};

/// Builder for constructing an IT8951 device.
///
/// Provides a fluent interface for configuring the device before creation.
///
/// # Examples
///
/// ```ignore
/// use it8951::IT8951;
///
/// let display = IT8951::builder()
///     .vcom(1500)
///     .build_mock()?; // For testing
/// ```
#[derive(Debug, Clone)]
pub struct IT8951Builder {
    vcom: u16,
}

impl IT8951Builder {
    /// Creates a new builder with default values.
    pub fn new() -> Self {
        Self { vcom: 1500 }
    }

    /// Sets the VCOM voltage value.
    ///
    /// # Arguments
    ///
    /// * `vcom` - VCOM value (e.g., 1500 for -1.50V)
    ///
    /// # Examples
    ///
    /// ```
    /// use it8951::IT8951Builder;
    ///
    /// let builder = IT8951Builder::new().vcom(1530);
    /// ```
    pub fn vcom(mut self, vcom: u16) -> Self {
        self.vcom = vcom;
        self
    }

    /// Validates the builder configuration.
    #[cfg(test)]
    fn validate(&self) -> Result<()> {
        if self.vcom > 5000 {
            return Err(Error::InvalidVcom(self.vcom));
        }
        Ok(())
    }

    /// Builds an IT8951 device with mock hardware (for testing).
    ///
    /// This creates a device with mock SPI and GPIO interfaces,
    /// useful for testing without real hardware.
    #[cfg(test)]
    pub fn build_mock(
        self,
    ) -> Result<IT8951<
        crate::hal::mock::MockSpi,
        crate::hal::mock::MockInputPin,
        crate::hal::mock::MockOutputPin,
        crate::hal::mock::MockOutputPin,
    >> {
        use crate::hal::mock::{MockInputPin, MockOutputPin, MockSpi};
        use crate::hal::PinState;

        self.validate()?;

        let spi = MockSpi::new();
        let hrdy = MockInputPin::new(PinState::High);
        let cs = MockOutputPin::new(PinState::High);
        let reset = MockOutputPin::new(PinState::High);

        Ok(IT8951::new(spi, hrdy, cs, reset, self.vcom))
    }
}

impl Default for IT8951Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let builder = IT8951Builder::new();
        assert_eq!(builder.vcom, 1500);
    }

    #[test]
    fn test_builder_vcom() {
        let builder = IT8951Builder::new().vcom(1530);
        assert_eq!(builder.vcom, 1530);
    }

    #[test]
    fn test_builder_validation() {
        let builder = IT8951Builder::new().vcom(6000);
        assert!(matches!(builder.validate(), Err(Error::InvalidVcom(6000))));
    }

    #[test]
    fn test_build_mock() {
        let device = IT8951Builder::new().vcom(1500).build_mock().unwrap();
        assert_eq!(device.vcom(), 1500);
    }

    #[test]
    fn test_build_mock_invalid_vcom() {
        let result = IT8951Builder::new().vcom(6000).build_mock();
        assert!(matches!(result, Err(Error::InvalidVcom(6000))));
    }
}
