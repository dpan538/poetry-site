pub const COLS: usize = 200;
pub const ROWS: usize = COLS / 2;

#[path = "light.rs"]
mod light;
#[path = "mountain.rs"]
mod mountain;
#[path = "sprite.rs"]
mod sprite;
#[path = "wivenhoe.rs"]
mod wivenhoe;

#[derive(Clone, Copy)]
enum Motion {
    Flare,
    Winter,
    Screen,
    Moonlight,
    Wivenhoe,
    Clouds,
}

pub struct Painting {
    pub piece: &'static str,
    pub number: u8,
    pub label: &'static str,
    pub caption: &'static str,
    pub source: &'static str,
    pub soft: bool,
    rgb: &'static [u8],
    photo_y: [f32; 2],
    motion: Motion,
}

impl Painting {
    pub fn rows(&self) -> usize {
        self.rgb.len() / (COLS * 3)
    }

    pub fn grid(&self, narrow: bool) -> (usize, usize) {
        let cols = if self.source.is_empty() {
            if narrow { 68 } else { 102 }
        } else if narrow {
            COLS / 2
        } else {
            COLS
        };
        (cols, (self.rows() * cols + COLS / 2) / COLS)
    }
}

include!(concat!(env!("OUT_DIR"), "/paintings.rs"));

fn luminance([r, g, b]: [f32; 3]) -> f32 {
    r * 0.2126 + g * 0.7152 + b * 0.0722
}

pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let channel = |offset: f32| {
        let pos = (offset + h * 6.0).rem_euclid(6.0);
        let ramp = pos.min(4.0 - pos).clamp(0.0, 1.0);
        ((v - v * s * ramp) * 255.0).round() as u8
    };
    [channel(5.0), channel(3.0), channel(1.0)]
}

pub fn noise(seed: u32) -> f32 {
    let mut hash = seed.wrapping_mul(0x9e37_79b9);
    hash = (hash ^ (hash >> 16)).wrapping_mul(0x85eb_ca6b);
    ((hash ^ (hash >> 13)) >> 8) as f32 / 16_777_216.0
}

pub struct Ink {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub density: f32,
}

pub struct Study {
    painting: &'static Painting,
    mountain: Option<mountain::Layers>,
    black: f32,
    white: f32,
}

impl Study {
    pub fn new(painting: &'static Painting) -> Self {
        let mut light_counts = [0; 256];
        for pixel in painting.rgb.chunks_exact(3) {
            let luma = luminance([pixel[0] as f32, pixel[1] as f32, pixel[2] as f32]);
            light_counts[luma.round() as usize] += 1;
        }
        let percentile = |part: f32| {
            let cut = (part * (COLS * painting.rows()) as f32) as usize;
            let mut total = 0;
            for (value, count) in light_counts.iter().enumerate() {
                total += count;
                if total >= cut {
                    return value as f32 / 255.0;
                }
            }
            1.0
        };
        Self {
            painting,
            mountain: matches!(painting.motion, Motion::Clouds)
                .then(|| mountain::Layers::new(painting.rgb)),
            black: percentile(0.025),
            white: percentile(0.97),
        }
    }

    fn pixel(pixels: &[u8], x: usize, y: usize) -> [f32; 3] {
        let start = (y * COLS + x) * 3;
        let rgb = &pixels[start..start + 3];
        [
            rgb[0] as f32 / 255.0,
            rgb[1] as f32 / 255.0,
            rgb[2] as f32 / 255.0,
        ]
    }

    fn sample(&self, x: f32, y: f32) -> [f32; 3] {
        Self::sample_pixels(self.painting.rgb, x, y)
    }

    fn sample_pixels(pixels: &[u8], x: f32, y: f32) -> [f32; 3] {
        let x = x.clamp(0.0, (COLS - 1) as f32);
        let rows = pixels.len() / (COLS * 3);
        let y = y.clamp(0.0, (rows - 1) as f32);
        let (left, top) = (x as usize, y as usize);
        let right = (left + 1).min(COLS - 1);
        let bottom = (top + 1).min(rows - 1);
        let [a, b, c, d] = [
            Self::pixel(pixels, left, top),
            Self::pixel(pixels, right, top),
            Self::pixel(pixels, left, bottom),
            Self::pixel(pixels, right, bottom),
        ];
        let mix = |a: f32, b: f32, at: f32| a + (b - a) * at;
        std::array::from_fn(|i| {
            mix(
                mix(a[i], b[i], x.fract()),
                mix(c[i], d[i], x.fract()),
                y.fract(),
            )
        })
    }

    pub fn painting(&self) -> &'static Painting {
        self.painting
    }

    pub fn exposure(&self, seconds: f32) -> f32 {
        light::exposure(self.painting.motion, seconds)
    }

    pub fn photo_ink(&self, column: f32, row: f32, exposure: f32) -> [f32; 3] {
        let x = column / (COLS - 1) as f32;
        let [top, height] = self.painting.photo_y;
        let y = top + row / (self.painting.rows() - 1) as f32 * height;
        let source = self.sample(column, row);
        let motion = self.painting.motion;
        light::photograph(motion, source, x, y, exposure, light::region(motion, x, y))
    }

    pub fn motion_strength(&self, column: f32, row: f32) -> f32 {
        let x = column / (COLS - 1) as f32;
        let y = row / (self.painting.rows() - 1) as f32;
        match self.painting.motion {
            Motion::Wivenhoe => f32::from(u8::from(wivenhoe::moves(column, row))),
            Motion::Clouds => mountain::region(x, y),
            motion => light::region(motion, x, y),
        }
    }

    pub fn ink(
        &self,
        column: f32,
        row: f32,
        seconds: f32,
        exposure: f32,
        grain: f32,
        strength: f32,
    ) -> Ink {
        let x = column / (COLS - 1) as f32;
        let y = row / (self.painting.rows() - 1) as f32;
        let source = self.sample(column, row);
        let color = if strength == 0.0 {
            source
        } else {
            match self.painting.motion {
                Motion::Wivenhoe => wivenhoe::compose(self, column, row, seconds),
                Motion::Clouds => self.mountain.as_ref().map_or(source, |layers| {
                    let clouds = layers.compose(column, row, seconds);
                    std::array::from_fn(|i| source[i] + (clouds[i] - source[i]) * strength)
                }),
                Motion::Moonlight => light::moonlit(source, column, row, seconds, exposure),
                motion => light::photograph(motion, source, x, y, exposure, strength),
            }
        };
        let [r, g, b] = color;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        let hue = (if delta < 0.003 {
            0.0
        } else if max == r {
            (g - b) / delta
        } else if max == g {
            (b - r) / delta + 2.0
        } else {
            (r - g) / delta + 4.0
        } / 6.0)
            .rem_euclid(1.0);
        let saturation = delta / max.max(0.001);
        let red_distance = hue.min(1.0 - hue);
        let red_weight = (1.0 - red_distance * 12.0).max(0.0);
        let pigment = ((saturation - 0.06) / 0.18).clamp(0.0, 1.0);
        let graded =
            ((luminance(color) - self.black) / (self.white - self.black).max(0.13)).clamp(0.0, 1.0);
        // Layered scenes set colour and dot size from the same composed frame.
        let density = (0.06 + graded * 0.88 + (grain - 0.5) * 0.32).clamp(0.0, 1.0);
        let (chroma, lift) = if self.painting.soft {
            let greens = (1.0 - (hue - 0.35).abs() / 0.23).max(0.0);
            (0.94 - 0.16 * greens, 0.085 * (1.0 - max).powi(2))
        } else {
            (1.0, 0.0)
        };
        Ink {
            hue,
            saturation: (saturation * chroma * (1.0 + 0.22 * red_weight * pigment)).min(1.0),
            value: (max + lift).clamp(0.0, 1.0),
            density,
        }
    }
}
