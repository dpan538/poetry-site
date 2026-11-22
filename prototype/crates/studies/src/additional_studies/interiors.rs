use super::super::{gain, hash, mix, noise, smooth, tint};
use super::paint::{Painting, shifted};

pub(super) fn cat_background(p: &mut Painting) {
    p.wash(|[x, y], _| {
        let light = smooth(1.0, 0.08, x) * (0.75 + 0.25 * (1. - y));
        gain(
            mix(tint(0x49376A), tint(0xDB9277), light),
            0.96 + 0.07 * noise(x * 38., y * 47.),
        )
    });
    p.rect([0.118, 0.046, 0.883, 0.822], 0xE8B298);
    p.rect([0.140, 0.074, 0.858, 0.771], 0x653E67);
    p.wash(|[x, y], c| {
        if !(0.156..0.842).contains(&x) || !(0.105..0.766).contains(&y) {
            return c;
        }
        let evening = smooth(0.12, 0.76, y);
        mix(tint(0x3346A3), tint(0x7760C5), evening)
    });
    p.glow([0.263, 0.225], [0.095, 0.152], 0xB99AEF, 0.62);
    p.oval([0.262, 0.225], [0.028, 0.047], 0xFFEEC0);
    for i in 0..18 {
        let [x, top, edge] = building(i);
        p.rect(
            [x, top, edge, 0.766],
            if i % 2 == 0 { 0x40568C } else { 0x3C3981 },
        );
        p.line([x, top], [edge, top], 0.002, 0x748BCD);
    }
    p.rect([0.146, 0.416, 0.851, 0.445], 0x9F6C84);
    p.rect([0.490, 0.103, 0.509, 0.770], 0x9F6C84);
    p.line([0.493, 0.103], [0.493, 0.770], 0.003, 0xE2A694);
    p.line([0.154, 0.417], [0.842, 0.417], 0.004, 0xF5B89C);
    p.polygon(
        &[
            [0.133, 0.758],
            [0.865, 0.758],
            [0.927, 0.846],
            [0.072, 0.846],
        ],
        0xCA856C,
    );
    p.rect([0.072, 0.846, 0.927, 0.875], 0x764D6A);
    p.line([0.073, 0.844], [0.927, 0.844], 0.003, 0xF8BD95);
    p.glow([0.533, 0.790], [0.363, 0.055], 0x4B305A, 0.76);
}

fn building(i: i32) -> [f32; 3] {
    let x = 0.157 + i as f32 * 0.038;
    let w = 0.021 + 0.022 * hash(i, 33);
    let top = 0.437 + 0.16 * hash(i, 31);
    [x, top, (x + w).min(0.84)]
}

pub(super) fn cat(p: &mut Painting, t: f32) {
    for i in 0..18 {
        let [x, top, edge] = building(i);
        for row in 0..4 {
            let y = top + 0.022 + row as f32 * 0.032;
            let seed = hash(i, row + 72);
            if y >= 0.749 || seed < 0.18 || (0.474..0.509).contains(&x) {
                continue;
            }
            let change = 0.9 + 11. * hash(i, row + 138);
            let switch = smooth(change, change + 0.34, t);
            let flicker = (t * 1.65 + seed * 19.).sin() * 0.07;
            let power = if seed < 0.51 {
                0.12 + 0.87 * switch
            } else if seed > 0.80 {
                0.96 - 0.81 * switch
            } else {
                0.87
            };
            let glow = (power + flicker).clamp(0., 1.);
            let lamp = [0xFFB778, 0xFFD5A1, 0x91C4F4, 0xDCA5E7][(i + row * 3) as usize % 4];
            let left = x + 0.005;
            let right = (edge - 0.004).min(left + 0.012);
            p.glow(
                [0.5 * (left + right), y + 0.008],
                [0.014, 0.018],
                lamp,
                glow * 0.59,
            );
            p.translucent(
                &[[left, y], [right, y], [right, y + 0.015], [left, y + 0.015]],
                lamp,
                glow,
            );
        }
        if i % 5 == 1 {
            let pulse =
                (1. - smooth(0.04, 0.14, (t * 0.82 + 0.32 * i as f32).fract())) * 0.73 + 0.16;
            p.glow([x + 0.014, top - 0.004], [0.010, 0.015], 0xFF6675, pulse);
        }
    }
    let travel = ((t - 1.15) / 10.3).clamp(0., 1.);
    let x = 0.203 + travel * 0.588;
    let y = 0.339 - 0.106 * travel;
    let visibility = smooth(0., 0.12, travel) * (1. - smooth(0.88, 1., travel));
    let q = |dx, dy| [x + dx, y + dy - dx * 0.16];
    p.translucent(
        &[
            q(-0.046, -0.003),
            q(-0.022, -0.009),
            q(0.031, -0.009),
            q(0.046, 0.001),
            q(0.036, 0.011),
            q(-0.039, 0.009),
        ],
        0xBCC6EA,
        visibility,
    );
    p.translucent(
        &[
            q(-0.012, -0.004),
            q(-0.026, -0.033),
            q(-0.014, -0.033),
            q(0.009, -0.007),
            q(0.005, 0.005),
            q(-0.008, 0.036),
            q(-0.024, 0.035),
        ],
        0x93A6D8,
        visibility,
    );
    p.translucent(
        &[
            q(-0.046, -0.001),
            q(-0.055, -0.024),
            q(-0.043, -0.024),
            q(-0.029, -0.006),
        ],
        0xA3B4E3,
        visibility,
    );
    p.glow(
        q(-0.021, 0.033),
        [0.013, 0.017],
        0xFF5E77,
        visibility * 0.90,
    );
    p.glow(
        q(-0.024, -0.031),
        [0.012, 0.015],
        0x7CFFCD,
        visibility * 0.84,
    );
    let strobe = 1. - smooth(0.035, 0.103, ((t + 0.09) * 1.13).fract());
    p.glow(
        q(0.030, -0.007),
        [0.016, 0.023],
        0xFFF1CF,
        visibility * (0.23 + 0.77 * strobe),
    );
    // The near mullion covers the plane as it crosses between the two panes.
    p.rect([0.490, 0.103, 0.509, 0.770], 0x9F6C84);
    p.line([0.493, 0.103], [0.493, 0.770], 0.003, 0xE2A694);
    sleeping_cat(p);
}

fn sleeping_cat(p: &mut Painting) {
    let head = [0.365, 0.704];
    let mut tail = [0.674, 0.736];
    for i in 1..25 {
        let s = i as f32 / 24.;
        let point = [
            0.674 + 0.200 * s,
            0.736 + 0.084 * (1. - (2.5 * s).cos()) * 0.48 - 0.075 * s.powi(5),
        ];
        let r = 0.028 - 0.014 * s;
        p.line(tail, point, r + 0.003, 0xD8A5A0);
        p.line(
            [tail[0], tail[1] + 0.006],
            [point[0], point[1] + 0.006],
            r,
            0x373052,
        );
        tail = point;
    }
    p.oval([0.386, 0.778], [0.073, 0.030], 0xE8B7A4);
    p.oval([0.353, 0.787], [0.051, 0.021], 0xFFF0CA);
    p.oval([0.492, 0.718], [0.131, 0.090], 0xFBE8C1);
    p.oval([0.607, 0.696], [0.122, 0.106], 0xFCDEB3);
    p.polygon(
        &[
            [0.491, 0.632],
            [0.548, 0.620],
            [0.579, 0.646],
            [0.567, 0.711],
            [0.524, 0.722],
            [0.489, 0.687],
        ],
        0x423452,
    );
    p.oval([0.633, 0.696], [0.083, 0.088], 0x453652);
    p.glow([0.653, 0.665], [0.085, 0.071], 0x97697E, 0.62);
    p.oval([0.648, 0.751], [0.061, 0.041], 0xFFE2B9);
    p.line([0.594, 0.765], [0.624, 0.784], 0.005, 0xD5988B);
    p.oval([head[0] + 0.068, 0.682], [0.066, 0.083], 0xF5DBB9);
    let q = |x, y| shifted(head, [x, y], -0.06);
    p.oval(q(0., 0.), [0.078, 0.086], 0xF5DABD);
    p.oval(q(0.014, -0.021), [0.066, 0.070], 0x503858);
    p.oval(q(-0.023, 0.033), [0.054, 0.053], 0xFFE8CB);
    p.polygon(
        &[
            q(-0.052, -0.052),
            q(-0.022, -0.069),
            q(-0.021, 0.014),
            q(-0.051, 0.033),
            q(-0.070, 0.012),
        ],
        0xFFF1C9,
    );
    for (dx, forward) in [(-0.045, -1.), (0.039, 1.)] {
        let ear_x = dx + forward * 0.013;
        let top = -0.109;
        p.polygon(
            &[q(dx - 0.027, -0.045), q(ear_x, top), q(dx + 0.027, -0.051)],
            0x9C667E,
        );
        p.polygon(
            &[
                q(dx - 0.014, -0.052),
                q(ear_x, top + 0.018),
                q(dx + 0.016, -0.056),
            ],
            0xF2A8A2,
        );
        p.line(q(dx - 0.026, -0.047), q(ear_x, top), 0.004, 0xE7BBAD);
    }
    for x in [-0.041, 0.024] {
        p.line(q(x - 0.013, 0.001), q(x + 0.012, 0.006), 0.0034, 0x4C394F);
    }
    p.polygon(
        &[q(-0.033, 0.031), q(-0.008, 0.029), q(-0.020, 0.046)],
        0xA95F72,
    );
    p.line(q(-0.020, 0.043), q(-0.029, 0.058), 0.002, 0x9B5F73);
    for i in 0..3 {
        let y = i as f32 * 0.011;
        p.line(
            q(-0.044, 0.044 + y),
            q(-0.106, 0.029 + y * 1.65),
            0.0018,
            0xEFD4BD,
        );
        p.line(
            q(0.012, 0.048 + y),
            q(0.078, 0.040 + y * 1.58),
            0.0015,
            0xEFD4BD,
        );
    }
}

const WINDOW: [f32; 4] = [0.643, 0.154, 0.887, 0.585];

pub(super) fn candle_background(p: &mut Painting) {
    p.wash(|[x, y], _| {
        // Large charcoal planes establish the recess. Their low luminance
        // leaves empty cells in the shadows instead of a texture of colored ink.
        let plaster = 0.035 + 0.055 * smooth(0.84, 0.11, x) * (1. - 0.34 * y);
        [plaster * 1.03, plaster, plaster * 0.94]
    });
    p.polygon(
        &[[0.544, 0.], [1., 0.], [1., 0.715], [0.548, 0.632]],
        0x191C20,
    );
    p.line([0.547, 0.002], [0.547, 0.630], 0.0035, 0x292B2C);
    p.polygon(
        &[
            [0.614, 0.119],
            [0.922, 0.118],
            [0.922, 0.626],
            [0.609, 0.613],
        ],
        0x24282C,
    );
    p.polygon(
        &[
            [0.620, 0.124],
            [0.646, 0.154],
            [0.646, 0.586],
            [0.617, 0.606],
        ],
        0x343637,
    );
    p.polygon(
        &[
            [0.617, 0.605],
            [0.647, 0.584],
            [0.887, 0.583],
            [0.932, 0.621],
            [0.913, 0.636],
            [0.613, 0.621],
        ],
        0x353838,
    );
    p.line([0.643, 0.155], [0.888, 0.155], 0.006, 0x444548);
    p.rect(WINDOW, 0x23364F);
    p.oval([0.804, 0.249], [0.020, 0.034], 0xD8DDE0);
    p.glow([0.805, 0.246], [0.085, 0.105], 0x8DABD0, 0.32);
    p.polygon(
        &[
            [0.643, 0.398],
            [0.705, 0.351],
            [0.752, 0.385],
            [0.800, 0.365],
            [0.855, 0.384],
            [0.887, 0.372],
            [0.887, 0.585],
            [0.643, 0.585],
        ],
        0x192831,
    );
    p.polygon(
        &[
            [0.643, 0.501],
            [0.704, 0.448],
            [0.768, 0.459],
            [0.845, 0.431],
            [0.887, 0.464],
            [0.887, 0.585],
            [0.643, 0.585],
        ],
        0x213F51,
    );
    p.polygon(
        &[
            [0.643, 0.530],
            [0.720, 0.483],
            [0.762, 0.472],
            [0.802, 0.490],
            [0.887, 0.508],
            [0.887, 0.585],
            [0.643, 0.585],
        ],
        0x152635,
    );
    p.rect([0.757, 0.154, 0.766, 0.585], 0x171A1C);
    p.rect([0.643, 0.355, 0.887, 0.368], 0x1D2429);

    // The near table is angled across the frame, with a warm top, a darker
    // front lip and the candle and vessel sitting on the same projected plane.
    p.polygon(
        &[
            [0., 0.700],
            [0.383, 0.671],
            [0.751, 0.738],
            [1., 0.855],
            [1., 1.],
            [0., 1.],
        ],
        0x32271F,
    );
    p.polygon(
        &[[0., 0.872], [0.393, 0.827], [1., 0.924], [1., 1.], [0., 1.]],
        0x181716,
    );
    p.line([0., 0.873], [0.393, 0.829], 0.005, 0x5F4831);
    p.line([0.393, 0.829], [1., 0.924], 0.0036, 0x493825);
    for i in 0..4 {
        let y = i as f32 * 0.024;
        p.line([0.0, 0.746 + y], [0.75, 0.765 + y], 0.0018, 0x4F3C2A);
    }
    p.polygon(
        &[
            [0.474, 0.780],
            [0.647, 0.739],
            [0.737, 0.793],
            [0.554, 0.847],
        ],
        0x3B322A,
    );
    p.line([0.485, 0.781], [0.649, 0.742], 0.0036, 0x6A5642);

    p.glow([0.195, 0.736], [0.09, 0.033], 0x131415, 0.74);
    p.oval([0.198, 0.707], [0.033, 0.036], 0x343A39);
    p.oval([0.198, 0.723], [0.029, 0.012], 0x2B2D2B);
    p.rect([0.188, 0.671, 0.211, 0.703], 0x4B4C41);
    p.oval([0.200, 0.672], [0.012, 0.006], 0x81745D);
    p.line([0.200, 0.674], [0.215, 0.508], 0.0039, 0x4D4839);
    p.line([0.208, 0.588], [0.185, 0.540], 0.003, 0x635740);
    p.oval([0.214, 0.509], [0.006, 0.009], 0x856747);
    p.oval([0.184, 0.540], [0.006, 0.008], 0x856747);

    p.glow([0.379, 0.783], [0.167, 0.051], 0x171713, 0.80);
    p.oval([0.359, 0.746], [0.112, 0.038], 0x634931);
    p.oval([0.357, 0.737], [0.105, 0.030], 0x997144);
    p.oval([0.357, 0.736], [0.084, 0.021], 0x65472C);
    p.rect([0.329, 0.562, 0.388, 0.733], 0x847559);
    p.oval([0.358, 0.730], [0.030, 0.011], 0x716343);
    p.rect([0.330, 0.574, 0.345, 0.723], 0xAE9471);
    p.line([0.377, 0.572], [0.379, 0.642], 0.004, 0xCEB58A);
    p.oval([0.359, 0.562], [0.030, 0.011], 0xDED0AB);
    p.oval([0.360, 0.562], [0.019, 0.006], 0x87734E);
    p.line([0.358, 0.557], [0.357, 0.544], 0.004, 0x2D2921);
}

pub(super) fn candle(p: &mut Painting, t: f32) {
    let flame = 1. - smooth(8.61, 9.02, t);
    let lean = smooth(8.30, 8.69, t) * 0.059;
    let sway = 0.006 * (4.6 * t).sin() + 0.003 * (8.2 * t).sin();
    let tip_x = 0.356 + (lean + sway) * flame;
    let height = (0.087 + 0.007 * (t * 6.5).sin()) * flame;
    let source_x = 0.356 + (tip_x - 0.356) * 0.45;
    let source_y = 0.553 - height * 0.43;
    p.wash(|[x, y], c| {
        if (WINDOW[0]..WINDOW[2]).contains(&x) && (WINDOW[1]..WINDOW[3]).contains(&y) {
            if (0.757..0.766).contains(&x) || (0.355..0.368).contains(&y) {
                return c;
            }
            let cloud = 1.
                - smooth(
                    0.007,
                    0.027,
                    (y - 0.211 - 0.018 * (x * 14. - t * 0.55).sin()).abs(),
                );
            if y < 0.31 && (x - 0.804).abs() > 0.023 {
                return mix(c, tint(0x445973), cloud * 0.38);
            }
            if y > 0.482 {
                let shore = 1. - smooth(0.021, 0.105, (x - 0.804).abs());
                let wave = (y * 165. - t * 2.3 + 1.3 * (x * 42.).sin())
                    .sin()
                    .max(0.)
                    .powi(5);
                return mix(c, tint(0x7C9EBB), shore * wave * 0.41);
            }
            return c;
        }
        let distance = ((x - source_x) * 1.7).hypot((y - source_y) * 1.11);
        let pool = (1. - smooth(0.03, 0.48, distance)) * flame;
        let cast_y = source_y + (y - source_y) * (source_x - 0.207) / (source_x - x).max(0.001);
        let stem_x = 0.212 + (source_x - 0.356 - 0.139) * (1.58 + 0.45 * (y - 0.55)) + 0.139;
        let shadow = (1. - smooth(0.006, 0.020, (x - stem_x).abs()))
            * smooth(0.50, 0.525, cast_y)
            * (1. - smooth(0.646, 0.678, cast_y))
            * smooth(0.203, 0.162, x);
        let reflection = pool * (1. - 0.80 * shadow);
        // Light gives the wood and back wall a warm response, with charcoal
        // elsewhere. It does not add a second colored lighting field.
        let surface = if y > 0.66 { 0xAF7942 } else { 0x967044 };
        mix(
            gain(c, 0.72 + 0.30 * reflection),
            tint(surface),
            0.44 * reflection,
        )
    });
    if flame > 0.002 {
        let shade = [source_x, source_y];
        p.glow(shade, [0.074, 0.125], 0xE4A25E, flame * 0.27);
        let shape = |u: f32, v: f32| {
            [
                0.357 + u * flame.sqrt() + (tip_x - 0.357) * v,
                0.550 - height * v,
            ]
        };
        p.polygon(
            &[
                shape(-0.011, 0.01),
                shape(-0.019, 0.32),
                shape(-0.013, 0.62),
                shape(0., 1.),
                shape(0.011, 0.61),
                shape(0.017, 0.18),
                shape(0.007, -0.031),
            ],
            0xF2AD59,
        );
        p.oval(
            [0.355 + 0.21 * (tip_x - 0.356), 0.536 - height * 0.16],
            [0.009 * flame, 0.019 * flame],
            0xFFF0A5,
        );
    }
    if t > 8.82 {
        let age = t - 8.82;
        for i in 0..23 {
            let k = i as f32 / 22.;
            let drift = (age - 2.7 * k).max(0.);
            let x = 0.356 + 0.044 * drift + 0.028 * (drift * 5.).sin() * smooth(0., 0.56, drift);
            let y = 0.537 - drift * 0.121;
            let opacity = smooth(0.06, 0.28, drift) * (1. - smooth(1.14, 3.05, drift)) * 0.24;
            p.glow(
                [x, y],
                [0.007 + 0.011 * drift, 0.012 + 0.018 * drift],
                0xA0A7A5,
                opacity,
            );
        }
    }
}
