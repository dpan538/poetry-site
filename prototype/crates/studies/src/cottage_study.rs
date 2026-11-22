//! A continuous evening outside the cottage.
use super::Point;
use crate::scene::{Color, ellipse, gain, hash, luminance, mix, noise, rect, smooth};

pub(super) const PLATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/cottage-afternoon.rgb"));
const WINDOWS: [[f32; 4]; 2] = [[0.275, 0.445, 0.360, 0.600], [0.428, 0.446, 0.513, 0.601]];

struct Leaf {
    position: Point,
    glancing: f32,
}

pub(super) struct Cottage {
    hour: f32,
    time: f32,
    lamp: f32,
    leaves: [Leaf; 5],
}

impl Cottage {
    pub(super) fn at(seconds: f32) -> Self {
        let time = seconds.min(13.05);
        let hour = 0.62 * smooth(0.16, 8.0, time) + 0.38 * smooth(5.55, 13.05, time);
        let lamp = smooth(2.5, 5.54, time) * (1. - smooth(9.58, 11.91, time));
        let leaves = std::array::from_fn(|i| {
            let seed = hash(i as i32, 412);
            let start = if i < 3 { 1.36 } else { 8.12 };
            let age = (time - start - 0.62 * seed).max(0.);
            let pass = (age / (1.54 + seed * 1.10)).min(1.);
            let glide = smooth(0., 1., pass);
            let lift = (age * (5.4 + seed * 3.7)).sin().abs()
                * smooth(0., 0.21, pass)
                * (1. - smooth(0.74, 1., pass));
            Leaf {
                position: [
                    0.292 + seed * 0.126 + glide * (0.24 + seed * 0.17),
                    0.843 + seed * 0.094 - 0.020 * glide - 0.017 * lift,
                ],
                glancing: 0.55 + 0.45 * lift,
            }
        });
        Self {
            hour,
            time,
            lamp,
            leaves,
        }
    }

    pub(super) fn shade(&self, base: Color, x: f32, y: f32) -> Color {
        let mut color = self.ambient(base, x, y);
        if y > 0.807 {
            for (i, leaf) in self.leaves.iter().enumerate() {
                let hit = ellipse(x, y, leaf.position, [0.005 + leaf.glancing * 0.007, 0.007]);
                let ochre = gain(
                    [0.86, 0.455, 0.260],
                    0.85 + i as f32 * 0.045 - 0.21 * self.hour,
                );
                color = mix(color, ochre, hit * 0.90);
            }
        }
        if self.lamp > 0. {
            for pane in WINDOWS {
                color = glaze(color, base, x, y, pane, self.lamp);
            }
        }
        color
    }

    fn ambient(&self, base: Color, x: f32, y: f32) -> Color {
        let cool = [0.82, 0.952, 1.169];
        let grade = std::array::from_fn(|i| base[i] * cool[i]);
        let exposure = 1.18 - 0.109 * self.hour + 0.059 * smooth(0.53, 0.92, y);
        let mut color = gain(mix(base, grade, 0.83 * self.hour), exposure);
        if y < 0.48 && (y < 0.151 || x > 0.829 || x < 0.147) {
            let sky = smooth(0.041, 0.188, base[2] - base[0]) * smooth(0.41, 0.59, base[2]);
            let drift = x - self.time * 0.017;
            let band = smooth(0.48, 0.76, noise(drift * 6.5, y * 49. + 3.3));
            let evening = mix([1.04, 0.824, 0.697], [0.779, 0.765, 0.928], self.hour);
            color = mix(color, evening, band * sky * 0.36);
        }
        color
    }
}

fn glaze(color: Color, base: Color, x: f32, y: f32, pane: [f32; 4], lamp: f32) -> Color {
    let glass = rect(x, y, pane);
    let [left, top, right, bottom] = pane;
    let centre = (left + right) * 0.5;
    let band = (top + bottom) * 0.5;
    let sash = rect(x, y, [centre - 0.0027, top, centre + 0.0027, bottom]).max(rect(
        x,
        y,
        [left, band - 0.0025, right, band + 0.0025],
    ));
    let pane_detail = luminance(base);
    let depth = smooth(top, bottom, y);
    let lit = gain(
        mix([0.862, 0.568, 0.291], [1.15, 0.865, 0.504], depth),
        0.86 + pane_detail * 0.37,
    );
    mix(color, lit, glass * lamp * (1. - 0.96 * sash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::plate_pixels::plate_pixel;

    #[test]
    fn the_scene_continues_through_the_previous_cut_points() {
        for time in [4.58, 11.28] {
            let before = Cottage::at(time - 1. / 30.);
            let after = Cottage::at(time + 1. / 30.);
            for (x, y) in [(0.525, 0.12), (0.295, 0.48), (0.495, 0.55), (0.334, 0.79)] {
                let base = plate_pixel(PLATE, x, y);
                let from = before.shade(base, x, y);
                let to = after.shade(base, x, y);
                assert!(from.into_iter().zip(to).all(|(a, b)| (a - b).abs() < 0.045));
            }
        }
    }
}
