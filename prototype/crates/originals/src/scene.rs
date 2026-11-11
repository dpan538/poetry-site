use std::{
    collections::VecDeque,
    f32::consts::{PI, TAU},
};

pub const PERIOD: f32 = 20.0;
const WIDTH: usize = 340;
const HEIGHT: usize = 200;
pub type Color = [f32; 3];

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t.clamp(0.0, 1.0))
}

fn scale(color: Color, gain: f32) -> Color {
    color.map(|c| (c * gain).clamp(0.0, 1.0))
}

pub fn smooth(a: f32, b: f32, v: f32) -> f32 {
    let t = ((v - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn hash(x: u32, y: u32) -> f32 {
    let n = x.wrapping_mul(374_761_393) ^ y.wrapping_mul(668_265_263);
    let n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    (n ^ (n >> 16)) as f32 / u32::MAX as f32
}

fn line(x: f32, y: f32, a: [f32; 2], b: [f32; 2], width: f32) -> f32 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let at = (((x - a[0]) * dx + (y - a[1]) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    let distance = ((x - a[0] - dx * at).powi(2) + (y - a[1] - dy * at).powi(2)).sqrt();
    1.0 - smooth(width * 0.5, width * 0.5 + 0.0018, distance)
}

fn rect(x: f32, y: f32, bounds: [f32; 4]) -> f32 {
    let [left, top, right, bottom] = bounds;
    smooth(left - 0.001, left + 0.001, x)
        * smooth(right + 0.001, right - 0.001, x)
        * smooth(top - 0.001, top + 0.001, y)
        * smooth(bottom + 0.001, bottom - 0.001, y)
}

fn ellipse(x: f32, y: f32, center: [f32; 2], radius: [f32; 2]) -> f32 {
    let d = ((x - center[0]) / radius[0]).powi(2) + ((y - center[1]) / radius[1]).powi(2);
    1.0 - smooth(0.85, 1.12, d)
}

fn polygon(x: f32, y: f32, vertices: &[[f32; 2]]) -> f32 {
    let mut inside = false;
    for (a, b) in vertices.iter().zip(vertices.iter().cycle().skip(1)) {
        if (a[1] > y) != (b[1] > y) && x < a[0] + (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]) {
            inside = !inside;
        }
    }
    f32::from(u8::from(inside))
}

fn noise(x: f32, y: f32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let u = smooth(0.0, 1.0, x.fract().rem_euclid(1.0));
    let v = smooth(0.0, 1.0, y.fract().rem_euclid(1.0));
    let a = hash(ix as u32, iy as u32);
    let b = hash((ix + 1) as u32, iy as u32);
    let c = hash(ix as u32, (iy + 1) as u32);
    let d = hash((ix + 1) as u32, (iy + 1) as u32);
    (a + (b - a) * u) * (1.0 - v) + (c + (d - c) * u) * v
}

fn sample(pixels: &[Color], x: f32, y: f32) -> Color {
    let px = (x * (WIDTH - 1) as f32).clamp(0.0, (WIDTH - 1) as f32);
    let py = (y * (HEIGHT - 1) as f32).clamp(0.0, (HEIGHT - 1) as f32);
    let (left, top) = (px as usize, py as usize);
    let (right, bottom) = ((left + 1).min(WIDTH - 1), (top + 1).min(HEIGHT - 1));
    mix(
        mix(
            pixels[top * WIDTH + left],
            pixels[top * WIDTH + right],
            px.fract(),
        ),
        mix(
            pixels[bottom * WIDTH + left],
            pixels[bottom * WIDTH + right],
            px.fract(),
        ),
        py.fract(),
    )
}

fn sample_mask(mask: &[f32], x: f32, y: f32) -> f32 {
    let px = (x * (WIDTH - 1) as f32).clamp(0.0, (WIDTH - 1) as f32);
    let py = (y * (HEIGHT - 1) as f32).clamp(0.0, (HEIGHT - 1) as f32);
    let (left, top) = (px as usize, py as usize);
    let (right, bottom) = ((left + 1).min(WIDTH - 1), (top + 1).min(HEIGHT - 1));
    let upper =
        mask[top * WIDTH + left] * (1.0 - px.fract()) + mask[top * WIDTH + right] * px.fract();
    let lower = mask[bottom * WIDTH + left] * (1.0 - px.fract())
        + mask[bottom * WIDTH + right] * px.fract();
    upper * (1.0 - py.fract()) + lower * py.fract()
}

// A common wind drives the leaves first, then the cloth and its projected light.
// Integer harmonics keep every moving surface continuous across the 20 s loop.
fn breeze(phase: f32, lag: f32) -> f32 {
    0.72 * (phase * 4.0 - lag).sin() + 0.28 * (phase * 7.0 - lag * 1.7).sin()
}

struct GauzeLayers {
    background: Vec<Color>,
    foliage: Vec<f32>,
    curtain: Vec<f32>,
}

struct TwilightLayers {
    sky: Vec<f32>,
    clear: Vec<Color>,
    sunset: Vec<Color>,
}

impl TwilightLayers {
    fn new(image: &[Color]) -> Self {
        // Trace connected sky from an open-sky seed. Dark tree branches and
        // roof edges block the flood; the foreground is never a moving layer.
        let eligible: Vec<bool> = image
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let x = (i % WIDTH) as f32 / (WIDTH - 1) as f32;
                let y = (i / WIDTH) as f32 / (HEIGHT - 1) as f32;
                let luma = c[0] * 0.2126 + c[1] * 0.7152 + c[2] * 0.0722;
                y < 0.602 - 0.14 * x * x && c[2] > 0.24 && luma > 0.205
            })
            .collect();
        let mut sky = vec![0.0; WIDTH * HEIGHT];
        let mut queue = VecDeque::new();
        for (x, y) in [(170, 20), (240, 15), (120, 55), (60, 95)] {
            let i = y * WIDTH + x;
            if eligible[i] && sky[i] == 0.0 {
                sky[i] = 1.0;
                queue.push_back(i);
            }
        }
        while let Some(i) = queue.pop_front() {
            let x = i % WIDTH;
            let y = i / WIDTH;
            for (nx, ny) in [
                (x.saturating_sub(1), y),
                ((x + 1).min(WIDTH - 1), y),
                (x, y.saturating_sub(1)),
                (x, (y + 1).min(HEIGHT - 1)),
            ] {
                let n = ny * WIDTH + nx;
                if eligible[n] && sky[n] == 0.0 {
                    sky[n] = 1.0;
                    queue.push_back(n);
                }
            }
        }
        let mut clear = Vec::with_capacity(image.len());
        let mut sunset = Vec::with_capacity(image.len());
        for (i, &original) in image.iter().enumerate() {
            let warmth = sky[i]
                * smooth(0.015, 0.13, original[0] - original[1])
                * smooth(-0.17, 0.12, original[0] - original[2]);
            // Revealed gaps between sunset clouds belong to blue sky. A
            // capped red channel with unchanged green would create cyan stains
            // where a warm cloud moves away, especially beside dark branches.
            let cool_green = original[1] * 0.62 + 0.02;
            let cool = [
                original[0].min(0.22),
                cool_green,
                (original[2] * 0.76 + 0.14).max(cool_green + 0.11),
            ];
            let background = mix(original, cool, warmth);
            clear.push(background);
            sunset.push(std::array::from_fn(|c| original[c] - background[c]));
        }
        Self { sky, clear, sunset }
    }
}

const DUST_COLUMNS: usize = 32;
const DUST_ROWS: usize = 20;

struct DustParticle {
    center: [f32; 2],
    radius: [f32; 2],
    opacity: f32,
    color: Color,
    foreground: bool,
}

const CURTAIN: &[[f32; 2]] = &[
    [0.509, 0.0],
    [0.651, 0.0],
    [0.674, 0.734],
    [0.650, 0.743],
    [0.618, 0.733],
    [0.587, 0.738],
    [0.550, 0.723],
    [0.519, 0.733],
    [0.493, 0.687],
];

pub struct Scene {
    pub kind: u8,
    image: Vec<Color>,
    gauze_layers: Option<GauzeLayers>,
    twilight: Option<TwilightLayers>,
    dust: Vec<DustParticle>,
    dust_bins: Vec<Vec<usize>>,
    light_levels: Vec<f32>,
    light_bins: Vec<Vec<usize>>,
}

impl Scene {
    pub fn new(kind: u8) -> Self {
        let bytes: &[u8] = match kind {
            2 => include_bytes!(concat!(env!("OUT_DIR"), "/dusk-gauze.rgb")),
            3 => include_bytes!(concat!(env!("OUT_DIR"), "/crown-tiles.rgb")),
            5 => include_bytes!(concat!(env!("OUT_DIR"), "/drowned-water.rgb")),
            _ => &[],
        };
        let image: Vec<Color> = bytes
            .chunks_exact(3)
            .map(|p| {
                [
                    p[0] as f32 / 255.0,
                    p[1] as f32 / 255.0,
                    p[2] as f32 / 255.0,
                ]
            })
            .collect();
        let gauze_layers = if kind == 2 {
            let mut background = image.clone();
            let curtain: Vec<f32> = (0..WIDTH * HEIGHT)
                .map(|i| {
                    polygon(
                        (i % WIDTH) as f32 / (WIDTH - 1) as f32,
                        (i / WIDTH) as f32 / (HEIGHT - 1) as f32,
                        CURTAIN,
                    )
                })
                .collect();
            let mask: Vec<usize> = (0..WIDTH * HEIGHT).filter(|&i| curtain[i] > 0.5).collect();
            // Reconstruct the narrow background exposed by cloth movement once.
            // The generated curtain is then rendered as its own anchored surface.
            for &i in &mask {
                let y = i / WIDTH;
                let x = (i % WIDTH) as f32 / WIDTH as f32;
                background[i] = mix(
                    image[y * WIDTH + 165],
                    image[y * WIDTH + 235],
                    (x - 0.49) / 0.2,
                );
            }
            for _ in 0..180 {
                for &i in &mask {
                    let up = i.saturating_sub(WIDTH);
                    let down = (i + WIDTH).min(WIDTH * HEIGHT - 1);
                    background[i] = std::array::from_fn(|c| {
                        (background[up][c]
                            + background[down][c]
                            + background[i - 1][c]
                            + background[i + 1][c])
                            * 0.25
                    });
                }
            }
            // Extract the dark foliage inside the open window. Its frame and
            // sill never enter the moving layer. Reconstruct only the sky that
            // can be exposed at a leaf edge, rather than sliding a second image
            // over a stationary copy of the same branch.
            let foliage: Vec<f32> = image
                .iter()
                .enumerate()
                .map(|(i, &c)| {
                    let x = (i % WIDTH) as f32 / (WIDTH - 1) as f32;
                    let y = (i / WIDTH) as f32 / (HEIGHT - 1) as f32;
                    let peak = c.iter().copied().fold(0.0_f32, f32::max);
                    rect(x, y, [0.159, 0.042, 0.481, 0.545])
                        * smooth(0.52, 0.35, peak)
                        * smooth(-0.005, 0.025, c[1] - c[0])
                })
                .collect();
            let foliage_indices: Vec<usize> =
                (0..WIDTH * HEIGHT).filter(|&i| foliage[i] > 0.01).collect();
            for &i in &foliage_indices {
                let y = (i / WIDTH) as f32 / (HEIGHT - 1) as f32;
                let sky = mix(
                    [0.36, 0.57, 0.89],
                    [0.98, 0.58, 0.42],
                    smooth(0.07, 0.37, y),
                );
                background[i] = mix(sky, [0.07, 0.19, 0.26], smooth(0.35, 0.54, y));
            }
            for _ in 0..100 {
                for &i in &foliage_indices {
                    let up = i.saturating_sub(WIDTH);
                    let down = (i + WIDTH).min(WIDTH * HEIGHT - 1);
                    background[i] = std::array::from_fn(|c| {
                        (background[up][c]
                            + background[down][c]
                            + background[i - 1][c]
                            + background[i + 1][c])
                            * 0.25
                    });
                }
            }
            Some(GauzeLayers {
                background,
                foliage,
                curtain,
            })
        } else {
            None
        };
        let twilight = if kind == 5 {
            Some(TwilightLayers::new(&image))
        } else {
            None
        };
        let light_bins = if kind == 5 {
            (0..DUST_COLUMNS)
                .map(|col| {
                    let left = col as f32 / DUST_COLUMNS as f32;
                    let right = (col + 1) as f32 / DUST_COLUMNS as f32;
                    LATE_LIGHTS
                        .iter()
                        .enumerate()
                        .filter_map(|(i, light)| {
                            (light[0] + 0.055 >= left && light[0] - 0.055 <= right).then_some(i)
                        })
                        .collect()
                })
                .collect()
        } else {
            Vec::new()
        };
        let mut scene = Self {
            kind,
            image,
            gauze_layers,
            twilight,
            dust: Vec::new(),
            dust_bins: if matches!(kind, 2 | 3) {
                (0..DUST_COLUMNS * DUST_ROWS).map(|_| Vec::new()).collect()
            } else {
                Vec::new()
            },
            light_levels: if kind == 5 {
                vec![0.0; LATE_LIGHTS.len()]
            } else {
                Vec::new()
            },
            light_bins,
        };
        scene.prepare_frame(0.0);
        scene
    }

    pub fn prepare_frame(&mut self, seconds: f32) {
        if self.kind == 5 {
            let q = seconds.rem_euclid(PERIOD) / PERIOD;
            for (i, level) in self.light_levels.iter_mut().enumerate() {
                *level = late_light_level(i, q);
            }
            return;
        }
        if !matches!(self.kind, 2 | 3) {
            return;
        }
        let t = seconds.rem_euclid(PERIOD);
        let phase = t * TAU / PERIOD;
        let drift = 0.020 * breeze(phase, 0.4);
        self.dust.clear();
        for bin in &mut self.dust_bins {
            bin.clear();
        }
        // Compute trajectories once per frame and index their footprints.
        // A denser room must not require hundreds of particle evaluations for
        // every cell of the mobile renderer.
        let count = if self.kind == 2 { 54 } else { 330 };
        for i in 0..count {
            let seed = hash(i, 37);
            let foreground = self.kind == 2 || i >= 210;
            let circuits = if self.kind == 2 && hash(i, 71) > 0.55 {
                2.0
            } else {
                1.0
            };
            let age = (hash(i, 12) + t * circuits / PERIOD).fract();
            let (px, py, fade, settled) = if self.kind == 2 {
                let vertical = if hash(i, 51) > 0.25 { age } else { 1.0 - age };
                (
                    0.255 + seed * 0.40 + 0.015 * (phase * 3.0 + seed * 9.0).sin(),
                    0.20 + vertical * 0.61,
                    (PI * age).sin().powi(2),
                    false,
                )
            } else {
                // Dust falls through the air, then remains on a surface.
                // Only an invisible particle is respawned at the top of its
                // next life; its visible vertical trajectory never reverses.
                let base_x = if foreground {
                    0.055 + seed * 0.89
                } else {
                    0.268 + seed * 0.47
                };
                let surface = if foreground {
                    0.943 + hash(i, 81) * 0.028
                } else if base_x < 0.38 {
                    0.503
                } else if base_x < 0.46 || base_x > 0.65 {
                    0.605
                } else {
                    0.707
                };
                let top = if foreground {
                    0.065 + hash(i, 61) * 0.16
                } else {
                    0.155 + hash(i, 61) * 0.06
                };
                let fall = (age / 0.77).min(1.0).powf(1.15);
                let px = base_x + 0.012 * (phase * 2.0 + seed * 9.0).sin() * (1.0 - fall);
                let py = top + (surface - top) * fall;
                let fade = smooth(0.0, 0.055, age) * (1.0 - smooth(0.93, 1.0, age));
                (px, py, fade, age >= 0.77)
            };
            let beam_center = if self.kind == 2 {
                0.65 - py * 0.32
            } else {
                0.81 - py * 0.57
            };
            let in_beam = (-(px - (beam_center + drift)).powi(2) / 0.014).exp();
            let radius = if self.kind == 2 {
                0.0028 + hash(i, 93) * 0.0022
            } else if foreground {
                0.0035 + hash(i, 93) * 0.0035
            } else {
                0.0025 + hash(i, 93) * 0.0035
            };
            let sparkle = if !settled && (self.kind == 2 || foreground) {
                0.58 + 0.42 * (0.5 + 0.5 * (phase * 13.0 + seed * TAU).sin()).powi(2)
            } else {
                1.0
            };
            let particle = DustParticle {
                center: [px, py],
                radius: [radius, radius * if settled { 0.45 } else { 1.7 }],
                opacity: fade * (0.20 + 0.80 * in_beam) * 0.8 * sparkle,
                color: mix([0.70, 0.72, 0.67], [1.0, 0.85, 0.50], in_beam),
                foreground,
            };
            let left = ((px - radius * 1.1) * DUST_COLUMNS as f32).floor().max(0.0) as usize;
            let right = (((px + radius * 1.1) * DUST_COLUMNS as f32).floor() as usize)
                .min(DUST_COLUMNS - 1);
            let top = ((py - radius * 1.9) * DUST_ROWS as f32).floor().max(0.0) as usize;
            let bottom =
                (((py + radius * 1.9) * DUST_ROWS as f32).floor() as usize).min(DUST_ROWS - 1);
            let index = self.dust.len();
            self.dust.push(particle);
            for row in top..=bottom {
                for col in left..=right {
                    self.dust_bins[row * DUST_COLUMNS + col].push(index);
                }
            }
        }
    }

    pub fn color(&self, x: f32, y: f32, seconds: f32) -> Color {
        let t = seconds.rem_euclid(PERIOD);
        match self.kind {
            0 => powerlines(x, y, t),
            1 => private_sky(x, y, t),
            2 => self.gauze(x, y, t),
            3 => self.tiles(x, y, t),
            4 => night_light(x, y, t),
            _ => self.water(x, y, t),
        }
    }

    fn dust_light(&self, mut color: Color, x: f32, y: f32, glass: f32) -> Color {
        let col = ((x * DUST_COLUMNS as f32) as usize).min(DUST_COLUMNS - 1);
        let row = ((y * DUST_ROWS as f32) as usize).min(DUST_ROWS - 1);
        for &index in &self.dust_bins[row * DUST_COLUMNS + col] {
            let mote = &self.dust[index];
            let visibility = if mote.foreground { 1.0 } else { glass };
            let dot = ellipse(x, y, mote.center, mote.radius);
            color = mix(color, mote.color, dot * mote.opacity * visibility);
        }
        color
    }

    fn gauze(&self, x: f32, y: f32, t: f32) -> Color {
        let layers = self.gauze_layers.as_ref().expect("Prepared gauze layers");
        let phase = t * TAU / PERIOD;
        let mut color = sample(&layers.background, x, y);

        // Branches bend from the left attachment, with faster, smaller leaf
        // flutter. The lower hedge responds less than the exposed branch tips.
        let root = smooth(0.163, 0.225, x) * smooth(0.545, 0.49, y);
        let tip = root * (0.35 + 0.65 * smooth(0.49, 0.12, y));
        let leaf_x = x - 0.018 * tip * breeze(phase, y * 1.4);
        let leaf_y = y - tip
            * (0.011 * breeze(phase, x * 2.8)
                + 0.0025 * (phase * 11.0 + x * 35.0 + y * 19.0).sin());
        let leaves =
            sample_mask(&layers.foliage, leaf_x, leaf_y) * rect(x, y, [0.159, 0.042, 0.481, 0.545]);
        color = mix(color, sample(&self.image, leaf_x, leaf_y), leaves);

        // Keep the heading fixed; displacement and lift grow toward the hem.
        let free = smooth(0.0, 0.75, y).powf(1.4);
        let sway = 0.018 * free * breeze(phase, 0.45 + y * 1.4);
        let lift = 0.004 * free * (phase * 4.0 - 0.75).sin();
        let cloth_x = x - sway - 0.0015 * free * (phase * 7.0 - y * 12.0).sin();
        let cloth_y = y + lift;
        let cloth = sample_mask(&layers.curtain, cloth_x, cloth_y);
        let cloth_color = scale(
            sample(&self.image, cloth_x, cloth_y),
            1.0 + 0.035 * free * breeze(phase, 0.8),
        );
        color = mix(color, cloth_color, cloth);

        // Transmitted light and leaf shadows travel across the existing warm
        // pool on the floor. No board, bed or window geometry is displaced.
        let original = sample(&self.image, x, y);
        let floor = smooth(0.775, 0.81, y)
            * smooth(0.995, 0.94, y)
            * smooth(0.06, 0.16, x)
            * smooth(0.685, 0.65, x)
            * smooth(0.02, 0.12, original[0] - original[1]);
        let cast = breeze(phase, 1.05);
        let shadow = smooth(
            0.43,
            0.73,
            noise(x * 26.0 + y * 17.0 - cast * 1.1, y * 48.0 - cast * 0.65),
        );
        let transmission = 1.015 + 0.055 * breeze(phase, 1.2) - 0.25 * shadow;
        color = mix(color, scale(color, transmission), floor);

        // Small openings between moving leaves throw warm flecks onto the
        // floor. Each has its own short rise/fall and bounded displacement;
        // these are changes in projected light, not moving floor texture.
        if floor > 0.0 {
            let mut flecks = 0.0_f32;
            for (i, [cx, cy]) in [
                [0.23, 0.827],
                [0.32, 0.842],
                [0.41, 0.862],
                [0.49, 0.893],
                [0.59, 0.916],
                [0.31, 0.889],
                [0.44, 0.929],
                [0.55, 0.850],
                [0.61, 0.874],
            ]
            .iter()
            .enumerate()
            {
                if (x - cx).abs() > 0.055 || (y - cy).abs() > 0.034 {
                    continue;
                }
                let seed = hash(i as u32, 103);
                let offset = seed * TAU;
                let cycle = 13.0 + (i % 4) as f32 * 2.0;
                let pulse = (0.5 + 0.5 * (phase * cycle + offset).sin()).powi(3);
                let px = cx + 0.009 * (phase * 11.0 + offset).sin() + cast * 0.004;
                let py = cy + 0.004 * (phase * 17.0 + offset * 1.3).sin();
                let rx = 0.015 + seed * 0.010;
                let ry = 0.008 + seed * 0.005;
                let footprint = (-((x - px) / rx).powi(2) - ((y - py) / ry).powi(2)).exp();
                flecks = flecks.max(footprint * pulse);
            }
            color = mix(color, [1.0, 0.76, 0.34], floor * flecks * 0.42);
        }
        // Smaller transmitted shadows reach the bedside fabric and adjacent
        // floor. Change illumination only; the bed silhouette stays fixed.
        let bedside = smooth(0.645, 0.685, x)
            * smooth(0.82, 0.77, x)
            * smooth(0.61, 0.67, y)
            * smooth(0.925, 0.875, y);
        let bedside_shadow = smooth(
            0.40,
            0.72,
            noise(x * 35.0 + y * 11.0 - cast * 0.55, y * 51.0 - cast * 0.35),
        );
        color = mix(
            color,
            scale(color, 1.035 + 0.025 * cast - 0.11 * bedside_shadow),
            bedside,
        );
        self.dust_light(color, x, y, 0.0)
    }

    fn tiles(&self, x: f32, y: f32, t: f32) -> Color {
        let mut color = sample(&self.image, x, y);
        let phase = t * TAU / PERIOD;
        let pane =
            rect(x, y, [0.264, 0.153, 0.480, 0.715]).max(rect(x, y, [0.519, 0.153, 0.738, 0.715]));
        let drift = 0.020 * breeze(phase, 0.4);
        let beam = (-(x - (0.81 - y * 0.57 + drift)).powi(2) / 0.009).exp() * pane;
        let light = 0.5 + 0.5 * breeze(phase, 0.0);
        color = mix(color, [0.98, 0.70, 0.34], beam * (0.035 + 0.13 * light));
        self.dust_light(color, x, y, pane)
    }

    fn water(&self, x: f32, y: f32, t: f32) -> Color {
        let shore = 0.603;
        let near_bank = 0.775 + x * 0.36;
        let bank = smooth(near_bank + 0.004, near_bank - 0.012, y)
            * smooth(0.019, 0.035, x)
            * (1.0
                - polygon(
                    x,
                    y,
                    &[
                        [0.028, 0.666],
                        [0.154, 0.674],
                        [0.171, 0.766],
                        [0.237, 0.779],
                        [0.242, 0.804],
                        [0.216, 0.828],
                        [0.211, 0.904],
                        [0.183, 0.905],
                        [0.177, 0.842],
                        [0.150, 0.851],
                        [0.137, 0.895],
                        [0.110, 0.897],
                        [0.106, 0.819],
                        [0.047, 0.803],
                    ],
                ));
        let water = smooth(shore, shore + 0.028, y) * bank;
        // Color reaches the shoreline sooner than wave displacement. Reusing
        // the broad ripple feather here would leave a pink strip at blue hour.
        let wet_surface = smooth(shore - 0.002, shore + 0.004, y) * bank;
        // The rigid-object mask controls motion only. Ambient color must also
        // reach the bench, the water seen between slats and the nearby paving.
        let ambient_surface = smooth(shore - 0.002, shore + 0.004, y);
        let phase = TAU * t / PERIOD;
        // Preserve the original lit waterfront, then add windows after the
        // midpoint. Only the final loop transition returns to the opening sky.
        let q = t / PERIOD;
        let nightfall = smooth(0.12, 0.68, q) * (1.0 - smooth(0.90, 1.0, q));
        let depth = ((y - shore) / (1.0 - shore)).clamp(0.0, 1.0);
        let dx = (0.003 + 0.012 * depth)
            * ((y * 110.0 - phase * 6.0).sin() + 0.35 * (y * 233.0 + phase * 4.0).sin());
        let dy = (0.002 + depth * 0.0045) * (x * 23.0 + y * 76.0 - phase * 4.0).sin();
        let base = evening_light(self.twilight_sky(x, y, q), x, y, nightfall, ambient_surface);
        let color = if water > 0.0 {
            let ripple = evening_light(
                sample(&self.image, x + dx, y + dy),
                x + dx,
                y + dy,
                nightfall,
                wet_surface,
            );
            let glint = (x * 31.0 + y * 173.0 - phase * 7.0).sin() * (0.025 + depth * 0.07);
            mix(base, scale(ripple, 1.0 + glint), water)
        } else {
            base
        };
        let col = ((x * DUST_COLUMNS as f32) as usize).min(DUST_COLUMNS - 1);
        prepared_shore_lights(
            color,
            x,
            y,
            t,
            water,
            dx,
            &self.light_levels,
            &self.light_bins[col],
        )
    }

    fn twilight_sky(&self, x: f32, y: f32, q: f32) -> Color {
        let original = sample(&self.image, x, y);
        if q == 0.0 {
            return original;
        }
        let layers = self.twilight.as_ref().expect("Prepared sky layers");
        let sky = sample_mask(&layers.sky, x, y);
        if sky <= 0.0 {
            return original;
        }
        let close = smooth(0.90, 1.0, q);
        let nightfall = smooth(0.12, 0.68, q) * (1.0 - close);
        let visibility = (1.0 - smooth(0.08, 0.65, q)) * (1.0 - close) + close;
        // A one-way drift belongs to the sunset layer only. It has dispersed
        // before its hidden reset, so trees and roofs retain their silhouette.
        let travel = if q < 0.90 {
            0.052 * smooth(0.0, 0.65, q)
        } else {
            0.0
        };
        let cloud = sample(&layers.sunset, x - travel, y + travel * 0.10);
        let clear = sample(&layers.clear, x, y);
        let blue = [
            clear[0] * 0.25 + clear[1] * 0.06 + 0.009,
            clear[1] * 0.58 + clear[2] * 0.06 + 0.008,
            clear[2] * 0.76 + 0.065,
        ];
        let mut color = mix(clear, blue, nightfall);
        for c in 0..3 {
            color[c] += cloud[c] * visibility;
        }
        mix(original, color, sky)
    }
}

fn evening_light(original: Color, x: f32, y: f32, nightfall: f32, river: f32) -> Color {
    // Warm sunset water is ambient illumination, not an emissive surface.
    // Preserve gold only in the reflection columns of actual source lamps.
    let depth = ((y - 0.60) / 0.40).clamp(0.0, 1.0);
    let mut lamp_column = 0.0_f32;
    for [cx, radius] in [
        [0.535, 0.005],
        [0.563, 0.004],
        [0.600, 0.005],
        [0.661, 0.014],
        [0.779, 0.017],
        [0.843, 0.007],
        [0.905, 0.008],
        [0.936, 0.016],
        [0.972, 0.009],
    ] {
        let width = radius + depth * 0.010;
        lamp_column = lamp_column.max((-((x - cx) / width).powi(2)).exp());
    }
    let emission = lamp_column
        * smooth(0.06, 0.18, original[0] - original[1])
        * smooth(0.22, 0.65, original[0])
        * smooth(0.62, 0.38, original[2] / original[0].max(0.01));
    let brightness = original[0] * 0.2126 + original[1] * 0.7152 + original[2] * 0.0722;
    let blue_water = [
        original[0] * 0.13 + original[2] * 0.10 + 0.008,
        original[1] * 0.40 + original[2] * 0.12 + 0.018,
        original[2] * 0.64 + brightness * 0.18 + 0.045,
    ];
    mix(original, blue_water, river * nightfall * (1.0 - emission))
}

include!("shore_lights.rs");

fn late_light_level(index: usize, q: f32) -> f32 {
    let seed = hash(index as u32, 313);
    smooth(0.47 + seed * 0.12, 0.64 + seed * 0.13, q) * (1.0 - smooth(0.92, 1.0, q))
}

#[cfg(test)]
fn additional_shore_lights(
    color: Color,
    x: f32,
    y: f32,
    t: f32,
    river: f32,
    ripple_x: f32,
) -> Color {
    let levels: Vec<f32> = (0..LATE_LIGHTS.len())
        .map(|i| late_light_level(i, t / PERIOD))
        .collect();
    let indices: Vec<usize> = (0..LATE_LIGHTS.len()).collect();
    prepared_shore_lights(color, x, y, t, river, ripple_x, &levels, &indices)
}

fn prepared_shore_lights(
    mut color: Color,
    x: f32,
    y: f32,
    t: f32,
    river: f32,
    ripple_x: f32,
    levels: &[f32],
    indices: &[usize],
) -> Color {
    let q = t / PERIOD;
    if q <= 0.47 || y < 0.44 || x < 0.06 {
        return color;
    }
    let phase = TAU * q;
    let mut radiance = [0.0_f32; 3];
    for &i in indices {
        let [cx, cy, rx, ry, energy] = LATE_LIGHTS[i];
        if (x - cx).abs() > 0.055 {
            continue;
        }
        let seed = hash(i as u32, 313);
        let on = levels[i];
        if on <= 0.0 {
            continue;
        }
        if (x - cx).abs() < rx * 3.0 && (y - cy).abs() < ry * 3.0 {
            let u = (x - cx) / rx;
            let v = (y - cy) / ry;
            let core = (-u.powi(4) - v.powi(4)).exp();
            let halo = (-(u * u + v * v) * 0.38).exp() * 0.035;
            let strength = (core + halo) * energy * on;
            for (channel, warm) in [1.0, 0.69, 0.28].iter().enumerate() {
                radiance[channel] += warm * strength;
            }
        }
        if river > 0.0 && y > 0.61 {
            let depth = ((y - 0.61) / 0.39).clamp(0.0, 1.0);
            let center = cx - ripple_x * 0.55;
            let spread = 0.0035 + depth * 0.014;
            let across = (-((x - center) / spread).powi(2)).exp();
            let falloff = (-(y - 0.61) / 0.19).exp();
            let bands =
                (0.5 + 0.5 * (y * 195.0 - phase * 6.0 + seed * 7.0 + x * 45.0).sin()).powi(4);
            let strength = across * falloff * bands * energy * on * river * 0.23;
            for (channel, warm) in [1.0, 0.58, 0.19].iter().enumerate() {
                radiance[channel] += warm * strength;
            }
        }
    }
    let peak = radiance.iter().copied().fold(0.0_f32, f32::max);
    let gain = ((if river > 0.0 { 0.40 } else { 0.85 }) / peak.max(0.001)).min(1.0);
    for channel in 0..3 {
        color[channel] = (color[channel] + radiance[channel] * gain).min(1.0);
    }
    color
}

fn powerlines(x: f32, y: f32, t: f32) -> Color {
    let phase = TAU * t / PERIOD;
    let horizon = 0.655;
    let mut color = mix([0.77, 0.12, 0.095], [0.92, 0.19, 0.115], y / horizon);
    let haze = (-((y - 0.60) / 0.16).powi(2)).exp() * 0.025 * (phase.sin() * 0.5 + 0.5);
    color = mix(color, [0.94, 0.40, 0.18], haze);
    if y >= horizon {
        let depth = ((y - horizon) / (1.0 - horizon)).clamp(0.0, 1.0);
        let grass = noise(x * 240.0, y * 190.0) * 0.5 + noise(x * 70.0, y * 75.0) * 0.5;
        color = mix([0.025, 0.11, 0.07], [0.14, 0.28, 0.16], grass);
        let trail = (-((x - 0.53) / (0.013 + depth * 0.08)).powi(2)).exp();
        color = mix(color, [0.31, 0.32, 0.19], trail * 0.14);
    }
    let dark = [0.035, 0.07, 0.045];
    for i in (0..7).rev() {
        let s = 0.72_f32.powi(i);
        let foot = [0.49 + (1.0 - s) * 0.22, horizon + 0.11 * s];
        let top = [foot[0] + 0.026 * s, foot[1] - 0.68 * s];
        let pole = line(x, y, foot, top, 0.006 * s);
        color = mix(color, dark, pole);
        for level in [0.09, 0.23, 0.40] {
            let arm_y = top[1] + level * s;
            let arm_x = top[0] - level * 0.038 * s;
            color = mix(
                color,
                dark,
                line(
                    x,
                    y,
                    [arm_x - 0.04 * s, arm_y],
                    [arm_x + 0.04 * s, arm_y + 0.008 * s],
                    0.0023 * s,
                ),
            );
            for side in [-1.0, 1.0] {
                let wire_x = arm_x + side * 0.037 * s;
                color = mix(
                    color,
                    dark,
                    line(
                        x,
                        y,
                        [wire_x, arm_y],
                        [wire_x, arm_y + 0.016 * s],
                        0.002 * s,
                    ),
                );
            }
        }
    }
    for side in [-1.0, 1.0] {
        let wire_at = |i| {
            let s = 0.72_f32.powi(i);
            [
                0.49 + (1.0 - s) * 0.22 + 0.026 * s - 0.23 * 0.038 * s + side * 0.037 * s,
                horizon + 0.11 * s - 0.68 * s + 0.23 * s,
            ]
        };
        color = mix(
            color,
            dark,
            line(x, y, [0.35 + side * 0.045, -0.03], wire_at(0), 0.0026),
        );
        for i in 0..6 {
            color = mix(color, dark, line(x, y, wire_at(i), wire_at(i + 1), 0.0019));
        }
    }
    // A continuous departure and return, with a rest on the same wire.
    let flight = ((t - 3.0) / 14.0).clamp(0.0, 1.0);
    let lift = (PI * flight).sin();
    let bx = 0.51 + 0.54 * (TAU * flight).sin() * lift.sqrt();
    let by = 0.229 - lift * 0.18;
    let wing = if lift > 0.01 {
        (TAU * t * 1.8).sin() * 0.007
    } else {
        0.003
    };
    let bird = ellipse(x, y, [bx, by], [0.0045, 0.007])
        .max(line(x, y, [bx - 0.012, by - wing], [bx, by], 0.002))
        .max(line(x, y, [bx, by], [bx + 0.012, by - wing], 0.002));
    mix(color, dark, bird)
}

fn clouds(x: f32, y: f32, t: f32) -> f32 {
    // One faint high wisp crosses a small distance in twenty seconds. It
    // disperses before the loop closes; the sky remains overwhelmingly blue.
    let q = t / PERIOD;
    let cx = 0.54 + q * 0.14;
    let cy = 0.145;
    let envelope = smooth(0.0, 0.18, q) * (1.0 - smooth(0.78, 1.0, q));
    let dx = (x - cx) / 0.083;
    let dy = (y - cy) / 0.011;
    let shape = (-dx * dx - dy * dy).exp();
    let strands = 0.64 + 0.36 * noise((x - cx) * 65.0 + 20.0, y * 95.0 + 10.0);
    shape * strands * envelope * 0.27
}

fn shop_wall(u: f32, y: f32, sunlit: bool) -> Color {
    let row = (y * 22.0).floor();
    let brick_u = u * 4.0 + row * 0.5;
    let grain = noise(u * 80.0 + 100.0, y * 185.0);
    let stone = hash(brick_u.floor() as u32, row as u32);
    let base = if sunlit {
        [0.66, 0.57, 0.42]
    } else {
        [0.29, 0.34, 0.38]
    };
    let mut color = scale(base, 0.84 + 0.14 * stone + 0.12 * grain);
    let mortar = smooth(0.07, 0.025, (y * 22.0).fract()).max(smooth(0.08, 0.025, brick_u.fract()));
    color = mix(
        color,
        if sunlit {
            [0.43, 0.42, 0.36]
        } else {
            [0.17, 0.23, 0.28]
        },
        mortar * 0.60,
    );
    // Recessed shop glass, jambs, a transom and two panes. All material noise
    // is anchored to the wall; no brick or reflection swims with the sky.
    let outer = rect(u, y, [0.16, 0.17, 0.86, 0.74]);
    color = mix(color, [0.15, 0.19, 0.20], outer);
    let glass = rect(u, y, [0.22, 0.20, 0.80, 0.69]);
    let reflection = 0.5 + 0.5 * (u * 5.0 + y * 2.0).sin();
    let pane = mix([0.065, 0.16, 0.24], [0.24, 0.39, 0.51], reflection * 0.67);
    color = mix(color, pane, glass);
    let mullion = rect(u, y, [0.47, 0.19, 0.53, 0.70]).max(rect(u, y, [0.20, 0.455, 0.82, 0.472]));
    color = mix(color, [0.11, 0.17, 0.19], mullion);
    let edge = smooth(0.90, 0.96, u);
    mix(
        color,
        if sunlit {
            [0.81, 0.74, 0.56]
        } else {
            [0.47, 0.54, 0.57]
        },
        edge * 0.75,
    )
}

fn private_sky(x: f32, y: f32, t: f32) -> Color {
    let mut color = mix([0.09, 0.37, 0.79], [0.39, 0.68, 0.90], smooth(0.0, 0.85, y));
    color = mix(color, [0.88, 0.94, 0.96], clouds(x, y, t));
    let left = 0.20 - y * 0.15;
    let right = 0.83 + y * 0.12;
    if x < left {
        color = shop_wall(x / left, y, false);
    }
    if x > right {
        color = shop_wall((1.0 - x) / (1.0 - right), y, true);
    }
    if y > 0.78 {
        let depth = (y - 0.78) / 0.22;
        let distance = 1.0 / (0.28 + depth);
        let u = (x - 0.52) * distance * 7.0;
        let v = distance * 4.0;
        let grain = noise(x * 170.0, y * 130.0);
        color = mix(
            [0.20, 0.26, 0.27],
            [0.38, 0.40, 0.35],
            grain * 0.55 + depth * 0.30,
        );
        let seams =
            smooth(0.07, 0.015, u.fract().rem_euclid(1.0)).max(smooth(0.08, 0.02, v.fract()));
        color = mix(color, [0.12, 0.18, 0.19], seams * 0.70);
        let gate_shadow = smooth(
            0.17,
            0.06,
            ((x - depth * 0.13) * 20.0).fract().rem_euclid(1.0),
        );
        color = scale(color, 1.0 - gate_shadow * 0.28);
    }
    let gate = [0.055, 0.10, 0.12];
    for i in 0..19 {
        let gx = 0.065 + i as f32 * 0.048;
        let height = 0.63 - 0.016 * (i as f32 - 9.0).abs();
        let bar = line(x, y, [gx, 1.0], [gx, height], 0.008);
        if bar > 0.0 {
            let patina = smooth(0.62, 0.86, noise(x * 290.0 + 30.0, y * 57.0 + i as f32));
            color = mix(color, mix(gate, [0.30, 0.21, 0.12], patina * 0.65), bar);
            let edge = line(x, y, [gx + 0.0025, 1.0], [gx + 0.0025, height], 0.0015);
            color = mix(color, [0.24, 0.33, 0.36], edge * 0.45);
        }
        color = mix(
            color,
            gate,
            polygon(
                x,
                y,
                &[
                    [gx - 0.006, height + 0.01],
                    [gx, height - 0.018],
                    [gx + 0.006, height + 0.01],
                ],
            ),
        );
        color = mix(
            color,
            [0.28, 0.33, 0.33],
            rect(
                x,
                y,
                [gx - 0.006, height + 0.033, gx + 0.006, height + 0.044],
            ) * 0.8,
        );
    }
    for gy in [0.68, 0.88] {
        color = mix(color, gate, line(x, y, [0.04, gy], [0.96, gy], 0.014));
        color = mix(
            color,
            [0.25, 0.34, 0.37],
            line(x, y, [0.04, gy - 0.005], [0.96, gy - 0.005], 0.0025) * 0.6,
        );
    }
    for post in [0.477, 0.506] {
        color = mix(
            color,
            [0.10, 0.16, 0.18],
            rect(x, y, [post, 0.606, post + 0.013, 1.0]),
        );
    }
    // A readable shackle, brass lock body and keyhole survive low sampling.
    let shackle = ellipse(x, y, [0.500, 0.766], [0.014, 0.026])
        * (1.0 - ellipse(x, y, [0.500, 0.767], [0.008, 0.019]));
    color = mix(color, [0.51, 0.57, 0.56], shackle);
    let lock = rect(x, y, [0.485, 0.776, 0.515, 0.812]);
    color = mix(
        color,
        mix([0.49, 0.32, 0.11], [0.83, 0.64, 0.27], (x - 0.485) / 0.03),
        lock,
    );
    mix(
        color,
        [0.065, 0.11, 0.12],
        ellipse(x, y, [0.500, 0.795], [0.0035, 0.007]),
    )
}

fn night_light(x: f32, y: f32, t: f32) -> Color {
    let warm = [0.98, 0.64, 0.25];
    let phase = t / PERIOD;
    let on = smooth(0.06, 0.25, phase) * (1.0 - smooth(0.62, 0.88, phase));
    let mut color = mix([0.045, 0.10, 0.22], [0.13, 0.22, 0.35], y);
    let lamp = [0.31, 0.55];
    let glow = (-((x - lamp[0]) / 0.31).powi(2) - ((y - lamp[1]) / 0.45).powi(2)).exp();
    color = mix(color, warm, glow * on * 0.38);
    // Window, door, furniture and their outlines stay in fixed coordinates.
    color = mix(
        color,
        [0.055, 0.09, 0.15],
        rect(x, y, [0.68, 0.16, 0.91, 0.65]),
    );
    color = mix(
        color,
        [0.10, 0.29, 0.51],
        rect(x, y, [0.692, 0.179, 0.898, 0.634]),
    );
    color = mix(
        color,
        [0.72, 0.83, 0.88],
        ellipse(x, y, [0.84, 0.265], [0.027, 0.046]),
    );
    color = mix(
        color,
        [0.055, 0.09, 0.15],
        line(x, y, [0.79, 0.18], [0.79, 0.64], 0.008),
    );
    color = mix(
        color,
        [0.055, 0.09, 0.15],
        line(x, y, [0.69, 0.44], [0.90, 0.44], 0.009),
    );
    if y > 0.77 {
        let pool = (-((x - 0.32) / 0.35).powi(2) - ((y - 0.84) / 0.16).powi(2)).exp();
        color = mix([0.09, 0.115, 0.17], [0.50, 0.31, 0.15], pool * on);
        let board = (x * 15.0 + y * 6.0).fract();
        if board < 0.025 {
            color = scale(color, 0.72);
        }
    }
    color = mix(
        color,
        [0.045, 0.07, 0.12],
        rect(x, y, [0.08, 0.19, 0.19, 0.78]),
    );
    color = mix(
        color,
        [0.69, 0.42, 0.19],
        rect(x, y, [0.185, 0.20, 0.19, 0.77]) * on * 0.7,
    );
    color = mix(
        color,
        [0.12, 0.09, 0.09],
        rect(x, y, [0.235, 0.678, 0.40, 0.705]),
    );
    for leg in [0.25, 0.38] {
        color = mix(
            color,
            [0.07, 0.07, 0.09],
            rect(x, y, [leg, 0.70, leg + 0.012, 0.91]),
        );
    }
    color = mix(
        color,
        [0.28, 0.22, 0.17],
        line(x, y, [0.31, 0.55], [0.31, 0.68], 0.008),
    );
    let shade = polygon(
        x,
        y,
        &[[0.269, 0.49], [0.352, 0.49], [0.376, 0.585], [0.245, 0.585]],
    );
    color = mix(color, mix([0.25, 0.27, 0.29], [1.0, 0.79, 0.42], on), shade);
    mix(
        color,
        [1.0, 0.87, 0.60],
        ellipse(x, y, [0.31, 0.584], [0.045, 0.008]) * on,
    )
}

#[cfg(test)]
mod motion_regressions {
    use super::*;

    #[test]
    fn shop_dust_falls_then_rests_on_its_surface() {
        let mut scene = Scene::new(3);
        for index in 0..330 {
            let offset = hash(index as u32, 12);
            let mut last_y = 0.0;
            let mut resting = None;
            for age in [0.08, 0.2, 0.4, 0.6, 0.75, 0.81, 0.88, 0.92] {
                let time = (age - offset).rem_euclid(1.0) * PERIOD;
                scene.prepare_frame(time);
                let particle = &scene.dust[index];
                assert!(
                    particle.center[1] + 1e-6 >= last_y,
                    "Dust {index} moved upwards during its visible life"
                );
                last_y = particle.center[1];
                if age >= 0.81 {
                    if let Some(position) = resting {
                        assert_eq!(particle.center, position, "Settled dust must stay still");
                    }
                    resting = Some(particle.center);
                }
            }
        }
    }

    #[test]
    fn original_waterfront_lamps_never_start_switched_off() {
        for original in [[1.0, 0.55, 0.12], [0.72, 0.42, 0.08]] {
            for nightfall in [0.0, 0.3, 0.7, 1.0] {
                assert_eq!(
                    evening_light(original, 0.779, 0.541, nightfall, 0.0),
                    original
                );
                assert_eq!(
                    evening_light(original, 0.779, 0.72, nightfall, 1.0),
                    original
                );
            }
        }
    }

    #[test]
    fn sunset_reflection_fades_but_lamp_reflection_stays_warm() {
        for sunset in [[0.8, 0.4, 0.28], [0.7, 0.32, 0.45]] {
            let late = evening_light(sunset, 0.30, 0.72, 1.0, 1.0);
            assert!(
                late[0] < late[1] && late[2] > late[0] * 1.7,
                "Sunset reflection must become blue water rather than survive as a false lamp"
            );
        }
        let lamp = [0.95, 0.52, 0.18];
        assert_eq!(evening_light(lamp, 0.779, 0.72, 1.0, 1.0), lamp);
    }

    #[test]
    fn later_lights_add_distinct_sources_without_replacing_the_opening() {
        let base = [0.04, 0.08, 0.15];
        assert!(
            LATE_LIGHTS.len() >= 66,
            "Add at least fifty sources beyond the former sixteen"
        );
        for &[x, y, _, _, _] in LATE_LIGHTS {
            assert_eq!(additional_shore_lights(base, x, y, 0.0, 0.0, 0.0), base);
            let late = additional_shore_lights(base, x, y, 16.0, 0.0, 0.0);
            assert!(
                late[0] > base[0] + 0.3,
                "A new window or shore light must appear"
            );
            assert_eq!(additional_shore_lights(base, x, y, PERIOD, 0.0, 0.0), base);
        }
        for [x, y] in [[0.661, 0.542], [0.779, 0.541], [0.936, 0.507]] {
            let lit = additional_shore_lights(base, x, y, 16.0, 0.0, 0.0);
            assert!(
                lit.iter().zip(base).all(|(a, b)| *a >= b),
                "Original lamps may only gain nearby spill light"
            );
        }
    }

    #[test]
    fn blue_hour_removes_the_sunset_and_preserves_foreground_geometry() {
        let scene = Scene::new(5);
        let layers = scene.twilight.as_ref().unwrap();
        let mut original_warmth = 0.0_f32;
        let mut late_warmth = 0.0_f32;
        let mut covered = 0;
        for row in 65..110 {
            for col in 30..190 {
                let x = (col as f32 + 0.5) / WIDTH as f32;
                let y = (row as f32 + 0.5) / HEIGHT as f32;
                if sample_mask(&layers.sky, x, y) < 0.99 {
                    continue;
                }
                let first = scene.twilight_sky(x, y, 0.0);
                let late = scene.twilight_sky(x, y, 0.8);
                original_warmth += (first[0] - first[1]).max(0.0);
                late_warmth += (late[0] - late[1]).max(0.0);
                covered += 1;
            }
        }
        assert!(
            covered > 1000,
            "Sky matte must cover the broad sunset field"
        );
        assert!(
            original_warmth > 20.0,
            "Opening still contains a warm sunset"
        );
        assert!(
            late_warmth < original_warmth * 0.05,
            "Warm sunset must actually disappear at blue hour"
        );
        for [x, y] in [[0.02, 0.30], [0.10, 0.73], [0.14, 0.86], [0.98, 0.50]] {
            let original = sample(&scene.image, x, y);
            assert_eq!(
                scene.twilight_sky(x, y, 0.8),
                original,
                "Sky motion must not deform foreground or facades"
            );
        }
    }

    #[test]
    fn blue_hour_cools_bench_highlights_without_sky_cyan_patches() {
        let mut scene = Scene::new(5);
        scene.prepare_frame(16.0);
        let mut warm_bench_samples = 0;
        for row in 135..180 {
            for col in 20..75 {
                let x = (col as f32 + 0.5) / WIDTH as f32;
                let y = (row as f32 + 0.5) / HEIGHT as f32;
                let original = sample(&scene.image, x, y);
                if original[0] - original[1] > 0.045 && original[0] > 0.12 {
                    let late = scene.color(x, y, 16.0);
                    assert!(
                        late[0] < late[1],
                        "Old pink illumination must not remain inside the protected bench footprint"
                    );
                    warm_bench_samples += 1;
                }
            }
        }
        assert!(warm_bench_samples > 20);
        let sky = &scene.twilight.as_ref().unwrap().sky;
        for q in [0.05, 0.15, 0.25, 0.4, 0.6, 0.8] {
            for row in 70..115 {
                for col in 15..75 {
                    let x = (col as f32 + 0.5) / WIDTH as f32;
                    let y = (row as f32 + 0.5) / HEIGHT as f32;
                    if sample_mask(sky, x, y) < 0.99 {
                        continue;
                    }
                    let color = scene.twilight_sky(x, y, q);
                    assert!(
                        !(color[1] > color[0] + 0.04 && color[1] > color[2] - 0.025),
                        "Revealed sky must not become cyan beside the foreground branches"
                    );
                }
            }
        }
    }
}
