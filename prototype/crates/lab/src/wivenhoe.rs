use super::{COLS, ROWS, Study, light::ease, sprite::Sprite};

const GROUND: &[u8; COLS * ROWS * 3] =
    include_bytes!(concat!(env!("OUT_DIR"), "/wivenhoe-ground.rgb"));

include!(concat!(env!("OUT_DIR"), "/wivenhoe-sprites.rs"));

fn source_xy(column: f32, row: f32) -> (f32, f32) {
    (column * 4.0 + 1.5, row * 4.0 + 1.5)
}

pub(super) fn moves(column: f32, row: f32) -> bool {
    let (x, y) = source_xy(column, row);
    row < 36.0 || SPRITES.iter().any(|head| head.near(x, y))
}

pub(super) fn compose(study: &Study, column: f32, row: f32, seconds: f32) -> [f32; 3] {
    let phase = (seconds / 18.0).fract();
    if row < 36.0 {
        let edge = (COLS - 1) as f32;
        let sky = |position: f32| {
            let x = position.rem_euclid(edge * 2.0);
            study.sample(x.min(2.0 * edge - x), row)
        };
        let cloud = sky(column - 21.6 * phase);
        let entering = sky(column - 21.6 * (phase - 1.0));
        let join = ease((phase - 0.65) / 0.35);
        let horizon = study.sample(column, row);
        let weight = ease((36.0 - row) / 6.0);
        return std::array::from_fn(|i| {
            let drift = cloud[i] + (entering[i] - cloud[i]) * join;
            horizon[i] + (drift - horizon[i]) * weight
        });
    }

    let (x, y) = source_xy(column, row);
    let mut color = Study::sample_pixels(GROUND, column, row);
    for (index, head) in SPRITES
        .iter()
        .enumerate()
        .filter(|(_, head)| head.near(x, y))
    {
        // Different grazing intervals. Feet, bodies and the fence stay registered.
        let time = phase * 18.0 - [0.0, 2.6, 5.0, 7.4][index];
        let lift = ease((time - 2.0) / 0.95) * (1.0 - ease((time - 4.1) / 1.55));
        let angle = [0.55, 0.60, 0.48, -0.58][index] * lift;
        let [r, g, b, alpha] = head.sample(x, y, angle);
        for (under, over) in color.iter_mut().zip([r, g, b]) {
            *under = *under * (1.0 - alpha) + over;
        }
    }
    color
}
