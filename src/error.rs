//! Error types for the IT8951 library.

/// Errors that can occur when using the IT8951 library.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// SPI communication error
    #[error("SPI error: {0}")]
    Spi(String),

    /// GPIO error
    #[error("GPIO error: {0}")]
    Gpio(String),

    /// Timeout waiting for device ready
    #[error("Timeout waiting for device ready after {0}ms")]
    Timeout(u64),

    /// Invalid parameter provided
    #[error("Invalid parameter: {0}")]
    InvalidParameter(&'static str),

    /// Device not ready for operation
    #[error("Device not ready")]
    NotReady,

    /// Invalid area (out of bounds)
    #[error("Invalid area: x={}, y={}, width={}, height={} exceeds display bounds", .0.x, .0.y, .0.width, .0.height)]
    InvalidArea(crate::Area),

    /// Invalid VCOM value
    #[error("Invalid VCOM value: {0} (must be between 0 and 5000)")]
    InvalidVcom(u16),

    /// Invalid image dimensions
    #[error("Invalid image dimensions: {0}x{1}")]
    InvalidDimensions(u16, u16),

    /// Image format error
    #[cfg(feature = "image-support")]
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Device error with description
    #[error("Device error: {0}")]
    Device(String),

    /// Initialization error
    #[error("Initialization error: {0}")]
    Init(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Memory operation error
    #[error("Memory operation error: {0}")]
    Memory(String),

    /// Display operation error
    #[error("Display operation error: {0}")]
    Display(String),
}

/// Result type alias for IT8951 operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Timeout(1000);
        assert_eq!(err.to_string(), "Timeout waiting for device ready after 1000ms");

        let err = Error::InvalidParameter("test");
        assert_eq!(err.to_string(), "Invalid parameter: test");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }
}
