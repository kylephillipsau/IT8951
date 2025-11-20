# IT8951 E-Paper Display Driver

A safe, ergonomic Rust driver for the IT8951 e-paper controller chip, commonly used in e-paper displays such as the Waveshare 6-inch e-Paper HAT.

## Features

- Type-safe API with compile-time guarantees
- Hardware abstraction layer for portability
- Support for multiple display modes and pixel formats
- Comprehensive error handling
- Mock implementations for testing without hardware
- 72 unit tests passing

## Quick Start

```rust
use it8951::{IT8951, DisplayMode, Area};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build with mock hardware (for testing)
    let mut display = IT8951::builder()
        .vcom(1500)
        .build_mock()?;

    display.init()?;

    // Clear to white
    display.clear(0xFF)?;
    display.refresh(DisplayMode::Gc16)?;

    // Partial update
    let area = Area::new(100, 100, 200, 200);
    display.fill_area(&area, 0x80)?;
    display.refresh_area(&area, DisplayMode::Du)?;

    Ok(())
}
```

## Building

```bash
# Build the library
cargo build

# Run tests
cargo test

# Build for Raspberry Pi (cross-compile)
cargo build --target armv7-unknown-linux-gnueabihf --release

# Generate documentation
cargo doc --open
```

## Implementation Status

### Phase 1: Foundation
- Error types and handling
- HAL traits for SPI and GPIO
- Mock HAL implementations for testing
- Core data structures (Area, DeviceInfo, DisplayMode, etc.)

### Phase 2: Protocol Layer
- Command and register definitions
- Low-level SPI transport with preambles
- Hardware ready synchronization
- Register read/write operations
- Batch data transfer support

### Phase 3: Device Management
- IT8951 device struct with builder pattern
- Device initialization and hardware reset
- Device information retrieval
- VCOM voltage configuration
- Power state management (run/standby/sleep)

### Phase 4: Display Operations
- Clear and fill operations
- Full and partial area refresh
- Image loading with format validation
- Pixel packing for efficient transfer

### Phase 5: Graphics Layer
- Framebuffer for in-memory drawing
- Line drawing (Bresenham's algorithm)
- Rectangle drawing (filled and outline)
- Circle drawing (midpoint algorithm)
- Display integration for framebuffer rendering

## Display Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `Init` | Initialization/clear | Clear ghosting |
| `Du` | Direct Update | Fast monochrome |
| `Gc16` | Grayscale Clearing | High quality 16-level |
| `Gl16` | Grayscale Level | Faster 16-level |
| `A2` | Animation | Very fast updates |

## Pixel Formats

- `Bpp2` - 4 grayscale levels
- `Bpp3` - 8 grayscale levels
- `Bpp4` - 16 grayscale levels
- `Bpp8` - 256 grayscale levels

## Graphics Example

```rust
use it8951::Framebuffer;

// Create a framebuffer
let mut fb = Framebuffer::new(800, 600);

// Draw primitives
fb.clear(0xFF);  // White background
fb.draw_rect(100, 100, 200, 150, 0x00, false)?;  // Black outline
fb.draw_circle(400, 300, 50, 0x80, true)?;  // Gray filled circle
fb.draw_line(0, 0, 799, 599, 0x00)?;  // Diagonal line

// Transfer to display
display.draw_framebuffer(&fb, DisplayMode::Gc16)?;
```

## Hardware Compatibility

- Waveshare 6-inch e-Paper HAT (800x600)
- Other IT8951-based displays
- Raspberry Pi (via SPI)
- Linux systems with SPI support

## Project Structure

```
src/
├── lib.rs              # Public API and re-exports
├── error.rs            # Error types and Result
├── types.rs            # Core data structures
├── hal/                # Hardware abstraction
│   ├── spi.rs          # SPI traits
│   ├── gpio.rs         # GPIO traits
│   └── mock.rs         # Mock implementations
├── protocol/           # IT8951 protocol
│   ├── commands.rs     # Command definitions
│   ├── registers.rs    # Register addresses
│   └── transport.rs    # Low-level operations
├── device/             # Device management
│   ├── mod.rs          # IT8951 struct
│   └── builder.rs      # Builder pattern
├── display/            # Display operations
│   └── mod.rs          # Clear, refresh, load
└── graphics/           # Drawing primitives
    ├── mod.rs          # Framebuffer
    └── display.rs      # Display integration
```

## Comparison to C Implementation

| Feature | C | Rust |
|---------|---|------|
| Memory Safety | Manual | Automatic |
| Error Handling | Return codes | Result<T, E> |
| Testing | Manual | 72 automated tests |
| Documentation | Minimal | Comprehensive |
| Type Safety | Limited | Strong |
| Build System | Make | Cargo |

## References

- [IT8951 Datasheet](http://www.waveshare.net/w/upload/c/c4/IT8951_D_V0.2.4.3_20170728.pdf)
- [Waveshare 6inch e-Paper HAT Wiki](https://www.waveshare.com/wiki/6inch_e-Paper_HAT)
- [Product Page (EN)](https://www.waveshare.com/6inch-e-paper-hat.htm)
- [Product Page (CN)](http://www.waveshare.net/shop/6inch-e-Paper-HAT.htm)

## License

MIT OR Apache-2.0
