use super::{COLS, Motion, ROWS, Study, luminance, sprite::Sprite};

const MOONLIT_GROUND: &[u8; COLS * ROWS * 3] =
    include_bytes!(concat!(env!("OUT_DIR"), "/moonlit-ground.rgb"));
include!(concat!(env!("OUT_DIR"), "/moonlit-sprites.rs"));

pub(super) fn ease(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

pub(super) fn exposure(motion: Motion, seconds: f32) -> f32 {
    let period = match motion {
        Motion::Flare | Motion::Winter | Motion::Screen | Motion::Moonlight => 18.0,
        Motion::Wivenhoe | Motion::Clouds => return 0.0,
    };
    let phase = (seconds / period).fract();
    ease((phase - 0.08) / 0.35) * (1.0 - ease((phase - 0.56) / 0.34))
}

fn mix(base: [f32; 3], target: [f32; 3], amount: f32) -> [f32; 3] {
    std::array::from_fn(|i| base[i] + (target[i] - base[i]) * amount)
}

fn band(value: f32, from: f32, to: f32, feather: f32) -> f32 {
    ease((value - from) / feather) * ease((to - value) / feather)
}

fn roofline(x: f32) -> f32 {
    const HORIZON: &[(f32, f32)] = &[
        (0.0, 0.558),
        (0.10, 0.570),
        (0.29, 0.608),
        (0.36, 0.614),
        (0.65, 0.552),
        (0.82, 0.524),
        (0.95, 0.526),
        (1.0, 0.522),
    ];
    HORIZON
        .windows(2)
        .find_map(|p| {
            let [(a, y0), (b, y1)] = [p[0], p[1]];
            (x <= b).then(|| y0 + (y1 - y0) * (x - a) / (b - a))
        })
        .unwrap_or(0.522)
}

fn chapel(x: f32, y: f32) -> f32 {
    // The cross, front gable and white walls form a fixed matte.
    let slope = if x < 0.506 { 1.84 } else { 1.71 };
    let roof = 0.227 + (x - 0.506).abs() * slope;
    let gable =
        band(x, 0.345, 0.674, 0.012) * ease((y - roof + 0.008) / 0.020) * ease((0.794 - y) / 0.024);
    let cross = band(x, 0.496, 0.516, 0.009) * band(y, 0.183, 0.239, 0.018);
    gable.max(cross)
}

fn moon_sky(x: f32, y: f32) -> f32 {
    let horizon = 0.49 + x * 0.26 - 0.10 * (1.0 - x * 4.0).max(0.0);
    let trees = 0.635 - y * 0.36;
    ease((horizon - y) / 0.09) * ease((trees - x) / 0.09)
}

pub(super) fn region(motion: Motion, x: f32, y: f32) -> f32 {
    match motion {
        Motion::Flare => {
            let r2 = ((x - 0.474) / 0.305).powi(2) + ((y - 0.511) / 0.307).powi(2);
            ease(1.0 - r2)
        }
        Motion::Winter => 1.0 - chapel(x, y),
        Motion::Screen => band(x, 0.246, 0.786, 0.013) * band(y, 0.355, 0.675, 0.016),
        Motion::Moonlight => moon_sky(x, y).max(f32::from(u8::from(SPRITES[0].near(
            x * (COLS - 1) as f32 * 4.0 + 1.5,
            y * (ROWS - 1) as f32 * 4.0 + 1.5,
        )))),
        Motion::Wivenhoe | Motion::Clouds => 0.0,
    }
}

pub(super) fn photograph(
    motion: Motion,
    source: [f32; 3],
    x: f32,
    y: f32,
    light: f32,
    amount: f32,
) -> [f32; 3] {
    let [r, g, b] = source;
    match motion {
        Motion::Flare => {
            let heat = amount * ease((luminance(source) - 0.44) / 0.43);
            let core = ease(1.0 - ((x - 0.474).powi(2) + (y - 0.511).powi(2)) / 0.0024);
            let dim = mix(source, [r * 0.65, g * 0.49, b * 0.31], heat);
            let faint = mix(dim, [1.0, 0.81, 0.24], 0.58 * core);
            let hot = mix(source, [1.0, 0.986, 0.83], heat * heat * 0.64);
            // The source blooms out from the centre. The outer rays follow the core.
            let radius = ((x - 0.474).powi(2) + (y - 0.511).powi(2)).sqrt();
            let arrive = ease((light - radius * 1.1) / (1.0 - radius * 1.1).max(0.1));
            mix(faint, hot, arrive)
        }
        Motion::Winter => {
            let horizon = roofline(x);
            let sky = ease((horizon - y + 0.010) / 0.024);
            let ground = ease((y - horizon - 0.008) / 0.045);
            let dark_blue = [r * 0.29, g * 0.38, b * 0.50];
            let night = mix(source, dark_blue, sky * light * amount);
            let brightness = luminance(source);
            // Dark cast shadows remain visible as blue shade on the snow.
            let snow = 0.69 + 0.29 * brightness;
            let white = [snow * 0.97, snow, snow * 1.015];
            let cover = ease((brightness - 0.045) / 0.17);
            mix(night, white, ground * light * amount * cover)
        }
        Motion::Screen => {
            let emission = ease((g - b - 0.16) / 0.20) * ease((g - 0.41) / 0.24);
            let off = [0.040 + r * 0.012, 0.039 + g * 0.012, 0.040 + b * 0.012];
            let on = mix(source, [1.0, 0.952, 0.08], 0.28);
            let reflection = band(x, 0.03, 0.98, 0.18) * band(y, 0.615, 0.785, 0.047);
            let red_cast = (r - (g + b) * 0.5).max(0.0);
            let ambient = [(r - 0.64 * red_cast).max(0.0), g * 0.93, b * 0.96];
            let ground = mix(source, ambient, reflection * (1.0 - light));
            mix(ground, mix(off, on, light), amount * emission)
        }
        _ => source,
    }
}

pub(super) fn moonlit(
    source: [f32; 3],
    column: f32,
    row: f32,
    seconds: f32,
    light: f32,
) -> [f32; 3] {
    let x = column * 4.0 + 1.5;
    let y = row * 4.0 + 1.5;
    let figure = &SPRITES[0];
    if figure.near(x, y) {
        // The figure leans from the waist. The legs and the companion stay planted.
        let phase = (seconds / 18.0).fract();
        let turn = ease((phase - 0.14) / 0.16) * (1.0 - ease((phase - 0.54) / 0.22));
        let [r, g, b, alpha] = figure.sample(x, y, -0.25 * turn);
        let ground = Study::sample_pixels(MOONLIT_GROUND, column, row);
        return std::array::from_fn(|i| ground[i] * (1.0 - alpha) + [r, g, b][i]);
    }
    let weight = moon_sky(column / (COLS - 1) as f32, row / (ROWS - 1) as f32);
    let factor = 1.0 + (light - 0.56) * 0.66 * weight;
    source.map(|v| (v * factor).min(1.0))
}
