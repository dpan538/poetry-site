//! The photographic plates supply recognisable places, surfaces and light.
//! All silhouettes, sprite transforms, water and steam are composed as
//! colour samples before the shared ASCII glyph encoder.
use super::super::{
    Color, Material, Sample, ellipse, gain, line, mix, noise, polygon, rect, smooth,
};

use crate::scene::plate_pixels::{plate_pixel, rgba};

#[path = "poem_plates.rs"]
mod poem_plates;

const SHADOW: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shadow-landing.rgb"));
const TABLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/thermos-table.rgb"));
const BOTTLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/thermos-bottle.rgba"));
const BOTTLE_W: usize = 80;
const BOTTLE_H: usize = 241;
const ASPECT: f32 = 1.7;
const MUG_MOUTH: [f32; 2] = [0.558, 0.575];
const BOTTLE_ANCHOR: [f32; 2] = [0.610, 0.620];
const POUR_LIP: [f32; 2] = [0.843, 0.052];
const BOTTLE_HEIGHT: f32 = 0.56;
const BOTTLE_WIDTH: f32 = 0.224;
const POUR_START: f32 = 7.72;
const POUR_END: f32 = 12.42;
const FALL_TIME: f32 = 0.32;

enum Motion {
    Shadow(Shadow),
    Thermos(Thermos),
    Poem(poem_plates::Poem),
}

pub(super) struct Plate {
    base: Vec<Color>,
    width: usize,
    height: usize,
    motion: Motion,
}

impl Plate {
    pub(super) fn shadow(width: usize, height: usize) -> Self {
        Self::new(SHADOW, width, height, Motion::Shadow(Shadow::at(0.)))
    }

    pub(super) fn thermos(width: usize, height: usize) -> Self {
        Self::new(TABLE, width, height, Motion::Thermos(Thermos::at(0.)))
    }

    pub(super) fn poem(kind: u8, width: usize, height: usize) -> Self {
        let (source, poem) = poem_plates::Poem::new(kind, width, height);
        Self::new(source, width, height, Motion::Poem(poem))
    }

    fn new(source: &[u8], width: usize, height: usize, motion: Motion) -> Self {
        let mural = matches!(motion, Motion::Poem(poem_plates::Poem::DancingChildren(_)));
        let mut base = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                let mut color = [0.; 3];
                for (ox, oy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                    let u = (x as f32 + ox) / width as f32;
                    let v = (y as f32 + oy) / height as f32;
                    let mut pixel = plate_pixel(source, u, v);
                    if mural {
                        pixel = poem_plates::mural_on_facade(pixel, u, v);
                    }
                    for i in 0..3 {
                        color[i] += pixel[i] * 0.25;
                    }
                }
                let u = (x as f32 + 0.5) / width as f32;
                let v = (y as f32 + 0.5) / height as f32;
                let mut color = color.map(|channel| channel.powf(0.88));
                if matches!(motion, Motion::Shadow(_)) {
                    let lift = (1. - smooth(0.10, 0.155, u))
                        * (1. - smooth(0.08, 0.28, color[0].max(color[1]).max(color[2])));
                    color = mix(color, [0.24, 0.26, 0.30], lift * 0.43);
                } else if matches!(motion, Motion::Thermos(_)) {
                    color = thermos_grade(color, u, v);
                }
                base.push(color);
            }
        }
        Self {
            base,
            width,
            height,
            motion,
        }
    }

    pub(super) fn update(&mut self, seconds: f32) {
        match &mut self.motion {
            Motion::Shadow(frame) => *frame = Shadow::at(seconds),
            Motion::Thermos(frame) => *frame = Thermos::at(seconds),
            Motion::Poem(frame) => frame.update(seconds),
        }
    }

    pub(super) fn sample(&self, x: usize, y: usize) -> Sample {
        let color = self.base[y * self.width + x];
        let u = (x as f32 + 0.5) / self.width as f32;
        let v = (y as f32 + 0.5) / self.height as f32;
        let radiance = match &self.motion {
            Motion::Shadow(frame) => frame.shade(color, u, v),
            Motion::Thermos(frame) => frame.shade(color, u, v, self.width, self.height),
            Motion::Poem(frame) => frame.shade(color, u, v),
        };
        let mut sample = Sample::new(radiance, Material::Solid);
        sample.floor = 0.035;
        sample
    }
}

fn thermos_grade(color: Color, x: f32, y: f32) -> Color {
    let [r, g, b] = color;
    let daylight =
        smooth(0.48, 0.61, x) * (1. - smooth(0.45, 0.56, y)) * smooth(0.035, 0.18, b - r);
    let lit: Color = std::array::from_fn(|i| color[i] + daylight * [0.012, 0.040, 0.061][i]);
    let red = rect(x, y, [0.484, 0.594, 0.682, 0.826])
        * smooth(0.07, 0.29, r - g)
        * (1. - smooth(0.62, 0.89, r));
    let sage = rect(x, y, [0.751, 0.585, 0.937, 0.822]) * smooth(0.05, 0.18, g - b);
    [
        lit[0] + red * 0.128 + sage * 0.048,
        lit[1] + red * 0.033 + sage * 0.105,
        lit[2] + red * 0.014 + sage * 0.029,
    ]
}

struct Shadow {
    near: f32,
    distant: f32,
    merge: f32,
    fade: f32,
    face_turn: f32,
}

impl Shadow {
    fn at(seconds: f32) -> Self {
        let t = (seconds * 1.18).rem_euclid(34.);
        let approach = smooth(2., 17.0, t);
        let merge = smooth(14., 20., t);
        Self {
            near: 0.487 + 0.146 * approach,
            distant: 0.851 - 0.218 * approach,
            merge,
            fade: smooth(0., 2.4, t) * (1. - smooth(27., 33., t)),
            face_turn: smooth(9., 16.5, t),
        }
    }

    fn shade(&self, color: Color, u: f32, v: f32) -> Color {
        // The door lining and rail are in front of the wall. At the skirting
        // the projection turns onto the floor and loses contrast.
        if !(0.346..0.986).contains(&u) || !(0.16..0.93).contains(&v) || self.fade <= 0. {
            return color;
        }
        let wall = smooth(0.343, 0.355, u);
        let edge = 1. - 0.62 * smooth(0.842, 0.877, v);
        let distant = soft_profile(u, v, self.distant, 0.278, self.face_turn) * (1. - self.merge);
        let near = soft_profile(u, v, self.near, 0.272, 1.);
        let transmittance = 1. - self.fade * wall * edge * (0.62 * near + 0.58 * distant).min(0.80);
        let lit = gain(color, transmittance);
        mix(
            lit,
            [lit[0] * 0.67, lit[1] * 0.85, lit[2] * 1.06],
            1. - transmittance,
        )
    }
}

fn soft_profile(x: f32, y: f32, center: f32, top: f32, turn: f32) -> f32 {
    [(-0.005, 0.), (0.005, 0.), (0., -0.007), (0., 0.007)]
        .into_iter()
        .map(|(dx, dy)| {
            let u = (x + dx - center) / 0.112;
            let v = (y + dy - top) / 0.31;
            let cranial = ellipse(u, v, [-0.15, 0.35], [0.49, 0.40]);
            let profile = mix([face(-u, v), 0., 0.], [face(u, v), 0., 0.], turn)[0];
            let neck = rect(u, v, [-0.39, 0.64, 0.19, 1.11]);
            let shoulders = ellipse(u, v, [-0.16, 1.17], [0.94, 0.43]);
            let torso = rect(u, v, [-0.99, 1.19, 0.66, 2.13]);
            cranial.max(profile).max(neck).max(shoulders).max(torso)
        })
        .sum::<f32>()
        * 0.25
}

fn face(x: f32, y: f32) -> f32 {
    polygon(
        x,
        y,
        &[
            [-0.34, 0.11],
            [0.23, 0.08],
            [0.39, 0.26],
            [0.40, 0.35],
            [0.59, 0.48],
            [0.42, 0.54],
            [0.43, 0.65],
            [0.24, 0.74],
            [-0.30, 0.62],
        ],
    )
}

struct Thermos {
    pivot: [f32; 2],
    sin: f32,
    cos: f32,
    mouth: [f32; 2],
    height: f32,
    time: f32,
    water: f32,
    warmth: f32,
    seconds: f32,
}

impl Thermos {
    fn at(seconds: f32) -> Self {
        let seconds = seconds * 1.30;
        let t = seconds.rem_euclid(32.);
        let lift = smooth(3.1, 6.9, t) * (1. - smooth(14.1, 17.6, t));
        let lean = smooth(4.4, 7.64, t) * (1. - smooth(12.87, 15.82, t));
        let (sin, cos) = (lean * 1.66).sin_cos();
        let pivot = [0.300 + 0.045 * lift, 0.610 - 0.191 * lift - 0.077 * lean];
        let x = (POUR_LIP[0] - BOTTLE_ANCHOR[0]) * BOTTLE_WIDTH;
        let y = (POUR_LIP[1] - BOTTLE_ANCHOR[1]) * BOTTLE_HEIGHT;
        Self {
            pivot,
            sin,
            cos,
            mouth: [
                pivot[0] + (cos * x - sin * y) / ASPECT,
                pivot[1] + sin * x + cos * y,
            ],
            height: lift,
            time: t,
            water: smooth(POUR_START + FALL_TIME, POUR_END + FALL_TIME, t)
                * (1. - smooth(30.5, 32., t)),
            warmth: smooth(8.45, 12.7, t) * (1. - smooth(26., 31., t)),
            seconds,
        }
    }

    fn shade(&self, base: Color, x: f32, y: f32, cols: usize, rows: usize) -> Color {
        let mut color = self.rain(base, x, y);
        let ground_shadow = ellipse(x, y, [0.310 + 0.026 * self.height, 0.828], [0.112, 0.047]);
        color = gain(color, 1. - ground_shadow * (0.43 - 0.23 * self.height));

        let height = MUG_MOUTH[1] - y;
        if (0.006..0.33).contains(&height) && (x - MUG_MOUTH[0]).abs() < 0.13 {
            let bend = (height * 17. - self.seconds * 1.05).sin();
            let center = MUG_MOUTH[0] + 0.045 * height + bend * (0.009 + 0.101 * height);
            let cloud = 1. - smooth(0.004, 0.023 + 0.101 * height, (x - center).abs());
            let plume = smooth(0.004, 0.061, height) * (1. - smooth(0.15, 0.33, height));
            let twist = noise(x * 85. + y * 11., y * 24. + self.seconds * 1.64);
            color = mix(
                color,
                [0.84, 0.90, 0.87],
                self.warmth * cloud * plume * (0.24 + 0.41 * twist),
            );
        }
        let surface = ellipse(x, y, [MUG_MOUTH[0], MUG_MOUTH[1] + 0.004], [0.051, 0.014]);
        if surface > 0. && self.water > 0. {
            let strike = pour_flow(self.time - FALL_TIME);
            let distance = ((x - MUG_MOUTH[0]).powi(2) + ((y - MUG_MOUTH[1]) * 2.7).powi(2)).sqrt();
            let ripple = 0.5 + 0.5 * (distance * 245. - self.seconds * 18.).cos();
            color = mix(color, [0.59, 0.39, 0.25], surface * self.water * 0.53);
            color = mix(
                color,
                [1.04, 0.76, 0.47],
                surface * strike * (0.18 + 0.36 * ripple),
            );
        }

        let bottle = self.bottle(color, x, y, cols, rows);
        self.water_stream(bottle, x, y, cols, rows)
    }

    fn water_stream(&self, color: Color, x: f32, y: f32, cols: usize, rows: usize) -> Color {
        if !(POUR_START..POUR_END + FALL_TIME).contains(&self.time)
            || !(self.mouth[1] - 0.015..MUG_MOUTH[1] + 0.013).contains(&y)
            || !(self.mouth[0] - 0.024..MUG_MOUTH[0] + 0.023).contains(&x)
        {
            return color;
        }
        let mut edge = 0.;
        let mut core = 0.;
        for (ox, oy) in [(-0.27, -0.27), (0.27, -0.27), (-0.27, 0.27), (0.27, 0.27)] {
            let x = x + ox / cols as f32;
            let y = y + oy / rows as f32;
            let along = (y - self.mouth[1]) / (MUG_MOUTH[1] - self.mouth[1]);
            if !(-0.035..=1.0).contains(&along) {
                continue;
            }
            let travel = along.max(0.).sqrt();
            let launched = self.time - travel * FALL_TIME;
            let flow = pour_flow(launched);
            let center = self.mouth[0] + (MUG_MOUTH[0] - self.mouth[0]) * travel;
            let wobble = 0.0014 * travel * (y * 104. - launched * 15.).sin();
            let from_center = (x - center - wobble).abs();
            let reach = 0.012 - 0.004 * along.max(0.);
            edge += flow * (1. - smooth(0.0035, reach, from_center));
            core += flow
                * (1. - smooth(0.001, reach * 0.56, from_center))
                * (0.78 + 0.22 * (y * 112. - launched * 19.).cos());
        }
        let tinted = mix(color, [0.57, 0.82, 0.91], edge * 0.21);
        let lip = ellipse(x, y, self.mouth, [0.010, 0.015]) * pour_flow(self.time);
        mix(tinted, [1.16, 1.07, 0.87], (core * 0.25).max(lip * 0.81))
    }

    fn rain(&self, base: Color, x: f32, y: f32) -> Color {
        if x < 0.51 || y > 0.525 || base[2] - base[0] < 0.1 {
            return base;
        }
        let mut amount: f32 = 0.;
        for i in 0..9 {
            let i = i as f32;
            let u = 0.529 + i * 0.052 + noise(i * 1.41, 17.) * 0.026;
            if (x - u).abs() > 0.010 {
                continue;
            }
            let v = (i * 0.171 + self.seconds * (0.065 + 0.048 * noise(i, 5.9))).fract() * 0.55;
            amount = amount.max(line(x, y, [u - 0.004, v - 0.044], [u, v], 0.002));
        }
        mix(base, [0.62, 0.83, 0.96], amount * 0.24)
    }

    fn bottle(&self, background: Color, x: f32, y: f32, cols: usize, rows: usize) -> Color {
        if (x - self.pivot[0]).abs() > 0.225 || (y - self.pivot[1]).abs() > 0.38 {
            return background;
        }
        let mut result = [0.; 3];
        for (ox, oy) in [(-0.26, -0.26), (0.26, -0.26), (-0.26, 0.26), (0.26, 0.26)] {
            let px = (x + ox / cols as f32 - self.pivot[0]) * ASPECT;
            let py = y + oy / rows as f32 - self.pivot[1];
            let u = (self.cos * px + self.sin * py) / BOTTLE_WIDTH + BOTTLE_ANCHOR[0];
            let v = (-self.sin * px + self.cos * py) / BOTTLE_HEIGHT + BOTTLE_ANCHOR[1];
            let sample = sprite_pixel(u, v, background);
            for i in 0..3 {
                result[i] += sample[i] * 0.25;
            }
        }
        result
    }
}

fn pour_flow(time: f32) -> f32 {
    smooth(POUR_START, POUR_START + 0.24, time) * (1. - smooth(POUR_END - 0.36, POUR_END, time))
}

fn sprite_pixel(u: f32, v: f32, background: Color) -> Color {
    let Some((rgb, alpha)) = rgba(BOTTLE, BOTTLE_W, BOTTLE_H, u, v) else {
        return background;
    };
    let mut paint = rgb.map(|channel| channel.powf(0.83));
    if u > 0.24 && v > 0.127 {
        paint = mix(paint, [0.94, 0.93, 0.84], 0.21);
    }
    let rim = smooth(0.026, 0.044, v) * (1. - smooth(0.084, 0.105, v)) * smooth(0.34, 0.58, u);
    paint = mix(paint, [0.87, 0.87, 0.78], rim * 0.37);
    // The manuscript's simple price sticker shares the same bottle transform.
    if (0.46..0.78).contains(&u) && (0.55..0.63).contains(&v) {
        let printed = (0.582..0.600).contains(&v) && (0.50..0.69).contains(&u);
        paint = mix(
            paint,
            if printed {
                [0.42, 0.25, 0.16]
            } else {
                [0.81, 0.59, 0.34]
            },
            0.86,
        );
    }
    mix(background, paint, alpha)
}
