//! Render a webpage screenshot on the e-paper display.
//!
//! This example fetches a URL, renders it as an image, and displays it.
//! Requires wkhtmltoimage to be installed: sudo apt install wkhtmltopdf
//!
//! Usage: cargo run --example display_url -- <url>

use it8951::types::{Area, DisplayMode, PixelFormat};
use it8951::IT8951Builder;
use std::env;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <url>", args[0]);
        eprintln!("Example: {} https://example.com", args[0]);
        std::process::exit(1);
    }

    let url = &args[1];
    println!("Fetching: {}", url);

    // Initialize display first to get dimensions
    println!("Initializing IT8951...");
    let mut display = IT8951Builder::new().vcom(1480).build()?;
    display.init()?;

    let panel_width = display.width();
    let panel_height = display.height();
    println!("Panel size: {}x{}", panel_width, panel_height);

    // Render webpage to image using wkhtmltoimage
    let temp_path = "/tmp/webpage.png";
    println!("Rendering webpage...");

    let output = Command::new("wkhtmltoimage")
        .args([
            "--width", &panel_width.to_string(),
            "--height", &panel_height.to_string(),
            "--quality", "100",
            url,
            temp_path,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("wkhtmltoimage failed: {}", stderr);
        eprintln!("Make sure wkhtmltoimage is installed: sudo apt install wkhtmltopdf");
        std::process::exit(1);
    }

    // Load the rendered image
    let img = image::open(temp_path)?;
    let gray = img.to_luma8();

    println!("Rendered image size: {}x{}", gray.width(), gray.height());

    let width = (gray.width() as u16).min(panel_width);
    let height = (gray.height() as u16).min(panel_height);
    println!("Using size: {}x{}", width, height);

    // Extract pixel data
    let mut data = Vec::with_capacity((width as usize) * (height as usize));
    for py in 0..height {
        for px in 0..width {
            let pixel = gray.get_pixel(px as u32, py as u32);
            data.push(pixel.0[0]);
        }
    }

    // Clear and display
    println!("Clearing display...");
    display.clear(0xFF)?;
    display.refresh(DisplayMode::Init)?;
    display.wait_display_ready()?;

    println!("Loading image...");
    let area = Area::new(0, 0, width, height);
    display.load_image(&data, &area, PixelFormat::Bpp8)?;

    println!("Refreshing display...");
    display.refresh(DisplayMode::Gc16)?;
    display.wait_display_ready()?;

    // Clean up temp file
    std::fs::remove_file(temp_path).ok();

    println!("Done!");
    Ok(())
}
