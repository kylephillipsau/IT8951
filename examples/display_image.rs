//! Display an image file on the e-paper display.
//!
//! Usage: cargo run --example display_image -- <image_path>

use it8951::types::{Area, DisplayMode, PixelFormat};
use it8951::IT8951Builder;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <image_path>", args[0]);
        eprintln!("Supported formats: PNG, JPEG, BMP, etc.");
        std::process::exit(1);
    }

    let image_path = &args[1];
    println!("Loading image: {}", image_path);

    // Load and convert image to grayscale
    let img = image::open(image_path)?;
    let gray = img.to_luma8();

    println!("Image size: {}x{}", gray.width(), gray.height());

    // Initialize display
    println!("Initializing IT8951...");
    let mut display = IT8951Builder::new().vcom(1480).build()?;
    display.init()?;

    let panel_width = display.width();
    let panel_height = display.height();
    println!("Panel size: {}x{}", panel_width, panel_height);

    // Scale/crop image to fit display
    let (img_width, img_height) = (gray.width() as u16, gray.height() as u16);

    // Calculate display position (center the image)
    let x = if img_width < panel_width {
        (panel_width - img_width) / 2
    } else {
        0
    };
    let y = if img_height < panel_height {
        (panel_height - img_height) / 2
    } else {
        0
    };

    // Crop to panel size if needed
    let width = img_width.min(panel_width);
    let height = img_height.min(panel_height);

    // Extract pixel data
    let mut data = Vec::with_capacity((width as usize) * (height as usize));
    for py in 0..height {
        for px in 0..width {
            let pixel = gray.get_pixel(px as u32, py as u32);
            data.push(pixel.0[0]);
        }
    }

    // Clear display first
    println!("Clearing display...");
    display.clear(0xFF)?;
    display.refresh(DisplayMode::Init)?;
    display.wait_display_ready()?;

    // Load image
    println!("Loading image at ({}, {}) size {}x{}", x, y, width, height);
    let area = Area::new(x, y, width, height);
    display.load_image(&data, &area, PixelFormat::Bpp8)?;

    // Refresh with high quality mode
    println!("Refreshing display...");
    display.refresh_area(&area, DisplayMode::Gc16)?;
    display.wait_display_ready()?;

    println!("Done!");
    Ok(())
}
