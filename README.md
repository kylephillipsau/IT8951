# it8951

Rust driver for the IT8951 e-paper controller used by the Waveshare e-Paper HATs
(6", 7.8", 9.7", 10.3"). Talks to the controller over SPI plus two GPIOs, exposes the
IT8951 command set behind a small typed API, and includes a mock HAL so the protocol
layer is unit-tested without hardware.

Tested on a Raspberry Pi 4 with the Waveshare 9.7" HAT (1200×825, firmware `SWv_0.2.1T`,
M841 waveforms). The numbers in this document were measured on that setup.

## Contents

- [Hardware setup (Raspberry Pi)](#hardware-setup-raspberry-pi)
- [Using the driver](#using-the-driver)
- [How the IT8951 behaves](#how-the-it8951-behaves)
- [Display modes](#display-modes)
- [Pixel formats and buffer layout](#pixel-formats-and-buffer-layout)
- [Getting the most out of it](#getting-the-most-out-of-it)
- [Building and testing](#building-and-testing)
- [Project structure](#project-structure)
- [Related projects and references](#related-projects-and-references)

## Hardware setup (Raspberry Pi)

### Wiring

The Waveshare HAT plugs onto the 40-pin header and uses:

| Signal | Pi pin | Default in this crate |
|---|---|---|
| SPI (MOSI/MISO/SCLK/CE0) | SPI0, chip select CE0 | `/dev/spidev0.0` |
| HRDY (host ready, active high) | GPIO 24 | `hal::linux::pins::HRDY` |
| RST (reset, active low) | GPIO 17 | `hal::linux::pins::RST` |

Chip select is driven by the kernel SPI driver, not by the crate.

### `/boot/firmware/config.txt`

```ini
dtparam=spi=on

# Pin the GPU core clock. The SPI clock divider is computed for a 500 MHz core, but a
# Pi 4's core floats between ~200 and 500 MHz when the GPU is idle, which silently
# drops a "24 MHz" bus to 9-12 MHz (measured 1.15 MB/s instead of 2.76 MB/s).
core_freq=500
core_freq_min=500
```

### `/boot/firmware/cmdline.txt`

Add `spidev.bufsiz=65536` so one SPI transfer can carry 64 KiB. The default of 4 KiB
would turn every image load into hundreds of tiny transfers.

### Permissions

The user running the driver needs the `spi` and `gpio` groups
(`sudo usermod -aG spi,gpio $USER`, then log in again).

### Verify

```bash
ls /dev/spidev0.0                       # SPI device present
cat /sys/module/spidev/parameters/bufsiz # 65536
vcgencmd measure_clock core             # 500000000, constantly
```

## Using the driver

```rust
use it8951::{Area, DisplayMode, IT8951Builder, PixelFormat};

fn main() -> it8951::Result<()> {
    let mut display = IT8951Builder::new()
        .vcom(1500)              // the value printed on the panel's flex cable, in mV
        .spi_data_hz(24_000_000) // datasheet maximum; see "SPI clock" below
        .build()?;               // or .build_with_spi("/dev/spidev0.1")
    display.init()?;             // hardware reset, device info, VCOM
    println!("{}x{}", display.width(), display.height());

    // Fill the panel white and refresh it.
    display.clear(0xFF)?;
    display.refresh(DisplayMode::Gc16)?;
    display.wait_display_ready()?;

    // Load an image into a region and refresh only that region.
    let area = Area::new(100, 100, 400, 300);
    let pixels = vec![0x00u8; 400 * 300]; // 8bpp, row-major, 0x00 = black
    display.load_image(&pixels, &area, PixelFormat::Bpp8)?;
    display.refresh_area(&area, DisplayMode::Du)?;

    // Do host-side work while the waveform runs; poll before the next update.
    while !display.is_display_ready()? {
        std::thread::sleep(std::time::Duration::from_millis(3));
    }

    display.sleep()?;
    Ok(())
}
```

### Framebuffer

`Framebuffer` is an 8bpp in-memory canvas with line/rect/circle primitives and
scrolling; `draw_framebuffer` and `draw_framebuffer_full` load it and optionally
refresh.

```rust
let mut fb = display.create_framebuffer()?;
fb.clear(0xFF);
fb.draw_rect(100, 100, 200, 150, 0x00, false);
fb.draw_circle(400, 300, 50, 0x80, true);
display.draw_framebuffer_full(&fb, DisplayMode::Gc16)?;
```

### SPI clock

Commands always run at 1 MHz; pixel data runs at the clock given to
`IT8951Builder::spi_data_hz` (default 24 MHz, the datasheet maximum).

Higher clocks work on this hardware — `verify_spi_integrity` loads a pseudo-random
pattern and reads it back through the memory burst-read command, and on a Pi 4 with the
9.7" HAT 32, 40 and 48 MHz all read back byte-exact over 1.4 MB each. The Pi rounds the
divider, so those are really 31.3, 35.7 and 41.7 MHz. Since this is outside the
datasheet, verify at start-up and fall back:

```rust
display.set_spi_speeds(1_000_000, 32_000_000);
if !display.verify_spi_integrity(50)? {
    display.set_spi_speeds(1_000_000, 24_000_000);
}
```

| Data clock | Measured throughput |
|---|---|
| 24 MHz | 2.73 MB/s |
| 32 MHz | 3.70 MB/s |
| 40 MHz | 4.21 MB/s |
| 48 MHz | 4.85 MB/s |

### Reading the image buffer back

`read_memory(addr, words)` returns raw words from the controller's memory. The image
buffer starts at `img_buf_addr()` (see [buffer layout](#pixel-formats-and-buffer-layout)).
This is how the pixel packing and SPI clocks above were verified.

## How the IT8951 behaves

Everything below was measured, not taken from the datasheet.

**Waveform time is a fixed cost per update.** It depends on the mode and barely on the
area: DU takes 195 ms for 200×100 and 224 ms for the full 1200×825 panel. Sending one
merged region is almost always cheaper than several small ones.

**The controller blocks the host for the whole waveform.** While an update runs, HRDY
stays low. A second `refresh_area` waits until the first waveform finishes; so does a
`load_image`, even into an unrelated area (a 1 ms load takes 179 ms if issued mid-DU).
SPI transfer and panel refresh are therefore strictly serial; the only work that can
overlap a waveform is host-side.

**Per-update floor** = SPI time for the bytes sent + ~12 ms for the `DisplayArea`
command + waveform time. `LoadImgArea` setup and `LoadImgEnd` together cost about 1 ms;
a register read (`is_display_ready`) about 4 ms.

**`LUTAFSR` reads idle for a moment right after a refresh is accepted.** Wait ~20 ms
after `refresh_area` before trusting `is_display_ready`.

**The first load after `init()` is slow** (one-off, ~0.5 s).

## Display modes

Values are for M841 firmware (9.7" and larger panels); 6" M641 firmware numbers A2 = 4.

| Mode | Value | Levels | Waveform (200×100 → full) | Flash | Use |
|---|---|---|---|---|---|
| `A2` | 6 | 2 | 159 → 187 ms | no | pointer, animation; lighter blacks, most ghosting |
| `Du` | 1 | 2 | 195 → 224 ms | no | typing, scrolling |
| `Du4` | 7 | 4 | 330 → 358 ms | no | anti-aliased text without dithering |
| `Gl16` | 3 | 16 | 502 → 530 ms | slight | cleaning DU ghosting, general greyscale |
| `Gc16` | 2 | 16 | 502 → 530 ms | full white | images, periodic full refresh |
| `Glr16` / `Gld16` | 4 / 5 | 16 | — | | REGAL variants; behave like GL16 without preprocessing |
| `Init` | 0 | — | — | white | clear to white |

Binary modes threshold the stored grey value; grey input to DU/A2 does not produce
grey output, it produces inconsistently dropped strokes. Quantise (threshold or dither)
on the host before a DU/A2 update.

## Pixel formats and buffer layout

`load_image` accepts `Bpp8`, `Bpp4`, `Bpp2` (and `Bpp3`). Sub-byte formats put the
**first pixel in the low bits** of each byte, matching the 8bpp layout where the first
pixel is the low byte of each little-endian word:

| Format | Byte layout | Word alignment (x and width) |
|---|---|---|
| `Bpp8` | one pixel per byte | 2 px |
| `Bpp4` | `p0 \| p1 << 4` (top nibble of each 8-bit grey) | 4 px |
| `Bpp2` | `p0 \| p1 << 2 \| p2 << 4 \| p3 << 6` (top 2 bits) | 8 px |

The controller's buffer is 8bpp, byte-addressed, with a stride of one panel width;
row *y* of the panel starts at `img_buf_addr() + y * width`. A 4bpp load lands
byte-identical to the equivalent 8bpp load (`0x10 0x20 …` reads back as `0x10 0x20 …`).
A 2bpp load is stored shifted, `0x00/0x40/0x80/0xC0`, so 2bpp "white" is grey level 12,
not 15 — fine for DU4, slightly off-white for greyscale modes.

Regions whose x or width violate the alignment column produce diagonal or smeared
output; widen the region rather than the data.

## Getting the most out of it

These follow directly from the measurements:

1. **Fix the clock first.** Without `core_freq_min=500` every transfer is ~2.4× slower
   than it should be.
2. **Choose the mode by intent, not by area.** DU for anything interactive; GL16 to
   clean up once the screen is still; GC16 only for images or a periodic full refresh.
   A GC16 costs 2.5 DU waveforms *and* flashes.
3. **One update per change set.** Merge dirty regions into one word-aligned bounding
   box. Two regions cost two waveforms; at 4bpp even a whole 1040×785 viewport is only
   ~110 ms of SPI at 32 MHz, less than one waveform.
4. **Send fewer bytes.** 4bpp is verified byte-exact and halves SPI time with no visible
   change. 2bpp halves it again for binary/4-level modes with the level-12 caveat.
5. **Never poll HRDY in a tight loop.** The driver's `wait_ready` spins briefly, then
   sleeps; a caller that polls `is_display_ready` should sleep a few ms between reads.
6. **Capture after the panel is idle**, not before, so the freshest frame is what gets
   sent; the waveform is the pacing element, nothing else needs a timer.

A worked example of all of this — headless Sway captured to the panel with
damage-driven updates — is [paper-tty-rs](https://github.com/kylephillipsau/paper-tty-rs).
There, a keystroke costs ~13 ms of host time plus one waveform, and a full scroll
~41 ms plus one waveform, with no flash.

## Building and testing

```bash
cargo build --release          # default features: std (spidev + gpio-cdev)
cargo test                     # 74 unit tests against the mock HAL, no hardware needed
cargo doc --open
```

On a 64-bit Raspberry Pi OS the simplest route is to build on the Pi itself. To
cross-compile from another machine:

```bash
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

(32-bit Pi OS uses `armv7-unknown-linux-gnueabihf`.)

### Features

| Feature | Default | Provides |
|---|---|---|
| `std` | yes | `LinuxSpi`, `LinuxInputPin`, `LinuxOutputPin` via `spidev` and `gpio-cdev` |
| `rpi` | no | `rppal` dependency (reserved) |
| `image-support` | no | `image` crate |
| `graphics` | no | `embedded-graphics` |
| `async` | no | `tokio` |
| `config` | no | `serde` + `toml` |
| `full` | no | all of the above |

### Mock HAL

`hal::mock::{MockSpi, MockInputPin, MockOutputPin}` record every SPI transfer and let
tests script responses; `IT8951Builder::build_mock()` wires them up. They are compiled
only under `cfg(test)`.

## Project structure

```
src/
├── lib.rs           public API and re-exports
├── error.rs         Error / Result
├── types.rs         Area, DeviceInfo, DisplayMode, PixelFormat, Rotation, …
├── hal/             SpiTransfer / InputPin / OutputPin traits, Linux impls, mocks
├── protocol/        command + register constants, Transport (preambles, HRDY, chunking)
├── device/          IT8951 struct, builder, init/reset/VCOM/power, memory read-back
├── display/         clear, fill, load_image, refresh, refresh_area
└── graphics/        Framebuffer primitives and draw_framebuffer
```

## Related projects and references

- [paper-tty-rs](https://github.com/kylephillipsau/paper-tty-rs) — terminal and Sway
  compositor on an IT8951 panel, built on this crate
- [GregDMeyer/IT8951](https://github.com/GregDMeyer/IT8951) — Python driver
- [Modos-Labs/Glider](https://github.com/Modos-Labs/Glider) — open FPGA e-paper
  controller; useful background on waveforms and why controller latency is what it is
- [IT8951 datasheet](http://www.waveshare.net/w/upload/c/c4/IT8951_D_V0.2.4.3_20170728.pdf)
- [Waveshare 9.7" e-Paper HAT wiki](https://www.waveshare.com/wiki/9.7inch_e-Paper_HAT)

## License

MIT OR Apache-2.0
