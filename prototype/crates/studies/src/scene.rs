#[path = "artwork.rs"]
mod artwork;
use std::f32::consts::PI;
const SHRINE_AXIS: f32 = 0.535;
const GATE_TOP: f32 = 0.430;
const GATE_HEIGHT: f32 = 0.52;
const SHRINE_FOOT: f32 = 0.755;
const SHRINE_HEIGHT: f32 = GATE_HEIGHT * 0.28;
const SHRINE_WIDTH: f32 = SHRINE_HEIGHT * 1.24;
const PATH_REAR: f32 = SHRINE_FOOT + 0.036;
const BRIDGE_FAR: [f32; 2] = [0.68, 0.625];
const BRIDGE_NEAR: [f32; 2] = [0.18, 1.025];
const BRIDGE_HOUSE: [f32; 4] = [0.82, 0.582, 0.15, 0.13];

fn bridge_station(depth: f32) -> (f32, f32, f32) {
    (
        BRIDGE_FAR[0] + (BRIDGE_NEAR[0] - BRIDGE_FAR[0]) * depth,
        BRIDGE_FAR[1] + (BRIDGE_NEAR[1] - BRIDGE_FAR[1]) * depth,
        0.012 + depth * 0.105,
    )
}

fn shrine_path_half(y: f32) -> f32 {
    let foot = GATE_TOP + GATE_HEIGHT;
    let inner_opening = GATE_HEIGHT * (0.36 * 0.91 - 0.026);
    let rear_half = 0.027;
    // Both road edges pass through the inside edges of the pillar feet. The
    // foreground width is derived from that constraint, never guessed apart.
    let slope = (inner_opening - rear_half) / (foot - PATH_REAR);
    rear_half + slope * (y - PATH_REAR).max(0.)
}
pub type Color = [f32; 3];
pub const NEUTRAL_INKS: [u32; 11] = [
    0x080808, 0x141414, 0x242424, 0x383838, 0x525252, 0x707070, 0x949494, 0xB8B8B8, 0xDCDCDC,
    0xF3F3F3, 0xFFFFFF,
];
pub fn luminance(c: Color) -> f32 {
    c[0] * 0.2126 + c[1] * 0.7152 + c[2] * 0.0722
}
#[derive(Clone, Copy, PartialEq)]
pub enum Material {
    Air,
    Solid,
    Leaves,
    Water,
    Floor,
    Window,
    Metal,
}
fn material_floor(material: Material) -> f32 {
    match material {
        Material::Air => 0.04,
        Material::Solid | Material::Metal => 0.02,
        Material::Leaves => 0.0,
        Material::Water => 0.10,
        Material::Floor => 0.04,
        Material::Window => 0.12,
    }
}
#[derive(Clone, Copy)]
pub struct Sample {
    pub color: Color,
    pub material: Material,
    pub floor: f32,
    pub opacity: f32,
}
impl Sample {
    pub fn new(color: Color, material: Material) -> Self {
        Self {
            color,
            material,
            floor: material_floor(material),
            opacity: 1.,
        }
    }
    fn coat(&mut self, color: Color, alpha: f32, material: Material) {
        self.color = mix(self.color, color, alpha);
        if alpha > 0.45 {
            self.material = material;
            self.floor = material_floor(material);
        }
    }
}
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0., 1.);
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}
fn gain(a: Color, t: f32) -> Color {
    a.map(|v| (v * t).max(0.))
}
fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let q = ((x - a) / (b - a)).clamp(0., 1.);
    q * q * (3. - 2. * q)
}
fn hash(x: i32, y: i32) -> f32 {
    let n = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263);
    let n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    (n ^ (n >> 16)) as f32 / u32::MAX as f32
}
fn noise(x: f32, y: f32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (u, v) = (
        smooth(0., 1., x.rem_euclid(1.)),
        smooth(0., 1., y.rem_euclid(1.)),
    );
    let a = hash(ix, iy) * (1. - u) + hash(ix + 1, iy) * u;
    let b = hash(ix, iy + 1) * (1. - u) + hash(ix + 1, iy + 1) * u;
    a * (1. - v) + b * v
}
fn rect(x: f32, y: f32, b: [f32; 4]) -> f32 {
    smooth(b[0] - 0.001, b[0] + 0.001, x)
        * smooth(b[2] + 0.001, b[2] - 0.001, x)
        * smooth(b[1] - 0.002, b[1] + 0.002, y)
        * smooth(b[3] + 0.002, b[3] - 0.002, y)
}
fn ellipse(x: f32, y: f32, c: [f32; 2], r: [f32; 2]) -> f32 {
    1. - smooth(
        0.9,
        1.1,
        ((x - c[0]) / r[0]).powi(2) + ((y - c[1]) / r[1]).powi(2),
    )
}
fn line(x: f32, y: f32, a: [f32; 2], b: [f32; 2], w: f32) -> f32 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let t = (((x - a[0]) * dx + (y - a[1]) * dy) / (dx * dx + dy * dy).max(0.00001)).clamp(0., 1.);
    let d = ((x - a[0] - dx * t).powi(2) + (y - a[1] - dy * t).powi(2)).sqrt();
    1. - smooth(w, w + 0.002, d)
}
fn polygon(x: f32, y: f32, v: &[[f32; 2]]) -> f32 {
    let mut inside = false;
    for (a, b) in v.iter().zip(v.iter().cycle().skip(1)) {
        if (a[1] > y) != (b[1] > y) && x < a[0] + (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]) {
            inside = !inside;
        }
    }
    if inside { 1. } else { 0. }
}
fn tint(hex: u32) -> Color {
    [
        (hex >> 16) as f32 / 255.,
        ((hex >> 8) & 255) as f32 / 255.,
        (hex & 255) as f32 / 255.,
    ]
}

pub struct Scene {
    pub cols: usize,
    pub rows: usize,
    pub palette: Vec<Color>,
    pub ground: Color,
    artwork: artwork::Artwork,
}
impl Scene {
    pub fn new(kind: u8, cols: usize, rows: usize) -> Self {
        let colors: &[u32] = match kind {
            0 => &[
                0x101824, 0x243249, 0x3A4C61, 0x5B6B7D, 0x8896A3, 0x302C24, 0x484C3F, 0x6B745B,
                0x58251C, 0x962B1D, 0xCD3922, 0xED5B2B, 0xEF9560, 0xFFC58C, 0xFFE4BD, 0xBD914E,
                0xECD3A0,
            ],
            1 => &[
                0x344952, 0x526E7B, 0x94ACB1, 0xC4D2CE, 0x2F3938, 0x6F4C2C, 0xAA7340, 0xE6B85F,
                0xFFE0A0, 0xFFF0CE, 0x6E221A, 0xBE3924, 0xED6030, 0xFFF0D2, 0xD6BC91, 0x9B8059,
                0x58452E, 0xE8DDC6, 0xBCA88A, 0x46633C,
            ],
            2 => &[
                0x16252C, 0x354B55, 0x657D87, 0xA6B8BF, 0xDAE1E2, 0x143D38, 0x21675A, 0x29987E,
                0x55BEA3, 0xA0D9C6, 0x594B38, 0x997C4C, 0xC5A36B, 0xE2CAA0, 0xF2E8D5,
            ],
            3 => &[
                0x101625, 0x202C46, 0x344B6C, 0x526E8C, 0x8199AD, 0xB9C7D0, 0xE3E9EC, 0xB4A68B,
                0xE1D6BD, 0xFFF0CE,
            ],
            4 => &[
                0x426E9B, 0x82AACC, 0xCDDDE5, 0xD6EEEE, 0x1E515B, 0x2E666F, 0x4A9EAA, 0x9AD0D0,
                0x4C311F, 0x8D4C24, 0xD49545, 0xF6CD80, 0xEEE9DB, 0x2E9BF0, 0x7BC6FF, 0xB9E2FF,
                0xF5A7C9, 0xFFCADB, 0xFFEBF2,
            ],
            _ => &[
                0x182635, 0x304B68, 0x4F7CA1, 0x80A8C4, 0xB9CFDC, 0xE2EAF0, 0x1D2920, 0x354735,
                0x58654D, 0x4D211A, 0x8D291D, 0xC52D1D, 0xED4A25, 0xFF9B50, 0xFFE1AE, 0xE6DEC9,
                0xB9AE96, 0x584338, 0x8B5C3B,
            ],
        };
        let palette = NEUTRAL_INKS
            .iter()
            .chain(colors)
            .copied()
            .map(tint)
            .collect::<Vec<_>>();
        Self {
            cols,
            rows,
            ground: palette[0],
            palette,
            artwork: artwork::Artwork::new(kind, cols, rows),
        }
    }
    pub fn prepare(&mut self, seconds: f32) {
        self.artwork.update(seconds);
    }
    pub fn sample(&self, x: usize, y: usize, _seconds: f32) -> Sample {
        self.artwork.sample(x, y)
    }
}
fn moon(c: Color, x: f32, y: f32, centre: [f32; 2], radius: f32, crescent: bool) -> Color {
    let disc = ellipse(x, y, centre, [radius, radius * 1.7]);
    let surface = 0.87 + 0.10 * noise(x * 280., y * 164.);
    let mut result = mix(c, [surface; 3], disc);
    if crescent {
        result = mix(
            result,
            c,
            ellipse(
                x,
                y,
                [centre[0] - radius * 0.4, centre[1] - radius * 0.28],
                [radius * 0.97, radius * 1.65],
            ),
        );
    }
    result
}
fn torii(s: &mut Sample, x: f32, y: f32, cx: f32, top: f32, height: f32) {
    let half = height * 0.36;
    let foot = top + height;
    for sign in [-1., 1.] {
        let up = [cx + sign * half * 0.78, top + height * 0.09];
        let down = [cx + sign * half * 0.91, foot];
        let post_end = [down[0], foot - height * 0.058];
        s.coat(
            [0.30, 0.14, 0.16],
            line(
                x,
                y,
                [up[0] + 0.006, up[1]],
                [post_end[0] + height * 0.009, post_end[1]],
                height * 0.036,
            ),
            Material::Solid,
        );
        s.coat(
            [0.91, 0.20, 0.09],
            line(x, y, up, post_end, height * 0.026),
            Material::Solid,
        );
        s.coat(
            [0.20; 3],
            rect(
                x,
                y,
                [
                    down[0] - height * 0.034,
                    foot - height * 0.054,
                    down[0] + height * 0.034,
                    foot,
                ],
            ),
            Material::Solid,
        );
        s.coat(
            [0.46; 3],
            line(
                x,
                y,
                [down[0] - height * 0.033, foot - height * 0.052],
                [down[0] + height * 0.033, foot - height * 0.052],
                height * 0.003,
            ),
            Material::Solid,
        );
    }
    // A swept dark cap over the vermilion lintel; no letter or symbol is required.
    let lift = ((x - cx) / half).powi(2) * height * 0.035;
    s.coat(
        [0.94, 0.24, 0.10],
        rect(
            x,
            y + lift,
            [
                cx - half * 1.25,
                top,
                cx + half * 1.25,
                top + height * 0.045,
            ],
        ),
        Material::Solid,
    );
    s.coat(
        [0.045, 0.045, 0.045],
        rect(
            x,
            y + lift,
            [
                cx - half * 1.32,
                top - height * 0.025,
                cx + half * 1.32,
                top,
            ],
        ),
        Material::Solid,
    );
    s.coat(
        [0.96, 0.29, 0.10],
        rect(
            x,
            y,
            [
                cx - half,
                top + height * 0.19,
                cx + half,
                top + height * 0.225,
            ],
        ),
        Material::Solid,
    );
    s.coat(
        [0.85, 0.21, 0.09],
        rect(
            x,
            y,
            [
                cx - height * 0.021,
                top + height * 0.04,
                cx + height * 0.021,
                top + height * 0.2,
            ],
        ),
        Material::Solid,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stone_approach_fits_between_the_gate_pillar_feet() {
        let opening = GATE_HEIGHT * (0.36 * 0.91 - 0.026);
        assert!((shrine_path_half(GATE_TOP + GATE_HEIGHT) - opening).abs() < 0.000001);
        assert!(shrine_path_half(1.) > shrine_path_half(PATH_REAR));
        assert!(
            PATH_REAR > SHRINE_FOOT + 0.034,
            "approach cuts through shrine steps"
        );
        assert!(
            GATE_HEIGHT * 0.36 * 2.64 > SHRINE_WIDTH * 1.20 * 2.,
            "distant hall dominates near gate"
        );
    }
    #[test]
    fn gate_posts_stop_at_their_stone_bases() {
        let foot = GATE_TOP + GATE_HEIGHT;
        for sign in [-1., 1.] {
            let cx = SHRINE_AXIS + sign * GATE_HEIGHT * 0.36 * 0.91;
            for dx in [-0.02, 0., 0.02] {
                for below in [0.006, 0.015, 0.03] {
                    let mut sample = Sample::new([0.60; 3], Material::Floor);
                    torii(
                        &mut sample,
                        cx + dx,
                        foot + below,
                        SHRINE_AXIS,
                        GATE_TOP,
                        GATE_HEIGHT,
                    );
                    assert_eq!(
                        sample.color, [0.60; 3],
                        "red stroke extends below the grounded base"
                    );
                }
            }
        }
    }
    #[test]
    fn bridge_lands_beside_the_house_with_a_clear_quay_between() {
        let (x, y, half) = bridge_station(0.);
        let house_left = BRIDGE_HOUSE[0] - BRIDGE_HOUSE[2] * 0.43;
        assert!(
            x + half < house_left - 0.02,
            "rail enters the house footprint"
        );
        assert!(y > BRIDGE_HOUSE[1] + 0.025, "deck lands inside the facade");
        assert!(y < 0.655, "landing misses the quay");
    }
    #[test]
    fn pool_stays_blue_with_local_pink_beads_and_unchanged_arcade() {
        let mut scene = Scene::new(4, 170, 100);
        let mut wall = Vec::new();
        for time in [0., 12., 24., 48., 72.] {
            scene.prepare(time);
            let (mut water, mut blue, mut pink) = (0, 0, 0);
            for y in 65..100 {
                for x in 0..170 {
                    let sample = scene.sample(x, y, time);
                    if sample.material == Material::Water {
                        water += 1;
                        blue += usize::from(sample.color[2] > sample.color[0] + 0.12);
                        pink += usize::from(
                            sample.color[0] > sample.color[2] + 0.035
                                && sample.color[2] > sample.color[1] + 0.08,
                        );
                    }
                }
            }
            assert!(
                blue * 100 > water * 85,
                "pool loses its blue material at {time}s"
            );
            assert!(pink > 4, "pink beads disappear at {time}s");
            assert!(
                pink * 100 < water * 4,
                "pink spreads across the pool at {time}s"
            );
            wall.push(scene.sample(150, 27, time).color);
        }
        assert!(
            wall.windows(2).all(|pair| pair[0] == pair[1]),
            "beads recolour the arcade"
        );
    }
    #[test]
    fn scene_time_is_continuous_beyond_the_review_window() {
        for kind in [0, 1, 3, 4, 5] {
            let mut s = Scene::new(kind, 102, 60);
            let mut fingerprints = Vec::new();
            for t in [2., 12., 25., 45., 65.] {
                s.prepare(t);
                let values = (0..60)
                    .flat_map(|y| (0..102).map(move |x| (x, y)))
                    .map(|(x, y)| s.sample(x, y, t).color)
                    .collect::<Vec<_>>();
                assert!(
                    values.iter().flatten().all(|c| c.is_finite() && *c >= 0.),
                    "scene {kind} invalid radiance"
                );
                fingerprints.push(values);
            }
            for pair in fingerprints.windows(2) {
                let changed = pair[0]
                    .iter()
                    .zip(&pair[1])
                    .filter(|(a, b)| a.iter().zip(b.iter()).any(|(a, b)| (a - b).abs() > 0.015))
                    .count();
                assert!(
                    changed > 80,
                    "scene {kind} stops evolving: only {changed} changed cells"
                );
            }
        }
    }
    #[test]
    fn stone_and_people_are_fixed_while_atmosphere_changes() {
        for (kind, points) in [
            (1, vec![(40, 49), (63, 46), (86, 48)]),
            (5, vec![(46, 86), (63, 86)]),
        ] {
            let mut s = Scene::new(kind, 102, 100);
            s.prepare(3.);
            let first = points
                .iter()
                .map(|&(x, y)| s.sample(x, y, 3.).color)
                .collect::<Vec<_>>();
            for t in [10., 27., 55.] {
                s.prepare(t);
                for (&(x, y), color) in points.iter().zip(&first) {
                    assert_eq!(s.sample(x, y, t).color, *color);
                }
            }
        }
    }
}
