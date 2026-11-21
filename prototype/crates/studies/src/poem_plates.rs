//! Original photographic plates with time-dependent lighting and sprite layers.
use super::{plate_pixel, rgba};
use crate::scene::{
    Color, DANCING_CHILDREN, FEATHER, MANIFESTO, WHITE_TREE, ellipse, gain, hash, line, luminance,
    mix, noise, smooth,
};
use std::f32::consts::TAU;

#[path = "white_tree.rs"]
mod white_tree;

pub(super) enum Poem {
    Manifesto(f32),
    WhiteTree(white_tree::Tree),
    DancingChildren(Dance),
    Feather(Feather),
    BlackPaint(f32),
}

impl Poem {
    pub(super) fn new(kind: u8, width: usize, height: usize) -> (&'static [u8], Self) {
        match kind {
            MANIFESTO => (
                include_bytes!(concat!(env!("OUT_DIR"), "/manifesto-crt.rgb")),
                Self::Manifesto(0.),
            ),
            WHITE_TREE => (
                white_tree::TRUNK_PLATE,
                Self::WhiteTree(white_tree::Tree::new(width, height)),
            ),
            DANCING_CHILDREN => (
                include_bytes!(concat!(env!("OUT_DIR"), "/dancing-facade.rgb")),
                Self::DancingChildren(Dance::at(0.)),
            ),
            FEATHER => (
                include_bytes!(concat!(env!("OUT_DIR"), "/feather-ground.rgb")),
                Self::Feather(Feather::at(0.)),
            ),
            _ => (
                include_bytes!(concat!(env!("OUT_DIR"), "/black-paint-canvas.rgb")),
                Self::BlackPaint(0.),
            ),
        }
    }

    pub(super) fn update(&mut self, seconds: f32) {
        match self {
            Self::Manifesto(time) | Self::BlackPaint(time) => *time = seconds,
            Self::WhiteTree(tree) => tree.update(seconds),
            Self::DancingChildren(frame) => *frame = Dance::at(seconds),
            Self::Feather(frame) => *frame = Feather::at(seconds),
        }
    }

    pub(super) fn shade(&self, base: Color, x: f32, y: f32) -> Color {
        match self {
            Self::Manifesto(time) => manifesto(base, x, y, *time),
            Self::WhiteTree(tree) => tree.shade(base, x, y),
            Self::DancingChildren(frame) => frame.shade(base, x, y),
            Self::Feather(frame) => frame.shade(base, x, y),
            Self::BlackPaint(time) => black_paint(base, x, y, *time),
        }
    }
}

fn manifesto(base: Color, x: f32, y: f32, time: f32) -> Color {
    let t = time.rem_euclid(22.);
    let ignition = smooth(6.3, 9.5, t) * (1. - smooth(16.1, 21.0, t));
    let tremble = smooth(2., 5.0, t) * (1. - ignition);
    let u = ((x - 0.499).abs() - 0.166).max(0.) / 0.030;
    let v = ((y - 0.431).abs() - 0.202).max(0.) / 0.040;
    let glass = 1. - smooth(0.89, 1.04, u * u + v * v);
    if glass == 0. {
        // The display lights nearby parts of the casing and the adjoining wall.
        let wash =
            ellipse(x, y, [0.689, 0.456], [0.186, 0.323]) * (0.38 + 0.62 * smooth(0.64, 0.76, x));
        return mix(base, [0.89, 0.87, 0.73], wash * ignition * 0.20);
    }

    let scan = 0.5 + 0.5 * (y * 258. - t * 5.5).cos();
    let ink = hash(
        (x * 160.).floor() as i32,
        (y * 122. - t * 3.).floor() as i32,
    );
    let speck = smooth(0.905 - 0.08 * tremble, 0.993, ink);
    let floor = [0.11, 0.133, 0.14];
    let dark = mix(base, floor, glass * (0.18 + 0.38 * tremble));
    let mut color = mix(
        dark,
        [0.73, 0.81, 0.79],
        glass * speck * (0.18 + 0.48 * tremble),
    );

    for (row, words) in [
        b"I WANT".as_slice(),
        b"TO SEE".as_slice(),
        b"THE WORLD".as_slice(),
    ]
    .into_iter()
    .enumerate()
    {
        let row = row as f32;
        let slip = tremble * (t * 5.2 - row * 1.41).sin();
        let by = 0.241 + 0.118 * row + 0.022 * slip;
        let bx = 0.500 - words.len() as f32 * 0.018 + 0.022 * slip;
        let mark = type_bits(words, (x - bx) / 0.009, (y - by) / 0.0165);
        let broken = 1. - 0.66 * tremble * smooth(0.47, 1., ink);
        color = mix(
            color,
            [0.92, 0.91, 0.81],
            glass * mark * broken * (1. - 0.95 * ignition),
        );
    }

    let height = (0.601 - y) / 0.312;
    if (0. ..1.18).contains(&height) && (x - 0.5).abs() < 0.14 {
        let bend = (height * 6.9 + time * 2.54).sin() * height * 0.036;
        let flutter = noise(time * 3.25, height * 7.0) * 0.026;
        let tip = 0.90 + flutter * 4.;
        let body = 0.051
            * (1. - smooth(0.23, tip, height))
            * (0.68 + 0.32 * (height * 4.7 + 0.58).sin().max(0.));
        let wave = (height * 28. - time * 8.).sin() * flutter * height * 0.42;
        let mid = 0.496 + bend + wave;
        let edge = ((x - mid).abs() - body) + 0.008 * (height * 8. - time * 4.).sin();
        let main = (1. - smooth(-0.007, 0.011, edge))
            * (1. - smooth(tip - 0.09, tip + 0.02, height))
            * smooth(0., 0.08, height);
        let split = mid - 0.055 * smooth(0.09, 0.51, height);
        let split_tip = 0.573 + 0.084 * (time * 2.72 + 0.52).sin();
        let split_width = 0.027 * (1. - smooth(0.19, split_tip, height));
        let side = (1. - smooth(-0.006, 0.01, (x - split).abs() - split_width))
            * (1. - smooth(split_tip - 0.10, split_tip + 0.012, height))
            * smooth(0.03, 0.22, height);
        let flame = main.max(side * 0.83);
        let core =
            (1. - smooth(0.008, 0.031, (x - mid).abs())) * (0.7 + 0.3 * (height * 7.8).sin());
        color = mix(color, [0.37, 0.62, 0.74], glass * flame * ignition);
        color = mix(
            color,
            [1.24, 1.21, 1.12],
            glass * flame * ignition * (0.26 + 0.63 * core),
        );
    }
    gain(color, 0.94 + 0.06 * scan * glass)
}

fn type_bits(words: &[u8], x: f32, y: f32) -> f32 {
    if x < 0. || y < 0. || x >= (words.len() * 4) as f32 || y >= 5. {
        return 0.;
    }
    let col = x as usize;
    let column = col % 4;
    if column == 3 {
        return 0.;
    }
    let glyph = match words[col / 4] {
        b'A' => [2, 5, 7, 5, 5],
        b'D' => [6, 5, 5, 5, 6],
        b'E' => [7, 4, 6, 4, 7],
        b'H' => [5, 5, 7, 5, 5],
        b'I' => [7, 2, 2, 2, 7],
        b'L' => [4, 4, 4, 4, 7],
        b'N' => [5, 7, 7, 7, 5],
        b'O' => [7, 5, 5, 5, 7],
        b'R' => [6, 5, 6, 5, 5],
        b'S' => [7, 4, 7, 1, 7],
        b'T' => [7, 2, 2, 2, 2],
        b'W' => [5, 5, 7, 7, 5],
        _ => return 0.,
    };
    if glyph[y as usize] & (4 >> column) == 0 {
        0.
    } else {
        1.
    }
}

const MURAL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/dancing-mural.rgb"));

fn mural_coordinates(x: f32, y: f32) -> [f32; 2] {
    // Invert the projection of the tile plane under the canopy. Its inset
    // corners in the cropped image are (.153,.388), (.805,.258),
    // (.805,.803), and (.148,.789), clockwise from the top left.
    let a = 0.439460 + 0.264025 * x;
    let b = -0.006126 + 0.007610 * x;
    let c = -0.198118 + 0.264025 * y;
    let d = 0.394995 + 0.007610 * y;
    let (x, y) = (x - 0.153, y - 0.388);
    let det = a * d - b * c;
    [(d * x - b * y) / det, (a * y - c * x) / det]
}

pub(super) fn mural_on_facade(tiles: Color, x: f32, y: f32) -> Color {
    let [u, v] = mural_coordinates(x, y);
    if !(0. ..1.).contains(&u) || !(0. ..1.).contains(&v) {
        return tiles;
    }
    let paint = plate_pixel(MURAL, 0.012 + 0.978 * u, 0.041 + 0.863 * v);
    let high = paint[0].max(paint[1]).max(paint[2]);
    let low = paint[0].min(paint[1]).min(paint[2]);
    let vivid = smooth(0.235, 0.38, high - low);
    let contour = 1. - smooth(0.26, 0.56, luminance(paint));
    let coverage = vivid.max(contour) * smooth(0., 0.017, 1. - u.max(1. - u).max(v.max(1. - v)));
    mix(tiles, gain(paint, 0.83 + 0.25 * luminance(tiles)), coverage)
}

pub(super) struct Dance {
    reflection: f32,
}

impl Dance {
    fn at(seconds: f32) -> Self {
        let phase = seconds * TAU / 20.;
        Self {
            reflection: 0.19 + 0.62 * (0.5 - 0.5 * phase.cos()),
        }
    }

    fn shade(&self, base: Color, x: f32, y: f32) -> Color {
        let [x, y] = mural_coordinates(x, y);
        if !(0. ..1.).contains(&x) || !(0. ..1.).contains(&y) {
            return base;
        }
        let edge = smooth(0., 0.15, 1. - x.max(1. - x).max(y.max(1. - y)));
        let distance = (x - self.reflection + 0.21 * (y - 0.52)).abs();
        let daylight = edge * (1. - smooth(0.03, 0.52, distance));
        let gloss = daylight * smooth(0.37, 0.88, luminance(base));
        let brighter = gain(base, 1. + daylight * 0.084);
        mix(brighter, [1.12, 1.04, 0.88], gloss * 0.095)
    }
}

const FEATHER_PLATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/feather-cutout.rgba"));
const FEATHER_WIDTH: f32 = 0.570;
const FEATHER_HEIGHT: f32 = 0.289;
const QUILL: [f32; 2] = [0.97, 0.951];
const CONTACT: [f32; 2] = [0.746, 0.785];
const LANDING: f32 = 4.30;
const LANDING_ANGLE: f32 = 0.115;

fn fall_distance(t: f32) -> f32 {
    // Integrate the downward velocity with linear drag. It rises from
    // rest and approaches terminal speed before the feather hits the ground.
    let drag_time = 0.69;
    t - drag_time * (1. - (-t / drag_time).exp())
}

fn quill_offset(angle: f32) -> [f32; 2] {
    let (sin, cos) = angle.sin_cos();
    let x = (QUILL[0] - 0.5) * FEATHER_WIDTH;
    let y = (QUILL[1] - 0.5) * FEATHER_HEIGHT;
    [(cos * x - sin * y) / 1.7, sin * x + cos * y]
}

pub(super) struct Feather {
    t: f32,
    center: [f32; 2],
    sin: f32,
    cos: f32,
    settle: f32,
    damp: f32,
    dark: f32,
    opacity: f32,
    rim: f32,
}

impl Feather {
    fn at(seconds: f32) -> Self {
        let t = seconds.rem_euclid(13.);
        let fall = fall_distance(t.min(LANDING)) / fall_distance(LANDING);
        let (center, angle) = if t < LANDING {
            let offset = quill_offset(LANDING_ANGLE);
            let landing = [CONTACT[0] - offset[0], CONTACT[1] - offset[1]];
            let air = (1. - smooth(0.65, 1., fall)) * smooth(0.0, 0.23, fall);
            let yaw = (t * 2.63 + 0.19).sin() * air;
            let center = [
                0.395 + (landing[0] - 0.395) * fall + 0.049 * yaw,
                0.117 + (landing[1] - 0.117) * fall,
            ];
            (center, LANDING_ANGLE + 0.38 * yaw)
        } else {
            // Land on the quill, then let the vane come down around that
            // point. The footprint does not slide while the feather settles.
            let hinge = smooth(LANDING, LANDING + 0.62, t);
            let angle = LANDING_ANGLE - 0.50 * hinge;
            let offset = quill_offset(angle);
            ([CONTACT[0] - offset[0], CONTACT[1] - offset[1]], angle)
        };
        let (sin, cos) = angle.sin_cos();
        Self {
            t,
            center,
            sin,
            cos,
            settle: smooth(3.4, 4.95, t),
            damp: smooth(4.51, 8.61, t),
            dark: smooth(7.35, 10.64, t),
            opacity: smooth(0., 0.32, t) * (1. - smooth(11.43, 12.66, t)),
            rim: smooth(9.55, 10.56, t) * (1. - smooth(11.0, 12.0, t)),
        }
    }

    fn shade(&self, base: Color, x: f32, y: f32) -> Color {
        let puddle = smooth(0.021, 0.095, base[2] - base[0]) * smooth(0.55, 0.69, y);
        let ripple = ((x * 64. + y * 128. - self.t * TAU * 0.86).sin())
            .max(0.)
            .powf(4.);
        let mut color = mix(base, [0.62, 0.74, 0.78], ripple * puddle * 0.13);
        let shadow = ellipse(x, y, [self.center[0] + 0.029, 0.777], [0.18, 0.072]);
        color = gain(
            color,
            1. - shadow * self.opacity * (0.06 + 0.28 * self.settle),
        );
        if (x - self.center[0]).abs() > 0.239 || (y - self.center[1]).abs() > 0.259 {
            return color;
        }
        let px = (x - self.center[0]) * 1.7;
        let py = y - self.center[1];
        let u = (self.cos * px + self.sin * py) / FEATHER_WIDTH + 0.5;
        let v = (-self.sin * px + self.cos * py) / FEATHER_HEIGHT + 0.5;
        let Some((feather, alpha)) = rgba(FEATHER_PLATE, 128, 76, u, v) else {
            return color;
        };
        let shaft = line(u, v, [0.093, 0.205], [0.51, 0.602], 0.013).max(line(
            u,
            v,
            [0.51, 0.602],
            [0.97, 0.951],
            0.016,
        ));
        let clean = mix(
            mix(feather, [1.08, 1.05, 0.97], 0.27),
            [0.70, 0.64, 0.52],
            shaft * 0.53,
        );
        let contact = (1. - u) * 0.75 + (1. - v) * 0.25 + noise(u * 21., v * 17.) * 0.10;
        let damp =
            smooth(contact - 0.08, contact + 0.087, self.damp * 0.86) * smooth(0., 0.12, self.damp);
        let dark =
            smooth(contact - 0.11, contact + 0.047, self.dark * 0.75) * smooth(0., 0.17, self.dark);
        let ochre = mix([0.70, 0.46, 0.23], clean, 0.22 + 0.12 * feather[0]);
        let mud = mix([0.18, 0.13, 0.11], ochre, 0.36 * feather[0]);
        let stained = mix(mix(clean, ochre, damp), mud, damp * dark * 0.87);
        let edge = smooth(0.21, 0.66, alpha) * (1. - smooth(0.70, 1.00, alpha));
        let pearl = mix(
            [0.80, 0.83, 1.0],
            [1.13, 0.91, 0.72],
            0.5 + 0.5 * (u * 9. + v * 2.).sin(),
        );
        let lit = mix(stained, pearl, self.rim * edge * (1. - 0.79 * dark));
        mix(color, lit, alpha * self.opacity)
    }
}

fn black_paint(base: Color, x: f32, y: f32, time: f32) -> Color {
    let t = time.rem_euclid(17.2);
    let onset = smooth(0., 0.86, t) * (1. - smooth(12., 16.6, t));
    let center = 0.403 + 0.365 * smooth(0.04, 10.4, t);
    let distance = (x - center + (y - 0.50) * 0.11).abs();
    let beam = (1. - smooth(0.050, 0.22, distance)) * onset;
    let mask =
        ellipse(x, y, [0.556, 0.438], [0.182, 0.445]) * (1. - smooth(0.45, 0.63, luminance(base)));
    let b = luminance(base);
    let muted = mix(base, [b * 0.89, b * 0.87, b * 0.85], mask * 0.65);
    let texture = mix(
        gain(base, 1.72 + 0.38 * beam),
        [0.75, 0.46, 0.25],
        0.18 * mask,
    );
    let mut color = mix(muted, texture, beam * (0.95 * mask + 0.08 * (1. - mask)));
    for (sx, sy, rx, ry) in [
        (0.686, 0.205, 0.017, 0.023),
        (0.695, 0.484, 0.018, 0.036),
        (0.661, 0.605, 0.012, 0.020),
    ] {
        let local = 1. - smooth(0.014, 0.088, (sx - center).abs());
        let plate = ellipse(x, y, [sx, sy], [rx, ry]);
        let glint = line(
            x,
            y,
            [sx - rx * 0.6, sy + ry * 0.44],
            [sx + rx * 0.6, sy - ry * 0.48],
            0.003,
        );
        color = mix(
            color,
            [1.11, 0.96, 0.68],
            (plate * 0.44 + glint * plate * 0.51) * local * onset,
        );
    }
    color
}

#[cfg(test)]
mod tests {
    use super::{CONTACT, Feather, LANDING, quill_offset};

    #[test]
    fn the_feather_descends_until_its_quill_touches() {
        let mut previous_y = Feather::at(0.).center[1];
        for frame in 1..=120 {
            let t = LANDING * frame as f32 / 120.;
            let y = Feather::at(t).center[1];
            assert!(y >= previous_y, "feather rose at {t}");
            previous_y = y;
        }
    }

    #[test]
    fn the_quill_stays_at_the_point_of_contact_while_the_vane_settles() {
        for offset in [0., 0.1, 0.3, 0.62, 3.] {
            let frame = Feather::at(LANDING + offset);
            let quill = quill_offset(frame.sin.atan2(frame.cos));
            for i in 0..2 {
                assert!((frame.center[i] + quill[i] - CONTACT[i]).abs() < 0.0001);
            }
        }
    }
}
