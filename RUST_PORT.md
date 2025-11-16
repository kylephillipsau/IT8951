# IT8951 Rust Port

## Quick Start

This directory contains a C implementation of the IT8951 e-paper controller library. We are in the process of porting this to Rust for improved safety, ergonomics, and maintainability.

## Project Status

**Current Phase**: Planning ✅
**Next Phase**: Foundation Implementation

See [claude.md](./claude.md) for the comprehensive implementation plan.

## Why Rust?

### Safety Benefits
- **Memory Safety**: No buffer overflows, null pointer dereferences, or use-after-free bugs
- **Thread Safety**: Compile-time prevention of data races
- **Type Safety**: Strong typing prevents many classes of errors

### Developer Experience
- **Modern Tooling**: Cargo for build, test, and dependency management
- **Documentation**: Built-in doc generation and testing
- **Error Handling**: Explicit error handling with Result types
- **Package Ecosystem**: Access to crates.io ecosystem

### Performance
- **Zero-Cost Abstractions**: High-level code compiles to efficient machine code
- **No Garbage Collection**: Predictable performance without GC pauses
- **LLVM Backend**: Advanced optimizations

## Planned Project Structure

```
it8951-rs/                      # New Rust crate
├── Cargo.toml                  # Rust project manifest
├── README.md                   # Rust-specific README
├── src/
│   ├── lib.rs                  # Main library
│   ├── hal/                    # Hardware abstraction
│   ├── protocol/               # IT8951 protocol
│   ├── device/                 # Device management
│   ├── display/                # Display operations
│   ├── graphics/               # Graphics primitives
│   ├── image/                  # Image handling
│   └── memory/                 # Memory operations
├── examples/                   # Example programs
│   ├── basic_display.rs
│   ├── image_display.rs
│   └── graphics_demo.rs
├── tests/                      # Integration tests
└── benches/                    # Benchmarks
```

## Feature Parity Matrix

| Feature | C Implementation | Rust Implementation | Status |
|---------|-----------------|---------------------|--------|
| SPI Communication | ✅ bcm2835 | 🔄 spidev/rppal | Planned |
| Device Initialization | ✅ | 🔄 | Planned |
| Image Loading (8bpp) | ✅ | 🔄 | Planned |
| Image Loading (1bpp) | ✅ | 🔄 | Planned |
| Display Modes | ✅ | 🔄 | Planned |
| VCOM Management | ✅ | 🔄 | Planned |
| Memory Operations | ✅ | 🔄 | Planned |
| Graphics Primitives | ✅ | 🔄 | Planned |
| Text Rendering | ✅ | 🔄 | Planned |
| BMP Loading | ✅ | 🔄 | Planned |
| Power Management | ✅ | 🔄 | Planned |
| Error Handling | ⚠️ Basic | 🔄 Enhanced | Planned |
| Testing | ⚠️ Manual | 🔄 Automated | Planned |
| Documentation | ⚠️ Minimal | 🔄 Comprehensive | Planned |

Legend: ✅ Complete | 🔄 In Progress | ⚠️ Limited | ❌ Missing

## Building the Rust Port

Once implemented, building will be as simple as:

```bash
# Build the library
cargo build --release

# Run tests
cargo test

# Run an example
cargo run --example basic_display

# Generate documentation
cargo doc --open
```

## Cross-Compilation for Raspberry Pi

```bash
# Install cross-compilation tools
rustup target add armv7-unknown-linux-gnueabihf

# Build for Raspberry Pi
cargo build --release --target armv7-unknown-linux-gnueabihf
```

## API Preview

Here's what the Rust API will look like:

```rust
use it8951::{IT8951, DisplayMode, PixelFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the display
    let mut display = IT8951::builder()
        .spi_device("/dev/spidev0.0")?
        .spi_hz(24_000_000)
        .vcom(1500)
        .build()?;

    display.init()?;

    // Get device information
    let info = display.device_info();
    println!("Panel: {}x{}", info.panel_width, info.panel_height);

    // Clear the screen
    display.clear(0xFF)?;

    // Load and display an image
    display.load_image("test.bmp", 0, 0)?;
    display.refresh(DisplayMode::Gc16)?;

    // Draw some graphics
    display.draw_rect(100, 100, 200, 150, 0x00)?;
    display.draw_text("Hello from Rust!", 10, 10, 0x00)?;
    display.refresh_area((0, 0, 600, 400), DisplayMode::Du)?;

    Ok(())
}
```

Compare this to the C version:

```c
// C version - more verbose and error-prone
IT8951_Init();
GetIT8951SystemInfo(&gstI80DevInfo);

IT8951LdImgInfo stLdImgInfo;
IT8951AreaImgInfo stAreaImgInfo;

memset(gpFrameBuf, 0xF0, gstI80DevInfo.usPanelW * gstI80DevInfo.usPanelH);
stLdImgInfo.ulStartFBAddr = (uint32_t)gpFrameBuf;
stLdImgInfo.usEndianType = IT8951_LDIMG_L_ENDIAN;
stLdImgInfo.usPixelFormat = IT8951_8BPP;
stLdImgInfo.usRotate = IT8951_ROTATE_0;
stLdImgInfo.ulImgBufBaseAddr = gulImgBufAddr;
stAreaImgInfo.usX = 0;
stAreaImgInfo.usY = 0;
stAreaImgInfo.usWidth = gstI80DevInfo.usPanelW;
stAreaImgInfo.usHeight = gstI80DevInfo.usPanelH;

IT8951HostAreaPackedPixelWrite(&stLdImgInfo, &stAreaImgInfo);
IT8951DisplayArea(0, 0, gstI80DevInfo.usPanelW, gstI80DevInfo.usPanelH, 0);

IT8951_Cancel();
```

## Performance Comparison

Expected performance characteristics:

| Operation | C Implementation | Rust Implementation (Target) |
|-----------|-----------------|------------------------------|
| Full Screen Update | ~2s | ~2s (same) |
| Partial Update | ~500ms | ~400ms (optimized) |
| Memory Allocation | Manual | Zero-copy where possible |
| Error Handling | Return codes | Result<T, E> |
| Compile Time | Fast | Slower (but only once) |
| Runtime Safety | Manual | Automatic |

## Testing Approach

The Rust implementation will include:

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_format_conversion() {
        let bpp8 = PixelFormat::Bpp8;
        assert_eq!(bpp8.to_bits(), 3);
    }

    #[test]
    fn test_area_validation() {
        let area = Area::new(0, 0, 800, 600);
        assert!(area.is_valid(800, 600));
    }
}
```

### Integration Tests
```rust
#[test]
#[ignore] // Requires hardware
fn test_device_initialization() {
    let display = IT8951::builder()
        .spi_device("/dev/spidev0.0")
        .unwrap()
        .build()
        .unwrap();

    let info = display.device_info();
    assert!(info.panel_width > 0);
    assert!(info.panel_height > 0);
}
```

### Hardware Tests
- Conditional compilation with `--features hardware-tests`
- Visual regression testing
- Performance benchmarking with Criterion

## Migration Path

For existing users of the C library:

1. **Side-by-side**: Run both implementations during transition
2. **Feature Flags**: Use Cargo features to toggle implementations
3. **FFI Bridge**: Optionally expose Rust implementation to C
4. **Gradual Migration**: Migrate one feature at a time

## Contributing to the Rust Port

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/IT8951.git
cd IT8951

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Format code
cargo fmt

# Lint code
cargo clippy

# Run tests
cargo test
```

### Code Style

- Follow Rust standard style (enforced by rustfmt)
- Use clippy for linting
- Write doc comments for all public items
- Include examples in doc comments
- Prefer iterators over loops
- Use the type system to prevent errors

## Timeline

See [claude.md](./claude.md) for detailed phase breakdown.

**Estimated Duration**: 6-8 weeks for full implementation

- **Phase 1**: Foundation (Week 1-2)
- **Phase 2**: Protocol Layer (Week 2-3)
- **Phase 3**: Device Management (Week 3-4)
- **Phase 4**: Display Operations (Week 4-5)
- **Phase 5**: Graphics Layer (Week 5-6)
- **Phase 6**: Polish and Optimization (Week 6-7)
- **Phase 7**: Documentation and Release (Week 7-8)

## Resources

### Learning Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust Embedded Book](https://rust-embedded.github.io/book/)

### Related Crates
- [spidev](https://crates.io/crates/spidev) - Linux SPI device access
- [gpio-cdev](https://crates.io/crates/gpio-cdev) - GPIO access
- [rppal](https://crates.io/crates/rppal) - Raspberry Pi peripheral access
- [embedded-graphics](https://crates.io/crates/embedded-graphics) - 2D graphics library
- [image](https://crates.io/crates/image) - Image processing

### IT8951 Resources
- [IT8951 Datasheet](http://www.waveshare.net/w/upload/c/c4/IT8951_D_V0.2.4.3_20170728.pdf)
- [Waveshare Wiki](https://www.waveshare.com/wiki/6inch_e-Paper_HAT)
- [Python Implementation](https://github.com/GregDMeyer/IT8951)

## License

The Rust port will maintain the same license as the original C implementation.

## Questions or Issues?

- Check [claude.md](./claude.md) for the comprehensive plan
- Open an issue on GitHub
- Refer to the Rust documentation when complete

---

**Status**: Planning Complete ✅
**Next Step**: Begin Phase 1 - Foundation Implementation
