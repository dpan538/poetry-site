use super::super::{mix, smooth, tint};
use super::paint::{Painting, Point};
use std::f32::consts::TAU;

const CENTER: Point = [0.51, 0.50];
const RIM: [Point; 14] = [
    [0.506, 0.117],
    [0.419, 0.151],
    [0.361, 0.257],
    [0.351, 0.399],
    [0.376, 0.555],
    [0.402, 0.667],
    [0.414, 0.785],
    [0.466, 0.857],
    [0.539, 0.845],
    [0.585, 0.766],
    [0.611, 0.648],
    [0.657, 0.507],
    [0.669, 0.342],
    [0.613, 0.180],
];
const FOLD: [Point; 12] = [
    [0.408, 0.638],
    [0.393, 0.506],
    [0.399, 0.333],
    [0.453, 0.224],
    [0.524, 0.198],
    [0.589, 0.246],
    [0.612, 0.335],
    [0.594, 0.432],
    [0.551, 0.469],
    [0.510, 0.423],
    [0.485, 0.466],
    [0.472, 0.574],
];
const CONCHA: [Point; 10] = [
    [0.442, 0.595],
    [0.457, 0.506],
    [0.491, 0.450],
    [0.523, 0.468],
    [0.551, 0.520],
    [0.576, 0.493],
    [0.575, 0.590],
    [0.537, 0.631],
    [0.505, 0.604],
    [0.472, 0.663],
];
const LOBE: [Point; 7] = [
    [0.501, 0.605],
    [0.491, 0.674],
    [0.461, 0.728],
    [0.460, 0.779],
    [0.505, 0.809],
    [0.548, 0.759],
    [0.568, 0.668],
];

pub(super) fn background(p: &mut Painting) {
    p.wash(|[x, y], _| {
        let depth =
            (1. - ((x - CENTER[0]) * 1.9).powi(2) - ((y - CENTER[1]) * 1.3).powi(2)).max(0.);
        mix(tint(0x14152F), tint(0x25294F), depth)
    });
}

fn beat(t: f32) -> f32 {
    let impulse = |age| smooth(-0.083, 0., age) * (1. - smooth(0.009, 0.151, age));
    [0.72, 1.79, 2.94, 4.15, 5.43, 6.84, 8.37, 10.02]
        .into_iter()
        .map(|start| impulse(t - start) + 0.57 * impulse(t - start - 0.198))
        .fold(0., f32::max)
}

fn spline(x: f32, y: f32, z: f32, w: f32, f: f32) -> f32 {
    0.5 * (2. * y
        + (-x + z) * f
        + (2. * x - 5. * y + 4. * z - w) * f * f
        + (-x + 3. * y - 3. * z + w) * f * f * f)
}

fn follow(
    p: &mut Painting,
    nodes: &[Point],
    closed: bool,
    shift: impl Fn(Point, f32) -> Point,
    width: f32,
    color: u32,
) {
    let count = if closed { nodes.len() } else { nodes.len() - 1 };
    let at = |i: isize| {
        let index = if closed {
            i.rem_euclid(nodes.len() as isize)
        } else {
            i.clamp(0, nodes.len() as isize - 1)
        };
        nodes[index as usize]
    };
    let mut previous = shift(nodes[0], 0.);
    for i in 0..count {
        let k = i as isize;
        let [a, b, c, d] = [at(k - 1), at(k), at(k + 1), at(k + 2)];
        for j in 1..9 {
            let t = j as f32 / 8.;
            let point = std::array::from_fn(|axis| spline(a[axis], b[axis], c[axis], d[axis], t));
            let next = shift(point, (i as f32 + t) / count as f32);
            p.line(previous, next, width, color);
            previous = next;
        }
    }
}

pub(super) fn pulse(p: &mut Painting, t: f32) {
    let energy = beat(t);
    let radiance = 0.10 + 0.16 * energy;
    p.glow(CENTER, [0.33, 0.45], 0x7C4D79, radiance);
    let contour_inks = [
        0xEC8E88, 0xBE6789, 0xF6AA88, 0xFBD5AA, 0xB975A8, 0xE48E9D, 0x948DC8,
    ];
    for (band, path) in [&RIM[..], &FOLD[..], &CONCHA[..], &LOBE[..]]
        .into_iter()
        .enumerate()
    {
        let strands = if band == 0 { 8 } else { 6 };
        for i in 0..strands {
            let offset = i as f32 - (strands - 1) as f32 * 0.5;
            let spawn = TAU * i as f32 / strands as f32;
            let map = |[x, y]: Point, progress: f32| {
                let angle = progress * TAU;
                let braid =
                    (angle * (2. + (band % 2) as f32) + spawn + 0.24 * (t * 1.1).sin()).sin();
                let tie = (angle * 3. - spawn + 0.45 * t).cos();
                let tighten = 1. - 0.34 * energy;
                let stretch = 1. + 0.10 * energy + 0.008 * (t * 1.10).sin();
                let u = (x - CENTER[0]) * stretch;
                let v = (y - CENTER[1]) * (stretch + 0.025 * energy);
                [
                    CENTER[0] + u + 0.0047 * offset * tighten + 0.012 * braid,
                    CENTER[1] + v + 0.0083 * offset * tighten + 0.010 * tie,
                ]
            };
            let ink = contour_inks[(i + 2 * band) % contour_inks.len()];
            follow(p, path, band == 0, map, 0.0023 + 0.0014 * energy, ink);
        }
    }
    // A warm thread carries the pulse from the ear canal into the lower fold.
    if energy > 0.02 {
        p.glow([0.523, 0.553], [0.073, 0.119], 0xE9969B, energy * 0.42);
    }
}
