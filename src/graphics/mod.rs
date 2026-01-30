//! Graphics primitives and framebuffer operations
//!
//! This module provides in-memory framebuffer management and drawing primitives
//! for the IT8951 e-paper display. All drawing operations work on a framebuffer
//! which can then be transferred to the display hardware.
//!
//! # Examples
//!
//! ```
//! use it8951::graphics::Framebuffer;
//! use it8951::Area;
//!
//! // Create a framebuffer for an 800x600 display
//! let mut fb = Framebuffer::new(800, 600);
//!
//! // Draw some primitives
//! fb.clear(0xFF); // White background
//! fb.draw_rect(100, 100, 200, 150, 0x00, false); // Black outline
//! fb.draw_circle(400, 300, 50, 0x80, true); // Gray filled circle
//! fb.draw_line(0, 0, 799, 599, 0x00); // Diagonal line
//!
//! // Get the pixel data to send to display
//! let data = fb.data();
//! ```

mod display;

use crate::error::{Error, Result};
use crate::types::Area;

/// In-memory framebuffer for 8-bit grayscale graphics
///
/// The framebuffer stores pixel data in 8bpp format (0x00=black, 0xFF=white)
/// and provides drawing primitives. All coordinates are bounds-checked.
///
/// # Examples
///
/// ```
/// use it8951::graphics::Framebuffer;
///
/// let mut fb = Framebuffer::new(800, 600);
/// fb.clear(0xFF);
/// fb.set_pixel(400, 300, 0x00).unwrap();
/// assert_eq!(fb.get_pixel(400, 300).unwrap(), 0x00);
/// ```
#[derive(Debug, Clone)]
pub struct Framebuffer {
    width: u16,
    height: u16,
    data: Vec<u8>,
}

impl Framebuffer {
    /// Create a new framebuffer with the specified dimensions
    ///
    /// All pixels are initialized to 0x00 (black).
    ///
    /// # Examples
    ///
    /// ```
    /// use it8951::graphics::Framebuffer;
    ///
    /// let fb = Framebuffer::new(800, 600);
    /// assert_eq!(fb.width(), 800);
    /// assert_eq!(fb.height(), 600);
    /// ```
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            data: vec![0; size],
        }
    }

    /// Get the framebuffer width
    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Get the framebuffer height
    #[inline]
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Get a reference to the pixel data
    ///
    /// The data is in row-major order, with each byte representing one pixel
    /// in 8-bit grayscale (0x00=black, 0xFF=white).
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get a mutable reference to the pixel data
    #[inline]
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Clear the entire framebuffer to a specific grayscale value
    ///
    /// # Examples
    ///
    /// ```
    /// use it8951::graphics::Framebuffer;
    ///
    /// let mut fb = Framebuffer::new(100, 100);
    /// fb.clear(0xFF); // Set all pixels to white
    /// assert_eq!(fb.get_pixel(50, 50).unwrap(), 0xFF);
    /// ```
    pub fn clear(&mut self, value: u8) {
        self.data.fill(value);
    }

    /// Set a single pixel to a grayscale value
    ///
    /// Returns an error if the coordinates are out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use it8951::graphics::Framebuffer;
    ///
    /// let mut fb = Framebuffer::new(100, 100);
    /// fb.set_pixel(50, 50, 0x80).unwrap();
    /// assert_eq!(fb.get_pixel(50, 50).unwrap(), 0x80);
    /// ```
    pub fn set_pixel(&mut self, x: u16, y: u16, value: u8) -> Result<()> {
        if x >= self.width || y >= self.height {
            return Err(Error::InvalidArea(Area::new(x, y, 1, 1)));
        }
        let index = (y as usize) * (self.width as usize) + (x as usize);
        self.data[index] = value;
        Ok(())
    }

    /// Get the grayscale value of a single pixel
    ///
    /// Returns an error if the coordinates are out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use it8951::graphics::Framebuffer;
    ///
    /// let mut fb = Framebuffer::new(100, 100);
    /// fb.set_pixel(25, 75, 0xAA).unwrap();
    /// assert_eq!(fb.get_pixel(25, 75).unwrap(), 0xAA);
    /// ```
    pub fn get_pixel(&self, x: u16, y: u16) -> Result<u8> {
        if x >= self.width || y >= self.height {
            return Err(Error::InvalidArea(Area::new(x, y, 1, 1)));
        }
        let index = (y as usize) * (self.width as usize) + (x as usize);
        Ok(self.data[index])
    }

    /// Draw a line using Bresenham's line algorithm
    ///
    /// Draws a line from (x0, y0) to (x1, y1) with the specified grayscale value.
    /// Coordinates are clipped to the framebuffer bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use it8951::graphics::Framebuffer;
    ///
    /// let mut fb = Framebuffer::new(100, 100);
    /// fb.draw_line(10, 10, 90, 90, 0x00); // Black diagonal line
    /// ```
    pub fn draw_line(&mut self, x0: u16, y0: u16, x1: u16, y1: u16, value: u8) {
        let mut x0 = x0 as i32;
        let mut y0 = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            // Only draw if within bounds
            if x0 >= 0 && x0 < self.width as i32 && y0 >= 0 && y0 < self.height as i32 {
                let _ = self.set_pixel(x0 as u16, y0 as u16, value);
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Draw a rectangle
    ///
    /// Draws a rectangle with top-left corner at (x, y) and the specified
    /// width and height. If `filled` is true, draws a solid rectangle;
    /// otherwise draws only the outline.
    ///
    /// # Examples
    ///
    /// ```
    /// use it8951::graphics::Framebuffer;
    ///
    /// let mut fb = Framebuffer::new(200, 200);
    /// fb.draw_rect(50, 50, 100, 80, 0x00, false); // Black outline
    /// fb.draw_rect(10, 10, 30, 30, 0x80, true);   // Gray filled square
    /// ```
    pub fn draw_rect(&mut self, x: u16, y: u16, width: u16, height: u16, value: u8, filled: bool) {
        if filled {
            // Draw filled rectangle
            for py in y..y.saturating_add(height).min(self.height) {
                for px in x..x.saturating_add(width).min(self.width) {
                    let _ = self.set_pixel(px, py, value);
                }
            }
        } else {
            // Draw rectangle outline
            // Top and bottom edges
            for px in x..x.saturating_add(width).min(self.width) {
                let _ = self.set_pixel(px, y, value);
                if height > 0 {
                    let bottom = y.saturating_add(height - 1).min(self.height - 1);
                    let _ = self.set_pixel(px, bottom, value);
                }
            }
            // Left and right edges
            for py in y..y.saturating_add(height).min(self.height) {
                let _ = self.set_pixel(x, py, value);
                if width > 0 {
                    let right = x.saturating_add(width - 1).min(self.width - 1);
                    let _ = self.set_pixel(right, py, value);
                }
            }
        }
    }

    /// Draw a circle using the midpoint circle algorithm
    ///
    /// Draws a circle centered at (x, y) with the specified radius.
    /// If `filled` is true, draws a solid circle; otherwise draws only the outline.
    ///
    /// # Examples
    ///
    /// ```
    /// use it8951::graphics::Framebuffer;
    ///
    /// let mut fb = Framebuffer::new(200, 200);
    /// fb.draw_circle(100, 100, 50, 0x00, false); // Black circle outline
    /// fb.draw_circle(100, 100, 30, 0x80, true);  // Gray filled circle
    /// ```
    pub fn draw_circle(&mut self, cx: u16, cy: u16, radius: u16, value: u8, filled: bool) {
        if radius == 0 {
            let _ = self.set_pixel(cx, cy, value);
            return;
        }

        let cx = cx as i32;
        let cy = cy as i32;
        let radius = radius as i32;

        if filled {
            // Draw filled circle using horizontal lines
            let mut x = 0;
            let mut y = radius;
            let mut d = 3 - 2 * radius;

            while x <= y {
                // Draw horizontal lines for each octant
                self.draw_horizontal_line(cx - x, cx + x, cy + y, value);
                self.draw_horizontal_line(cx - x, cx + x, cy - y, value);
                self.draw_horizontal_line(cx - y, cx + y, cy + x, value);
                self.draw_horizontal_line(cx - y, cx + y, cy - x, value);

                if d < 0 {
                    d += 4 * x + 6;
                } else {
                    d += 4 * (x - y) + 10;
                    y -= 1;
                }
                x += 1;
            }
        } else {
            // Draw circle outline using midpoint algorithm
            let mut x = 0;
            let mut y = radius;
            let mut d = 3 - 2 * radius;

            while x <= y {
                // Draw 8 octants
                self.plot_circle_points(cx, cy, x, y, value);

                if d < 0 {
                    d += 4 * x + 6;
                } else {
                    d += 4 * (x - y) + 10;
                    y -= 1;
                }
                x += 1;
            }
        }
    }

    // Helper: Draw horizontal line for filled circle
    fn draw_horizontal_line(&mut self, x0: i32, x1: i32, y: i32, value: u8) {
        if y < 0 || y >= self.height as i32 {
            return;
        }

        let x_start = x0.max(0).min(self.width as i32 - 1);
        let x_end = x1.max(0).min(self.width as i32 - 1);

        for x in x_start..=x_end {
            let _ = self.set_pixel(x as u16, y as u16, value);
        }
    }

    // Helper: Plot 8 symmetric circle points
    fn plot_circle_points(&mut self, cx: i32, cy: i32, x: i32, y: i32, value: u8) {
        let points = [
            (cx + x, cy + y),
            (cx - x, cy + y),
            (cx + x, cy - y),
            (cx - x, cy - y),
            (cx + y, cy + x),
            (cx - y, cy + x),
            (cx + y, cy - x),
            (cx - y, cy - x),
        ];

        for (px, py) in points {
            if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                let _ = self.set_pixel(px as u16, py as u16, value);
            }
        }
    }

    /// Fill a rectangular area with a value
    ///
    /// This is a convenience method that fills the specified area.
    ///
    /// # Examples
    ///
    /// ```
    /// use it8951::graphics::Framebuffer;
    /// use it8951::Area;
    ///
    /// let mut fb = Framebuffer::new(200, 200);
    /// let area = Area::new(50, 50, 100, 100);
    /// fb.fill_area(&area, 0x80).unwrap();
    /// ```
    pub fn fill_area(&mut self, area: &Area, value: u8) -> Result<()> {
        // Validate area
        if area.x + area.width > self.width || area.y + area.height > self.height {
            return Err(Error::InvalidArea(*area));
        }

        let stride = self.width as usize;
        let w = area.width as usize;
        for y in area.y..area.y + area.height {
            let start = (y as usize) * stride + area.x as usize;
            self.data[start..start + w].fill(value);
        }
        Ok(())
    }

    /// Fill a rectangle with a value using direct slice operations (no bounds checking per pixel)
    ///
    /// Coordinates are clipped to framebuffer bounds.
    pub fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, value: u8) {
        let x_end = (x + w).min(self.width) as usize;
        let y_end = (y + h).min(self.height) as usize;
        let x = x.min(self.width) as usize;
        let y = y.min(self.height) as usize;
        let stride = self.width as usize;

        for row in y..y_end {
            let start = row * stride + x;
            let end = row * stride + x_end;
            self.data[start..end].fill(value);
        }
    }

    /// Scroll a rectangular region of the framebuffer up by the given number of pixel rows.
    /// Only pixels within (x, y, w, h) are shifted; everything outside is untouched.
    pub fn scroll_region_up(&mut self, x: u16, y: u16, w: u16, h: u16, pixel_rows: usize, fill: u8) {
        let stride = self.width as usize;
        let x = x as usize;
        let y = y as usize;
        let w = w as usize;
        let h = h as usize;

        if pixel_rows >= h {
            for row in y..y + h {
                let start = row * stride + x;
                self.data[start..start + w].fill(fill);
            }
            return;
        }

        for row in y..y + h - pixel_rows {
            let dst = row * stride + x;
            let src = (row + pixel_rows) * stride + x;
            self.data.copy_within(src..src + w, dst);
        }
        for row in (y + h - pixel_rows)..y + h {
            let start = row * stride + x;
            self.data[start..start + w].fill(fill);
        }
    }

    /// Scroll the framebuffer up by the given number of pixel rows.
    ///
    /// Shifts all pixel data up and fills the newly exposed bottom rows with `fill`.
    pub fn scroll_up(&mut self, pixel_rows: usize, fill: u8) {
        let stride = self.width as usize;
        let total = self.data.len();
        let offset = pixel_rows * stride;

        if offset >= total {
            self.data.fill(fill);
            return;
        }

        self.data.copy_within(offset..total, 0);
        self.data[total - offset..].fill(fill);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framebuffer_new() {
        let fb = Framebuffer::new(800, 600);
        assert_eq!(fb.width(), 800);
        assert_eq!(fb.height(), 600);
        assert_eq!(fb.data().len(), 800 * 600);
    }

    #[test]
    fn test_clear() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(0xFF);
        assert_eq!(fb.get_pixel(0, 0).unwrap(), 0xFF);
        assert_eq!(fb.get_pixel(50, 50).unwrap(), 0xFF);
        assert_eq!(fb.get_pixel(99, 99).unwrap(), 0xFF);
    }

    #[test]
    fn test_set_get_pixel() {
        let mut fb = Framebuffer::new(100, 100);
        fb.set_pixel(50, 50, 0x80).unwrap();
        assert_eq!(fb.get_pixel(50, 50).unwrap(), 0x80);
    }

    #[test]
    fn test_set_pixel_out_of_bounds() {
        let mut fb = Framebuffer::new(100, 100);
        assert!(fb.set_pixel(100, 50, 0x80).is_err());
        assert!(fb.set_pixel(50, 100, 0x80).is_err());
    }

    #[test]
    fn test_get_pixel_out_of_bounds() {
        let fb = Framebuffer::new(100, 100);
        assert!(fb.get_pixel(100, 50).is_err());
        assert!(fb.get_pixel(50, 100).is_err());
    }

    #[test]
    fn test_draw_line_horizontal() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(0xFF);
        fb.draw_line(10, 50, 90, 50, 0x00);

        // Check endpoints and middle
        assert_eq!(fb.get_pixel(10, 50).unwrap(), 0x00);
        assert_eq!(fb.get_pixel(50, 50).unwrap(), 0x00);
        assert_eq!(fb.get_pixel(90, 50).unwrap(), 0x00);

        // Check pixel not on line
        assert_eq!(fb.get_pixel(50, 51).unwrap(), 0xFF);
    }

    #[test]
    fn test_draw_line_vertical() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(0xFF);
        fb.draw_line(50, 10, 50, 90, 0x00);

        // Check endpoints and middle
        assert_eq!(fb.get_pixel(50, 10).unwrap(), 0x00);
        assert_eq!(fb.get_pixel(50, 50).unwrap(), 0x00);
        assert_eq!(fb.get_pixel(50, 90).unwrap(), 0x00);

        // Check pixel not on line
        assert_eq!(fb.get_pixel(51, 50).unwrap(), 0xFF);
    }

    #[test]
    fn test_draw_line_diagonal() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(0xFF);
        fb.draw_line(0, 0, 99, 99, 0x00);

        // Check some points on the diagonal
        assert_eq!(fb.get_pixel(0, 0).unwrap(), 0x00);
        assert_eq!(fb.get_pixel(50, 50).unwrap(), 0x00);
        assert_eq!(fb.get_pixel(99, 99).unwrap(), 0x00);
    }

    #[test]
    fn test_draw_rect_outline() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(0xFF);
        fb.draw_rect(20, 20, 60, 40, 0x00, false);

        // Check corners
        assert_eq!(fb.get_pixel(20, 20).unwrap(), 0x00);
        assert_eq!(fb.get_pixel(79, 20).unwrap(), 0x00);
        assert_eq!(fb.get_pixel(20, 59).unwrap(), 0x00);
        assert_eq!(fb.get_pixel(79, 59).unwrap(), 0x00);

        // Check interior is not filled
        assert_eq!(fb.get_pixel(50, 50).unwrap(), 0xFF);
    }

    #[test]
    fn test_draw_rect_filled() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(0xFF);
        fb.draw_rect(20, 20, 60, 40, 0x00, true);

        // Check corners and interior
        assert_eq!(fb.get_pixel(20, 20).unwrap(), 0x00);
        assert_eq!(fb.get_pixel(79, 59).unwrap(), 0x00);
        assert_eq!(fb.get_pixel(50, 50).unwrap(), 0x00);

        // Check outside is not filled
        assert_eq!(fb.get_pixel(10, 10).unwrap(), 0xFF);
    }

    #[test]
    fn test_draw_circle_outline() {
        let mut fb = Framebuffer::new(200, 200);
        fb.clear(0xFF);
        fb.draw_circle(100, 100, 50, 0x00, false);

        // Check center is not filled
        assert_eq!(fb.get_pixel(100, 100).unwrap(), 0xFF);

        // Check some points on the circle (approximately)
        // Top point
        assert_eq!(fb.get_pixel(100, 50).unwrap(), 0x00);
        // Right point
        assert_eq!(fb.get_pixel(150, 100).unwrap(), 0x00);
    }

    #[test]
    fn test_draw_circle_filled() {
        let mut fb = Framebuffer::new(200, 200);
        fb.clear(0xFF);
        fb.draw_circle(100, 100, 50, 0x00, true);

        // Check center is filled
        assert_eq!(fb.get_pixel(100, 100).unwrap(), 0x00);

        // Check interior point is filled
        assert_eq!(fb.get_pixel(100, 75).unwrap(), 0x00);

        // Check outside is not filled
        assert_eq!(fb.get_pixel(100, 40).unwrap(), 0xFF);
    }

    #[test]
    fn test_fill_area() {
        let mut fb = Framebuffer::new(200, 200);
        fb.clear(0xFF);

        let area = Area::new(50, 50, 100, 80);
        fb.fill_area(&area, 0x80).unwrap();

        // Check filled area
        assert_eq!(fb.get_pixel(50, 50).unwrap(), 0x80);
        assert_eq!(fb.get_pixel(100, 100).unwrap(), 0x80);
        assert_eq!(fb.get_pixel(149, 129).unwrap(), 0x80);

        // Check outside is not filled
        assert_eq!(fb.get_pixel(40, 50).unwrap(), 0xFF);
        assert_eq!(fb.get_pixel(50, 40).unwrap(), 0xFF);
    }

    #[test]
    fn test_fill_area_invalid() {
        let mut fb = Framebuffer::new(100, 100);
        let area = Area::new(50, 50, 100, 100); // Extends beyond bounds
        assert!(fb.fill_area(&area, 0x80).is_err());
    }
}
