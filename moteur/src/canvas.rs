/// Ultra-fast RGBA8 + Z-buffer software rasterizer with built-in bitmap font
#[derive(Clone)]
pub struct CanvasBuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub depth: Vec<f32>,
}

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn scale(self, factor: f32) -> Self {
        Self {
            r: ((self.r as f32 * factor).clamp(0.0, 255.0)) as u8,
            g: ((self.g as f32 * factor).clamp(0.0, 255.0)) as u8,
            b: ((self.b as f32 * factor).clamp(0.0, 255.0)) as u8,
            a: self.a,
        }
    }

    pub fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }
}

impl CanvasBuffer {
    pub fn new(width: u32, height: u32, bg: Color) -> Self {
        let count = (width * height) as usize;
        let mut pixels = Vec::with_capacity(count * 4);
        for _ in 0..count {
            pixels.extend_from_slice(&[bg.r, bg.g, bg.b, 255]);
        }
        Self {
            width,
            height,
            pixels,
            depth: vec![f32::INFINITY; count],
        }
    }

    #[inline]
    pub fn blend_pixel(&mut self, x: i32, y: i32, c: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 || c.a == 0 {
            return;
        }
        let idx = ((y as u32 * self.width + x as u32) * 4) as usize;
        if c.a == 255 {
            self.pixels[idx] = c.r;
            self.pixels[idx + 1] = c.g;
            self.pixels[idx + 2] = c.b;
            self.pixels[idx + 3] = 255;
        } else {
            let a = c.a as u32;
            let inv = 255 - a;
            self.pixels[idx] = ((c.r as u32 * a + self.pixels[idx] as u32 * inv) / 255) as u8;
            self.pixels[idx + 1] =
                ((c.g as u32 * a + self.pixels[idx + 1] as u32 * inv) / 255) as u8;
            self.pixels[idx + 2] =
                ((c.b as u32 * a + self.pixels[idx + 2] as u32 * inv) / 255) as u8;
            self.pixels[idx + 3] = 255;
        }
    }

    #[inline]
    pub fn put_pixel_depth(&mut self, x: i32, y: i32, z: f32, c: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 || c.a == 0 {
            return;
        }
        let p_idx = (y as u32 * self.width + x as u32) as usize;
        if z <= self.depth[p_idx] {
            if c.a >= 240 {
                self.depth[p_idx] = z;
            }
            self.blend_pixel(x, y, c);
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.width as i32);
        let y1 = (y + h).min(self.height as i32);
        for py in y0..y1 {
            for px in x0..x1 {
                self.blend_pixel(px, py, c);
            }
        }
    }

    pub fn stroke_rect(&mut self, x: i32, y: i32, w: i32, h: i32, thickness: i32, c: Color) {
        self.fill_rect(x, y, w, thickness, c);
        self.fill_rect(x, y + h - thickness, w, thickness, c);
        self.fill_rect(x, y, thickness, h, c);
        self.fill_rect(x + w - thickness, y, thickness, h, c);
    }

    pub fn fill_circle(&mut self, cx: f32, cy: f32, radius: f32, c: Color) {
        let r_sq = radius * radius;
        let x0 = (cx - radius - 1.0).floor() as i32;
        let x1 = (cx + radius + 1.0).ceil() as i32;
        let y0 = (cy - radius - 1.0).floor() as i32;
        let y1 = (cy + radius + 1.0).ceil() as i32;
        for py in y0..=y1 {
            for px in x0..=x1 {
                let dx = px as f32 + 0.5 - cx;
                let dy = py as f32 + 0.5 - cy;
                let d_sq = dx * dx + dy * dy;
                if d_sq <= r_sq {
                    self.blend_pixel(px, py, c);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_arc(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        start_rad: f32,
        end_rad: f32,
        thickness: f32,
        c: Color,
    ) {
        let steps = ((radius * (end_rad - start_rad).abs()).ceil() as usize).clamp(16, 96);
        for i in 0..steps {
            let t0 = start_rad + (end_rad - start_rad) * (i as f32 / steps as f32);
            let t1 = start_rad + (end_rad - start_rad) * ((i + 1) as f32 / steps as f32);
            self.draw_thick_line(
                cx + radius * t0.cos(),
                cy + radius * t0.sin(),
                cx + radius * t1.cos(),
                cy + radius * t1.sin(),
                thickness,
                c,
                false,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_thick_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        thickness: f32,
        c: Color,
        dashed: bool,
    ) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.5 {
            return;
        }
        let half = (thickness * 0.5).max(0.6);
        let min_x = (x1.min(x2) - half - 1.0).floor().max(0.0) as i32;
        let max_x = (x1.max(x2) + half + 1.0)
            .ceil()
            .min(self.width as f32 - 1.0) as i32;
        let min_y = (y1.min(y2) - half - 1.0).floor().max(0.0) as i32;
        let max_y = (y1.max(y2) + half + 1.0)
            .ceil()
            .min(self.height as f32 - 1.0) as i32;

        let ux = dx / len;
        let uy = dy / len;

        for py in min_y..=max_y {
            for px in min_x..=max_x {
                let vx = px as f32 + 0.5 - x1;
                let vy = py as f32 + 0.5 - y1;
                let along = (vx * ux + vy * uy).clamp(0.0, len);
                if dashed && ((along / 7.0) as i32 % 2 == 1) {
                    continue;
                }
                let proj_x = x1 + along * ux;
                let proj_y = y1 + along * uy;
                let dist = ((px as f32 + 0.5 - proj_x).powi(2)
                    + (py as f32 + 0.5 - proj_y).powi(2))
                .sqrt();
                if dist <= half {
                    let coverage = (half + 0.5 - dist).clamp(0.0, 1.0);
                    let alpha = (c.a as f32 * coverage) as u8;
                    self.blend_pixel(px, py, c.with_alpha(alpha));
                }
            }
        }
    }

    pub fn fill_triangle_2d(&mut self, p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), c: Color) {
        self.fill_triangle_3d((p0.0, p0.1, 0.0), (p1.0, p1.1, 0.0), (p2.0, p2.1, 0.0), c);
    }

    pub fn fill_quad_2d(
        &mut self,
        p0: (f32, f32),
        p1: (f32, f32),
        p2: (f32, f32),
        p3: (f32, f32),
        c: Color,
    ) {
        self.fill_triangle_2d(p0, p1, p2, c);
        self.fill_triangle_2d(p0, p2, p3, c);
    }

    pub fn fill_triangle_3d(
        &mut self,
        p0: (f32, f32, f32),
        p1: (f32, f32, f32),
        p2: (f32, f32, f32),
        c: Color,
    ) {
        let min_x = p0.0.min(p1.0).min(p2.0).floor().max(0.0) as i32;
        let max_x = p0.0.max(p1.0).max(p2.0).ceil().min(self.width as f32 - 1.0) as i32;
        let min_y = p0.1.min(p1.1).min(p2.1).floor().max(0.0) as i32;
        let max_y =
            p0.1.max(p1.1)
                .max(p2.1)
                .ceil()
                .min(self.height as f32 - 1.0) as i32;

        let edge = |a: (f32, f32, f32), b: (f32, f32, f32), x: f32, y: f32| -> f32 {
            (x - a.0) * (b.1 - a.1) - (y - a.1) * (b.0 - a.0)
        };

        let area = edge(p0, p1, p2.0, p2.1);
        if area.abs() < 1e-4 {
            return;
        }

        for py in min_y..=max_y {
            for px in min_x..=max_x {
                let x = px as f32 + 0.5;
                let y = py as f32 + 0.5;
                let w0 = edge(p1, p2, x, y) / area;
                let w1 = edge(p2, p0, x, y) / area;
                let w2 = edge(p0, p1, x, y) / area;
                if w0 >= -1e-4 && w1 >= -1e-4 && w2 >= -1e-4 {
                    let z = w0 * p0.2 + w1 * p1.2 + w2 * p2.2;
                    self.put_pixel_depth(px, py, z, c);
                }
            }
        }
    }

    pub fn fill_quad_3d(
        &mut self,
        p0: (f32, f32, f32),
        p1: (f32, f32, f32),
        p2: (f32, f32, f32),
        p3: (f32, f32, f32),
        c: Color,
    ) {
        self.fill_triangle_3d(p0, p1, p2, c);
        self.fill_triangle_3d(p0, p2, p3, c);
    }

    pub fn draw_text_centered(&mut self, cx: i32, cy: i32, text: &str, scale: i32, c: Color) {
        let clean = normalize_ascii(text);
        let w = (clean.len() as i32) * 6 * scale;
        let h = 7 * scale;
        self.draw_text(cx - w / 2, cy - h / 2, &clean, scale, c);
    }

    pub fn draw_badge_text(
        &mut self,
        cx: i32,
        cy: i32,
        text: &str,
        scale: i32,
        fg: Color,
        bg: Color,
    ) {
        let clean = normalize_ascii(text);
        let w = (clean.len() as i32) * 6 * scale;
        let h = 7 * scale;
        let pad_x = 5 * scale;
        let pad_y = 3 * scale;
        self.fill_rect(
            cx - w / 2 - pad_x,
            cy - h / 2 - pad_y,
            w + pad_x * 2,
            h + pad_y * 2,
            bg,
        );
        self.draw_text(cx - w / 2, cy - h / 2, &clean, scale, fg);
    }

    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, scale: i32, c: Color) {
        let mut cursor_x = x;
        for ch in text.chars() {
            let glyph = glyph_5x7(ch);
            for (row_idx, row_bits) in glyph.iter().enumerate() {
                for col_idx in 0..5 {
                    if (row_bits >> (4 - col_idx)) & 1 == 1 {
                        self.fill_rect(
                            cursor_x + col_idx * scale,
                            y + (row_idx as i32) * scale,
                            scale,
                            scale,
                            c,
                        );
                    }
                }
            }
            cursor_x += 6 * scale;
        }
    }
}

fn normalize_ascii(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'é' | 'è' | 'ê' | 'ë' => out.push('e'),
            'É' | 'È' | 'Ê' => out.push('E'),
            'à' | 'â' | 'ä' => out.push('a'),
            'À' | 'Â' => out.push('A'),
            'î' | 'ï' => out.push('i'),
            'ô' | 'ö' => out.push('o'),
            'ù' | 'û' | 'ü' => out.push('u'),
            'ç' => out.push('c'),
            '²' => out.push('2'),
            '³' => out.push('3'),
            '€' => out.push_str("EUR"),
            '×' => out.push('x'),
            '—' | '–' => out.push('-'),
            _ => out.push(c),
        }
    }
    out
}

fn glyph_5x7(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'D' => [
            0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
        '.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100,
        ],
        ',' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '+' => [
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ],
        '/' => [
            0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000,
        ],
        ':' => [
            0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000,
        ],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ],
        '&' => [
            0b01100, 0b10010, 0b10100, 0b01000, 0b10101, 0b10010, 0b01101,
        ],
        '\'' => [
            0b00100, 0b00100, 0b01000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '%' => [
            0b11001, 0b11010, 0b00010, 0b00100, 0b01000, 0b01011, 0b10011,
        ],
        _ => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
    }
}
