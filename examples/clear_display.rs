//! Example: Clear the e-paper display to white.
//!
//! This example demonstrates basic display initialization and clearing.
//!
//! Run with: cargo run --example clear_display

use it8951::{DisplayMode, IT8951Builder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IT8951 E-Paper Display Test");
    println!("===========================");

    // Build the display with default settings
    // VCOM should match your display (check label on flex cable)
    let mut display = IT8951Builder::new()
        .vcom(1500)  // -1.50V, adjust for your display
        .build()?;

    println!("Initializing display...");
    display.init()?;

    // Print device info
    if let Some(info) = display.device_info() {
        println!("Panel size: {}x{}", info.panel_width, info.panel_height);
        println!("Firmware: {}", info.fw_version);
        println!("LUT version: {}", info.lut_version);
        println!("Image buffer: 0x{:08X}", info.img_buf_addr);
    }

    // Read and display current VCOM
    let current_vcom = display.read_vcom()?;
    println!("Current VCOM: {} (-{:.2}V)", current_vcom, current_vcom as f32 / 1000.0);

    // Wait for display to be ready before loading
    println!("Waiting for display ready...");
    display.wait_display_ready()?;

    // Clear to white (0xF0 is white in 8bpp)
    println!("Clearing display to white...");
    display.clear(0xF0)?;

    // Refresh with Init mode (clears ghosting)
    println!("Refreshing display (Init mode)...");
    display.refresh(DisplayMode::Init)?;

    // Wait for display to complete
    display.wait_display_ready()?;

    println!("Done!");

    Ok(())
}
