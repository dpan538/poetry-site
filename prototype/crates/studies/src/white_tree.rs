//! The crown grows from a white trunk, then falls in a single release.
use crate::scene::{Color, gain, luminance, mix, noise, smooth};

#[path = "white_tree_debris.rs"]
mod debris;
use debris::Debris;

pub(super) const TRUNK_PLATE: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/white-tree-trunk.rgb"));
const ROOT: [f32; 2] = [0.625, 0.494];
const ROOT_RADIUS: f32 = 0.012;
const ASPECT: f32 = 1.7;
const DROP: f32 = 5.5;
const CURVE_STEPS: usize = 4;

fn growth_clock(seconds: f32) -> f32 {
    // Increase the rate linearly as the crown opens.
    seconds * (1.3 + 0.24 * seconds)
}

struct Shoot {
    parent: usize,
    attachment: f32,
    tip: [f32; 2],
    radius: f32,
    duration: f32,
}

const fn shoot(parent: usize, x: f32, y: f32, radius: f32, duration: f32) -> Shoot {
    Shoot {
        parent,
        attachment: 1.,
        tip: [x, y],
        radius,
        duration,
    }
}

impl Shoot {
    const fn from_side(mut self, attachment: f32) -> Self {
        self.attachment = attachment;
        self
    }
}

const CROWN: &[Shoot] = &[
    // The leader attaches to the existing trunk, followed by its continuation.
    shoot(0, 0.622, 0.416, 0.0100, 1.22),
    shoot(0, 0.621, 0.352, 0.0082, 1.12),
    shoot(1, 0.627, 0.291, 0.0066, 1.19),
    shoot(2, 0.619, 0.233, 0.0057, 1.28),
    shoot(3, 0.632, 0.170, 0.0040, 1.51),
    shoot(4, 0.623, 0.111, 0.0020, 1.73),
    // An open, low branch to the right and its forks.
    shoot(0, 0.665, 0.383, 0.0078, 1.32).from_side(0.57),
    shoot(6, 0.707, 0.345, 0.0067, 1.44),
    shoot(7, 0.758, 0.331, 0.0052, 1.36),
    shoot(8, 0.797, 0.307, 0.0036, 1.17),
    shoot(9, 0.832, 0.266, 0.0023, 1.48),
    shoot(6, 0.679, 0.331, 0.0047, 1.51).from_side(0.80),
    shoot(11, 0.668, 0.280, 0.0028, 1.15),
    shoot(11, 0.714, 0.287, 0.0025, 1.53).from_side(0.73),
    shoot(7, 0.746, 0.368, 0.0037, 1.41).from_side(0.83),
    shoot(14, 0.787, 0.369, 0.0024, 1.56),
    shoot(8, 0.773, 0.272, 0.0030, 1.74).from_side(0.78),
    // The long left branch and its smaller, unequal splits.
    shoot(0, 0.582, 0.385, 0.0074, 1.48).from_side(0.91),
    shoot(17, 0.541, 0.359, 0.0060, 1.61),
    shoot(18, 0.504, 0.321, 0.0044, 1.58),
    shoot(19, 0.464, 0.300, 0.0028, 1.28),
    shoot(17, 0.570, 0.325, 0.0045, 1.57).from_side(0.72),
    shoot(21, 0.545, 0.278, 0.0029, 1.32),
    shoot(18, 0.503, 0.378, 0.0038, 1.53).from_side(0.83),
    shoot(23, 0.459, 0.366, 0.0023, 1.35),
    shoot(19, 0.487, 0.264, 0.0026, 1.50).from_side(0.79),
    // Higher branches grow out between the lower ones and the leader.
    shoot(2, 0.666, 0.265, 0.0055, 1.18).from_side(0.65),
    shoot(26, 0.705, 0.226, 0.0044, 1.27),
    shoot(27, 0.755, 0.207, 0.0034, 1.52),
    shoot(28, 0.785, 0.177, 0.0020, 1.58),
    shoot(26, 0.681, 0.212, 0.0036, 1.40).from_side(0.73),
    shoot(30, 0.674, 0.165, 0.0023, 1.53),
    shoot(27, 0.731, 0.247, 0.0029, 1.81).from_side(0.84),
    shoot(1, 0.582, 0.306, 0.0057, 1.42).from_side(0.71),
    shoot(33, 0.561, 0.254, 0.0044, 1.27),
    shoot(34, 0.531, 0.219, 0.0030, 1.62),
    shoot(35, 0.515, 0.178, 0.0019, 1.42),
    shoot(34, 0.572, 0.207, 0.0026, 1.71).from_side(0.81),
    shoot(33, 0.544, 0.313, 0.0027, 1.66).from_side(0.79),
    shoot(3, 0.590, 0.195, 0.0043, 1.28).from_side(0.79),
    shoot(39, 0.574, 0.142, 0.0030, 1.40),
    shoot(40, 0.579, 0.092, 0.0019, 1.46),
    shoot(39, 0.552, 0.159, 0.0024, 1.71).from_side(0.83),
    shoot(4, 0.654, 0.137, 0.0030, 1.34).from_side(0.66),
    shoot(43, 0.666, 0.097, 0.0019, 1.66),
    // The last shoots keep opening the crown as the pace picks up.
    shoot(5, 0.632, 0.060, 0.0015, 2.25),
    shoot(5, 0.595, 0.077, 0.0016, 2.05).from_side(0.76),
    shoot(10, 0.877, 0.243, 0.0024, 2.00),
    shoot(47, 0.922, 0.228, 0.0015, 2.08),
    shoot(47, 0.864, 0.200, 0.0016, 1.85).from_side(0.67),
    shoot(20, 0.425, 0.273, 0.0022, 2.04),
    shoot(50, 0.386, 0.250, 0.0016, 2.14),
    shoot(50, 0.423, 0.231, 0.0016, 1.78).from_side(0.65),
    shoot(24, 0.419, 0.350, 0.0019, 2.28),
    shoot(53, 0.385, 0.365, 0.0015, 2.15),
    shoot(29, 0.819, 0.144, 0.0018, 2.10),
    shoot(55, 0.860, 0.134, 0.0014, 1.85),
    shoot(36, 0.493, 0.131, 0.0017, 2.33),
    shoot(57, 0.465, 0.112, 0.0013, 2.15),
];

type Point = [f32; 2];

struct Branch {
    curve: [Point; 3],
    bounds: [Point; 2],
    radius: Point,
    from: f32,
    until: f32,
    progress: f32,
    visible: [Point; CURVE_STEPS + 1],
}

impl Branch {
    fn at(&self, along: f32) -> Point {
        let [start, middle, end] = self.curve;
        let inverse = 1. - along;
        std::array::from_fn(|i| {
            inverse * inverse * start[i] + 2. * inverse * along * middle[i] + along * along * end[i]
        })
    }

    fn tangent(&self, along: f32) -> Point {
        let [start, middle, end] = self.curve;
        std::array::from_fn(|i| {
            (1. - along) * (middle[i] - start[i]) + along * (end[i] - middle[i])
        })
    }

    fn width_at(&self, along: f32) -> f32 {
        self.radius[1] + (self.radius[0] - self.radius[1]) * (1. - along).powi(2)
    }

    fn time_at(&self, along: f32) -> f32 {
        self.from + along * (self.until - self.from)
    }

    fn progress_at(&self, growth: f32) -> f32 {
        ((growth - self.from) / (self.until - self.from)).clamp(0., 1.)
    }
}

pub(in crate::scene::third_batch::plates) struct Tree {
    time: f32,
    branches: Vec<Branch>,
    debris: Debris,
}

impl Tree {
    pub(super) fn new(width: usize, height: usize) -> Self {
        let mut branches: Vec<Branch> = Vec::with_capacity(CROWN.len());
        for (i, shoot) in CROWN.iter().enumerate() {
            let (start, direction, radius, from) = if i == 0 {
                (ROOT, [0.01, -1.], ROOT_RADIUS, 0.34)
            } else {
                let parent = &branches[shoot.parent];
                let at = shoot.attachment;
                (
                    parent.at(at),
                    parent.tangent(at),
                    parent.width_at(at).min(shoot.radius * 1.72 + 0.0006),
                    parent.time_at(at) + 0.065,
                )
            };
            let chord = [shoot.tip[0] - start[0], shoot.tip[1] - start[1]];
            let length = (chord[0] * ASPECT).hypot(chord[1]);
            let direction_length = (direction[0] * ASPECT).hypot(direction[1]);
            // The base follows the parent, then bends outward toward the new tip.
            let follow = if shoot.attachment < 1. { 0.35 } else { 0.22 };
            let control = std::array::from_fn(|axis| {
                start[axis]
                    + chord[axis] * 0.28
                    + direction[axis] * length * follow / direction_length
            });
            let bounds = [
                std::array::from_fn(|axis| start[axis].min(control[axis]).min(shoot.tip[axis])),
                std::array::from_fn(|axis| start[axis].max(control[axis]).max(shoot.tip[axis])),
            ];
            branches.push(Branch {
                curve: [start, control, shoot.tip],
                bounds,
                radius: [radius, shoot.radius],
                from,
                until: from + shoot.duration,
                progress: 0.,
                visible: [start; CURVE_STEPS + 1],
            });
        }
        let mut origins = Vec::new();
        let growth = growth_clock(DROP);
        for branch in &branches {
            let reached = branch.progress_at(growth);
            let [start, _, tip] = branch.curve;
            let length = ((tip[0] - start[0]) * ASPECT).hypot(tip[1] - start[1]);
            let count = (length * reached * 106.).ceil() as usize;
            for i in 0..count {
                let along = (i as f32 + 0.5) * reached / count as f32;
                origins.push(branch.at(along));
            }
        }
        let debris = Debris::new(width, height, origins);
        Self {
            time: 0.,
            branches,
            debris,
        }
    }

    pub(super) fn update(&mut self, seconds: f32) {
        self.time = seconds.min(DROP + self.debris.settle_time());
        if self.time >= DROP {
            self.debris.update(self.time - DROP);
            return;
        }
        let growth = growth_clock(seconds);
        for branch in &mut self.branches {
            branch.progress = branch.progress_at(growth);
            for i in 1..=CURVE_STEPS {
                branch.visible[i] = branch.at(branch.progress * i as f32 / CURVE_STEPS as f32);
            }
        }
    }

    pub(super) fn shade(&self, base: Color, x: f32, y: f32) -> Color {
        let color = self.landscape(base, x, y);
        if self.time >= DROP {
            return self.debris.shade(color, x, y);
        }
        if !(0.35..0.955).contains(&x) || !(0.034..0.514).contains(&y) {
            return color;
        }

        let mut coverage: f32 = 0.;
        let mut bark = [0.; 3];
        let feathering = 0.68 / self.debris.height() as f32;
        for branch in &self.branches {
            if branch.progress <= 0. {
                continue;
            }
            let [[left, top], [right, bottom]] = branch.bounds;
            let margin = branch.radius[0] + feathering;
            if x < left - margin / ASPECT
                || x > right + margin / ASPECT
                || y < top - margin
                || y > bottom + margin
            {
                continue;
            }
            for (i, span) in branch.visible.windows(2).enumerate() {
                let [[sx, sy], [ex, ey]] = [span[0], span[1]];
                let dx = (ex - sx) * ASPECT;
                let dy = ey - sy;
                let px = (x - sx) * ASPECT;
                let py = y - sy;
                let length = (dx * dx + dy * dy).max(0.000001);
                let at = ((px * dx + py * dy) / length).clamp(0., 1.);
                let position = (i as f32 + at) / CURVE_STEPS as f32;
                let radius = branch.width_at(position * branch.progress);
                let taper = smooth(0.75, 1., position) * (1. - branch.progress);
                let width = radius * (1. - taper * 0.58);
                let distance = ((px - dx * at).powi(2) + (py - dy * at).powi(2)).sqrt();
                let ink = 1. - smooth(width * 0.53, width + feathering, distance);
                if ink <= coverage {
                    continue;
                }
                let side = (dy * px - dx * py) / length.sqrt();
                let grain = noise(x * 121., y * 234.);
                let light = smooth(-width, width, side);
                bark = gain(
                    mix([0.61, 0.55, 0.48], [1.15, 1.04, 0.88], 0.66 + 0.31 * light),
                    0.92 + 0.13 * grain,
                );
                coverage = ink;
            }
        }
        mix(color, bark, coverage)
    }

    fn landscape(&self, base: Color, x: f32, y: f32) -> Color {
        if y >= 0.738 && base[0] > base[2] + 0.09 {
            let flow = x * 7.0 + y * 6.2 - self.time * 0.86;
            let stems = noise(x * 74., y * 51.);
            let gust = smooth(0.36, 0.96, 0.48 + 0.52 * flow.sin())
                * (0.48 + 0.52 * stems)
                * smooth(0.748, 0.95, y);
            return mix(base, [1.00, 0.75, 0.41], gust * 0.20);
        }
        let cool = smooth(0.024, 0.093, base[2] - base[0]);
        let bank = noise(x * 4.1 - self.time * 0.13, y * 6.6 + 22.3);
        let cloud = smooth(0.32, 0.74, bank);
        let sky = smooth(0.35, 0.67, luminance(base)) * (1. - smooth(0.19, 0.34, y));
        let mountain = gain(base, 0.93 + cool * (0.05 + cloud * 0.17));
        mix(mountain, [0.71, 0.77, 0.82], sky * cloud * cool * 0.20)
    }
}

#[cfg(test)]
mod tests {
    use super::{CROWN, DROP, Tree, growth_clock};
    use crate::scene::WHITE_TREE_DURATION;

    #[test]
    fn the_crown_grows_only_from_existing_bark() {
        let mut tree = Tree::new(119, 70);
        assert!(tree.branches.iter().all(|branch| branch.progress == 0.));
        let mut concurrent_growth = false;
        for tick in 0..(DROP * 15.) as usize {
            tree.update(tick as f32 / 15.);
            for (i, branch) in tree.branches.iter().enumerate().skip(1) {
                if branch.progress > 0. {
                    let shoot = &CROWN[i];
                    let parent = &tree.branches[shoot.parent];
                    assert_eq!(branch.curve[0], parent.at(shoot.attachment));
                    assert!(parent.progress >= shoot.attachment);
                    concurrent_growth |= parent.progress < 1.;
                }
            }
        }
        assert!(concurrent_growth);
        assert!(
            tree.branches
                .iter()
                .filter(|branch| branch.progress == 1.)
                .count()
                > 47
        );
        assert!(growth_clock(5.) - growth_clock(4.) > growth_clock(2.) - growth_clock(1.));
    }

    #[test]
    fn the_trunk_remains_and_the_whole_scene_stops_after_the_drop() {
        let mut tree = Tree::new(119, 70);
        assert!(DROP + tree.debris.settle_time() <= WHITE_TREE_DURATION);
        let point = [0.625, 0.66];
        let trunk = [0.81, 0.77, 0.66];
        tree.update(WHITE_TREE_DURATION);
        let end = tree.shade(trunk, point[0], point[1]);
        assert_eq!(end, tree.landscape(trunk, point[0], point[1]));
        let settled_at = tree.time;
        tree.update(30.);
        assert_eq!(tree.time, settled_at);
        assert_eq!(end, tree.shade(trunk, point[0], point[1]));
    }
}
