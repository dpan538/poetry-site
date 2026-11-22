use super::Color;

pub(super) fn plate_pixel(source: &[u8], u: f32, v: f32) -> Color {
    let x = ((u * 340.) as usize).min(339);
    let y = ((v * 200.) as usize).min(199);
    let p = (y * 340 + x) * 3;
    std::array::from_fn(|i| source[p + i] as f32 / 255.)
}

pub(super) fn rgba(
    pixels: &[u8],
    width: usize,
    height: usize,
    u: f32,
    v: f32,
) -> Option<(Color, f32)> {
    if !(0. ..1.).contains(&u) || !(0. ..1.).contains(&v) {
        return None;
    }
    let x = (u * width as f32 - 0.5).clamp(0., (width - 1) as f32);
    let y = (v * height as f32 - 0.5).clamp(0., (height - 1) as f32);
    let (left, top) = (x as usize, y as usize);
    let (right, bottom) = ((left + 1).min(width - 1), (top + 1).min(height - 1));
    let (dx, dy) = (x.fract(), y.fract());
    let mut alpha = 0.;
    let mut rgb = [0.; 3];
    for (sx, sy, w) in [
        (left, top, (1. - dx) * (1. - dy)),
        (right, top, dx * (1. - dy)),
        (left, bottom, (1. - dx) * dy),
        (right, bottom, dx * dy),
    ] {
        let at = (sy * width + sx) * 4;
        let coverage = w * pixels[at + 3] as f32 / 255.;
        alpha += coverage;
        for i in 0..3 {
            rgb[i] += coverage * pixels[at + i] as f32 / 255.;
        }
    }
    if alpha < 0.001 {
        return None;
    }
    Some((rgb.map(|channel| channel / alpha), alpha))
}
