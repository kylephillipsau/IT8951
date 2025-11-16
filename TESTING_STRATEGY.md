# IT8951 Rust Port - Testing Strategy

## Overview

This document outlines the comprehensive testing strategy for the IT8951 Rust port. Our goal is to achieve high code quality, reliability, and maintainability through multiple layers of testing.

## Testing Philosophy

1. **Test Early, Test Often**: Write tests alongside implementation
2. **Pyramid Approach**: Many unit tests, fewer integration tests, selective hardware tests
3. **Automation First**: All tests should be automatable via CI/CD
4. **Mock Hardware**: Use mocks for development without physical devices
5. **Document Tests**: Tests serve as living documentation

## Test Layers

### 1. Unit Tests

#### Scope
- Individual functions and methods
- Data structure conversions
- Protocol encoding/decoding
- Error handling logic
- Pure computation without hardware

#### Location
```rust
// In-module tests
// src/protocol/commands.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_encoding() {
        let cmd = Command::SystemRun;
        assert_eq!(cmd.to_u16(), 0x0001);
    }

    #[test]
    fn test_pixel_format_bits() {
        assert_eq!(PixelFormat::Bpp8.to_bits(), 3);
        assert_eq!(PixelFormat::Bpp4.to_bits(), 2);
        assert_eq!(PixelFormat::Bpp3.to_bits(), 1);
        assert_eq!(PixelFormat::Bpp2.to_bits(), 0);
    }
}
```

#### Coverage Target
- **Goal**: 90%+ code coverage for unit-testable code
- **Focus**: Logic-heavy modules (protocol, image processing, graphics)

#### Tools
- Built-in Rust test framework
- `cargo test`
- `cargo-tarpaulin` for coverage reporting

### 2. Property-Based Tests

#### Scope
- Test invariants that should hold for all inputs
- Fuzz testing for edge cases
- Validate conversions are lossless where expected

#### Examples
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_area_coordinates_valid(
        x in 0u16..800,
        y in 0u16..600,
        w in 1u16..=800,
        h in 1u16..=600
    ) {
        let area = Area { x, y, width: w, height: h };
        prop_assert!(area.x + area.width <= 800);
        prop_assert!(area.y + area.height <= 600);
    }

    #[test]
    fn test_grayscale_conversion_range(value in 0u8..=255) {
        let gray = GrayScale::from_u8(value);
        prop_assert!(gray.value() <= 255);
    }
}
```

### 3. Mock-Based Integration Tests

#### Scope
- Test component interactions without hardware
- Validate protocol sequences
- Test state management

#### Implementation
```rust
use mockall::*;

#[automock]
trait SpiInterface {
    fn transfer(&mut self, data: &[u8]) -> Result<Vec<u8>>;
}

#[test]
fn test_device_initialization_sequence() {
    let mut mock_spi = MockSpiInterface::new();

    // Expect reset sequence
    mock_spi
        .expect_transfer()
        .times(1)
        .returning(|_| Ok(vec![0x00, 0x00]));

    // Expect device info request
    mock_spi
        .expect_transfer()
        .times(1)
        .returning(|_| Ok(device_info_response()));

    let mut device = IT8951::new_with_spi(mock_spi);
    device.init().expect("Init should succeed");
}
```

### 4. Integration Tests

#### Scope
- Test public API without mocks
- End-to-end workflows (where possible without hardware)
- Virtual display mode testing

#### Location
```
tests/
├── integration/
│   ├── protocol_tests.rs
│   ├── display_tests.rs
│   ├── graphics_tests.rs
│   └── image_tests.rs
└── common/
    └── mod.rs  // Shared test utilities
```

#### Examples
```rust
// tests/integration/display_tests.rs
use it8951::*;

#[test]
#[cfg(feature = "virtual-display")]
fn test_full_display_workflow() {
    let mut display = IT8951::builder()
        .virtual_display()
        .panel_size(800, 600)
        .build()
        .expect("Failed to create virtual display");

    display.init().expect("Init failed");
    display.clear(0xFF).expect("Clear failed");

    let info = display.device_info();
    assert_eq!(info.panel_width, 800);
    assert_eq!(info.panel_height, 600);
}

#[test]
#[cfg(feature = "virtual-display")]
fn test_partial_update() {
    let mut display = setup_virtual_display();

    let area = Area::new(100, 100, 200, 200);
    display.fill_area(&area, 0x80).expect("Fill failed");
    display.refresh_area(&area, DisplayMode::Du).expect("Refresh failed");

    // Verify buffer contents
    let buffer = display.get_buffer_area(&area);
    assert!(buffer.iter().all(|&pixel| pixel == 0x80));
}
```

### 5. Hardware Tests

#### Scope
- Tests that require actual IT8951 hardware
- Visual verification tests
- Performance benchmarks on real hardware
- Long-running stability tests

#### Conditional Compilation
```rust
#[cfg(all(test, feature = "hardware-tests"))]
mod hardware_tests {
    use super::*;

    #[test]
    #[ignore]  // Ignored by default, run with --ignored
    fn test_real_device_initialization() {
        let mut display = IT8951::builder()
            .spi_device("/dev/spidev0.0")
            .expect("SPI init failed")
            .build()
            .expect("Build failed");

        display.init().expect("Init failed");

        let info = display.device_info();
        assert!(info.panel_width > 0);
        assert!(info.panel_height > 0);
        assert!(!info.fw_version.is_empty());
    }

    #[test]
    #[ignore]
    fn test_display_pattern() {
        let mut display = setup_real_device();

        // Display a test pattern
        for i in 0..16 {
            let gray = (i * 16) as u8;
            let y = i * (display.height() / 16);
            let area = Area::new(0, y, display.width(), display.height() / 16);
            display.fill_area(&area, gray).expect("Fill failed");
        }

        display.refresh(DisplayMode::Gc16).expect("Refresh failed");

        // Manual verification required
        println!("Check display shows 16 gray bars");
        println!("Press Enter to continue...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
    }
}
```

#### Running Hardware Tests
```bash
# Run hardware tests (requires --features hardware-tests --ignored)
cargo test --features hardware-tests -- --ignored --test-threads=1

# Run specific hardware test
cargo test --features hardware-tests test_real_device_initialization -- --ignored
```

### 6. Visual Regression Tests

#### Scope
- Capture display state at various points
- Compare against known-good baselines
- Detect unintended visual changes

#### Implementation
```rust
#[cfg(feature = "visual-regression")]
fn capture_display_state(display: &IT8951) -> DisplaySnapshot {
    DisplaySnapshot {
        buffer: display.get_buffer().to_vec(),
        width: display.width(),
        height: display.height(),
        timestamp: SystemTime::now(),
    }
}

#[test]
#[cfg(feature = "visual-regression")]
fn test_gradient_rendering() {
    let mut display = setup_virtual_display();

    // Render gradient
    for y in 0..display.height() {
        let gray = (y * 255 / display.height()) as u8;
        display.draw_line(0, y, display.width(), y, gray).unwrap();
    }

    let snapshot = capture_display_state(&display);
    let baseline = load_baseline("gradient_test.png");

    assert_images_similar(&snapshot, &baseline, 0.99);
}
```

### 7. Performance Benchmarks

#### Scope
- Measure operation timings
- Compare against C implementation
- Track performance regressions

#### Implementation
```rust
// benches/display_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use it8951::*;

fn benchmark_full_screen_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("display");

    for size in [600, 800, 1024].iter() {
        group.bench_with_input(
            BenchmarkId::new("full_update", size),
            size,
            |b, &size| {
                let mut display = setup_virtual_display_size(size, size);
                b.iter(|| {
                    display.clear(black_box(0xFF)).unwrap();
                    display.refresh(DisplayMode::Gc16).unwrap();
                });
            },
        );
    }

    group.finish();
}

fn benchmark_partial_update(c: &mut Criterion) {
    let mut display = setup_virtual_display();

    c.bench_function("partial_update_100x100", |b| {
        let area = Area::new(0, 0, 100, 100);
        b.iter(|| {
            display.fill_area(&area, black_box(0x80)).unwrap();
            display.refresh_area(&area, DisplayMode::Du).unwrap();
        });
    });
}

criterion_group!(benches, benchmark_full_screen_update, benchmark_partial_update);
criterion_main!(benches);
```

#### Running Benchmarks
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench display

# Generate HTML report
cargo bench -- --save-baseline master
```

### 8. Fuzz Testing

#### Scope
- Find edge cases and crashes
- Test robustness against malformed input
- Discover undefined behavior

#### Implementation
```rust
// fuzz/fuzz_targets/protocol_parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use it8951::protocol::parse_device_info;

fuzz_target!(|data: &[u8]| {
    // Should never panic, regardless of input
    let _ = parse_device_info(data);
});
```

#### Running Fuzz Tests
```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run fuzzer
cargo fuzz run protocol_parser

# Run with coverage
cargo fuzz coverage protocol_parser
```

## Test Organization

### Directory Structure
```
it8951/
├── src/
│   ├── lib.rs
│   ├── protocol/
│   │   ├── mod.rs
│   │   └── tests.rs        # Unit tests
│   └── ...
├── tests/
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── protocol_tests.rs
│   │   └── display_tests.rs
│   ├── hardware/
│   │   ├── mod.rs
│   │   └── device_tests.rs
│   └── common/
│       └── mod.rs           # Shared test utilities
├── benches/
│   └── display_benchmark.rs
├── fuzz/
│   └── fuzz_targets/
│       └── protocol_parser.rs
└── examples/
    └── ...
```

### Test Utilities
```rust
// tests/common/mod.rs
use it8951::*;

pub fn setup_virtual_display() -> IT8951<MockSpi, MockGpio, MockGpio, MockGpio> {
    IT8951::builder()
        .virtual_display()
        .panel_size(800, 600)
        .build()
        .expect("Failed to create virtual display")
}

pub fn assert_images_similar(img1: &[u8], img2: &[u8], threshold: f64) {
    assert_eq!(img1.len(), img2.len());
    let differences: usize = img1
        .iter()
        .zip(img2.iter())
        .filter(|(a, b)| a != b)
        .count();
    let similarity = 1.0 - (differences as f64 / img1.len() as f64);
    assert!(similarity >= threshold, "Images differ by {}%", (1.0 - similarity) * 100.0);
}
```

## Continuous Integration

### GitHub Actions Workflow
```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta, nightly]
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
          override: true
      - name: Run tests
        run: cargo test --all-features

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy
          override: true
      - name: Run clippy
        run: cargo clippy --all-features -- -D warnings

  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt
          override: true
      - name: Run rustfmt
        run: cargo fmt --all -- --check

  coverage:
    name: Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Generate coverage
        run: cargo tarpaulin --out Xml --all-features
      - name: Upload coverage
        uses: codecov/codecov-action@v3

  bench:
    name: Benchmarks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Run benchmarks
        run: cargo bench --no-fail-fast

  cross-compile:
    name: Cross Compile
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - armv7-unknown-linux-gnueabihf
          - aarch64-unknown-linux-gnu
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
          override: true
      - name: Build
        run: cargo build --target ${{ matrix.target }} --release
```

## Test Coverage Goals

| Component | Target Coverage | Priority |
|-----------|----------------|----------|
| Protocol Layer | 95% | Critical |
| Device Management | 90% | High |
| Display Operations | 85% | High |
| Graphics Primitives | 80% | Medium |
| Image Processing | 85% | Medium |
| Error Handling | 100% | Critical |
| HAL Abstraction | 90% | High |

## Testing Best Practices

### 1. Test Naming
```rust
// Good: Descriptive test names
#[test]
fn test_device_info_parsing_with_valid_data() { }

#[test]
fn test_display_area_returns_error_when_coordinates_out_of_bounds() { }

// Bad: Vague test names
#[test]
fn test1() { }

#[test]
fn test_display() { }
```

### 2. Arrange-Act-Assert Pattern
```rust
#[test]
fn test_area_intersection() {
    // Arrange
    let area1 = Area::new(0, 0, 100, 100);
    let area2 = Area::new(50, 50, 100, 100);

    // Act
    let intersection = area1.intersect(&area2);

    // Assert
    assert_eq!(intersection, Some(Area::new(50, 50, 50, 50)));
}
```

### 3. Test One Thing
```rust
// Good: Tests one specific behavior
#[test]
fn test_vcom_setter_validates_range() {
    let mut device = setup_device();
    assert!(device.set_vcom(5000).is_err());
}

#[test]
fn test_vcom_getter_returns_configured_value() {
    let mut device = setup_device();
    device.set_vcom(1500).unwrap();
    assert_eq!(device.get_vcom().unwrap(), 1500);
}
```

### 4. Use Assertions Wisely
```rust
// Good: Specific assertion
assert_eq!(result.width, 800, "Width should be 800");

// Better: Custom assertion message
assert!(
    result.is_valid(),
    "Area should be valid: {:?}",
    result
);
```

## Documentation Tests

All public APIs should have doc tests:

```rust
/// Clears the display to the specified grayscale value.
///
/// # Arguments
///
/// * `value` - Grayscale value (0x00 = black, 0xFF = white)
///
/// # Examples
///
/// ```
/// use it8951::IT8951;
///
/// let mut display = IT8951::builder().virtual_display().build()?;
/// display.clear(0xFF)?; // Clear to white
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an error if the SPI communication fails.
pub fn clear(&mut self, value: u8) -> Result<()> {
    // Implementation
}
```

## Test Maintenance

1. **Keep Tests Fast**: Unit tests should run in milliseconds
2. **Isolate Tests**: No dependencies between tests
3. **Clean Up**: Use RAII or explicit cleanup
4. **Mock External Dependencies**: Don't rely on network, filesystem, or hardware
5. **Regular Review**: Update tests when requirements change
6. **Refactor Tests**: Apply DRY principle to test code too

## Success Metrics

- [ ] 80%+ overall code coverage
- [ ] 95%+ coverage for critical paths
- [ ] All tests pass on CI
- [ ] No clippy warnings
- [ ] Benchmarks show acceptable performance
- [ ] Zero unsafe code violations detected
- [ ] All public APIs documented with examples
- [ ] Hardware tests validated on real device

---

**Last Updated**: 2025-11-16
**Status**: Planning Phase
