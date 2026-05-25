//! # OseCanvas - ASCII Graphics Canvas
//!
//! Provides rendering capabilities for the Ose (Graphics) domain.
//! Implements basic drawing primitives for ASCII art.

/// Ose Canvas for ASCII graphics
#[derive(Clone)]
pub struct OseCanvas {
    width: usize,
    height: usize,
    buffer: Vec<Vec<char>>,
    _cursor_x: usize,
    _cursor_y: usize,
}

impl OseCanvas {
    /// Create a new canvas with default 80x24 size
    pub fn new() -> Self {
        Self {
            width: 80,
            height: 24,
            buffer: vec![vec![' '; 80]; 24],
            _cursor_x: 0,
            _cursor_y: 0,
        }
    }

    /// Clear the canvas with a fill character
    pub fn clear(&mut self, fill: char) {
        for row in &mut self.buffer {
            row.fill(fill);
        }
    }

    /// Resize the canvas
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.buffer = vec![vec![' '; width]; height];
    }

    /// Set a single pixel/character
    pub fn set_pixel(&mut self, x: i64, y: i64, ch: char) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.buffer[y as usize][x as usize] = ch;
        }
    }

    /// Write text at position
    pub fn write_text(&mut self, x: i64, y: i64, text: &str) {
        for (i, ch) in text.chars().enumerate() {
            self.set_pixel(x + i as i64, y, ch);
        }
    }

    /// Draw a line using Bresenham's algorithm
    pub fn draw_line(&mut self, x1: i64, y1: i64, x2: i64, y2: i64, ch: char) {
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x1;
        let mut y = y1;

        loop {
            self.set_pixel(x, y, ch);
            if x == x2 && y == y2 {
                break;
            }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Draw a rectangle outline
    pub fn draw_rect(&mut self, x: i64, y: i64, w: i64, h: i64, ch: char) {
        for i in 0..w {
            self.set_pixel(x + i, y, ch);
            self.set_pixel(x + i, y + h - 1, ch);
        }
        for i in 0..h {
            self.set_pixel(x, y + i, ch);
            self.set_pixel(x + w - 1, y + i, ch);
        }
    }

    /// Fill a rectangle
    pub fn fill_rect(&mut self, x: i64, y: i64, w: i64, h: i64, ch: char) {
        for dy in 0..h {
            for dx in 0..w {
                self.set_pixel(x + dx, y + dy, ch);
            }
        }
    }

    /// Draw a circle using midpoint algorithm
    pub fn draw_circle(&mut self, xc: i64, yc: i64, r: i64, ch: char) {
        let mut x = 0;
        let mut y = r;
        let mut d = 1 - r;

        while x <= y {
            self.set_pixel(xc + x, yc + y, ch);
            self.set_pixel(xc - x, yc + y, ch);
            self.set_pixel(xc + x, yc - y, ch);
            self.set_pixel(xc - x, yc - y, ch);
            self.set_pixel(xc + y, yc + x, ch);
            self.set_pixel(xc - y, yc + x, ch);
            self.set_pixel(xc + y, yc - x, ch);
            self.set_pixel(xc - y, yc - x, ch);

            x += 1;
            if d < 0 {
                d += 2 * x + 1;
            } else {
                y -= 1;
                d += 2 * (x - y) + 1;
            }
        }
    }

    /// Render the canvas to a string
    pub fn render(&self) -> String {
        self.buffer
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get canvas dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

impl Default for OseCanvas {
    fn default() -> Self {
        Self::new()
    }
}
