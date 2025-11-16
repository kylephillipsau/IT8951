# IT8951 Rust API Design

## Design Principles

1. **Type Safety**: Use Rust's type system to prevent errors at compile time
2. **Ergonomic**: Make common operations simple, complex operations possible
3. **Zero Cost**: Abstractions should compile to efficient code
4. **Builder Pattern**: Flexible initialization with sensible defaults
5. **Error Handling**: Explicit error types, no panics in library code
6. **Ownership**: Clear ownership semantics, minimize copying
7. **Documentation**: Every public item thoroughly documented

## Core API

### Main Controller Type

```rust
/// IT8951 E-Paper Display Controller
///
/// This is the main entry point for controlling an IT8951-based e-paper display.
/// The controller manages SPI communication, device initialization, and display operations.
///
/// # Type Parameters
///
/// * `SPI` - SPI interface implementation
/// * `HRDY` - Hardware ready pin (input)
/// * `CS` - Chip select pin (output)
/// * `RESET` - Reset pin (output)
///
/// # Examples
///
/// ```no_run
/// use it8951::IT8951;
///
/// let mut display = IT8951::builder()
///     .spi_device("/dev/spidev0.0")?
///     .spi_hz(24_000_000)
///     .vcom(1500)
///     .build()?;
///
/// display.init()?;
/// display.clear(0xFF)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct IT8951<SPI, HRDY, CS, RESET> {
    spi: SPI,
    hrdy: HRDY,
    cs: CS,
    reset: RESET,
    device_info: DeviceInfo,
    img_buf_addr: u32,
    frame_buffer: Vec<u8>,
    config: DisplayConfig,
}

impl<SPI, HRDY, CS, RESET> IT8951<SPI, HRDY, CS, RESET>
where
    SPI: SpiInterface,
    HRDY: InputPin,
    CS: OutputPin,
    RESET: OutputPin,
{
    /// Creates a new builder for configuring the IT8951 controller.
    pub fn builder() -> IT8951Builder {
        IT8951Builder::new()
    }

    /// Initializes the display controller.
    ///
    /// This performs a hardware reset, retrieves device information,
    /// configures VCOM, and prepares the display for use.
    pub fn init(&mut self) -> Result<()>;

    /// Returns information about the connected display.
    pub fn device_info(&self) -> &DeviceInfo;

    /// Returns the width of the display in pixels.
    pub fn width(&self) -> u16;

    /// Returns the height of the display in pixels.
    pub fn height(&self) -> u16;
}
```

### Builder Pattern

```rust
/// Builder for configuring an IT8951 controller.
///
/// # Examples
///
/// ```no_run
/// use it8951::IT8951;
///
/// let display = IT8951::builder()
///     .spi_device("/dev/spidev0.0")?
///     .spi_hz(24_000_000)
///     .vcom(1500)
///     .cs_pin(8)
///     .hrdy_pin(24)
///     .reset_pin(17)
///     .build()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct IT8951Builder {
    spi_device: Option<String>,
    spi_hz: u32,
    vcom: u16,
    cs_pin: Option<u8>,
    hrdy_pin: Option<u8>,
    reset_pin: Option<u8>,
    rotation: Rotation,
}

impl IT8951Builder {
    pub fn new() -> Self {
        Self {
            spi_device: None,
            spi_hz: 24_000_000,
            vcom: 1500,
            cs_pin: None,
            hrdy_pin: None,
            reset_pin: None,
            rotation: Rotation::Rotate0,
        }
    }

    /// Sets the SPI device path (e.g., "/dev/spidev0.0").
    pub fn spi_device(mut self, device: impl Into<String>) -> Result<Self> {
        self.spi_device = Some(device.into());
        Ok(self)
    }

    /// Sets the SPI clock frequency in Hz (default: 24MHz).
    pub fn spi_hz(mut self, hz: u32) -> Self {
        self.spi_hz = hz;
        self
    }

    /// Sets the VCOM voltage value (default: 1500).
    ///
    /// The VCOM value should be obtained from the display's label.
    /// For example, -1.50V would be set as 1500.
    pub fn vcom(mut self, vcom: u16) -> Self {
        self.vcom = vcom;
        self
    }

    /// Sets the chip select GPIO pin number.
    pub fn cs_pin(mut self, pin: u8) -> Self {
        self.cs_pin = Some(pin);
        self
    }

    /// Sets the hardware ready GPIO pin number.
    pub fn hrdy_pin(mut self, pin: u8) -> Self {
        self.hrdy_pin = Some(pin);
        self
    }

    /// Sets the reset GPIO pin number.
    pub fn reset_pin(mut self, pin: u8) -> Self {
        self.reset_pin = Some(pin);
        self
    }

    /// Sets the display rotation.
    pub fn rotation(mut self, rotation: Rotation) -> Self {
        self.rotation = rotation;
        self
    }

    /// Builds the IT8951 controller with the configured settings.
    pub fn build(self) -> Result<IT8951<impl SpiInterface, impl InputPin, impl OutputPin, impl OutputPin>> {
        // Validate required fields
        let spi_device = self.spi_device.ok_or(Error::InvalidParameter("spi_device required"))?;

        // Initialize hardware interfaces
        // ... implementation
    }

    /// Creates a virtual display for testing without hardware.
    #[cfg(feature = "virtual-display")]
    pub fn virtual_display(self) -> Self {
        // Configure for virtual display
        self
    }

    /// Sets the panel size (only for virtual display).
    #[cfg(feature = "virtual-display")]
    pub fn panel_size(mut self, width: u16, height: u16) -> Self {
        // Set virtual panel size
        self
    }
}
```

### Device Information

```rust
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
    /// Parses device info from raw register data.
    pub(crate) fn from_raw(data: &[u16]) -> Result<Self>;

    /// Returns the total number of pixels.
    pub fn pixel_count(&self) -> usize {
        self.panel_width as usize * self.panel_height as usize
    }
}
```

### Display Operations

```rust
impl<SPI, HRDY, CS, RESET> IT8951<SPI, HRDY, CS, RESET>
where
    SPI: SpiInterface,
    HRDY: InputPin,
    CS: OutputPin,
    RESET: OutputPin,
{
    /// Clears the entire display to the specified grayscale value.
    ///
    /// # Arguments
    ///
    /// * `value` - Grayscale value (0x00 = black, 0xFF = white)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use it8951::IT8951;
    /// # let mut display = setup();
    /// display.clear(0xFF)?; // Clear to white
    /// display.clear(0x00)?; // Clear to black
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn clear(&mut self, value: u8) -> Result<()>;

    /// Refreshes the entire display with the specified mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - Display refresh mode
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use it8951::{IT8951, DisplayMode};
    /// # let mut display = setup();
    /// display.refresh(DisplayMode::Gc16)?; // High quality grayscale
    /// display.refresh(DisplayMode::Du)?;   // Fast monochrome
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn refresh(&mut self, mode: DisplayMode) -> Result<()>;

    /// Updates a specific area of the display.
    ///
    /// # Arguments
    ///
    /// * `area` - The rectangular area to update
    /// * `mode` - Display refresh mode
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use it8951::{IT8951, DisplayMode, Area};
    /// # let mut display = setup();
    /// let area = Area::new(0, 0, 200, 100);
    /// display.refresh_area(&area, DisplayMode::Du)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn refresh_area(&mut self, area: &Area, mode: DisplayMode) -> Result<()>;

    /// Fills a rectangular area with a solid color.
    ///
    /// # Arguments
    ///
    /// * `area` - The rectangular area to fill
    /// * `value` - Grayscale value to fill with
    pub fn fill_area(&mut self, area: &Area, value: u8) -> Result<()>;

    /// Waits for the display to be ready for the next operation.
    ///
    /// This blocks until the LUT (Look-Up Table) engine is free.
    pub fn wait_ready(&mut self) -> Result<()>;

    /// Puts the display into standby mode (low power).
    pub fn standby(&mut self) -> Result<()>;

    /// Resumes the display from standby mode.
    pub fn run(&mut self) -> Result<()>;

    /// Puts the display into sleep mode (lowest power).
    pub fn sleep(&mut self) -> Result<()>;
}
```

### VCOM Management

```rust
impl<SPI, HRDY, CS, RESET> IT8951<SPI, HRDY, CS, RESET>
where
    SPI: SpiInterface,
    HRDY: InputPin,
    CS: OutputPin,
    RESET: OutputPin,
{
    /// Gets the current VCOM voltage value.
    ///
    /// # Returns
    ///
    /// The VCOM value (e.g., 1500 for -1.50V)
    pub fn get_vcom(&mut self) -> Result<u16>;

    /// Sets the VCOM voltage value.
    ///
    /// # Arguments
    ///
    /// * `vcom` - VCOM value (e.g., 1500 for -1.50V)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use it8951::IT8951;
    /// # let mut display = setup();
    /// display.set_vcom(1500)?; // Set to -1.50V
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_vcom(&mut self, vcom: u16) -> Result<()>;
}
```

### Image Operations

```rust
impl<SPI, HRDY, CS, RESET> IT8951<SPI, HRDY, CS, RESET>
where
    SPI: SpiInterface,
    HRDY: InputPin,
    CS: OutputPin,
    RESET: OutputPin,
{
    /// Loads image data into the display buffer.
    ///
    /// # Arguments
    ///
    /// * `data` - Image pixel data (grayscale)
    /// * `x` - X coordinate (left edge)
    /// * `y` - Y coordinate (top edge)
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    /// * `format` - Pixel format
    pub fn load_image(
        &mut self,
        data: &[u8],
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        format: PixelFormat,
    ) -> Result<()>;

    /// Loads an image from a file.
    ///
    /// Supported formats: BMP, PNG, JPEG (with feature flags)
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image file
    /// * `x` - X coordinate (left edge)
    /// * `y` - Y coordinate (top edge)
    #[cfg(feature = "image-support")]
    pub fn load_image_file(
        &mut self,
        path: impl AsRef<std::path::Path>,
        x: u16,
        y: u16,
    ) -> Result<()>;

    /// Draws an image with automatic format conversion.
    ///
    /// # Arguments
    ///
    /// * `image` - Image implementing the `GrayImage` trait
    /// * `x` - X coordinate (left edge)
    /// * `y` - Y coordinate (top edge)
    #[cfg(feature = "image-support")]
    pub fn draw_image(
        &mut self,
        image: &impl GrayImage,
        x: u16,
        y: u16,
    ) -> Result<()>;
}
```

### Graphics Operations

```rust
impl<SPI, HRDY, CS, RESET> IT8951<SPI, HRDY, CS, RESET>
where
    SPI: SpiInterface,
    HRDY: InputPin,
    CS: OutputPin,
    RESET: OutputPin,
{
    /// Draws a pixel at the specified coordinates.
    pub fn draw_pixel(&mut self, x: u16, y: u16, color: u8) -> Result<()>;

    /// Draws a line between two points.
    pub fn draw_line(
        &mut self,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
        color: u8,
    ) -> Result<()>;

    /// Draws a rectangle outline.
    pub fn draw_rect(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        color: u8,
    ) -> Result<()>;

    /// Draws a filled rectangle.
    pub fn fill_rect(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        color: u8,
    ) -> Result<()>;

    /// Draws a circle outline.
    pub fn draw_circle(
        &mut self,
        center_x: u16,
        center_y: u16,
        radius: u16,
        color: u8,
    ) -> Result<()>;

    /// Draws a filled circle.
    pub fn fill_circle(
        &mut self,
        center_x: u16,
        center_y: u16,
        radius: u16,
        color: u8,
    ) -> Result<()>;

    /// Draws text at the specified position.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to draw
    /// * `x` - X coordinate (left edge)
    /// * `y` - Y coordinate (top edge)
    /// * `color` - Text color (grayscale)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use it8951::IT8951;
    /// # let mut display = setup();
    /// display.draw_text("Hello, World!", 10, 10, 0x00)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn draw_text(
        &mut self,
        text: &str,
        x: u16,
        y: u16,
        color: u8,
    ) -> Result<()>;

    /// Draws text with custom font.
    pub fn draw_text_with_font(
        &mut self,
        text: &str,
        x: u16,
        y: u16,
        color: u8,
        font: &Font,
    ) -> Result<()>;
}
```

### Supporting Types

```rust
/// Display refresh modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Pixel formats supported by the IT8951.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

/// Display rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Byte order for pixel data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    /// Little endian
    Little = 0,

    /// Big endian
    Big = 1,
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
        Self { x, y, width, height }
    }

    /// Returns true if the area is valid for the given display dimensions.
    pub fn is_valid(&self, display_width: u16, display_height: u16) -> bool {
        self.x < display_width
            && self.y < display_height
            && self.x + self.width <= display_width
            && self.y + self.height <= display_height
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
}

impl From<(u16, u16, u16, u16)> for Area {
    fn from((x, y, width, height): (u16, u16, u16, u16)) -> Self {
        Self::new(x, y, width, height)
    }
}
```

### Error Handling

```rust
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
    #[error("Timeout waiting for device ready")]
    Timeout,

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(&'static str),

    /// Device not ready
    #[error("Device not ready")]
    NotReady,

    /// Invalid area (out of bounds)
    #[error("Invalid area: {0:?}")]
    InvalidArea(Area),

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
}

/// Result type alias for IT8951 operations.
pub type Result<T> = std::result::Result<T, Error>;
```

## Advanced Features

### Embedded Graphics Integration

```rust
#[cfg(feature = "graphics")]
use embedded_graphics::{
    prelude::*,
    pixelcolor::Gray8,
};

#[cfg(feature = "graphics")]
impl<SPI, HRDY, CS, RESET> DrawTarget for IT8951<SPI, HRDY, CS, RESET>
where
    SPI: SpiInterface,
    HRDY: InputPin,
    CS: OutputPin,
    RESET: OutputPin,
{
    type Color = Gray8;
    type Error = Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<()>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            self.draw_pixel(point.x as u16, point.y as u16, color.luma())?;
        }
        Ok(())
    }
}

#[cfg(feature = "graphics")]
impl<SPI, HRDY, CS, RESET> OriginDimensions for IT8951<SPI, HRDY, CS, RESET>
where
    SPI: SpiInterface,
    HRDY: InputPin,
    CS: OutputPin,
    RESET: OutputPin,
{
    fn size(&self) -> Size {
        Size::new(self.width() as u32, self.height() as u32)
    }
}
```

### Async Support

```rust
#[cfg(feature = "async")]
impl<SPI, HRDY, CS, RESET> IT8951<SPI, HRDY, CS, RESET>
where
    SPI: AsyncSpiInterface,
    HRDY: AsyncInputPin,
    CS: AsyncOutputPin,
    RESET: AsyncOutputPin,
{
    /// Asynchronously clears the display.
    pub async fn clear_async(&mut self, value: u8) -> Result<()>;

    /// Asynchronously refreshes the display.
    pub async fn refresh_async(&mut self, mode: DisplayMode) -> Result<()>;

    /// Asynchronously waits for the display to be ready.
    pub async fn wait_ready_async(&mut self) -> Result<()>;
}
```

## Usage Examples

### Basic Example

```rust
use it8951::{IT8951, DisplayMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = IT8951::builder()
        .spi_device("/dev/spidev0.0")?
        .spi_hz(24_000_000)
        .vcom(1500)
        .build()?;

    display.init()?;

    println!("Display: {}x{}", display.width(), display.height());

    display.clear(0xFF)?;
    display.refresh(DisplayMode::Init)?;

    Ok(())
}
```

### Drawing Example

```rust
use it8951::{IT8951, DisplayMode, Area};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = IT8951::builder()
        .spi_device("/dev/spidev0.0")?
        .build()?;

    display.init()?;
    display.clear(0xFF)?;

    // Draw some shapes
    display.draw_rect(100, 100, 200, 150, 0x00)?;
    display.fill_circle(400, 300, 50, 0x80)?;
    display.draw_text("Hello, World!", 10, 10, 0x00)?;

    // Refresh only the area we drew
    let area = Area::new(0, 0, 600, 400);
    display.refresh_area(&area, DisplayMode::Gc16)?;

    Ok(())
}
```

### Image Display Example

```rust
use it8951::{IT8951, DisplayMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = IT8951::builder()
        .spi_device("/dev/spidev0.0")?
        .build()?;

    display.init()?;

    // Load and display an image
    display.load_image_file("photo.jpg", 0, 0)?;
    display.refresh(DisplayMode::Gc16)?;

    Ok(())
}
```

---

**Last Updated**: 2025-11-16
**Status**: Design Phase
