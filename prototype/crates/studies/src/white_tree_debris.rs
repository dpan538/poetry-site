//! White grains release together, descend at different speeds, and settle in the grass.
use super::{ASPECT, Point};
use crate::scene::{Color, hash, mix, noise, smooth};

const STEP: f32 = 1. / 90.;
const GRAVITY: f32 = 1.30;

struct Grain {
    path: Vec<Point>,
    radius: f32,
    brightness: f32,
}

impl Grain {
    fn new(origin: Point, index: usize) -> Self {
        let random = |seed| hash(index as i32 + 502, seed);
        let size = random(6);
        let speed = 0.12 + 0.09 * size + 0.08 * random(1);
        let floor = 0.81 + 0.18 * random(3).powf(0.9);
        // The fine grains follow the air more readily and reach a lower fall speed.
        let drag = 2.22 - 1.46 * size;
        Self {
            path: fall_path(origin, speed, drag, floor),
            radius: 0.0018 + 0.0031 * size,
            brightness: 0.50 + 0.42 * random(7),
        }
    }

    fn land_at(&self) -> f32 {
        (self.path.len() - 1) as f32 * STEP
    }

    fn position(&self, age: f32) -> Point {
        let last = self.path.len() - 1;
        if age >= self.land_at() {
            return self.path[last];
        }
        let frame = age / STEP;
        let i = (frame as usize).min(last - 1);
        let fraction = frame - i as f32;
        let a = self.path[i];
        let b = self.path[i + 1];
        [
            a[0] + (b[0] - a[0]) * fraction,
            a[1] + (b[1] - a[1]) * fraction,
        ]
    }
}

fn air_velocity([x, y]: Point, age: f32) -> Point {
    // A single gust travels across the crown. The grass shelters the lower layer.
    let arrival = age - 0.24 * x;
    let gust = smooth(0.10, 0.60, arrival) * (1. - 0.75 * smooth(0.81, 1.51, arrival));
    let exposure = 1. - 0.76 * smooth(0.62, 0.92, y);
    let eddy = noise(x * 8.2 - age * 1.5, y * 7.1 + 12.) - 0.5;
    [
        exposure * (0.016 + (0.188 + eddy * 0.105) * gust),
        exposure * (-0.095 * gust + 0.046 * eddy),
    ]
}

fn fall_path(origin: Point, speed: f32, drag: f32, floor: f32) -> Vec<Point> {
    // Integrate once so seeking and live playback use the same trajectories.
    let mut path = vec![origin];
    let mut position = origin;
    let mut velocity = [0., speed];
    while position[1] < floor {
        let air = air_velocity(position, (path.len() - 1) as f32 * STEP);
        velocity[0] += (air[0] - velocity[0]) * drag * STEP;
        velocity[1] += (GRAVITY + (air[1] - velocity[1]) * drag) * STEP;
        let landing = ((floor - position[1]) / (velocity[1] * STEP)).min(1.);
        position[0] += velocity[0] * STEP * landing;
        position[1] = (position[1] + velocity[1] * STEP).min(floor);
        path.push(position);
    }
    path
}

pub(super) struct Debris {
    width: usize,
    height: usize,
    dust: Vec<f32>,
    grains: Vec<Grain>,
    still_at: f32,
    drawn_at: f32,
}

impl Debris {
    pub(super) fn new(width: usize, height: usize, origins: Vec<Point>) -> Self {
        let grains: Vec<_> = origins
            .into_iter()
            .enumerate()
            .map(|(i, point)| Grain::new(point, i))
            .collect();
        let still_at = grains.iter().map(Grain::land_at).fold(0., f32::max);
        Self {
            width,
            height,
            dust: vec![0.; width * height],
            grains,
            still_at,
            drawn_at: -1.,
        }
    }

    pub(super) fn height(&self) -> usize {
        self.height
    }

    pub(super) fn settle_time(&self) -> f32 {
        self.still_at
    }

    pub(super) fn update(&mut self, age: f32) {
        let age = age.min(self.still_at);
        if age == self.drawn_at {
            return;
        }
        self.drawn_at = age;
        self.dust.fill(0.);
        let w = self.width as f32;
        let h = self.height as f32;
        let cell = 1. / h;
        for grain in &self.grains {
            let [x, y] = grain.position(age);
            let radius = grain.radius + cell * 0.35;
            let reach = radius + cell * 0.38;
            let x0 = ((x - reach / ASPECT) * w).floor().max(0.) as usize;
            let x1 = (((x + reach / ASPECT) * w).ceil() as usize).min(self.width);
            let y0 = ((y - reach) * h).floor().max(0.) as usize;
            let y1 = (((y + reach) * h).ceil() as usize).min(self.height);
            for row in y0..y1 {
                for col in x0..x1 {
                    let dx = ((col as f32 + 0.5) / w - x) * ASPECT;
                    let dy = (row as f32 + 0.5) / h - y;
                    let coverage =
                        grain.brightness * (1. - smooth(radius * 0.5, reach, dx.hypot(dy)));
                    let pixel = &mut self.dust[row * self.width + col];
                    *pixel = pixel.max(coverage);
                }
            }
        }
    }

    pub(super) fn shade(&self, background: Color, x: f32, y: f32) -> Color {
        let col = ((x * self.width as f32) as usize).min(self.width - 1);
        let row = ((y * self.height as f32) as usize).min(self.height - 1);
        mix(
            background,
            [1.10, 1.08, 1.03],
            self.dust[row * self.width + col],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_grain_falls_immediately_and_stops_on_the_ground() {
        let origin = [0.64, 0.33];
        for i in 0..200 {
            let grain = Grain::new(origin, i);
            assert_eq!(grain.position(0.), origin);
            assert!(grain.position(0.01)[1] > origin[1]);
            let mut y = origin[1];
            for frame in 1..45 {
                let next = grain.position(frame as f32 / 15.)[1];
                assert!(next >= y);
                y = next;
            }
            let settled = grain.position(grain.land_at());
            assert!((0.81..=0.99).contains(&settled[1]));
            assert_eq!(settled, grain.position(grain.land_at() + 10.));
        }
    }

    #[test]
    fn the_same_gust_carries_fine_grains_farther() {
        let origin = [0.58, 0.26];
        let fine = fall_path(origin, 0.14, 2.1, 0.90);
        let large = fall_path(origin, 0.23, 0.85, 0.90);
        assert!(fine.len() > large.len());
        assert!(fine.last().unwrap()[0] - large.last().unwrap()[0] > 0.025);
    }
}
