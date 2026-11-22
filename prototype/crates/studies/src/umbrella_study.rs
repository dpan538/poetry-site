//! Each pigmented raindrop leaves a wet trace where it crosses the café window.
use crate::scene::{Color, gain, hash, luminance, mix, rect, smooth};

const GLASS: [f32; 4] = [0.013, 0.088, 0.529, 0.653];
const WIND: f32 = 0.325;
const ENTRY_SPEED: f32 = 0.34;
const GRAVITY: f32 = 0.87;
const PIGMENT_START: f32 = 0.85;

pub(super) struct Umbrella {
    time: f32,
}

impl Umbrella {
    pub(super) fn at(seconds: f32) -> Self {
        Self {
            time: seconds.min(12.6),
        }
    }

    pub(super) fn shade(&self, base: Color, x: f32, y: f32) -> Color {
        let value = luminance(base);
        let inside = rect(x, y, GLASS);
        // All luminance comes from the plate. Rain is the only source of hue.
        let interior = gain(base.map(|c| c.powf(0.72)), 1.14);
        let brightness = luminance(interior);
        let grey_value = value + (brightness - value) * inside;
        let age = fall_time(y);
        let source_time = self.time - age;
        let drift = WIND * age;
        let primary = rain((x + drift) * 47., source_time, 17, inside > 0.);
        let secondary = rain((x + drift - 0.014) * 69., source_time, 29, inside > 0.);
        let wet = 1. - (1. - primary.stain) * (1. - 0.70 * secondary.stain);
        let pigment = gain([1.23, 1.115, 0.235], brightness + 0.031);
        let color = mix([grey_value; 3], pigment, wet * inside);

        let strength = primary.stroke.max(secondary.stroke * 0.58);
        let tinted = primary.pigmented.max(secondary.pigmented * 0.58);
        let rain_hue = mix(
            [0.88; 3],
            [0.90, 0.795, 0.261],
            inside * tinted / strength.max(0.00001),
        );
        let floor = 1. - smooth(0.910, 0.982, y);
        mix(color, rain_hue, strength * floor * 0.68)
    }
}

fn fall_time(y: f32) -> f32 {
    2. * y / (ENTRY_SPEED + (ENTRY_SPEED * ENTRY_SPEED + 2. * GRAVITY * y).sqrt())
}

struct Lane {
    phase: f32,
    center: f32,
    width: f32,
    interval: f32,
}

impl Lane {
    fn new(index: i32, seed: i32) -> Self {
        let phase = hash(index, seed);
        Self {
            phase,
            center: index as f32 + 0.16 + phase * 0.68,
            width: 0.057 + 0.078 * phase,
            interval: 0.20 + 0.19 * hash(index, seed + 5),
        }
    }

    fn trace(&self, x: f32, source_time: f32) -> f32 {
        let first_drop =
            ((PIGMENT_START + self.phase) / self.interval).ceil() * self.interval - self.phase;
        let wet_for = (source_time - first_drop).max(0.);
        if wet_for == 0. {
            return 0.;
        }
        // Repeated drops deepen the mark and adjacent wet tracks merge.
        // The track can only spread after a drop has passed this height.
        let width = self.width + 0.260 * wet_for.sqrt();
        let field = 1. - smooth(width * 0.45, width + 0.14, (x - self.center).abs());
        let concentration = wet_for / (wet_for + 0.91 + 0.53 * self.phase);
        field * concentration
    }
}

struct Rain {
    stroke: f32,
    pigmented: f32,
    stain: f32,
}

fn rain(column: f32, source_time: f32, seed: i32, on_glass: bool) -> Rain {
    let index = column.floor() as i32;
    let lane = Lane::new(index, seed);
    let across = 1. - smooth(lane.width * 0.40, lane.width, (column - lane.center).abs());
    let phase = (source_time + lane.phase).rem_euclid(lane.interval);
    let length = 0.069 + 0.045 * lane.phase;
    let stroke = across * smooth(0., 0.017, phase) * (1. - smooth(length - 0.024, length, phase));
    let pigmented = if source_time - phase >= PIGMENT_START {
        stroke
    } else {
        0.
    };
    let stain = if on_glass && source_time > PIGMENT_START {
        (index - 1..=index + 1).fold(0., |wet, nearby| {
            let trace = Lane::new(nearby, seed).trace(column, source_time);
            1. - (1. - wet) * (1. - trace)
        })
    } else {
        0.
    };
    Rain {
        stroke,
        pigmented,
        stain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stain_waits_for_the_same_raindrop_at_each_height() {
        let lane = Lane::new(16, 17);
        let launch =
            ((PIGMENT_START + lane.phase) / lane.interval).ceil() * lane.interval - lane.phase;
        let top = fall_time(0.16);
        let bottom = fall_time(0.57);
        let time = launch + (top + bottom) * 0.5;
        assert!(rain(lane.center, time - top, 17, true).stain > 0.);
        assert_eq!(rain(lane.center, time - bottom, 17, true).stain, 0.);
        assert!(rain(lane.center, launch + 0.5, 17, true).stain > 0.);
    }

    #[test]
    fn the_opening_is_monochrome_and_yellow_stays_in_the_window() {
        let base = [0.89, 0.57, 0.21];
        for time in [0., 0.3, 0.8] {
            let frame = Umbrella::at(time);
            for row in 0..80 {
                for col in 0..136 {
                    let shade = frame.shade(base, col as f32 / 136., row as f32 / 80.);
                    assert_eq!(shade, [shade[0]; 3]);
                }
            }
        }
        for time in [4., 8., 13.] {
            let frame = Umbrella::at(time);
            for (x, y) in [(0.70, 0.32), (0.79, 0.71), (0.24, 0.85)] {
                let shade = frame.shade(base, x, y);
                assert_eq!(shade, [shade[0]; 3]);
            }
        }
        let end = Umbrella::at(13.);
        for (x, y) in [(0.07, 0.13), (0.44, 0.13), (0.08, 0.60), (0.44, 0.59)] {
            let [r, g, b] = end.shade(base, x, y);
            assert!(r > b + 0.30 && g > b + 0.28);
        }
    }

    #[test]
    fn rainfall_traces_the_same_grain_across_the_window_edge() {
        let start_age = fall_time(0.28);
        let end_age = fall_time(0.57);
        let lane = Lane::new(34, 17);
        let start = start_age - lane.phase + lane.interval * 3. + 0.034;
        let source_x = lane.center / 47.;
        let start_x = source_x - WIND * start_age;
        let end_x = source_x - WIND * end_age;
        assert!(start_x > GLASS[2] && end_x < GLASS[2]);
        let a = rain(
            (start_x + WIND * start_age) * 47.,
            start - start_age,
            17,
            false,
        );
        let b = rain(
            (end_x + WIND * end_age) * 47.,
            start + (end_age - start_age) - end_age,
            17,
            true,
        );
        assert!(a.stroke > 0.9);
        assert!((a.stroke - b.stroke).abs() < 0.0001);
    }
}
