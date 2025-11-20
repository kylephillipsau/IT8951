//! Graphics demo showing drawing primitives.
//!
//! Usage: cargo run --example graphics_demo

use it8951::graphics::Framebuffer;
use it8951::types::DisplayMode;
use it8951::IT8951Builder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing IT8951...");
    let mut display = IT8951Builder::new().vcom(1480).build()?;
    display.init()?;

    let width = display.width();
    let height = display.height();
    println!("Panel size: {}x{}", width, height);

    // Create framebuffer
    let mut fb = Framebuffer::new(width, height);
    fb.clear(0xFF); // White background

    // Draw border
    fb.draw_rect(0, 0, width - 1, height - 1, 0x00, false);
    fb.draw_rect(2, 2, width - 3, height - 3, 0x00, false);

    // Draw title
    let title_y = 20;
    fb.draw_rect(10, title_y, width - 10, title_y + 40, 0x00, true);
    // Title text would go here if we had font rendering

    // Draw various shapes
    let center_x = width / 2;
    let center_y = height / 2;

    // Circles with different gray levels
    fb.draw_circle(center_x - 200, center_y - 100, 80, 0x00, false);
    fb.draw_circle(center_x - 200, center_y - 100, 60, 0x40, false);
    fb.draw_circle(center_x - 200, center_y - 100, 40, 0x80, true);

    // Rectangles
    fb.draw_rect(center_x + 50, center_y - 150, center_x + 250, center_y - 50, 0x00, false);
    fb.draw_rect(center_x + 70, center_y - 130, center_x + 230, center_y - 70, 0x60, true);

    // Lines in a star pattern
    let star_x = center_x - 200;
    let star_y = center_y + 150;
    for angle in (0..360).step_by(30) {
        let rad = (angle as f32).to_radians();
        let end_x = (star_x as f32 + 80.0 * rad.cos()) as u16;
        let end_y = (star_y as f32 + 80.0 * rad.sin()) as u16;
        fb.draw_line(star_x, star_y, end_x, end_y, 0x00);
    }

    // Gradient boxes
    let box_width = 50;
    let box_y = center_y + 80;
    for i in 0..8 {
        let gray = (i * 32) as u8;
        let x = center_x + 50 + (i as u16 * (box_width + 5));
        fb.draw_rect(x, box_y, x + box_width, box_y + 80, gray, true);
        fb.draw_rect(x, box_y, x + box_width, box_y + 80, 0x00, false);
    }

    // Grid pattern
    let grid_x = center_x + 50;
    let grid_y = center_y - 30;
    for i in 0..5u16 {
        fb.draw_line(
            grid_x,
            grid_y + i * 20,
            grid_x + 150,
            grid_y + i * 20,
            0x00,
        );
        fb.draw_line(
            grid_x + i * 30,
            grid_y,
            grid_x + i * 30,
            grid_y + 80,
            0x00,
        );
    }

    // Display the framebuffer
    println!("Clearing display...");
    display.clear(0xFF)?;
    display.refresh(DisplayMode::Init)?;
    display.wait_display_ready()?;

    println!("Drawing framebuffer...");
    display.draw_framebuffer_full(&fb, DisplayMode::Gc16)?;

    println!("Refreshing display...");
    display.refresh(DisplayMode::Gc16)?;
    display.wait_display_ready()?;

    println!("Done!");
    Ok(())
}
