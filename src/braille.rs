use std::fmt::Write as FmtWrite;
use std::io::{self, Write};

const DOT_OFFSETS: [(usize, usize); 8] = [
    (0, 0),
    (1, 0),
    (0, 1),
    (1, 1),
    (0, 2),
    (1, 2),
    (0, 3),
    (1, 3),
];

const BAYER: [f32; 16] = [
    0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0,
];

#[derive(Clone, Copy, Default)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone, Copy, Default)]
struct Pixel {
    light: f32,
    r: f32,
    g: f32,
    b: f32,
}

pub struct Framebuffer {
    cols: u16,
    rows: u16,
    width: usize,
    height: usize,
    pixels: Vec<Pixel>,
}

impl Framebuffer {
    pub fn new(cols: u16, rows: u16) -> Self {
        let cols = cols.max(1);
        let rows = rows.max(1);
        let width = cols as usize * 2;
        let height = rows as usize * 4;
        Self {
            cols,
            rows,
            width,
            height,
            pixels: vec![Pixel::default(); width * height],
        }
    }

    pub fn clear(&mut self) {
        self.pixels.fill(Pixel::default());
    }

    pub fn plot_normalized(&mut self, x: f32, y: f32, color: Rgb, energy: f32, radius: i32) {
        let x = x.clamp(0.0, 1.0) * (self.width.saturating_sub(1)) as f32;
        let y = y.clamp(0.0, 1.0) * (self.height.saturating_sub(1)) as f32;
        self.splat(x, y, color, energy, radius);
    }

    pub fn segment_normalized(
        &mut self,
        from: (f32, f32),
        to: (f32, f32),
        color: Rgb,
        energy: f32,
        radius: i32,
    ) {
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;
        let distance = (dx * dx + dy * dy).sqrt();
        let steps = (distance * (self.width.max(self.height) as f32) * 1.5)
            .ceil()
            .max(1.0) as usize;
        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            self.plot_normalized(from.0 + dx * t, from.1 + dy * t, color, energy, radius);
        }
    }

    pub fn present<W: Write>(&self, output: &mut W) -> io::Result<()> {
        output.write_all(self.frame().as_bytes())?;
        output.flush()
    }

    fn splat(&mut self, x: f32, y: f32, color: Rgb, energy: f32, radius: i32) {
        if energy <= 0.0 {
            return;
        }
        let center_x = x.round() as i32;
        let center_y = y.round() as i32;
        let radius = radius.max(0);
        let falloff_radius = radius as f32 + 1.0;
        for offset_y in -radius..=radius {
            for offset_x in -radius..=radius {
                let distance = ((offset_x * offset_x + offset_y * offset_y) as f32).sqrt();
                let falloff = (1.0 - distance / falloff_radius).max(0.0);
                if falloff <= 0.0 {
                    continue;
                }
                let pixel_x = center_x + offset_x;
                let pixel_y = center_y + offset_y;
                if pixel_x < 0 || pixel_y < 0 {
                    continue;
                }
                let pixel_x = pixel_x as usize;
                let pixel_y = pixel_y as usize;
                if pixel_x >= self.width || pixel_y >= self.height {
                    continue;
                }
                let amount = energy * falloff;
                let pixel = &mut self.pixels[pixel_y * self.width + pixel_x];
                pixel.light += amount;
                pixel.r += color.r * amount;
                pixel.g += color.g * amount;
                pixel.b += color.b * amount;
            }
        }
    }

    fn frame(&self) -> String {
        let mut output = String::with_capacity(self.cols as usize * self.rows as usize * 24);
        for row in 0..self.rows as usize {
            let _ = write!(&mut output, "\x1b[{};1H", row + 1);
            for col in 0..self.cols as usize {
                let mut dots = 0u8;
                let mut light = 0.0;
                let mut red = 0.0;
                let mut green = 0.0;
                let mut blue = 0.0;
                for (dot, (offset_x, offset_y)) in DOT_OFFSETS.iter().enumerate() {
                    let pixel_x = col * 2 + offset_x;
                    let pixel_y = row * 4 + offset_y;
                    let pixel = self.pixels[pixel_y * self.width + pixel_x];
                    let threshold = 0.055 + BAYER[(pixel_x + pixel_y) % BAYER.len()] / 16.0 * 0.24;
                    if pixel.light > threshold {
                        dots |= 1 << dot;
                        light += pixel.light;
                        red += pixel.r;
                        green += pixel.g;
                        blue += pixel.b;
                    }
                }
                if dots == 0 {
                    output.push(' ');
                    continue;
                }
                let scale = 255.0 / light.max(0.001);
                let red = (red * scale).round().clamp(0.0, 255.0) as u8;
                let green = (green * scale).round().clamp(0.0, 255.0) as u8;
                let blue = (blue * scale).round().clamp(0.0, 255.0) as u8;
                let glyph = char::from_u32(0x2800 + dots as u32).unwrap_or(' ');
                let _ = write!(&mut output, "\x1b[38;2;{red};{green};{blue}m{glyph}");
            }
            output.push_str("\x1b[0m\x1b[K");
        }
        output
    }
}
