use super::super::{gain, hash, mix, noise, smooth, tint};
use super::paint::{Painting, Point, shifted};

fn wire_y(x: f32) -> f32 {
    let s = (x - 0.07) / 0.85;
    0.573 - 0.083 * s + 0.25 * s * (1. - s)
}

pub(super) fn sparrow_background(p: &mut Painting) {
    p.wash(|[x, y], _| {
        let sky = mix(tint(0x287ACC), tint(0x85DEE9), smooth(0.02, 0.85, y));
        let sun = (1. - ((x - 0.83) * 2.1).powi(2) - ((y - 0.29) * 1.9).powi(2)).max(0.);
        mix(sky, tint(0xB9E8F1), sun * 0.30)
    });
    p.glow([0.810, 0.222], [0.19, 0.33], 0xFFF1B8, 0.52);
    p.oval([0.817, 0.207], [0.032, 0.055], 0xFFF3C2);
    p.polygon(
        &[
            [0.0, 0.728],
            [0.122, 0.637],
            [0.248, 0.655],
            [0.385, 0.619],
            [0.561, 0.683],
            [0.766, 0.624],
            [1., 0.665],
            [1., 0.91],
            [0., 0.91],
        ],
        0x5EAA9E,
    );
    for i in 0..15 {
        let x = 0.094 + i as f32 * 0.056;
        let height = 0.054 + hash(i, 330) * 0.114;
        let y = 0.825 - height;
        let width = 0.034 + 0.023 * hash(i, 331);
        p.polygon(
            &[
                [x, y],
                [x + width, y + 0.006],
                [x + width, 0.825],
                [x, 0.825],
            ],
            if i % 2 == 0 { 0xE8AC79 } else { 0xD5B68B },
        );
        p.polygon(
            &[
                [x + width * 0.72, y + 0.004],
                [x + width, y + 0.006],
                [x + width, 0.825],
                [x + width * 0.72, 0.825],
            ],
            0x937A8B,
        );
        p.polygon(
            &[
                [x - 0.004, y],
                [x + width * 0.45, y - 0.035],
                [x + width + 0.006, y + 0.009],
            ],
            0xC66D68,
        );
        let light = 0x5C6894;
        p.rect([x + 0.010, y + 0.026, x + 0.022, y + 0.048], light);
        p.rect([x + 0.010, y + 0.060, x + 0.022, y + 0.082], light);
    }
    p.line([0.132, 0.841], [0.154, 0.366], 0.009, 0x514269);
    p.line([0.113, 0.437], [0.191, 0.426], 0.005, 0x514269);
    for i in 0..32 {
        let x = 0.062 + 0.87 * i as f32 / 32.;
        let next = x + 0.87 / 32.;
        p.line([x, wire_y(x)], [next, wire_y(next)], 0.0042, 0x39405F);
    }
    p.rect([0., 0., 0.064, 0.86], 0x325188);
    p.rect([0.055, 0., 0.070, 0.825], 0x6688BA);
    p.rect([0.905, 0., 0.922, 0.825], 0x617BA5);
    p.rect([0.922, 0., 1., 0.86], 0x365384);
    p.rect([0., 0., 1., 0.038], 0x355888);
    p.polygon(&[[0., 0.830], [1., 0.830], [1., 1.], [0., 1.]], 0xBD826D);
    p.rect([0., 0.828, 1., 0.862], 0xF4C382);
    p.glow([0.198, 0.924], [0.173, 0.065], 0x77536A, 0.79);
    p.oval([0.192, 0.914], [0.124, 0.037], 0xFAE4C4);
    p.oval([0.192, 0.916], [0.108, 0.027], 0xDEC8AA);
    p.polygon(
        &[
            [0.133, 0.927],
            [0.149, 0.883],
            [0.226, 0.880],
            [0.267, 0.917],
            [0.240, 0.940],
            [0.161, 0.938],
        ],
        0xA46C52,
    );
    p.polygon(
        &[
            [0.144, 0.920],
            [0.157, 0.888],
            [0.223, 0.886],
            [0.255, 0.914],
            [0.234, 0.931],
            [0.168, 0.931],
        ],
        0xE8B675,
    );
    p.oval([0.196, 0.911], [0.038, 0.012], 0x915291);
}

pub(super) fn sparrow(p: &mut Painting, t: f32) {
    let perch = 0.395;
    let wire = wire_y(perch);
    let takeoff = 4.08;
    let flight = (t - takeoff).max(0.);
    let crouch = smooth(3.47, 3.79, t) * (1. - smooth(3.86, takeoff, t));
    let scan = 0.16 * smooth(0.84, 1.12, t) * (1. - smooth(2.89, 3.3, t));
    let center = [
        perch + 0.090 * flight + 0.035 * flight * flight,
        wire - 0.123 + 0.030 * crouch - 0.135 * flight + 0.012 * flight * flight,
    ];

    // Thin, translucent clouds drift behind both the bird and its wire.
    p.wash(|[x, y], c| {
        if !(0.07..0.905).contains(&x) || !(0.045..0.49).contains(&y) {
            return c;
        }
        let drift = x - 0.0043 * t;
        let layers = noise(drift * 11., y * 31.) * 0.59 + noise(drift * 27., y * 47.) * 0.41;
        let filament = smooth(0.48, 0.76, layers)
            * (1. - smooth(0.015, 0.102, (y - 0.17 - 0.041 * (x * 5.).sin()).abs()));
        mix(c, tint(0xE1F5F1), 0.51 * filament)
    });
    if center[0] < 1.07 {
        let incline = -0.16 + 0.28 * crouch - 0.26 * smooth(0., 0.38, flight);
        let q = |x, y| shifted(center, [x, y], incline);
        let wingbeat = (flight * 22.).sin();
        if flight > 0. {
            p.polygon(
                &[
                    q(-0.016, -0.016),
                    q(-0.061, -0.053 - 0.067 * wingbeat),
                    q(-0.125, -0.098 - 0.114 * wingbeat),
                    q(-0.105, -0.046 - 0.075 * wingbeat),
                    q(-0.057, 0.026),
                ],
                0x6D4A55,
            );
        }
        if flight == 0. {
            for (x, dy) in [(-0.015, 0.001), (0.023, 0.004)] {
                let claw = [perch + x, wire_y(perch + x) + dy];
                let hock = q(x - 0.014, 0.091);
                p.line(q(x - 0.024, 0.054), hock, 0.006, 0xAA6C57);
                p.line(hock, claw, 0.005, 0xC79067);
                p.line(
                    [claw[0] - 0.006, claw[1]],
                    [claw[0] + 0.009, claw[1] - 0.006],
                    0.004,
                    0x604556,
                );
            }
        }
        p.polygon(
            &[
                q(-0.060, -0.010),
                q(-0.163, 0.044),
                q(-0.145, 0.068),
                q(-0.100, 0.063),
                q(-0.024, 0.043),
            ],
            0x724956,
        );
        p.line(q(-0.148, 0.054), q(-0.068, 0.017), 0.005, 0xB67659);
        p.oval(q(-0.013, 0.004), [0.085, 0.088], 0xA86B50);
        p.oval(q(0.014, 0.031), [0.064, 0.068], 0xD8B094);
        p.oval(q(0.028, 0.033), [0.055, 0.048], 0xEAC9A5);
        if flight == 0. {
            p.polygon(
                &[
                    q(-0.071, 0.003),
                    q(-0.055, -0.047),
                    q(-0.024, -0.057),
                    q(0.020, -0.027),
                    q(0.013, 0.023),
                    q(-0.032, 0.050),
                    q(-0.089, 0.039),
                ],
                0x83534D,
            );
            p.line(q(-0.066, -0.003), q(-0.006, -0.033), 0.008, 0xD5A181);
            p.line(q(-0.086, 0.026), q(-0.016, 0.008), 0.005, 0xCFA077);
        } else {
            let joint = q(-0.044, -0.036 + 0.105 * wingbeat);
            let tip = q(-0.121, -0.071 + 0.202 * wingbeat);
            let secondary = q(-0.116, -0.025 + 0.166 * wingbeat);
            p.polygon(
                &[
                    q(0.004, -0.034),
                    joint,
                    tip,
                    secondary,
                    q(-0.068, 0.038),
                    q(-0.014, 0.045),
                ],
                0x926052,
            );
            p.line(q(-0.021, -0.024), joint, 0.009, 0xDEA37B);
            p.line(joint, tip, 0.005, 0xB57E60);
        }
        let face = q(0.054 + 0.021 * scan, -0.058 - 0.024 * scan);
        let f = |x, y| shifted(face, [x, y], incline + scan);
        p.oval(face, [0.047, 0.067], 0x9E644D);
        p.oval(f(0.007, 0.015), [0.036, 0.041], 0xF0C79D);
        p.oval(f(0.032, -0.007), [0.013, 0.030], 0x6D484D);
        p.oval(f(0.017, -0.020), [0.010, 0.014], 0x392D46);
        p.oval(f(0.020, -0.025), [0.003, 0.005], 0xFFF0CF);
        p.polygon(
            &[f(0.039, -0.020), f(0.069, -0.004), f(0.037, 0.009)],
            0x755058,
        );
        p.line(f(0.049, -0.005), f(0.064, -0.003), 0.002, 0xD79E72);
    }
    p.rect([0.905, 0., 0.922, 0.824], 0x617BA5);
    p.rect([0.922, 0., 1., 0.824], 0x365384);
    p.rect([0., 0., 1., 0.038], 0x355888);
    p.wash(|[x, y], c| {
        if !(0.070..0.904).contains(&x) || !(0.049..0.825).contains(&y) {
            return c;
        }
        let edge = (x - 0.069).min(0.904 - x).min((0.824 - y) * 0.66);
        let width = 0.018 + 0.042 * smooth(2., 12.8, t);
        let dew = noise(x * 66., y * 89.);
        mix(
            c,
            tint(0xD9F4ED),
            (1. - smooth(0., width, edge)) * 0.30 * dew,
        )
    });
}

pub(super) fn garden_background(p: &mut Painting) {
    p.wash(|[x, y], _| {
        let flecks = noise(x * 29., y * 30.) * 0.47 + noise(x * 107., y * 122.) * 0.25;
        let sun = ((x - 0.23).powi(2) + (y - 0.30).powi(2)) * 2.3;
        mix(
            tint(0x184F53),
            tint(0x8EBD79),
            0.29 + flecks * 0.72 - sun.min(0.6) * 0.39,
        )
    });
    for i in 0..105 {
        let x = hash(i, 61);
        let y = hash(i, 62) * 0.51;
        let scale = 0.018 + 0.037 * hash(i, 63);
        let inks = [0xA96376, 0xCE8494, 0xE5A2AD, 0xEEC4C1, 0x8C586E, 0x6E675E];
        p.oval([x, y], [scale, scale * 1.16], inks[i as usize % inks.len()]);
    }
    p.polygon(
        &[
            [0.09, 0.84],
            [0.136, 0.31],
            [0.069, 0.183],
            [0., 0.169],
            [0., 0.099],
            [0.157, 0.16],
            [0.176, 0.307],
            [0.163, 0.84],
        ],
        0x4D4140,
    );
    for (a, b, size) in [
        ([0.153, 0.37], [0.28, 0.16], 0.026),
        ([0.28, 0.16], [0.52, 0.065], 0.016),
        ([0.287, 0.157], [0.654, 0.184], 0.010),
        ([0.159, 0.29], [0.225, 0.054], 0.015),
    ] {
        p.line(a, b, size, 0x695250);
    }
    for i in 0..140 {
        let x = hash(i, 111);
        let spine = 0.134 + (0.19 + 0.065 * (x * 7.).sin()) * smooth(0.10, 0.95, x) - 0.14 * x;
        let y = spine + (hash(i, 112) - 0.56) * 0.27;
        let r = 0.007 + 0.020 * hash(i, 114);
        p.oval(
            [x, y],
            [r, r * 1.5],
            [0xAD6279, 0xD78EA1, 0xF6B8BF, 0xFEE3D7][i as usize % 4],
        );
    }
    p.wash(|[x, y], c| {
        let horizon = 0.524 + (x * 8.).sin() * 0.024;
        if y < horizon {
            return c;
        }
        let ground = noise(x * 11., y * 19.) * 0.38 + noise(x * 112., y * 103.) * 0.17;
        let path = smooth(0.33, 0.17, (x - 0.52 - (y - 0.73) * 0.17).abs());
        let color = mix(tint(0x305B59), tint(0xAE886F), path);
        mix(
            c,
            gain(color, 0.73 + ground),
            smooth(horizon, horizon + 0.058, y),
        )
    });
    for i in 0..45 {
        let x = hash(i, 81);
        let y = 0.6 + hash(i, 84) * 0.39;
        p.oval(
            [x, y],
            [0.005 + 0.014 * hash(i, 82), 0.003 + 0.006 * hash(i, 82)],
            if i % 4 == 0 { 0xC78895 } else { 0x8D8A70 },
        );
    }
    frog(p, [0.36, 0.714], 1.09, -1.);
    frog(p, [0.690, 0.658], 0.88, 1.);
}

fn frog(p: &mut Painting, [cx, cy]: Point, size: f32, facing: f32) {
    let q = |x, y| [cx + x * size * facing, cy + y * size];
    p.glow(
        q(0.006, 0.126),
        [0.194 * size, 0.052 * size],
        0x293849,
        0.88,
    );
    p.polygon(
        &[
            q(-0.173, 0.094),
            q(0.137, 0.088),
            q(0.169, 0.121),
            q(0.152, 0.156),
            q(-0.158, 0.157),
            q(-0.176, 0.132),
        ],
        0x555E69,
    );
    p.polygon(
        &[
            q(-0.173, 0.094),
            q(0.137, 0.088),
            q(0.169, 0.121),
            q(-0.159, 0.130),
        ],
        0xBDC0A7,
    );

    // In profile the haunch rises behind the low head, and the eye sits on
    // its own bump. The silhouette keeps the front and hind legs distinct.
    p.polygon(
        &[
            q(-0.154, -0.011),
            q(-0.132, -0.047),
            q(-0.111, -0.057),
            q(-0.076, -0.080),
            q(-0.041, -0.056),
            q(0.011, -0.066),
            q(0.049, -0.093),
            q(0.092, -0.081),
            q(0.117, -0.044),
            q(0.129, 0.014),
            q(0.105, 0.084),
            q(-0.071, 0.083),
            q(-0.132, 0.046),
        ],
        0x518E77,
    );
    p.oval(q(0.021, -0.022), [0.088 * size, 0.065 * size], 0x83BBA0);
    p.oval(q(0.070, 0.019), [0.059 * size, 0.067 * size], 0x54886E);
    p.oval(q(0.067, 0.009), [0.047 * size, 0.052 * size], 0x93C19C);
    p.line(q(0.087, 0.052), q(0.125, 0.080), 0.014 * size, 0x5B947D);
    p.line(q(0.125, 0.080), q(0.038, 0.103), 0.010 * size, 0x79A88A);
    p.line(q(0.043, 0.102), q(0.006, 0.104), 0.0045 * size, 0xC6D4A9);
    p.line(q(0.055, 0.109), q(0.015, 0.114), 0.004 * size, 0xB6CAA1);
    p.oval(q(-0.081, 0.009), [0.075 * size, 0.039 * size], 0xB5CEA3);
    p.oval(q(-0.080, -0.025), [0.073 * size, 0.034 * size], 0xACCEA8);
    p.oval(q(-0.064, -0.075), [0.041 * size, 0.044 * size], 0x7AAD8E);
    p.oval(q(-0.070, -0.083), [0.032 * size, 0.031 * size], 0xC7D8AE);
    p.oval(q(-0.079, -0.082), [0.014 * size, 0.013 * size], 0x3A6156);
    p.oval(q(-0.084, -0.091), [0.005 * size, 0.006 * size], 0xEFF0C6);
    p.oval(q(-0.129, -0.022), [0.005 * size, 0.006 * size], 0x688D79);
    p.line(q(-0.148, 0.006), q(-0.110, 0.020), 0.0045 * size, 0x4D795F);
    p.line(q(-0.110, 0.020), q(-0.040, 0.019), 0.0045 * size, 0x4D795F);
    p.line(q(-0.040, 0.019), q(-0.015, 0.010), 0.0045 * size, 0x4D795F);
    p.line(q(-0.039, 0.045), q(-0.062, 0.088), 0.015 * size, 0x659B80);
    p.line(q(-0.060, 0.088), q(-0.108, 0.106), 0.009 * size, 0x91BA94);
    for dy in [-0.006, 0.005] {
        p.line(
            q(-0.093, 0.102),
            q(-0.123, 0.109 + dy),
            0.004 * size,
            0xC7D7AC,
        );
    }
    for i in 0..26 {
        let x = hash(i, 126) * 0.134 - 0.030;
        let y = hash(i, 127) * 0.057 - 0.052;
        p.glow(q(x, y), [0.0045 * size, 0.007 * size], 0x466F68, 0.52);
    }
}

pub(super) fn petals(p: &mut Painting, t: f32) {
    for i in 0..10 {
        let age = t + 6. * hash(i, 113);
        let y = 0.10 + age * (0.033 + 0.011 * hash(i, 114));
        let x = hash(i, 131) + age * 0.019 + (age * 2. + i as f32).sin() * 0.012;
        if y < 0.58 {
            let r = 0.004 + 0.003 * hash(i, 154);
            p.oval(
                [x, y],
                [r, 0.006 + (age * 5.).sin().abs() * 0.009],
                0xFFD3CA,
            );
        }
    }
    let contact = 5.1;
    let after = (t - contact).max(0.);
    let x = 0.12 + 0.386 * smooth(0., contact, t) + 0.061 * (1. - (-after * 1.15).exp());
    let falling_y = 0.29
        + 0.444 * (t.min(contact) / contact).powf(1.22)
        + 0.045 * (t * 2.4).sin() * (1. - smooth(3.9, contact, t));
    let y = falling_y + 0.053 * smooth(contact, 9.2, t)
        - 0.014 * (after * 7.).sin().abs() * (-after).exp();
    let turn = t.min(contact) * 2.6 + 6.9 * (1. - (-after * 0.96).exp());
    let face = (turn * 1.4).cos().abs();
    let q = |dx, dy| shifted([x, y], [dx, dy * (0.38 + 0.62 * face)], turn);
    p.glow(
        [x, 0.795],
        [0.031, 0.014],
        0x3E3F3D,
        0.42 * smooth(3.0, contact, t),
    );
    p.polygon(
        &[
            q(-0.021, 0.),
            q(-0.006, -0.016),
            q(0.012, -0.015),
            q(0.020, -0.004),
            q(0.017, 0.010),
            q(0.007, 0.017),
            q(-0.009, 0.009),
        ],
        0xF8B4BB,
    );
    p.line(q(-0.012, 0.0), q(0.010, -0.009), 0.003, 0xFFE4DA);
}

pub(super) fn snow_background(p: &mut Painting) {
    p.wash(|[x, y], _| {
        let sky = mix(tint(0x4D61B5), tint(0xEDBCB4), smooth(0.03, 0.63, y));
        mix(
            sky,
            tint(0xD4D9E1),
            smooth(0.31, 0.76, noise(x * 5.4, y * 28.)) * 0.64,
        )
    });
    p.polygon(
        &[
            [0., 0.39],
            [0.19, 0.253],
            [0.28, 0.288],
            [0.39, 0.372],
            [0.54, 0.389],
            [0.72, 0.276],
            [0.81, 0.25],
            [1., 0.364],
            [1., 0.79],
            [0., 0.79],
        ],
        0x728CC0,
    );
    p.polygon(
        &[
            [0.11, 0.351],
            [0.19, 0.253],
            [0.28, 0.288],
            [0.34, 0.34],
            [0.25, 0.31],
            [0.222, 0.337],
            [0.182, 0.303],
        ],
        0xBED1DD,
    );
    p.polygon(
        &[
            [0.64, 0.327],
            [0.81, 0.25],
            [0.92, 0.315],
            [0.822, 0.302],
            [0.767, 0.316],
            [0.749, 0.298],
        ],
        0xB7C9D6,
    );
    p.polygon(
        &[
            [0., 0.46],
            [0.21, 0.36],
            [0.33, 0.42],
            [0.45, 0.48],
            [0.64, 0.494],
            [0.80, 0.415],
            [1., 0.397],
            [1., 0.91],
            [0., 0.91],
        ],
        0x496DA1,
    );
    for row in 0..5 {
        for i in 0..23 {
            let seed = row * 23 + i;
            let x = 0.042 * i as f32 + 0.021 * hash(seed, 131);
            let width = 0.019 + hash(seed, 132) * 0.024;
            let y = 0.457
                + row as f32 * 0.083
                + 0.051 * hash(seed, 134)
                + 0.075 * (0.55 - (x - 0.53).abs());
            let height = 0.015 + hash(seed, 135) * 0.073;
            let peak = y - height;
            p.rect(
                [x, peak, x + width, y],
                if seed % 3 == 0 { 0x687189 } else { 0x4E6483 },
            );
            p.polygon(
                &[
                    [x - 0.004, peak],
                    [x + width * 0.46, peak - 0.019],
                    [x + width + 0.004, peak],
                ],
                0xBBC9D5,
            );
            for w in 0..3 {
                let u = x + 0.004 + w as f32 * (width - 0.004) / 3.;
                let h = height * 0.15;
                p.rect(
                    [
                        u,
                        peak + height * 0.31,
                        u + 0.0060,
                        peak + height * 0.31 + h,
                    ],
                    if seed % 4 == 0 { 0x9EBBDE } else { 0xFFBD77 },
                );
            }
        }
    }
    // The nearest roofs have a side wall and a warm, distinct window.
    for (x, y, w, h, color) in [
        (0.232, 0.767, 0.104, 0.091, 0x9C5973),
        (0.409, 0.812, 0.133, 0.101, 0x5A81B5),
        (0.611, 0.793, 0.111, 0.093, 0xBD726F),
        (0.744, 0.742, 0.087, 0.076, 0x666EA5),
    ] {
        p.rect([x - w * 0.50, y - h, x + w * 0.5, y], color);
        p.polygon(
            &[
                [x + w * 0.5, y - h],
                [x + w * 0.69, y - h + 0.018],
                [x + w * 0.69, y + 0.012],
                [x + w * 0.50, y],
            ],
            0x53608D,
        );
        p.polygon(
            &[
                [x - w * 0.59, y - h],
                [x, y - h - 0.055],
                [x + w * 0.69, y - h + 0.014],
            ],
            0xEAF0F9,
        );
        p.polygon(
            &[
                [x, y - h - 0.055],
                [x + w * 0.24, y - h - 0.057],
                [x + w * 0.78, y - h + 0.015],
                [x + w * 0.69, y - h + 0.014],
            ],
            0x99ADD4,
        );
        for offset in [-0.23, 0.23] {
            let u = x + w * offset;
            p.glow([u, y - h * 0.49], [0.029, 0.034], 0xFFA46B, 0.50);
            p.rect([u - 0.009, y - h * 0.62, u + 0.009, y - h * 0.32], 0xFFD389);
            p.line([u, y - h * 0.63], [u, y - h * 0.31], 0.0014, 0xAF736A);
        }
    }
    p.polygon(
        &[
            [0., 0.741],
            [0.117, 0.768],
            [0.284, 0.833],
            [0.457, 0.893],
            [0.677, 0.877],
            [0.802, 0.839],
            [1., 0.830],
            [1., 1.],
            [0., 1.],
        ],
        0xA7C5E6,
    );
    p.polygon(
        &[
            [0., 0.739],
            [0.159, 0.77],
            [0.284, 0.833],
            [0.111, 0.809],
            [0.060, 0.803],
            [0., 0.799],
        ],
        0xE4E8EC,
    );
    p.polygon(
        &[
            [0., 0.933],
            [0.28, 0.866],
            [0.49, 0.916],
            [0.68, 0.912],
            [1., 0.85],
            [1., 1.],
            [0., 1.],
        ],
        0xD5E5F4,
    );
    for (x, y, size) in [
        (0.049, 0.965, 0.62),
        (0.111, 0.875, 0.45),
        (0.171, 0.944, 0.29),
        (0.892, 0.911, 0.30),
        (0.978, 0.970, 0.45),
    ] {
        p.line([x, y], [x, y - 0.44 * size], 0.006, 0x425469);
        for n in 0..7 {
            let top = y - size * (0.47 - n as f32 * 0.049);
            let w = size * (0.024 + n as f32 * 0.012);
            p.polygon(
                &[
                    [x, top],
                    [x - w, top + 0.125 * size],
                    [x + w, top + 0.125 * size],
                ],
                0x415A71,
            );
            p.line(
                [x - w * 0.83, top + 0.105 * size],
                [x, top + 0.028 * size],
                0.006,
                0x9FB6C9,
            );
        }
    }
}

pub(super) fn snow(p: &mut Painting, t: f32) {
    p.wash(|[x, y], c| {
        let band = 0.61 + 0.046 * (x * 6.0 - t * 0.21).sin();
        let field =
            noise(x * 4.3 - t * 0.09, y * 14.) * 0.72 + noise(x * 9.0 - t * 0.12, y * 29.) * 0.28;
        let trailing_edge = x - (-0.21 + 0.111 * t);
        let retreat = 0.12 + 0.79 * smooth(-0.18, 0.23, trailing_edge);
        let low = (1. - smooth(0.08, 0.25, (y - band).abs())) * retreat;
        let edge = (0.55 + 0.45 * smooth(0.26, 0.80, field)) * low;
        mix(c, tint(0xBED5F0), edge)
    });
    for i in 0..110 {
        let depth = hash(i, 92);
        let tick = t * (0.027 + 0.066 * depth);
        let y = (hash(i, 42) + tick).rem_euclid(1.1) - 0.06;
        let x = (hash(i, 43) + (0.023 + 0.032 * depth) * t + 0.013 * (t * 0.82 + i as f32).sin())
            .rem_euclid(1.06)
            - 0.03;
        let r = 0.0015 + 0.0056 * depth * depth;
        p.glow([x, y], [r * 1.38, r * 2.1], 0xFFF4EA, 0.44 + 0.46 * depth);
    }
}
