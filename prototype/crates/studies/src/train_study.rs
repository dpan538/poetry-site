use crate::scene::{Color, gain, hash, line, luminance, mix, noise, polygon, rect, smooth};

const TREES: &[(f32, f32, f32)] = &[
    (0.16, 0.046, 0.038),
    (0.987, 0.042, -0.032),
    (1.227, 0.022, 0.014),
    (1.66, 0.038, 0.064),
    (2.23, 0.045, -0.047),
    (2.43, 0.023, 0.018),
    (2.96, 0.030, -0.056),
    (3.48, 0.051, 0.010),
    (3.94, 0.034, -0.052),
    (4.39, 0.040, 0.038),
    (4.82, 0.023, -0.040),
    (5.06, 0.042, 0.020),
    (5.54, 0.031, 0.051),
    (5.92, 0.044, -0.036),
    (6.17, 0.023, 0.019),
    (6.65, 0.043, 0.025),
    (7.12, 0.031, -0.035),
    (7.29, 0.041, 0.029),
    (7.96, 0.040, -0.047),
    (8.38, 0.021, 0.020),
    (8.72, 0.039, -0.056),
];

pub(super) struct Train {
    time: f32,
    house: f32,
    foreground: f32,
    middle: f32,
}

impl Train {
    pub(super) fn at(seconds: f32) -> Self {
        Self {
            time: seconds,
            house: 0.820 - seconds * 0.104,
            foreground: seconds * 0.65,
            middle: seconds * 0.27,
        }
    }

    pub(super) fn shade(&self, mut color: Color, x: f32, y: f32) -> Color {
        let frame = window(x, y, [0.078, 0.065, 0.929, 0.89], 0.081);
        if frame < 1. {
            let opening = window(x, y, [0.065, 0.047, 0.942, 0.919], 0.087);
            let housing = [0.044, 0.050, 0.057];
            let metal = [0.156, 0.184, 0.199];
            color = mix(
                mix(housing, metal, opening),
                gain(landscape(color, x, y, self.time), 0.71),
                frame,
            );
            let sill = rect(x, y, [0.081, 0.917, 0.92, 0.926]);
            color = mix(color, [0.32, 0.30, 0.255], sill * 0.61);
            let latch = rect(x, y, [0.438, 0.894, 0.557, 0.919]);
            return mix(color, [0.40, 0.416, 0.386], latch * 0.73);
        }
        color = landscape(color, x, y, self.time);
        let nx = x + self.middle;
        let mid_edge = 0.812 + 0.053 * noise(nx * 18., 7.);
        let grass = smooth(mid_edge, mid_edge + 0.043, y);
        let warm = gain([0.60, 0.467, 0.25], 0.86 + 0.36 * noise(nx * 57., y * 92.));
        color = mix(color, warm, grass * 0.59);
        color = house(color, x - self.house, y);
        let near_x = x + self.foreground;
        for (i, &(trunk, width, lean)) in TREES.iter().enumerate() {
            if (near_x - trunk).abs() > 0.25 {
                continue;
            }
            let axis = trunk + lean * (0.8 - y) + 0.016 * (y * 5.0 + i as f32).sin();
            let taper = width * (0.30 + 0.65 * y);
            let bark = 1. - smooth(taper * 0.74, taper + 0.003, (near_x - axis).abs());
            let grain = 0.85 + 0.2 * noise((near_x - axis) * 395., y * 31.);
            let rim = smooth(-taper, taper, near_x - axis);
            let tree = gain(
                mix([0.105, 0.112, 0.092], [0.295, 0.239, 0.154], rim),
                grain,
            );
            let left = -0.13 - 0.075 * hash(i as i32, 3);
            let right = 0.11 + 0.078 * hash(i as i32, 5);
            let limbs = line(
                near_x,
                y,
                [trunk + lean * 0.28, 0.52],
                [trunk + left, 0.18],
                width * 0.20,
            )
            .max(line(
                near_x,
                y,
                [trunk + lean * 0.40, 0.39],
                [trunk + right, 0.085],
                width * 0.17,
            ));
            color = mix(color, tree, bark.max(limbs));
        }
        // The recessed glass shades the view at the seal, not across the landscape.
        let rim = window(x, y, [0.104, 0.095, 0.903, 0.857], 0.067);
        let edge = (x - 0.078).min(0.929 - x).min(y - 0.065).min(0.89 - y);
        gain(color, 0.63 + 0.34 * smooth(0., 0.043, edge) + 0.03 * rim)
    }
}

fn landscape(base: Color, x: f32, y: f32, time: f32) -> Color {
    let horizon = smooth(0.554, 0.602, y);
    let field = gain(base, 1.18 + 0.12 * smooth(0.61, 0.83, y));
    if horizon == 1. {
        return field;
    }
    let clear = mix(
        [0.255, 0.519, 0.838],
        [0.59, 0.761, 0.884],
        smooth(0.04, 0.59, y),
    );
    let cloud = 1. - smooth(0.018, 0.172, base[2] - base[0]);
    let value = luminance(base);
    let shaded = mix(
        [0.477, 0.578, 0.747],
        [0.699, 0.758, 0.834],
        smooth(0.49, 0.64, value),
    );
    let body = mix(shaded, [0.903, 0.845, 0.792], smooth(0.63, 0.78, value));
    let silver = mix(body, [1.11, 1.002, 0.865], smooth(0.77, 0.91, value));
    let mut sky = mix(clear, silver, cloud);

    // A thin lower veil advects beneath the fixed cumulus silhouettes.
    let filament_y = y + 0.018 * noise(x * 7. - time * 0.13, 3.8);
    let veil = smooth(0.386, 0.423, filament_y) * (1. - smooth(0.455, 0.495, filament_y));
    let fine = smooth(0.46, 0.82, noise(x * 14. - time * 0.29, y * 76.));
    sky = mix(
        sky,
        [0.729, 0.807, 0.907],
        veil * fine * (0.18 + 0.22 * cloud),
    );
    let glints = cloud * smooth(0.53, 0.63, value) * (1. - smooth(0.77, 0.86, value));
    if glints > 0.01 {
        sky = mix(
            sky,
            [1.05, 1.035, 0.973],
            glints * drift_grains(x, y, time) * 0.48,
        );
    }
    mix(sky, field, horizon)
}

fn drift_grains(x: f32, y: f32, time: f32) -> f32 {
    let row = (y * 72.).floor() as i32;
    let rate = 0.009 + 0.010 * hash(row, 25);
    let column = (x - rate * time) * 98.;
    let cell = column.floor() as i32;
    let density = hash(cell, row + 72);
    if density < 0.939 {
        return 0.;
    }
    let dx = column.fract() - (0.30 + 0.40 * hash(cell + 11, row));
    let dy = (y * 72.).fract() - (0.30 + 0.40 * hash(cell + 29, row + 10));
    let bead = 1. - smooth(0.07, 0.33, dx.hypot(dy * 1.16));
    let shimmer = 0.79 + 0.21 * (time * 2.50 + density * 37.).sin();
    bead * shimmer
}

fn window(x: f32, y: f32, [left, top, right, bottom]: [f32; 4], radius: f32) -> f32 {
    let dx = (left + radius - x).max(x - right + radius).max(0.);
    let dy = (top + radius - y).max(y - bottom + radius).max(0.);
    1. - smooth(radius - 0.004, radius + 0.004, dx.hypot(dy))
}

fn house(mut color: Color, x: f32, y: f32) -> Color {
    if !(-0.15..0.18).contains(&x) || !(0.532..0.795).contains(&y) {
        return color;
    }
    let body = polygon(
        x,
        y,
        &[
            [-0.097, 0.663],
            [-0.037, 0.573],
            [0.036, 0.649],
            [0.036, 0.767],
            [-0.097, 0.762],
        ],
    );
    let side = polygon(
        x,
        y,
        &[
            [0.036, 0.649],
            [0.149, 0.628],
            [0.149, 0.746],
            [0.036, 0.767],
        ],
    );
    color = mix(color, [0.78, 0.69, 0.49], side);
    color = mix(color, [0.97, 0.90, 0.70], body);
    let chimney = rect(x, y, [0.066, 0.551, 0.090, 0.622]);
    color = mix(color, [0.56, 0.31, 0.22], chimney);
    let roof = polygon(
        x,
        y,
        &[
            [-0.039, 0.562],
            [0.076, 0.547],
            [0.17, 0.631],
            [0.04, 0.653],
        ],
    );
    color = mix(
        color,
        gain([0.69, 0.27, 0.132], 0.83 + 0.24 * noise(x * 47., y * 70.)),
        roof,
    );
    let eave = line(x, y, [-0.109, 0.660], [-0.039, 0.562], 0.005).max(line(
        x,
        y,
        [-0.039, 0.562],
        [0.041, 0.655],
        0.005,
    ));
    color = mix(color, [0.37, 0.20, 0.13], eave);
    let glazing = rect(x, y, [-0.077, 0.671, -0.047, 0.706])
        .max(rect(x, y, [0.067, 0.658, 0.096, 0.697]))
        .max(rect(x, y, [0.107, 0.652, 0.131, 0.690]));
    color = mix(color, [0.21, 0.29, 0.28], glazing);
    color = mix(
        color,
        [0.31, 0.24, 0.15],
        rect(x, y, [-0.032, 0.688, 0.001, 0.764]),
    );
    mix(
        color,
        [0.44, 0.36, 0.25],
        line(x, y, [-0.093, 0.766], [0.149, 0.751], 0.004),
    )
}
