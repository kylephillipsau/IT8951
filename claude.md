# IT8951 E-Paper Controller - Rust Port Plan

## Project Overview

This document outlines a comprehensive plan to port the IT8951 e-paper controller library from C to Rust. The IT8951 is a controller chip used in e-paper displays (e.g., Waveshare 6-inch e-Paper HAT) that communicates via SPI interface.

### Reference Implementations
- **C Implementation**: Current codebase using bcm2835 library
- **Python Implementation**: https://github.com/GregDMeyer/IT8951

## Goals

1. **Safety**: Leverage Rust's type system and ownership model for memory safety
2. **Ergonomics**: Provide idiomatic Rust API with builder patterns and error handling
3. **Performance**: Match or exceed C implementation performance
4. **Compatibility**: Support Raspberry Pi and Linux systems with SPI
5. **Testing**: Comprehensive unit, integration, and hardware tests
6. **Documentation**: Full API documentation with examples

## Feature Requirements

### Core Features (Must Have)

#### 1. SPI Communication Layer
- [ ] Low-level SPI interface abstraction
- [ ] Hardware ready (HRDY) pin monitoring
- [ ] Chip select (CS) control
- [ ] Reset pin control
- [ ] Configurable SPI frequency (default 24 MHz, adjustable)
- [ ] Proper timing and wait states
- [ ] Error handling for SPI failures

#### 2. IT8951 Protocol Implementation
- [ ] Preamble-based command/data protocol
  - [ ] Write command (0x6000 preamble)
  - [ ] Write data (0x0000 preamble)
  - [ ] Read data (0x1000 preamble)
- [ ] Wait for ready state management
- [ ] Burst read/write operations
- [ ] Register read/write operations

#### 3. Device Management
- [ ] Device initialization and reset
- [ ] Get device information (panel size, firmware version, LUT version)
- [ ] VCOM voltage get/set operations
- [ ] System run/standby/sleep commands
- [ ] Image buffer base address configuration

#### 4. Memory Operations
- [ ] Memory burst read with trigger
- [ ] Memory burst write
- [ ] Memory burst end command
- [ ] Efficient bulk data transfer

#### 5. Image Loading
- [ ] Load image start/end commands
- [ ] Load image area (partial updates)
- [ ] Support for multiple pixel formats:
  - [ ] 2BPP (4 grayscale levels)
  - [ ] 3BPP (8 grayscale levels)
  - [ ] 4BPP (16 grayscale levels)
  - [ ] 8BPP (256 grayscale levels)
- [ ] Rotation support (0°, 90°, 180°, 270°)
- [ ] Endianness configuration (little/big endian)
- [ ] Packed pixel write operations

#### 6. Display Operations
- [ ] Display area update
- [ ] Display buffer area update
- [ ] Display modes (0-4 for different refresh patterns):
  - [ ] Mode 0: Init (clear to white)
  - [ ] Mode 1: DU (fast monochrome)
  - [ ] Mode 2: GC16 (16-level grayscale)
  - [ ] Mode 3: GL16 (fast 16-level grayscale)
  - [ ] Mode 4: A2 (animation mode)
- [ ] Wait for display ready (LUT engine status)
- [ ] 1BPP bitmap mode with FG/BG color table

#### 7. Graphics Primitives (miniGUI Port)
- [ ] Clear screen
- [ ] Draw pixel
- [ ] Draw line
- [ ] Draw rectangle (outline and filled)
- [ ] Draw circle (outline and filled)
- [ ] Draw ellipse
- [ ] Draw polygon
- [ ] Text rendering with ASCII font
- [ ] BMP image loading and display

### Extended Features (Should Have)

#### 8. Image Processing
- [ ] Image format conversions (RGB to grayscale)
- [ ] Dithering algorithms (Floyd-Steinberg, Atkinson, etc.)
- [ ] Image scaling and cropping
- [ ] Buffer management for partial updates

#### 9. Advanced Display Features
- [ ] Auto LUT (Look-Up Table) mode
- [ ] Partial update optimization
- [ ] Ghosting reduction algorithms
- [ ] Temperature compensation
- [ ] Waveform mode selection

#### 10. Configuration Management
- [ ] Configuration file support (TOML/JSON)
- [ ] Panel-specific configurations
- [ ] Calibration data storage
- [ ] Default settings management

### Nice to Have Features

#### 11. Virtual Display Mode
- [ ] Software emulator for development without hardware
- [ ] Frame buffer visualization
- [ ] Testing and debugging support

#### 12. High-Level Abstractions
- [ ] Canvas/drawing surface abstraction
- [ ] Image trait implementations
- [ ] Integration with `embedded-graphics` crate
- [ ] Double buffering support

#### 13. Performance Optimizations
- [ ] DMA transfer support
- [ ] Asynchronous operations (tokio support)
- [ ] Multi-threaded rendering pipeline
- [ ] Zero-copy operations where possible

## Architecture Design

### Module Structure

```
it8951/
├── Cargo.toml
├── README.md
├── LICENSE
├── examples/
│   ├── basic_display.rs
│   ├── image_display.rs
│   ├── graphics_demo.rs
│   └── benchmark.rs
├── src/
│   ├── lib.rs              # Public API and re-exports
│   ├── error.rs            # Error types and Result aliases
│   ├── hal/                # Hardware abstraction layer
│   │   ├── mod.rs
│   │   ├── spi.rs          # SPI trait and implementations
│   │   ├── gpio.rs         # GPIO control (CS, HRDY, RESET)
│   │   └── mock.rs         # Mock HAL for testing
│   ├── protocol/           # IT8951 protocol
│   │   ├── mod.rs
│   │   ├── commands.rs     # Command constants and types
│   │   ├── registers.rs    # Register addresses and helpers
│   │   └── transport.rs    # Low-level read/write operations
│   ├── device/             # Device management
│   │   ├── mod.rs
│   │   ├── info.rs         # Device info structures
│   │   ├── init.rs         # Initialization logic
│   │   └── vcom.rs         # VCOM management
│   ├── display/            # Display operations
│   │   ├── mod.rs
│   │   ├── modes.rs        # Display modes enumeration
│   │   ├── area.rs         # Area update operations
│   │   └── buffer.rs       # Frame buffer management
│   ├── image/              # Image handling
│   │   ├── mod.rs
│   │   ├── loader.rs       # Image loading (BMP, PNG, etc.)
│   │   ├── format.rs       # Pixel format conversions
│   │   └── dither.rs       # Dithering algorithms
│   ├── graphics/           # Graphics primitives
│   │   ├── mod.rs
│   │   ├── draw.rs         # Drawing operations
│   │   ├── text.rs         # Text rendering
│   │   └── font.rs         # Font management
│   └── memory/             # Memory operations
│       ├── mod.rs
│       └── transfer.rs     # Burst read/write operations
└── tests/
    ├── integration/
    │   ├── hardware_tests.rs
    │   └── protocol_tests.rs
    └── unit/
        └── ...
```

### Core Types and Traits

```rust
// Main controller type
pub struct IT8951<SPI, HRDY, CS, RESET> {
    spi: SPI,
    hrdy: HRDY,
    cs: CS,
    reset: RESET,
    device_info: DeviceInfo,
    img_buf_addr: u32,
    frame_buffer: Vec<u8>,
}

// Device information
pub struct DeviceInfo {
    pub panel_width: u16,
    pub panel_height: u16,
    pub img_buf_addr: u32,
    pub fw_version: String,
    pub lut_version: String,
}

// Display configuration
pub struct DisplayConfig {
    pub spi_hz: u32,
    pub vcom: u16,
    pub rotation: Rotation,
}

// Image loading information
pub struct LoadImageInfo {
    pub endian: Endian,
    pub pixel_format: PixelFormat,
    pub rotate: Rotation,
    pub start_fb_addr: u32,
    pub img_buf_base_addr: u32,
}

// Area specification
pub struct Area {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

// Enumerations
pub enum Rotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

pub enum PixelFormat {
    Bpp2,   // 4 gray levels
    Bpp3,   // 8 gray levels
    Bpp4,   // 16 gray levels
    Bpp8,   // 256 gray levels
}

pub enum DisplayMode {
    Init,       // Mode 0
    Du,         // Mode 1
    Gc16,       // Mode 2
    Gl16,       // Mode 3
    A2,         // Mode 4
}

pub enum Endian {
    Little,
    Big,
}

// Error type
pub enum Error {
    SpiError,
    TimeoutError,
    InvalidParameter,
    NotReady,
    DeviceError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### API Design Examples

```rust
// Initialization with builder pattern
let mut display = IT8951::builder()
    .spi_device("/dev/spidev0.0")?
    .spi_hz(24_000_000)
    .vcom(1500)
    .cs_pin(8)
    .hrdy_pin(24)
    .reset_pin(17)
    .build()?;

// Initialize the device
display.init()?;

// Get device info
let info = display.device_info();
println!("Panel: {}x{}", info.panel_width, info.panel_height);

// Clear screen to white
display.clear(0xFF)?;

// Load and display an image
let img = image::load_from_file("test.bmp")?;
display.draw_image(&img, 0, 0)?;
display.refresh(DisplayMode::Gc16)?;

// Draw graphics
display.draw_rect(100, 100, 200, 150, 0x00)?;
display.draw_circle(400, 300, 50, 0x80)?;
display.draw_text("Hello, World!", 10, 10, 0x00)?;
display.refresh_area(Area::new(0, 0, 600, 400), DisplayMode::Du)?;

// Partial update
display.update_area(Area::new(100, 100, 300, 200), |buf| {
    // Modify buffer in place
    buf.fill(0x80);
})?;

// Standby mode
display.standby()?;

// Wake up
display.run()?;

// Sleep mode
display.sleep()?;
```

## Dependencies

### Required Crates

```toml
[dependencies]
# SPI and GPIO
spidev = "0.6"                    # Linux SPI device access
gpio-cdev = "0.6"                  # GPIO character device
rppal = { version = "0.18", optional = true }  # Raspberry Pi specific

# Error handling
thiserror = "1.0"                  # Error derive macros
anyhow = "1.0"                     # Flexible error handling

# Image processing
image = "0.25"                     # Image format support
embedded-graphics = { version = "0.8", optional = true }

# Utilities
byteorder = "1.5"                  # Endianness handling
bitflags = "2.4"                   # Bit flag macros

# Async support (optional)
tokio = { version = "1.0", optional = true, features = ["full"] }

# Serialization for config
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

[dev-dependencies]
# Testing
criterion = "0.5"                  # Benchmarking
mockall = "0.12"                   # Mocking
proptest = "1.4"                   # Property testing

# Examples
structopt = "0.3"                  # CLI argument parsing
```

## Testing Strategy

### Unit Tests
- Mock SPI/GPIO interfaces for protocol testing
- Test command encoding/decoding
- Test data structure conversions
- Test error handling paths
- Test pixel format conversions
- Property-based tests for graphics primitives

### Integration Tests
- Test complete initialization flow
- Test image loading pipeline
- Test display update operations
- Test memory read/write operations
- Test VCOM configuration

### Hardware Tests
- Conditional compilation for hardware-specific tests
- Actual display operations on real hardware
- Performance benchmarks
- Visual regression testing (capture display states)
- Long-running stability tests

### Continuous Integration
- GitHub Actions for automated testing
- Cross-compilation for ARM targets
- Documentation generation and hosting
- Code coverage reporting
- Clippy and rustfmt checks

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
1. Set up Cargo project structure
2. Define error types and core data structures
3. Implement HAL traits for SPI and GPIO
4. Create mock implementations for testing
5. Write comprehensive documentation framework

### Phase 2: Protocol Layer (Week 2-3)
1. Implement low-level SPI read/write operations
2. Implement command protocol (preambles, waiting)
3. Implement register read/write
4. Implement memory burst operations
5. Add protocol-level unit tests

### Phase 3: Device Management (Week 3-4)
1. Implement device initialization
2. Implement device info retrieval
3. Implement VCOM management
4. Implement power management (run/standby/sleep)
5. Test initialization flow

### Phase 4: Display Operations (Week 4-5)
1. Implement image loading (area and full screen)
2. Implement display area update
3. Implement display modes
4. Implement wait for ready
5. Implement 1BPP mode support
6. Test basic display operations

### Phase 5: Graphics Layer (Week 5-6)
1. Port miniGUI drawing primitives
2. Implement text rendering
3. Implement BMP loader
4. Add additional image format support
5. Test graphics operations

### Phase 6: Polish and Optimization (Week 6-7)
1. Performance optimization
2. Memory usage optimization
3. Error handling improvements
4. API refinement based on usage
5. Comprehensive examples

### Phase 7: Documentation and Release (Week 7-8)
1. Complete API documentation
2. Write user guide
3. Create tutorial examples
4. Write migration guide from C/Python
5. Prepare crates.io release

## Safety Considerations

### Memory Safety
- Use Rust's ownership system to prevent buffer overflows
- Validate all array accesses
- Use checked arithmetic for address calculations
- Proper lifetime management for borrowed data

### Hardware Safety
- Validate pin configurations before use
- Timeout mechanisms for hardware waits
- Graceful handling of SPI errors
- Proper cleanup on Drop (reset device state)

### Type Safety
- Strong typing for commands, registers, and modes
- Type-state pattern for device states
- Builder pattern with compile-time validation
- Newtype pattern for domain-specific values

## Performance Targets

- Match or exceed C implementation speed
- SPI transfer rate: 24 MHz (default), up to 32 MHz tested
- Full screen refresh: < 2 seconds for GC16 mode
- Partial update: < 500ms for small areas
- Memory allocation: Minimize during hot paths
- Zero-copy where possible for image data

## Documentation Requirements

### API Documentation
- All public items must have doc comments
- Examples for all major features
- Document safety requirements
- Document error conditions
- Link related items

### User Guide
- Getting started tutorial
- Hardware setup instructions
- Configuration guide
- Common patterns and recipes
- Troubleshooting guide
- Performance tuning guide

### Examples
- Basic display operations
- Image display
- Graphics rendering
- Text display
- Partial updates
- Power management
- Performance benchmarks

## Migration Guide

For users coming from C or Python implementations:
- API comparison table
- Code migration examples
- Feature parity matrix
- Performance comparison
- Common pitfalls and solutions

## Open Questions

1. **Async Support**: Should we provide async APIs using tokio? Useful for non-blocking operations.
2. **embedded-hal**: Should we support embedded-hal traits for broader embedded ecosystem compatibility?
3. **Virtual Display**: Priority for software emulator for testing?
4. **Multi-threading**: Should we support concurrent operations (e.g., rendering while displaying)?
5. **FFI**: Should we provide C FFI for compatibility with existing systems?

## Success Criteria

- [ ] All core features implemented and tested
- [ ] API is idiomatic and ergonomic
- [ ] Performance matches or exceeds C implementation
- [ ] Comprehensive documentation
- [ ] CI/CD pipeline operational
- [ ] At least 80% code coverage
- [ ] Zero unsafe code where possible
- [ ] Published to crates.io
- [ ] Community adoption and feedback

## References

- [IT8951 Datasheet](http://www.waveshare.net/w/upload/c/c4/IT8951_D_V0.2.4.3_20170728.pdf)
- [Waveshare 6inch e-Paper HAT](https://www.waveshare.com/wiki/6inch_e-Paper_HAT)
- [Python Implementation](https://github.com/GregDMeyer/IT8951)
- [Rust Embedded Book](https://rust-embedded.github.io/book/)
- [spidev Documentation](https://docs.rs/spidev/)
- [gpio-cdev Documentation](https://docs.rs/gpio-cdev/)

---

**Last Updated**: 2025-11-16
**Status**: Planning Phase
**Next Review**: Start of Phase 1 Implementation
