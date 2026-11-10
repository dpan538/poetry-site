use super::{COLS, ROWS, Study, light::ease};

fn contour(points: &[(f32, f32)], x: f32) -> f32 {
    points
        .windows(2)
        .find_map(|edge| {
            let [(a, y0), (b, y1)] = [edge[0], edge[1]];
            (x <= b).then(|| y0 + (y1 - y0) * (x - a) / (b - a))
        })
        .unwrap_or(points.last().unwrap().1)
}

fn sky_edge(x: f32) -> f32 {
    contour(
        &[
            (0.0, 0.43),
            (0.12, 0.44),
            (0.25, 0.31),
            (0.37, 0.20),
            (0.46, 0.105),
            (0.55, 0.0),
            (0.59, 0.09),
            (0.67, 0.15),
            (0.71, 0.12),
            (0.75, 0.072),
            (0.83, 0.11),
            (0.90, 0.21),
            (0.97, 0.145),
            (1.0, 0.14),
        ],
        x,
    )
}

fn fog_edge(x: f32) -> f32 {
    contour(
        &[
            (0.0, 0.413),
            (0.075, 0.420),
            (0.132, 0.432),
            (0.23, 0.525),
            (0.33, 0.59),
            (0.42, 0.64),
            (0.54, 0.64),
            (0.60, 0.52),
            (0.66, 0.38),
            (1.0, 0.0),
        ],
        x,
    )
}

fn behind_pines(x: f32, y: f32) -> f32 {
    let silhouette = contour(
        &[
            (0.0, 0.72),
            (0.28, 0.65),
            (0.37, 0.62),
            (0.46, 0.597),
            (0.54, 0.568),
            (0.64, 0.557),
            (1.0, 0.55),
        ],
        y,
    );
    ease((silhouette - x) / 0.034)
}

fn clouds(x: f32, y: f32) -> f32 {
    let sky = ease((sky_edge(x) - y) / 0.04);
    let bank = ease((fog_edge(x) - y) / 0.11)
        * ease((0.655 - x) / 0.115)
        * ease((y - 0.07 - (x - 0.27).max(0.0) * 0.86) / 0.15)
        * behind_pines(x, y);
    sky.max(bank)
}

pub fn region(x: f32, y: f32) -> f32 {
    let sky = ease((sky_edge(x) - y) / 0.025);
    let fog = ease((fog_edge(x) - y) / 0.05)
        * ease((0.667 - x) / 0.055)
        * ease((y - 0.075 - (x - 0.27).max(0.0) * 0.80) / 0.10)
        * behind_pines(x, y);
    sky.max(fog)
}

pub struct Layers {
    plate: Vec<[f32; 3]>,
    upper: Vec<[f32; 4]>,
    lower: Vec<[f32; 4]>,
}

impl Layers {
    pub fn new(source: &[u8]) -> Self {
        let mut layers = Self {
            plate: Vec::with_capacity(COLS * ROWS),
            upper: Vec::with_capacity(COLS * ROWS),
            lower: Vec::with_capacity(COLS * ROWS),
        };
        for row in 0..ROWS {
            let y = row as f32 / (ROWS - 1) as f32;
            for col in 0..COLS {
                let x = col as f32 / (COLS - 1) as f32;
                let color = Study::pixel(source, col, row);
                let cloud = clouds(x, y);
                let deep = ease((y - 0.22) / 0.24);
                let lower = cloud * deep;
                let upper = ((cloud - lower) / (1.0 - lower).max(0.001)).clamp(0.0, 1.0);
                let distance = Study::sample_pixels(source, col as f32, row as f32 + 24.0);
                let clear: [f32; 3] = std::array::from_fn(|i| {
                    let blue = [0.35, 0.49, 0.56][i];
                    let ground = distance[i] * 0.76 + [0.27, 0.31, 0.31][i] * 0.24;
                    blue + (ground - blue) * deep
                });
                layers.plate.push(std::array::from_fn(|i| {
                    color[i] + (clear[i] - color[i]) * cloud
                }));
                for (layer, alpha) in [(&mut layers.upper, upper), (&mut layers.lower, lower)] {
                    layer.push([color[0] * alpha, color[1] * alpha, color[2] * alpha, alpha]);
                }
            }
        }
        layers
    }

    pub fn compose(&self, column: f32, row: f32, seconds: f32) -> [f32; 3] {
        let phase = (seconds / 18.0).fract();
        let join = ease((phase - 0.64) / 0.36);
        let mut color = sample(&self.plate, column, row);
        for (layer, travel) in [(&self.upper, 19.0), (&self.lower, 13.0)] {
            let first = sample(layer, column - travel * phase, row);
            let second = sample(layer, column - travel * (phase - 1.0), row);
            let [r, g, b, alpha] =
                std::array::from_fn(|i| first[i] + (second[i] - first[i]) * join);
            for (under, over) in color.iter_mut().zip([r, g, b]) {
                *under = *under * (1.0 - alpha) + over;
            }
        }
        color
    }
}

fn sample<const N: usize>(pixels: &[[f32; N]], x: f32, y: f32) -> [f32; N] {
    let x = x.clamp(0.0, (COLS - 1) as f32);
    let y = y.clamp(0.0, (ROWS - 1) as f32);
    let (left, top) = (x.floor() as usize, y.floor() as usize);
    let right = (left + 1).min(COLS - 1);
    let bottom = (top + 1).min(ROWS - 1);
    std::array::from_fn(|i| {
        let a = pixels[top * COLS + left][i] * (1.0 - x.fract())
            + pixels[top * COLS + right][i] * x.fract();
        let b = pixels[bottom * COLS + left][i] * (1.0 - x.fract())
            + pixels[bottom * COLS + right][i] * x.fract();
        a * (1.0 - y.fract()) + b * y.fract()
    })
}
