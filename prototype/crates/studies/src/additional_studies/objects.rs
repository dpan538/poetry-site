use super::super::{gain, hash, mix, noise, smooth, tint};
use super::paint::{Painting, Point};
use std::f32::consts::TAU;

pub(super) fn coin_background(p: &mut Painting) {
    p.wash(|[x, y], _| {
        let light = (1. - (x - 0.14).abs()) * (1. - 0.5 * y);
        let flecks = noise(185. * x, 140. * y);
        gain(
            mix(tint(0x43303C), tint(0xC6A28E), light),
            0.87 + 0.24 * flecks,
        )
    });
    for row in 0..12 {
        let y = row as f32 * 0.057;
        p.line([0., y], [1., y], 0.0014, 0x695257);
        for col in 0..11 {
            let x = (col as f32 + 0.5 * (row % 2) as f32) * 0.096;
            p.line([x, y], [x, y + 0.057], 0.0013, 0x725A59);
        }
    }
    p.rect([0.28, 0., 0.90, 0.676], 0x382629);
    p.rect([0.30, 0., 0.871, 0.67], 0xBE9A83);
    p.rect([0.317, 0., 0.847, 0.67], 0x501E25);
    p.rect([0.345, 0., 0.824, 0.663], 0xA44038);
    for (l, r) in [(0.364, 0.569), (0.595, 0.801)] {
        for (t, b) in [(0.026, 0.295), (0.326, 0.502), (0.528, 0.635)] {
            p.rect([l, t, r, b], 0xCA6A53);
            p.rect([l + 0.006, t + 0.010, r - 0.009, b - 0.010], 0x65262B);
            p.rect([l + 0.014, t + 0.022, r - 0.013, b - 0.018], 0x933432);
            p.line(
                [l + 0.015, b - 0.020],
                [r - 0.016, b - 0.020],
                0.003,
                0xAD5040,
            );
        }
    }
    p.oval([0.554, 0.357], [0.009, 0.023], 0x613E29);
    p.oval([0.552, 0.354], [0.006, 0.010], 0xFFDB8D);
    p.polygon(
        &[[0.27, 0.657], [0.91, 0.657], [0.985, 0.737], [0.20, 0.737]],
        0xB59287,
    );
    p.rect([0.20, 0.737, 0.985, 0.755], 0x51414E);
    p.wash(|[x, y], old| {
        if y < 0.752 {
            return old;
        }
        let perspective = (x - 0.59) / (0.39 + y);
        let mortar = (perspective * 9.).fract().abs() < 0.010 || ((y - 0.75) * 14.).fract() < 0.024;
        let stone = noise(perspective * 80., y * 92.) * 0.25 + noise(x * 230., y * 160.) * 0.13;
        gain(
            mix(tint(0x514249), tint(0xB08A78), (1. - x) * 0.9),
            0.77 + stone - f32::from(mortar) * 0.19,
        )
    });
    p.glow([0.18, 0.63], [0.68, 0.64], 0xF7CA9B, 0.13);
}

const LANDING: f32 = 1.24;
const BOUNCE: f32 = 0.24;
const ROLL_START: f32 = LANDING + BOUNCE;
const COIN_RADIUS: f32 = 0.091;
const PLANAR_SPEED: f32 = 0.285;
const ROLL_DURATION: f32 = 4.6;
const DEPTH_SCALE: f32 = 0.42;
const UP_SCALE: f32 = 0.9075;

fn rolling(t: f32) -> (f32, f32) {
    let elapsed = (t - ROLL_START).clamp(0., ROLL_DURATION);
    let distance = PLANAR_SPEED * elapsed * (1. - elapsed / (2. * ROLL_DURATION));
    (distance, -distance / COIN_RADIUS)
}

fn clearance(t: f32) -> f32 {
    const GRAVITY: f32 = 1.82;
    const INITIAL_HEIGHT: f32 = 0.094;
    if t <= LANDING {
        let velocity = 0.5 * GRAVITY * LANDING - INITIAL_HEIGHT / LANDING;
        return (INITIAL_HEIGHT + velocity * t - 0.5 * GRAVITY * t * t).max(0.);
    }
    let elapsed = t - LANDING;
    0.5 * GRAVITY * elapsed * (BOUNCE - elapsed).max(0.)
}

pub(super) fn coin(p: &mut Painting, t: f32) {
    let airborne = clearance(t);
    let (distance, roll_phase) = rolling(t);
    let air_time = t.min(ROLL_START);
    let turn = air_time / ROLL_START;
    let heading = -0.48_f32;
    let yaw = heading - TAU * (1. - turn);
    let (h_sin, h_cos) = yaw.sin_cos();
    let rest_time = ROLL_START + ROLL_DURATION;
    let fall = smooth(rest_time - 0.62, rest_time + 0.42, t);
    let upright = 0.53 + (std::f32::consts::FRAC_PI_2 - 0.53) * smooth(0.02, LANDING, t);
    let wobble = 0.085 * (t * 16.).sin() * smooth(0.36, 0.88, fall) * (1. - fall);
    let tilt = upright * (1. - fall) + wobble;
    let (v_sin, v_cos) = tilt.sin_cos();
    let spin = -PLANAR_SPEED * air_time / COIN_RADIUS + roll_phase;
    let travel = PLANAR_SPEED * air_time + distance;
    let contact = [
        0.237 + heading.cos() * travel / 1.7,
        0.735 - DEPTH_SCALE * heading.sin() * travel,
    ];
    // The lower point of the rim is the ground anchor. When the coin falls
    // onto its face the center moves around that point, rather than sinking.
    let center = [
        contact[0] - COIN_RADIUS * h_sin * v_cos / 1.7,
        contact[1] - airborne - COIN_RADIUS * (DEPTH_SCALE * h_cos * v_cos + UP_SCALE * v_sin),
    ];
    let project = |u: f32, v: f32| {
        [
            center[0] + COIN_RADIUS / 1.7 * (u * h_cos - v * v_cos * h_sin),
            center[1]
                - COIN_RADIUS
                    * (DEPTH_SCALE * (u * h_sin + v * v_cos * h_cos) + UP_SCALE * v * v_sin),
        ]
    };
    let disc = |size: f32| {
        std::array::from_fn::<_, 36, _>(|i| {
            let a = i as f32 * TAU / 36.;
            project(size * a.cos(), size * a.sin())
        })
    };
    p.glow(
        [contact[0] + airborne * 0.24, contact[1] + airborne * 0.10],
        [0.072 + airborne * 0.16, 0.021 + airborne * 0.075],
        0x33223D,
        0.85 - airborne * 1.2,
    );
    let edge = disc(1.).map(|[x, y]| [x + 0.004 * v_sin, y + 0.009]);
    p.polygon(&edge, 0x9D562B);
    p.polygon(&disc(1.), 0xFFD677);
    p.polygon(&disc(0.87), 0xAE6726);
    p.polygon(&disc(0.78), 0xE7A951);
    let stamp = |x: f32, y: f32| {
        let (s, c) = spin.sin_cos();
        project(x * c - y * s, x * s + y * c)
    };
    if DEPTH_SCALE * v_cos + UP_SCALE * h_cos * v_sin > 0. {
        p.line(stamp(-0.19, 0.24), stamp(0.03, 0.46), 0.005, 0xA05A30);
        p.line(stamp(0.03, 0.46), stamp(0.03, -0.39), 0.005, 0xA05A30);
        p.line(stamp(-0.21, -0.39), stamp(0.28, -0.39), 0.005, 0xA05A30);
    } else {
        p.polygon(&disc(0.49), 0xB87A40);
        p.polygon(&disc(0.37), 0xF1B860);
    }
    // The notch and the embossed face are both attached to the spinning rim.
    p.line(stamp(0.73, 0.09), stamp(0.97, 0.12), 0.008, 0x954A29);
    p.line(stamp(-0.43, 0.68), stamp(-0.50, 0.83), 0.004, 0xFFF0AF);
    let rim = project(-0.33, 0.84);
    p.glow(rim, [0.025, 0.027], 0xFFF0A9, 0.67);
}

const HORIZON: f32 = 0.397;
const VANISH: f32 = 0.601;
const SIGN_DEPTH: f32 = 1.68;
const SIGN_X: f32 = -1.45;

fn road(x: f32, z: f32, height: f32) -> Point {
    [
        VANISH + 0.30 * x / z.max(0.06),
        HORIZON + (0.77 - height) / z.max(0.06),
    ]
}

pub(super) fn sign_background(p: &mut Painting) {
    p.wash(|[x, y], _| {
        let sky = mix(tint(0x35297C), tint(0xF88D66), smooth(0., HORIZON, y));
        let mist = smooth(0.49, 0.80, noise(x * 8., y * 36.)) * 0.49;
        mix(sky, tint(0xFCB67E), mist * smooth(0.02, 0.41, y))
    });
    p.oval([0.712, 0.276], [0.049, 0.081], 0xFFCB82);
    p.polygon(
        &[
            [0., 0.35],
            [0.17, 0.274],
            [0.25, 0.328],
            [0.39, 0.32],
            [0.54, 0.384],
            [0.68, 0.369],
            [0.79, 0.332],
            [0.94, 0.290],
            [1., 0.339],
            [1., 1.],
            [0., 1.],
        ],
        0x563975,
    );
    p.polygon(
        &[
            [0., 0.392],
            [0.19, 0.342],
            [0.36, 0.376],
            [0.56, 0.397],
            [0.78, 0.389],
            [0.96, 0.345],
            [1., 0.340],
            [1., 1.],
            [0., 1.],
        ],
        0x253969,
    );
    p.polygon(
        &[
            [0., 0.449],
            [0.34, 0.414],
            [0.58, 0.407],
            [0.74, 0.441],
            [1., 0.477],
            [1., 1.],
            [0., 1.],
        ],
        0x163F52,
    );
    for i in 0..120 {
        let x = hash(i, 145);
        let y = 0.414 + hash(i, 163) * 0.61;
        let height = 0.010 + 0.079 * (y - HORIZON) * hash(i, 172);
        let ink = [0x337470, 0x265867, 0x4B8786, 0x225662][i as usize % 4];
        p.line([x, y], [x - 0.009, y - height], 0.0022, ink);
    }
    let surface = |p: &mut Painting, half: f32, ink| {
        p.polygon(
            &[
                road(-half, 45., 0.),
                road(half, 45., 0.),
                road(half, 0.94, 0.),
                road(-half, 0.94, 0.),
            ],
            ink,
        );
    };
    surface(p, 1.32, 0x73749B);
    surface(p, 1.14, 0x384574);
    p.wash(|[x, y], color| {
        if y < HORIZON + 0.77 / 45. {
            return color;
        }
        let depth = 0.77 / (y - HORIZON);
        // Paint boundaries in road coordinates and hold a one-cell minimum
        // width until they enter the distant haze. The white edges are solid.
        let edge_x = road(1.088, depth, 0.)[0] - VANISH;
        let from_edge = ((x - VANISH).abs() - edge_x).abs();
        let edge = 1. - smooth(0.0058, 0.0098, from_edge);
        let visibility = smooth(41., 7.5, depth);
        mix(color, tint(0xDBE3F7), edge * visibility)
    });
}

fn centerline(p: &mut Painting) {
    p.wash(|[x, y], color| {
        let across = (x - VANISH).abs();
        if across > 0.017 || y < HORIZON + 0.77 / 45. {
            return color;
        }
        let depth = 0.77 / (y - HORIZON);
        let cycle = (depth - 1.43).rem_euclid(1.83);
        let dash = smooth(0., 0.045, cycle) * (1. - smooth(0.78, 0.84, cycle));
        let half_width = (0.015 / depth).max(0.009) * (1. - 0.30 * smooth(7., 27., depth));
        let stripe = 1. - smooth(half_width * 0.80, half_width + 0.0022, across);
        mix(
            color,
            tint(0xFFD144),
            stripe * dash * smooth(41., 8., depth),
        )
    });
}

fn road_sign(p: &mut Painting, illumination: f32) {
    let material = |dark, lit| {
        mix(tint(dark), tint(lit), illumination)
            .into_iter()
            .fold(0_u32, |hex, v| (hex << 8) | (v * 255.) as u32)
    };
    let foot = road(SIGN_X, SIGN_DEPTH, 0.);
    let face = road(SIGN_X, SIGN_DEPTH, 0.69);
    p.glow(foot, [0.094, 0.034], 0x13223E, 0.79);
    p.line(face, foot, 0.010, material(0x707DB0, 0xF8D398));
    let diamond = |rx: f32, ry: f32| {
        [
            [face[0] - rx, face[1]],
            [face[0], face[1] - ry],
            [face[0] + rx, face[1]],
            [face[0], face[1] + ry],
        ]
    };
    p.polygon(&diamond(0.130, 0.201), 0x533055);
    p.polygon(&diamond(0.120, 0.190), material(0xFBA631, 0xFFD86A));
    p.polygon(&diamond(0.102, 0.163), 0x613465);
    p.polygon(&diamond(0.094, 0.150), material(0xF29A26, 0xFFDE70));
    let q = |x, y| [face[0] + x, face[1] + y];
    for (a, b) in [
        (q(-0.027, 0.076), q(0.022, 0.021)),
        (q(0.022, 0.021), q(-0.021, -0.023)),
        (q(-0.021, -0.023), q(0.018, -0.077)),
        (q(0.018, -0.077), q(-0.011, -0.073)),
        (q(0.018, -0.077), q(0.024, -0.045)),
    ] {
        p.line(a, b, 0.009, 0x372445);
    }
}

#[derive(Clone, Copy)]
struct Car {
    depth: f32,
    approaching: bool,
    paint: u32,
}

impl Car {
    fn lane(self) -> f32 {
        if self.approaching { -0.56 } else { 0.56 }
    }

    fn headlights(self, p: &mut Painting) {
        if !self.approaching {
            return;
        }
        let x = self.lane();
        let front = self.depth - 0.40;
        for side in [-1., 1.] {
            let start = x + side * 0.22;
            let far = (front - 4.2).max(0.92);
            let shoulder = start + side * 0.35;
            p.translucent(
                &[
                    road(start - 0.05, front, 0.005),
                    road(start + 0.05, front, 0.005),
                    road(shoulder + 0.50, far, 0.005),
                    road(shoulder - 0.50, far, 0.005),
                ],
                0xFFC878,
                0.30,
            );
        }
    }

    fn draw(self, p: &mut Painting) {
        let z = self.depth;
        let x = self.lane();
        let q = |u, d, v| road(x + u, z + d, v);
        let contact = q(0., 0., 0.);
        p.glow(contact, [0.155 / z, 0.14 / z], 0x131F46, 0.83);
        // Four wheels are projected on the same surface as the lane markings.
        for side in [-1., 1.] {
            for end in [-1., 1.] {
                let bearing = q(0.29 * side, 0.31 * end, 0.065);
                p.oval(bearing, [0.023 / z, 0.081 / z], 0x171C3A);
            }
        }
        p.polygon(
            &[
                q(-0.32, -0.49, 0.08),
                q(0.32, -0.49, 0.08),
                q(0.32, 0.50, 0.09),
                q(-0.32, 0.50, 0.09),
            ],
            0x25345A,
        );
        p.polygon(
            &[
                q(-0.32, -0.49, 0.24),
                q(0.32, -0.49, 0.24),
                q(0.32, 0.50, 0.25),
                q(-0.32, 0.50, 0.25),
            ],
            self.paint,
        );
        p.polygon(
            &[
                q(0.32, -0.49, 0.24),
                q(0.32, 0.50, 0.25),
                q(0.32, 0.50, 0.09),
                q(0.32, -0.49, 0.08),
            ],
            0x32436B,
        );
        p.polygon(
            &[
                q(-0.27, -0.25, 0.24),
                q(0.27, -0.25, 0.24),
                q(0.23, -0.12, 0.40),
                q(-0.23, -0.12, 0.40),
            ],
            if self.approaching { 0x6CAFD8 } else { 0x596A9E },
        );
        p.polygon(
            &[
                q(-0.23, -0.12, 0.40),
                q(0.23, -0.12, 0.40),
                q(0.22, 0.25, 0.40),
                q(-0.22, 0.25, 0.40),
            ],
            self.paint,
        );
        let fascia = if self.approaching { 0x3C5274 } else { 0x703C61 };
        p.polygon(
            &[
                q(-0.32, -0.491, 0.24),
                q(0.32, -0.491, 0.24),
                q(0.32, -0.491, 0.09),
                q(-0.32, -0.491, 0.09),
            ],
            fascia,
        );
        let (lamp, aura) = if self.approaching {
            (0xFFF0A2, 0xFFCA59)
        } else {
            (0xFF5467, 0xFF374B)
        };
        for side in [-1., 1.] {
            let position = q(side * 0.245, -0.50, 0.187);
            p.glow(position, [0.105 / z, 0.145 / z], aura, 0.82);
            p.oval(position, [0.034 / z, 0.029 / z], lamp);
        }
    }
}

pub(super) fn traffic(p: &mut Painting, t: f32) {
    let mut cars = [
        Car {
            depth: 21.4 - 3. * t,
            approaching: true,
            paint: 0x3DDADE,
        },
        Car {
            depth: 35. - 3. * t,
            approaching: true,
            paint: 0xD87FE3,
        },
        Car {
            depth: 3.2 + 3.8 * t,
            approaching: false,
            paint: 0xD65A63,
        },
        Car {
            depth: 0.78 + 3.5 * (t - 3.8),
            approaching: false,
            paint: 0xED8058,
        },
    ];
    cars.sort_by(|a, b| b.depth.total_cmp(&a.depth));
    for car in cars.iter().filter(|car| (1.30..=40.).contains(&car.depth)) {
        car.headlights(p);
    }
    // The yellow paint retains its color in the headlight beam. Vehicles
    // still occlude all of the markings when drawn over the road.
    centerline(p);
    let illumination = cars
        .iter()
        .filter(|car| car.approaching)
        .map(|car| {
            let ahead = car.depth - 0.40 - SIGN_DEPTH;
            let in_beam = smooth(0.85, 2.2, ahead);
            in_beam * (1. - smooth(3.2, 7.2, ahead))
        })
        .fold(0., f32::max);
    for car in cars
        .into_iter()
        .filter(|car| (0.36..=40.).contains(&car.depth))
    {
        car.draw(p);
    }
    road_sign(p, illumination);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coin_only_contacts_the_ground_at_landing_bounce_and_roll() {
        assert!(clearance(0.4) > clearance(0.));
        assert!(clearance(0.6) > clearance(1.1));
        assert!(clearance(LANDING - 0.001) > 0.);
        assert_eq!(clearance(LANDING), 0.);
        assert!(clearance(LANDING + BOUNCE / 2.) > 0.);
        assert!(clearance(LANDING + BOUNCE) < 0.00001);
        assert_eq!(clearance(6.), 0.);
    }

    #[test]
    fn coin_rotates_through_the_distance_traveled_on_its_rim() {
        let start = ROLL_START;
        assert_eq!(rolling(start), (0., 0.));
        for t in [0.4, 1., 2.8, 4.1, 5.2] {
            let (distance, phase) = rolling(start + t);
            assert!((distance + phase * COIN_RADIUS).abs() < 0.000001);
        }
        assert_eq!(rolling(start + ROLL_DURATION + 0.1), rolling(12.));
    }
}
