//! A procedural cube and two animated photographic plates.
#[path = "plates.rs"]
mod plates;

use super::{Color, Material, SHADOW, Sample, ellipse, gain, hash, mix, noise, smooth};
use std::f32::consts::TAU;

pub(super) struct Artwork(Study);

enum Study {
    Plate(plates::Plate),
    Cube(CubeFrame, usize, usize),
}

impl Artwork {
    pub(super) fn new(kind: u8, width: usize, height: usize) -> Self {
        Self(match kind {
            SHADOW => Study::Plate(plates::Plate::shadow(width, height)),
            super::CUBE => Study::Cube(CubeFrame::new(0.), width, height),
            super::THERMOS => Study::Plate(plates::Plate::thermos(width, height)),
            _ => Study::Plate(plates::Plate::poem(kind, width, height)),
        })
    }

    pub(super) fn update(&mut self, time: f32) {
        match &mut self.0 {
            Study::Plate(scene) => scene.update(time),
            Study::Cube(frame, _, _) => *frame = CubeFrame::new(time),
        }
    }

    pub(super) fn sample(&self, x: usize, y: usize) -> Sample {
        match &self.0 {
            Study::Plate(scene) => scene.sample(x, y),
            Study::Cube(frame, w, h) => {
                frame.shade((x as f32 + 0.5) / *w as f32, (y as f32 + 0.5) / *h as f32)
            }
        }
    }
}

fn marble(x: f32, y: f32) -> f32 {
    0.68 * noise(x * 19., y * 11.) + 0.32 * noise(x * 164., y * 117.)
}

fn solid(color: Color) -> Sample {
    let mut sample = Sample::new(color, Material::Solid);
    sample.floor = 0.09;
    sample
}

type Point3 = [f32; 3];

const CUBE_HALF: f32 = 0.482;
const CORNER_NORMAL: Point3 = [1., 1., -1.];
const CORNER_LIMIT: f32 = 3. * CUBE_HALF - 0.41;

struct CubeFrame {
    yaw: (f32, f32),
    pitch: (f32, f32),
    ray: Point3,
    key: Point3,
    fill: Point3,
    triangle: f32,
}

const PIGMENTS: [Color; 3] = [[0.27, 0.81, 0.83], [0.90, 0.62, 0.33], [0.71, 0.46, 0.86]];

fn inverse_rotate([x, y, z]: Point3, (sy, cy): (f32, f32), (sp, cp): (f32, f32)) -> Point3 {
    let (y, z) = (cp * y + sp * z, cp * z - sp * y);
    [cy * x - sy * z, y, cy * z + sy * x]
}

fn dot(a: Point3, b: Point3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

impl CubeFrame {
    fn new(time: f32) -> Self {
        let time = time * 1.12;
        let pace = time * TAU / 37.;
        let yaw = (0.30 + pace + 0.58 * (pace + 0.44).sin()).sin_cos();
        let pitch = (0.36 + 0.11 * (pace - 0.65).sin()).sin_cos();
        let triangle =
            smooth(14., 19.5, time.rem_euclid(37.)) * (1. - smooth(22., 27., time.rem_euclid(37.)));
        Self {
            yaw,
            pitch,
            ray: inverse_rotate([0., 0., -1.], yaw, pitch),
            key: inverse_rotate([-0.50, 0.67, 0.55], yaw, pitch),
            fill: inverse_rotate([0.74, 0.14, 0.50], yaw, pitch),
            triangle,
        }
    }

    fn shade(&self, x: f32, y: f32) -> Sample {
        let radius = ((x - 0.47) * 1.28).powi(2) + ((y - 0.48) * 0.71).powi(2);
        let halo = (1. - smooth(0., 0.38, radius)) * (0.54 + 0.07 * noise(x * 13., y * 9.));
        let mut background = mix([0.036; 3], [0.201; 3], halo);
        let shadow = ellipse(x, y, [0.537, 0.805], [0.269, 0.089]);
        background = mix(background, [0.040; 3], 0.58 * shadow);
        let origin = inverse_rotate(
            [(x - 0.50) * 3.18, (0.52 - y) * 1.87, 2.70],
            self.yaw,
            self.pitch,
        );
        let Some((p, axis, sign)) = cube_hit(origin, self.ray) else {
            return solid(background);
        };

        if axis == 3 {
            let glint = 0.83 + marble(p[0], p[1]) * 0.15;
            return solid(mix([0.35; 3], [1.04 * glint; 3], self.triangle));
        }
        let mut normal = [0.; 3];
        normal[axis] = sign;
        let diffuse = dot(normal, self.key).max(0.);
        let bounce = dot(normal, self.fill).max(0.);
        let uv = [(axis + 1) % 3, (axis + 2) % 3];
        let grain = marble(p[uv[0]] + axis as f32 * 1.7, p[uv[1]]);
        let edge = (CUBE_HALF - p[uv[0]].abs()).min(CUBE_HALF - p[uv[1]].abs());
        let rim = (1. - smooth(0., 0.018, edge)) * (0.17 + diffuse * 0.29);
        let roughness = 0.92 + grain * 0.14;
        let ambient = 0.34 + 0.07 * normal[1] + 0.02 * normal[0];
        let light = (ambient + diffuse * 0.72 + bounce * 0.12) * (1. - 0.43 * self.triangle);
        let mut color = [(0.72 * light + 0.82 * rim) * roughness; 3];
        if axis != 1 {
            color = gain(color, 0.97 + 0.022 * noise(p[uv[0]] * 79., p[uv[1]] * 27.));
        }
        solid(pigment(color, p[uv[0]], p[uv[1]], axis, sign, diffuse))
    }
}

fn pigment(color: Color, u: f32, v: f32, axis: usize, sign: f32, diffuse: f32) -> Color {
    // The flecks belong to the material coordinates so they turn with the
    // cube and never appear in the background or climb across its edges.
    let (u, v) = ((u + CUBE_HALF) * 35., (v + CUBE_HALF) * 35.);
    let face = 2 * axis as i32 + i32::from(sign > 0.);
    let (x, y) = (u.floor() as i32 + 71 * face, v.floor() as i32 + 23 * face);
    if hash(x, y) < 0.979 {
        return color;
    }
    let px = 0.28 + 0.44 * hash(x + 127, y);
    let py = 0.28 + 0.44 * hash(x, y - 191);
    let r2 = (u.fract() - px).powi(2) + (v.fract() - py).powi(2);
    let cover = 1. - smooth(0.053, 0.138, r2);
    let ink = PIGMENTS[(hash(y - 211, x) * 2.999) as usize];
    mix(color, gain(ink, 0.81 + 0.22 * diffuse), cover * 0.86)
}

fn cube_hit(origin: Point3, ray: Point3) -> Option<(Point3, usize, f32)> {
    let (mut entry, mut exit) = (0_f32, f32::INFINITY);
    let (mut face, mut sign) = (2, 1.);
    for i in 0..3 {
        if ray[i].abs() < 0.0001 {
            if origin[i].abs() > CUBE_HALF {
                return None;
            }
            continue;
        }
        let opposite = -ray[i].signum();
        let near = (opposite * CUBE_HALF - origin[i]) / ray[i];
        let far = (-opposite * CUBE_HALF - origin[i]) / ray[i];
        if near > entry {
            (entry, face, sign) = (near, i, opposite);
        }
        exit = exit.min(far);
        if entry > exit {
            return None;
        }
    }
    // Intersect one more plane so the triangle shares the cube's edges
    // and follows its rotation, just like the other six faces.
    let distance = CORNER_LIMIT - dot(origin, CORNER_NORMAL);
    let speed = dot(ray, CORNER_NORMAL);
    if speed.abs() < 0.0001 {
        if distance < 0. {
            return None;
        }
    } else {
        let corner = distance / speed;
        if speed > 0. {
            exit = exit.min(corner);
        } else if corner > entry {
            entry = corner;
            face = 3;
        }
        if entry > exit {
            return None;
        }
    }
    Some((
        std::array::from_fn(|i| origin[i] + ray[i] * entry),
        face,
        sign,
    ))
}
