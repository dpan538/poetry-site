//! Scene construction follows the spatial workflow in ascii.rest's Alpine Dawn,
//! Gangaur Ghat and Lake Pichola (MIT, bas3line). Geometry/lighting is computed
//! before character coverage. Time is continuous: the review ruler is not a loop.
use super::*;

const CW: usize = 640;
const CH: usize = 100;

pub(super) struct Artwork {
    kind: u8,
    w: usize,
    h: usize,
    base: Vec<Sample>,
    current: Vec<Sample>,
    cloud: Vec<f32>,
    cloud_edge: Vec<f32>,
    haze: Vec<f32>,
    reflected_terrain: Vec<Option<Color>>,
    last: f32,
}

fn fbm(mut x: f32, mut y: f32, octaves: usize) -> f32 {
    let (mut sum, mut norm, mut amp) = (0., 0., 0.5);
    for _ in 0..octaves {
        sum += noise(x, y) * amp;
        norm += amp;
        x = x * 2.03 + 13.17;
        y = y * 2.01 + 7.43;
        amp *= 0.5;
    }
    sum / norm
}
fn ridged(x: f32, y: f32) -> f32 {
    let a = 1. - (2. * noise(x, y) - 1.).abs();
    let b = 1. - (2. * noise(x * 2.1 + 17.3, y * 2.1) - 1.).abs();
    (a * a + 0.5 * b * b) / 1.5
}
fn periodic_noise(x: f32, y: f32, period: i32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let u = smooth(0., 1., x.fract());
    let v = smooth(0., 1., y.fract());
    let a = hash(ix.rem_euclid(period), iy);
    let b = hash((ix + 1).rem_euclid(period), iy);
    let c = hash(ix.rem_euclid(period), iy + 1);
    let d = hash((ix + 1).rem_euclid(period), iy + 1);
    a * (1. - u) * (1. - v) + b * u * (1. - v) + c * (1. - u) * v + d * u * v
}
fn cloud_noise(x: f32, y: f32, seed: f32) -> f32 {
    let warp = periodic_noise(x * 0.0125, y * 0.08 + seed, 8);
    let mut value = 0.;
    for (f, a) in [(1., 0.57), (2., 0.28), (4., 0.15)] {
        value += periodic_noise(
            (x * 0.025 + warp * 1.7) * f,
            (y * 0.14 + warp * 0.7 + seed) * f,
            (16. * f) as i32,
        ) * a;
    }
    value
}
fn sample_sheet(field: &[f32], x: f32, y: f32) -> f32 {
    let x = x.rem_euclid(CW as f32);
    let y = y.clamp(0., CH as f32 - 1.001);
    let (ix, iy) = (x.floor() as usize, y.floor() as usize);
    let (fx, fy) = (x.fract(), y.fract());
    let a = field[iy * CW + ix];
    let b = field[iy * CW + (ix + 1) % CW];
    let c = field[(iy + 1).min(CH - 1) * CW + ix];
    let d = field[(iy + 1).min(CH - 1) * CW + (ix + 1) % CW];
    (a + (b - a) * fx) * (1. - fy) + (c + (d - c) * fx) * fy
}

impl Artwork {
    pub(super) fn new(kind: u8, w: usize, h: usize) -> Self {
        let mut cloud = vec![0.; CW * CH];
        let mut cloud_edge = cloud.clone();
        let seed = [2.6, 9.3, 0., 24.7, 18.2, 41.8][kind as usize];
        for y in 0..CH {
            for x in 0..CW {
                let c = cloud_noise(x as f32, y as f32, seed);
                let above = cloud_noise(x as f32 + 1.5, y as f32 - 2.5, seed);
                cloud[y * CW + x] = smooth(0.39, 0.66, c);
                cloud_edge[y * CW + x] = (c - above).clamp(-0.20, 0.20);
            }
        }
        let mut base = Vec::with_capacity(w * h);
        let mut haze = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                let u = (x as f32 + 0.5) / w as f32;
                let v = (y as f32 + 0.5) / h as f32;
                haze.push(fbm(u * 6. + seed, v * 9. + 2.3, 3));
                base.push(match kind {
                    0 => bridge(u, v),
                    1 => supper_filtered(u, v, w, h),
                    3 | 5 => alpine(kind, u, v),
                    4 => arcade(u, v),
                    _ => unreachable!(),
                });
            }
        }
        for pixel in &mut base {
            let peak = pixel.color.into_iter().fold(0.0_f32, f32::max);
            if peak > 0.27 {
                pixel.floor = match pixel.material {
                    Material::Solid | Material::Metal => 0.14,
                    Material::Floor => 0.12,
                    _ => pixel.floor,
                };
            }
        }
        let mut reflected_terrain = vec![None; w * h];
        if kind == 3 {
            for y in 0..h {
                for x in 0..w {
                    if base[y * w + x].material != Material::Water {
                        continue;
                    }
                    let u = ((x as f32 + 0.5) / w as f32 - 0.5) * 1.24;
                    let v = (0.565 - (y as f32 + 0.5) / h as f32) * 0.73;
                    reflected_terrain[y * w + x] = terrain_ray(u, -v, -1.5, kind)
                        .map(|(px, z)| shade_terrain(px, -1.5 - v * z, z, kind));
                }
            }
        }
        let current = base.clone();
        Self {
            kind,
            w,
            h,
            base,
            current,
            cloud,
            cloud_edge,
            haze,
            reflected_terrain,
            last: -1.,
        }
    }
    fn atmosphere(&self, x: f32, y: f32, t: f32, variation: f32) -> Color {
        let kind = self.kind;
        let (mut c, light, band, thickness, speed) = match kind {
            0 => (
                mix([0.065, 0.10, 0.17], [0.36, 0.37, 0.38], smooth(0., 0.62, y)),
                [0.86, 0.55, 0.32],
                0.22,
                0.27,
                0.75,
            ),
            3 => (
                mix(
                    [0.025, 0.045, 0.085],
                    [0.20, 0.24, 0.28],
                    smooth(0., 0.64, y),
                ),
                [0.62, 0.65, 0.68],
                0.22,
                0.31,
                0.52,
            ),
            4 => (
                mix([0.15, 0.56, 0.95], [0.60, 0.82, 1.02], smooth(0., 0.64, y)),
                [0.96, 0.98, 1.02],
                0.20,
                0.30,
                0.9,
            ),
            _ => (
                mix([0.065, 0.17, 0.30], [0.39, 0.49, 0.54], smooth(0., 0.66, y)),
                [0.78, 0.80, 0.81],
                0.20,
                0.31,
                0.65,
            ),
        };
        c = gain(c, 0.86 + variation * 0.28);
        // Whole cloud density and its lit edges translate. No sine-shaped cloud
        // edge and no modulo at the end of the 24-second inspection ruler.
        let sx = x * 200. + t * speed;
        let sy = 38. + (y - band) * 115.;
        let density = sample_sheet(&self.cloud, sx, sy);
        let edge = sample_sheet(&self.cloud_edge, sx, sy);
        let envelope = (-((y - band) / thickness).powi(4)).exp();
        let body = gain(light, (0.72 + edge * 3.5).clamp(0.32, 1.25));
        c = mix(c, body, density * envelope * 0.83);
        // Spatial, source-centred glow: highlights do not tint the whole image.
        if kind == 0 {
            let halo = (-(((x - 0.78) / 0.25).powi(2) + ((y - 0.52) / 0.13).powi(2))).exp();
            c = mix(c, [0.90, 0.61, 0.30], halo * 0.63);
        }
        if kind == 3 {
            c = moon(c, x, y, [0.16, 0.18], 0.033, true);
            let halo = (-(((x - 0.16) / 0.15).powi(2) + ((y - 0.18) / 0.18).powi(2))).exp();
            c = mix(c, [0.65, 0.66, 0.68], halo * 0.055);
            let star = hash((x * 204.) as i32, (y * 120.) as i32);
            if y < 0.38 && star > 0.998 {
                c = [0.60 + 0.17 * (t * (0.5 + star) + x * 99.).sin(); 3];
            }
        }
        c
    }
    pub(super) fn update(&mut self, t: f32) {
        if t == self.last {
            return;
        }
        self.last = t;
        // Current above-water radiance is a first pass, including moving clouds
        // and current candle/lantern light. Water reads this pass, not old base.
        for y in 0..self.h {
            for x in 0..self.w {
                let k = y * self.w + x;
                let u = (x as f32 + 0.5) / self.w as f32;
                let v = (y as f32 + 0.5) / self.h as f32;
                let mut s = self.base[k];
                if s.material == Material::Air {
                    s.color = self.atmosphere(u, v, t, self.haze[k]);
                }
                if self.kind == 1 {
                    s = supper_motion(s, u, v, t);
                }
                if (self.kind == 0 || self.kind == 5)
                    && s.material != Material::Air
                    && s.material != Material::Water
                {
                    let flick = 0.87 + 0.10 * (t * 1.7).sin() + 0.03 * (t * 3.3).cos();
                    if s.material == Material::Window {
                        s.color = gain(s.color, flick);
                    }
                }
                self.current[k] = s;
            }
        }
        // Reflected column mapping uses nearby current radiance. Advecting
        // waves break it into perspective-scaled horizontal streaks.
        for y in 0..self.h {
            for x in 0..self.w {
                let k = y * self.w + x;
                if self.base[k].material != Material::Water {
                    continue;
                }
                let u = (x as f32 + 0.5) / self.w as f32;
                let v = (y as f32 + 0.5) / self.h as f32;
                let horizon = match self.kind {
                    0 => 0.575,
                    3 => 0.565,
                    4 => 0.62,
                    _ => 0.68,
                };
                let depth = ((v - horizon) / (1. - horizon)).clamp(0., 1.);
                let wave = (v * 147. - t * 1.8 + u * 8.).sin()
                    + 0.43 * (v * 237. - t * 2.7 - u * 19.).sin()
                    + (noise(u * 39. + t * 0.16, v * 81. - t * 0.21) - 0.5) * 1.4;
                let sx = (u + wave * (0.0015 + depth * 0.009)).clamp(0., 0.999);
                let mirror = horizon - (v - horizon) * if self.kind == 3 { 1. } else { 0.91 };
                let sy = (mirror + wave * 0.004).clamp(0., horizon - 0.002);
                let source_color = if self.kind == 3 {
                    // A reflected mountain ray can see a different face from
                    // the camera. Reusing a screen row sampled hidden terrain
                    // as water, creating the old flat dark band and bright bar.
                    let ry = (y as f32 + wave * 0.5).clamp(0., self.h as f32 - 1.) as usize;
                    let rx = (sx * self.w as f32) as usize;
                    self.reflected_terrain[ry * self.w + rx]
                        .unwrap_or_else(|| self.atmosphere(sx, sy, t, self.haze[k]))
                } else {
                    self.current
                        [(sy * self.h as f32) as usize * self.w + (sx * self.w as f32) as usize]
                        .color
                };
                let base = if self.kind == 4 {
                    [0.055, 0.23, 0.26]
                } else {
                    [0.055, 0.065, 0.075]
                };
                let mut s = self.base[k];
                s.color = if self.kind == 4 {
                    // Blue water remains a stable material. Pink belongs to
                    // individual floating beads, never to the entire pool.
                    let body = [0.13, 0.53, 0.87];
                    let shimmer = 0.94 + wave * 0.045;
                    mix(
                        gain(body, shimmer),
                        gain(source_color, 0.95),
                        0.25 + (1. - depth) * 0.12,
                    )
                } else if self.kind == 3 {
                    // Angle-dependent water reflection: a near bank must not
                    // become a hard white screen-row copy in the foreground.
                    let slope = ((v - 0.565) * 0.73).max(0.);
                    let view_normal =
                        slope / (1. + slope * slope + ((u - 0.5) * 1.24).powi(2)).sqrt();
                    let fresnel = 0.02 + 0.98 * (1. - view_normal).powi(5);
                    mix(
                        [0.12, 0.18, 0.26],
                        gain(source_color, 0.90 + wave * 0.035),
                        fresnel * 0.82,
                    )
                } else {
                    mix(
                        gain(source_color, 0.78 + wave * 0.055),
                        base,
                        0.23 + depth * 0.30,
                    )
                };
                let ridge = smooth(0.85, 1.45, wave) * (0.06 + depth * 0.13);
                s.color = mix(s.color, [0.68, 0.70, 0.72], ridge);
                if self.kind == 0 {
                    drifting_lamps(&mut s, u, v, t);
                }
                if self.kind == 4 {
                    pink_beads(&mut s, u, v, t);
                    boat(&mut s, u, v, t);
                }
                self.current[k] = s;
            }
        }
    }
    pub(super) fn sample(&self, x: usize, y: usize) -> Sample {
        self.current[y * self.w + x]
    }
}

// Geometry is shaded in world space; silhouettes alone are not a mountain.
fn terrain(x: f32, z: f32, kind: u8) -> f32 {
    let peaks: &[(f32, f32, f32, f32, f32)] = if kind == 3 {
        &[
            (0.34, 0.13, 145., 0.90, 0.28),
            (0.10, 0.30, 110., 0.96, -0.2),
            (0.70, 0.23, 175., 1.10, -0.25),
            (0.90, 0.34, 125., 0.80, 0.1),
            (0.52, 0.32, 245., 1.15, 0.45),
        ]
    } else {
        &[
            (0.54, 0.12, 175., 1.13, 0.24),
            (0.20, 0.30, 120., 1.15, -0.25),
            (0.85, 0.27, 140., 1.02, -0.3),
            (0.72, 0.32, 255., 1.15, 0.4),
            (0.04, 0.39, 150., 1.4, 0.3),
        ]
    };
    let mut h: f32 = 0.;
    for &(sx, sy, pz, f, a) in peaks {
        let ph = 1.5 + (0.565 - sy) * 0.73 * pz;
        let px = (sx - 0.5) * 1.24 * pz;
        let dx = x - px;
        let dz = z - pz;
        let rx = dx * a.cos() - dz * a.sin();
        let rz = dx * a.sin() + dz * a.cos();
        h = h.max(ph * (1. - (rx.abs() + rz.abs()) / (ph * f)));
    }
    h = h.max((1. + 3. * fbm(x * 0.04 + 19., z * 0.04, 2)) * smooth(48., 68., z));
    h + ((ridged(x * 0.06, z * 0.06) - 0.45) * 7.2 + (ridged(x * 0.2, z * 0.2) - 0.45) * 1.8)
        * smooth(4., 22., h)
}
fn terrain_hit(u: f32, v: f32, kind: u8) -> Option<(f32, f32)> {
    terrain_ray(u, v, 1.5, kind)
}
fn terrain_ray(u: f32, v: f32, origin_y: f32, kind: u8) -> Option<(f32, f32)> {
    let mut z = 48.;
    let mut prev = z;
    for _ in 0..200 {
        let gap = origin_y + v * z - terrain(u * z, z, kind);
        if gap < 0. {
            let (mut a, mut b) = (prev, z);
            for _ in 0..6 {
                let m = (a + b) * 0.5;
                if origin_y + v * m < terrain(u * m, m, kind) {
                    b = m;
                } else {
                    a = m;
                }
            }
            return Some((u * b, b));
        }
        prev = z;
        z += 0.35_f32.max(gap * 0.45) + z * 0.002;
        if z > 520. {
            break;
        }
    }
    None
}
fn lake_shore_y(x: f32) -> f32 {
    0.592
        + 0.012 * noise(x * 7.1, 15.7)
        + 0.040 * (-((x - 0.14) / 0.11).powi(2)).exp()
        + 0.033 * (-((x - 0.88) / 0.14).powi(2)).exp()
}
fn shade_terrain(px: f32, py: f32, z: f32, kind: u8) -> Color {
    let e = 0.35;
    let hx = (terrain(px + e, z, kind) - terrain(px - e, z, kind)) / (2. * e);
    let hz = (terrain(px, z + e, kind) - terrain(px, z - e, kind)) / (2. * e);
    let n = (hx * hx + hz * hz + 1.).sqrt();
    let up = 1. / n;
    let mut lit = ((-hx * 0.72 + 0.56 + hz * 0.40) / n).max(0.);
    if lit > 0. {
        let mut d = 0.9;
        while d < 95. {
            if py + 0.56 * d + 0.18 < terrain(px + 0.72 * d, z - 0.40 * d, kind) {
                lit = 0.;
                break;
            }
            d += 0.8 + d * 0.07;
        }
    }
    let grain = fbm(px * 0.4, py * 0.25, 2);
    let gully = ridged(px * 0.22 + z * 0.05, py * 0.045);
    let snow = smooth(0.18, 0.43, up + 0.27 * (grain - 0.5))
        * smooth(5., 12., py + 5. * grain)
        * (1. - 0.48 * smooth(0.46, 0.82, gully));
    let snow_light = if kind == 5 {
        0.35 + 0.72 * lit + 0.24 * up
    } else {
        0.29 + 0.79 * lit + 0.22 * up
    };
    let rock = 0.12 + 0.40 * lit + 0.11 * grain;
    let value = rock * (1. - snow) + snow_light * snow;
    let haze = if kind == 5 {
        smooth(75., 270., z) * 0.64
    } else {
        (1. - (-(z - 48.) / 250.).exp()).clamp(0., 1.) * 0.40
    };
    let distant_air = if kind == 5 {
        [0.57, 0.64, 0.70]
    } else {
        [0.24, 0.28, 0.31]
    };
    let color = mix([value; 3], distant_air, haze);
    let wood = smooth(9., 3.5, py) * smooth(120., 65., z);
    let canopy = if kind == 5 {
        [0.22, 0.24, 0.22]
    } else {
        [0.21 + 0.13 * up + 0.10 * lit; 3]
    };
    mix(color, canopy, wood * if kind == 5 { 0.5 } else { 0.55 })
}
fn alpine(kind: u8, x: f32, y: f32) -> Sample {
    let shore = lake_shore_y(x);
    let water = kind == 3 && y > shore;
    let mut s = Sample::new(
        [0.; 3],
        if water {
            Material::Water
        } else {
            Material::Air
        },
    );
    if y < if kind == 3 { shore } else { 0.69 } {
        if let Some((px, z)) = terrain_hit((x - 0.5) * 1.24, (0.565 - y) * 0.73, kind) {
            let py = 1.5 + (0.565 - y) * 0.73 * z;
            s = Sample::new(shade_terrain(px, py, z, kind), Material::Solid);
        }
    }
    if kind == 3 {
        // A far treeline separates real mountain relief from the waterline.
        for i in 0..37 {
            let cx = i as f32 / 36.;
            tree(
                &mut s,
                x,
                y,
                cx,
                lake_shore_y(cx),
                0.012 + hash(i, 4) * 0.024,
                0.37,
            );
        }
        if y > 0.88 + 0.09 * (x * 5.).sin().abs() && (x < 0.16 || x > 0.92) {
            s = Sample::new(
                [0.085 + fbm(x * 31., y * 35., 2) * 0.07; 3],
                Material::Solid,
            );
        }
        for (cx, foot, h) in [(0.018, 0.96, 0.36), (0.075, 0.89, 0.22), (0.98, 1.03, 0.36)] {
            tree(&mut s, x, y, cx, foot, h, 0.14);
        }
    } else {
        // A quiet stone forecourt replaces the noisy grey texture. Broad
        // planes and intentional joints describe space before character dither.
        if y > 0.53 {
            let value = 0.55 + 0.065 * smooth(0.57, 1., y);
            s.coat([value; 3], smooth(0.53, 0.635, y), Material::Solid);
        }
        // Foreground stays open. The distant hall and its approach share a
        // ground contact; the hall cannot float above or overlap the stairway.
        shrine(
            &mut s,
            x,
            y,
            SHRINE_AXIS,
            SHRINE_FOOT,
            SHRINE_WIDTH,
            SHRINE_HEIGHT,
        );
        let half = shrine_path_half(y);
        if y > PATH_REAR && (x - SHRINE_AXIS).abs() < half {
            let depth = (y - PATH_REAR) / (1. - PATH_REAR);
            // Equal world-space treads converge towards the shrine. Average
            // unresolved distant joints instead of producing a dark stripe.
            let step = (2.4 / (depth + 0.18)).fract();
            let joint = (x * 45.).fract() < 0.035;
            let value = if depth < 0.12 {
                0.51
            } else if step < 0.12 {
                0.36
            } else if step > 0.90 {
                0.68
            } else {
                0.65
            };
            s = Sample::new(
                [if joint { value * 0.68 } else { value }; 3],
                Material::Floor,
            );
        }
        torii(&mut s, x, y, SHRINE_AXIS, GATE_TOP, GATE_HEIGHT);
    }
    s
}
fn shrine(s: &mut Sample, x: f32, y: f32, cx: f32, foot: f32, w: f32, h: f32) {
    let eave = foot - h * 0.62;
    s.coat(
        [0.80, 0.77, 0.67],
        rect(x, y, [cx - w * 0.39, eave, cx + w * 0.39, foot]),
        Material::Solid,
    );
    // A broad hip roof with upturned eaves and a clearly lit ridge. This is
    // shrine architecture, separate from the distant cabin in Going Home.
    let roof = [
        [cx - w * 0.60, eave - h * 0.03],
        [cx - w * 0.40, eave - h * 0.10],
        [cx, foot - h],
        [cx + w * 0.40, eave - h * 0.10],
        [cx + w * 0.60, eave - h * 0.03],
        [cx + w * 0.48, eave + h * 0.06],
        [cx - w * 0.48, eave + h * 0.06],
    ];
    let roof_value = 0.14 + 0.045 * smooth(cx - w * 0.5, cx + w * 0.5, x);
    s.coat([roof_value; 3], polygon(x, y, &roof), Material::Solid);
    // The lower eave separates the shaded roof from the lit plaster facade.
    s.coat(
        [0.69; 3],
        line(x, y, roof[5], roof[6], 0.0028),
        Material::Solid,
    );
    for end in [roof[0], roof[4]] {
        s.coat(
            [0.82; 3],
            line(x, y, [cx, foot - h], end, 0.0023),
            Material::Solid,
        );
    }
    s.coat(
        [0.19, 0.17, 0.14],
        rect(
            x,
            y,
            [cx - w * 0.39, eave + 0.008, cx + w * 0.39, eave + 0.021],
        ),
        Material::Solid,
    );
    for dx in [-0.34, -0.22, -0.11, 0.11, 0.22, 0.34] {
        s.coat(
            [0.53, 0.29, 0.18],
            rect(
                x,
                y,
                [
                    cx + w * (dx - 0.018),
                    eave + 0.006,
                    cx + w * (dx + 0.018),
                    foot,
                ],
            ),
            Material::Solid,
        );
    }
    s.coat(
        [0.12; 3],
        rect(x, y, [cx - w * 0.11, eave + h * 0.12, cx + w * 0.11, foot]),
        Material::Solid,
    );
    for dx in [-0.245, 0.245] {
        s.coat(
            [0.75, 0.57, 0.32],
            rect(
                x,
                y,
                [
                    cx + w * (dx - 0.065),
                    eave + h * 0.16,
                    cx + w * (dx + 0.065),
                    eave + h * 0.35,
                ],
            ),
            Material::Window,
        );
    }
    // The upper veranda reads as a continuous timber structure, with a
    // shaded lower platform and broad, sunlit stone steps below it.
    for yy in [foot - h * 0.16, foot - h * 0.08] {
        s.coat(
            [0.48, 0.31, 0.22],
            line(x, y, [cx - w * 0.40, yy], [cx + w * 0.40, yy], 0.002),
            Material::Solid,
        );
    }
    s.coat(
        [0.62; 3],
        rect(
            x,
            y,
            [cx - w * 0.47, foot - 0.005, cx + w * 0.47, foot + 0.012],
        ),
        Material::Solid,
    );
    for (dy, half, value) in [(0.016, 0.49, 0.42), (0.026, 0.52, 0.75)] {
        s.coat(
            [value; 3],
            rect(
                x,
                y,
                [cx - w * half, foot + dy, cx + w * half, foot + dy + 0.008],
            ),
            Material::Solid,
        );
    }
}
fn tree(s: &mut Sample, x: f32, y: f32, cx: f32, foot: f32, h: f32, shade: f32) {
    let d = (y - (foot - h)) / h;
    if !(0. ..=1.).contains(&d) {
        return;
    }
    let tier = (d * 11.).fract();
    let hw = h
        * (0.028 + d * 0.24)
        * (0.57 + 0.43 * tier)
        * (0.82 + hash((y * 520.) as i32, (cx * 171.) as i32) * 0.30);
    let dx = (x - cx).abs();
    if dx < hw {
        let edge = smooth(hw - 0.004, hw, dx);
        let needles = noise(x * 380., y * 490.);
        let value = 0.045 + shade * (0.18 + 0.27 * needles) + edge * 0.13;
        *s = Sample::new([value; 3], Material::Leaves);
    }
    if (x - cx).abs() < 0.0016 * (0.4 + d) && d > 0.3 {
        s.coat([0.24; 3], 0.7, Material::Solid);
    }
}
fn cabin(s: &mut Sample, x: f32, y: f32, cx: f32, foot: f32, w: f32, h: f32) {
    if rect(x, y, [cx - w * 0.43, foot - h * 0.65, cx + w * 0.43, foot]) > 0.5 {
        let fine = w > 0.145;
        let plank = ((x - cx + w) * if fine { 81. } else { 270. }).fract();
        let light = (if fine { 0.60 } else { 0.39 } + 0.15 * (x - cx) / w)
            * (0.85 + 0.15 * noise(x * 98., y * 71.));
        *s = Sample::new(
            [if plank < 0.10 { light * 0.57 } else { light }; 3],
            Material::Solid,
        );
        for dx in [-0.25, 0., 0.25] {
            s.coat(
                [0.92, 0.66, 0.27],
                rect(
                    x,
                    y,
                    [
                        cx + w * (dx - 0.07),
                        foot - h * 0.48,
                        cx + w * (dx + 0.07),
                        foot - h * 0.22,
                    ],
                ),
                Material::Window,
            );
        }
    }
    let roof = polygon(
        x,
        y,
        &[
            [cx - w * 0.60, foot - h * 0.63],
            [cx, foot - h * 1.03],
            [cx + w * 0.60, foot - h * 0.63],
        ],
    );
    let tile = ((if w > 0.145 {
        x * 65. + y * 57.
    } else {
        x * 201. + y * 310.
    })
    .fract()
        > 0.18) as u8 as f32;
    s.coat([0.11 + 0.09 * tile; 3], roof, Material::Solid);
    s.coat(
        [0.59; 3],
        line(
            x,
            y,
            [cx - w * 0.60, foot - h * 0.63],
            [cx, foot - h * 1.03],
            0.0012,
        ),
        Material::Solid,
    );
    s.coat(
        [0.22; 3],
        rect(
            x,
            y,
            [cx - w * 0.51, foot - 0.006, cx + w * 0.51, foot + 0.009],
        ),
        Material::Solid,
    );
}

fn bridge(x: f32, y: f32) -> Sample {
    let mut s = Sample::new([0.; 3], Material::Air);
    if y > 0.575 {
        s = Sample::new([0.08; 3], Material::Water);
    }
    // The landing, quay and house have distinct footprints. The bridge ends
    // beside the house, not at a point inside its facade.
    let ridge = 0.53 + fbm(x * 5., 3.7, 3) * 0.055;
    if y > ridge && y < 0.605 {
        s = Sample::new([0.14 + fbm(x * 41., y * 49., 2) * 0.12; 3], Material::Solid);
    }
    for i in 0..20 {
        let cx = 0.48 + i as f32 * 0.028;
        tree(&mut s, x, y, cx, 0.575, 0.035 + hash(i, 12) * 0.055, 0.22);
    }
    cabin(
        &mut s,
        x,
        y,
        BRIDGE_HOUSE[0],
        BRIDGE_HOUSE[1],
        BRIDGE_HOUSE[2],
        BRIDGE_HOUSE[3],
    );
    let quay = polygon(
        x,
        y,
        &[[0.59, 0.588], [0.88, 0.588], [0.94, 0.655], [0.575, 0.655]],
    );
    s.coat([0.39; 3], quay, Material::Solid);
    s.coat(
        [0.66; 3],
        polygon(
            x,
            y,
            &[
                [0.662, 0.634],
                [0.701, 0.634],
                [0.824, 0.589],
                [0.799, 0.589],
            ],
        ),
        Material::Floor,
    );
    s.coat(
        [0.72; 3],
        line(x, y, [0.575, 0.655], [0.94, 0.655], 0.002),
        Material::Solid,
    );
    // Two stone piers and the dark underside below the deck establish volume.
    for d in [0.16, 0.40, 0.72] {
        let (cx, cy, width) = bridge_station(d);
        s.coat(
            [0.13; 3],
            polygon(
                x,
                y,
                &[
                    [cx - width, cy],
                    [cx + width, cy],
                    [cx + width * 0.8, cy + 0.07 * d],
                    [cx - width * 0.85, cy + 0.07 * d],
                ],
            ),
            Material::Solid,
        );
        s.coat(
            [0.32; 3],
            rect(
                x,
                y,
                [
                    cx - width * 0.7,
                    cy + 0.02,
                    cx - width * 0.35,
                    cy + 0.09 * d,
                ],
            ),
            Material::Solid,
        );
    }
    if y > BRIDGE_FAR[1] {
        let d = (y - BRIDGE_FAR[1]) / (BRIDGE_NEAR[1] - BRIDGE_FAR[1]);
        let (cx, _, width) = bridge_station(d);
        if (x - cx).abs() < width && d < 1.02 {
            let boards = (3. / (d + 0.10)).fract();
            let side = (x - cx) / width;
            let value = 0.61 + 0.10 * (1. - side.abs()) + 0.025 * noise(x * 81., y * 63.);
            let value = if d > 0.23 && boards < 0.11 {
                value * 0.64
            } else {
                value
            };
            s = Sample::new([value; 3], Material::Floor);
        }
    }
    for side in [-1., 1.] {
        let mut prev = None;
        for i in 0..24 {
            let d = (i as f32 / 23.).powf(1.65);
            let (axis, cy, width) = bridge_station(d);
            let cx = axis + side * width;
            let rail = 0.018 + d * 0.060;
            let top = [cx, cy - rail];
            if let Some(a) = prev {
                s.coat(
                    [1.04, 0.24, 0.105],
                    line(x, y, a, top, 0.002 + d * 0.0016),
                    Material::Solid,
                );
                s.coat(
                    [0.30, 0.10, 0.065],
                    line(
                        x,
                        y,
                        [a[0], a[1] + rail * 0.48],
                        [top[0], top[1] + rail * 0.48],
                        0.001,
                    ),
                    Material::Solid,
                );
            }
            s.coat(
                [0.46, 0.13, 0.075],
                line(x, y, [cx, cy], top, 0.001 + d * 0.0017),
                Material::Solid,
            );
            if i % 4 == 0 {
                s.coat(
                    [1.25, 0.89, 0.43],
                    ellipse(
                        x,
                        y,
                        [cx, cy - rail - 0.005],
                        [0.0018 + d * 0.0012, 0.004 + d * 0.002],
                    ),
                    Material::Window,
                );
            }
            prev = Some(top);
        }
    }
    let bank = 0.71 + 0.28 * smooth(0., 0.39, x) + noise(x * 21., 3.) * 0.02;
    if x < 0.21 && y > bank {
        let rough = fbm(x * 39., y * 58., 3);
        s = Sample::new([0.11 + rough * 0.16; 3], Material::Solid);
    }
    for (cx, foot, h) in [(0.01, 0.89, 0.34), (0.08, 0.81, 0.23), (0.135, 0.85, 0.18)] {
        tree(&mut s, x, y, cx, foot, h, 0.18);
    }
    s
}
fn drifting_lamps(s: &mut Sample, x: f32, y: f32, t: f32) {
    if y < 0.63 {
        return;
    }
    // A slow procession, not an intro event. Each lamp traverses the frame over
    // roughly 100 seconds, with its own reflected light and tiny trailing wake.
    for i in 0..9 {
        let depth = hash(i, 65);
        let cx = (hash(i, 18) * 1.16 + t * (0.003 + depth * 0.002)).rem_euclid(1.25) - 0.125;
        let cy = 0.67 + depth * 0.27;
        let z = 0.55 + depth * 0.65;
        if (x - cx).abs() > 0.045 * z || y < cy - 0.02 || y > cy + 0.12 * z {
            continue;
        }
        let flick =
            0.83 + 0.10 * (t * 2.7 + i as f32 * 4.).sin() + 0.06 * (t * 4.3 + i as f32).cos();
        let halo = (-((x - cx) / (0.009 * z)).powi(2) - ((y - cy) / (0.013 * z)).powi(2)).exp();
        for (c, a) in s.color.iter_mut().zip([0.84, 0.48, 0.14]) {
            *c += a * halo * flick;
        }
        s.coat(
            [1.4, 0.95, 0.40],
            ellipse(x, y, [cx, cy], [0.0025 * z, 0.004 * z]),
            Material::Window,
        );
        let wave = (y * 230. - t * 2.2 + x * 19.).sin();
        let streak = (-((x - cx) / (0.006 * z)).powi(2)).exp()
            * (-((y - cy - 0.035 * z) / (0.05 * z)).powi(2)).exp()
            * smooth(0.10, 0.8, wave)
            * smooth(cy, cy + 0.01, y);
        for (c, a) in s.color.iter_mut().zip([0.75, 0.48, 0.16]) {
            *c += a * streak * flick * 0.65;
        }
    }
}

fn supper_filtered(x: f32, y: f32, w: usize, h: usize) -> Sample {
    // Subcell area sampling keeps thin glass, plate rims and window bevels
    // coherent at both grids. This changes geometric coverage, not exposure.
    let mut result = supper(x, y);
    let mut color = [0.; 3];
    for dx in [-0.25, 0.25] {
        for dy in [-0.25, 0.25] {
            let sample = supper(x + dx / w as f32, y + dy / h as f32);
            for channel in 0..3 {
                color[channel] += sample.color[channel] * 0.25;
            }
        }
    }
    result.color = color;
    result
}
fn supper(x: f32, y: f32) -> Sample {
    // A dim interior with one warm light source, cool glass and depth outside.
    let wall = 0.28 + 0.20 * (1. - x) + 0.04 * fbm(x * 10., y * 11., 2);
    let mut s = Sample::new([wall * 1.18, wall * 0.88, wall * 0.58], Material::Solid);
    let wx = [0.13, 0.075, 0.91, 0.61];
    let frame = rect(
        x,
        y,
        [wx[0] - 0.023, wx[1] - 0.025, wx[2] + 0.023, wx[3] + 0.025],
    );
    s.coat([0.38, 0.31, 0.23], frame, Material::Solid);
    if rect(x, y, wx) > 0.5 {
        let value = 0.045 + 0.025 * (1. - y) + 0.02 * fbm(x * 11., y * 8., 3);
        s = Sample::new([value * 0.78, value * 0.90, value * 1.10], Material::Window);
        // Background building: a few distant windows rather than an empty slab.
        if x > 0.61 && y > 0.18 && y < 0.46 {
            let brick = (x * 83.).fract() < 0.08 || (y * 88.).fract() < 0.08;
            s.color = [if brick { 0.035 } else { 0.055 }; 3];
            if (x * 32.).fract() > 0.22
                && (x * 32.).fract() < 0.55
                && (y * 20.).fract() > 0.28
                && (y * 20.).fract() < 0.62
            {
                s.color = [0.15, 0.13, 0.09];
            }
        }
        // The crowd is outside, overlapping and staggered in depth. Their coats
        // have shoulders/lapels/side lighting; no row of identical pawn shapes.
        for (i, (cx, foot, h)) in [
            (0.21, 0.64, 0.29),
            (0.31, 0.59, 0.25),
            (0.405, 0.635, 0.33),
            (0.535, 0.61, 0.28),
            (0.63, 0.64, 0.34),
            (0.76, 0.60, 0.25),
            (0.86, 0.65, 0.32),
        ]
        .iter()
        .enumerate()
        {
            let z = h / 0.30;
            let head = foot - h;
            let body = polygon(
                x,
                y,
                &[
                    [cx - 0.012 * z, head + 0.037 * z],
                    [cx - 0.035 * z, head + 0.062 * z],
                    [cx - 0.044 * z, foot - 0.02],
                    [cx - 0.022 * z, *foot],
                    [cx + 0.031 * z, *foot],
                    [cx + 0.040 * z, head + 0.10 * z],
                    [cx + 0.024 * z, head + 0.055 * z],
                    [cx + 0.010 * z, head + 0.040 * z],
                ],
            );
            let face = ellipse(x, y, [*cx, head + 0.014 * z], [0.014 * z, 0.028 * z]);
            let side = (((x - cx) / (0.043 * z)) + 1.) * 0.5;
            let coat = 0.035 + 0.055 * side + 0.015 * noise(x * 171., y * 187.);
            s.coat([coat; 3], body, Material::Solid);
            let skin = 0.10 + 0.13 * side;
            s.coat([skin; 3], face, Material::Solid);
            let collar = line(
                x,
                y,
                [cx - 0.011 * z, head + 0.043 * z],
                [*cx, head + 0.092 * z],
                0.0012,
            ) + line(
                x,
                y,
                [cx + 0.011 * z, head + 0.043 * z],
                [*cx, head + 0.092 * z],
                0.0012,
            );
            s.coat([0.22; 3], collar * body, Material::Solid);
            if i % 2 == 0 {
                s.coat(
                    [0.16; 3],
                    line(
                        x,
                        y,
                        [cx + 0.015 * z, head + 0.10 * z],
                        [cx + 0.022 * z, foot - 0.02],
                        0.001,
                    ),
                    Material::Solid,
                );
            }
        }
    }
    for xx in [0.13, 0.39, 0.65, 0.91] {
        s.coat(
            [0.21, 0.18, 0.14],
            rect(x, y, [xx - 0.006, 0.07, xx + 0.007, 0.62]),
            Material::Solid,
        );
        s.coat(
            [0.53, 0.46, 0.34],
            line(x, y, [xx + 0.007, 0.075], [xx + 0.007, 0.61], 0.0011),
            Material::Solid,
        );
    }
    s.coat(
        [0.29; 3],
        rect(x, y, [0.12, 0.615, 0.93, 0.638]),
        Material::Solid,
    );
    s.coat(
        [0.63, 0.55, 0.41],
        line(x, y, [0.11, 0.613], [0.94, 0.613], 0.002),
        Material::Solid,
    );
    // Ivory linen and ceramic place settings keep material boundaries readable
    // beneath local candlelight. No noisy luminance texture on flat surfaces.
    let table = polygon(
        x,
        y,
        &[[0.23, 0.66], [0.81, 0.66], [1.05, 1.02], [-0.03, 1.02]],
    );
    if table > 0.5 {
        let fall = 0.74 + 0.10 * (1. - y);
        let fold = 0.025 * (x * 26. + y * 2.).sin();
        s = Sample::new(
            [fall + fold, (fall + fold) * 0.91, (fall + fold) * 0.78],
            Material::Floor,
        );
        let edge = line(x, y, [0.23, 0.665], [-0.03, 1.02], 0.004)
            + line(x, y, [0.81, 0.665], [1.05, 1.02], 0.004);
        s.coat([0.14; 3], edge, Material::Floor);
        for (cx, cy, z) in [(0.31, 0.84, 1.), (0.74, 0.87, 1.1), (0.53, 0.70, 0.55)] {
            s.coat(
                [0.12; 3],
                ellipse(x, y, [cx + 0.008, cy + 0.009], [0.076 * z, 0.027 * z]),
                Material::Floor,
            );
            s.coat(
                [0.97; 3],
                ellipse(x, y, [cx, cy], [0.069 * z, 0.024 * z]),
                Material::Solid,
            );
            s.coat(
                [0.62; 3],
                ellipse(x, y, [cx, cy], [0.057 * z, 0.018 * z]),
                Material::Solid,
            );
            s.coat(
                [0.84, 0.82, 0.76],
                ellipse(x, y, [cx, cy - 0.002], [0.048 * z, 0.013 * z]),
                Material::Solid,
            );
            s.coat(
                [0.38, 0.24, 0.14],
                ellipse(x, y, [cx + 0.010 * z, cy], [0.026 * z, 0.008 * z]),
                Material::Solid,
            );
            for (dx, dy) in [(-0.027, -0.004), (-0.019, 0.003), (-0.032, 0.003)] {
                s.coat(
                    [0.28, 0.39, 0.21],
                    ellipse(x, y, [cx + dx * z, cy + dy * z], [0.010 * z, 0.004 * z]),
                    Material::Solid,
                );
            }
            s.coat(
                [0.74; 3],
                line(
                    x,
                    y,
                    [cx - 0.085 * z, cy - 0.012],
                    [cx - 0.09 * z, cy + 0.029],
                    0.0015,
                ),
                Material::Metal,
            );
            for tine in [-1., 0., 1.] {
                s.coat(
                    [0.74; 3],
                    line(
                        x,
                        y,
                        [cx - 0.085 * z + tine * 0.003 * z, cy - 0.031 * z],
                        [cx - 0.085 * z + tine * 0.003 * z, cy - 0.010 * z],
                        0.0007 * z,
                    ),
                    Material::Metal,
                );
            }
            s.coat(
                [0.56; 3],
                line(
                    x,
                    y,
                    [cx + 0.083 * z, cy - 0.01],
                    [cx + 0.09 * z, cy + 0.028],
                    0.0012,
                ),
                Material::Metal,
            );
        }
        // Soup bowl and carafe. Light refracts at glass edges, not filled white.
        s.coat(
            [0.28; 3],
            ellipse(x, y, [0.52, 0.87], [0.076, 0.023]),
            Material::Solid,
        );
        s.coat(
            [0.90, 0.86, 0.76],
            ellipse(x, y, [0.52, 0.851], [0.070, 0.020]),
            Material::Solid,
        );
        s.coat(
            [0.67, 0.37, 0.15],
            ellipse(x, y, [0.52, 0.847], [0.060, 0.013]),
            Material::Floor,
        );
        let jug = polygon(
            x,
            y,
            &[
                [0.80, 0.71],
                [0.797, 0.744],
                [0.78, 0.773],
                [0.78, 0.825],
                [0.831, 0.825],
                [0.831, 0.773],
                [0.817, 0.744],
                [0.814, 0.71],
            ],
        );
        s.coat([0.15; 3], jug * 0.75, Material::Solid);
        s.coat(
            [0.71; 3],
            line(x, y, [0.785, 0.774], [0.785, 0.821], 0.0015),
            Material::Metal,
        );
        for (cx, cy, z) in [(0.235, 0.762, 0.8), (0.80, 0.78, 0.85)] {
            let bowl = ellipse(x, y, [cx, cy], [0.018 * z, 0.027 * z]);
            let inner = ellipse(x, y, [cx, cy - 0.003], [0.014 * z, 0.024 * z]);
            s.coat(
                [0.87, 0.84, 0.75],
                (bowl - inner).max(0.) * 0.75,
                Material::Metal,
            );
            s.coat(
                [0.73; 3],
                line(x, y, [cx, cy + 0.024 * z], [cx, cy + 0.058 * z], 0.0012),
                Material::Metal,
            );
            s.coat(
                [0.79; 3],
                ellipse(x, y, [cx, cy + 0.060 * z], [0.014 * z, 0.004 * z]),
                Material::Metal,
            );
        }
    }
    for (cx, cy, z) in [(0.40, 0.74, 1.), (0.65, 0.76, 1.1)] {
        s.coat(
            [0.16; 3],
            ellipse(x, y, [cx + 0.012, cy + 0.046 * z], [0.024 * z, 0.008 * z]),
            Material::Solid,
        );
        s.coat(
            [0.56; 3],
            ellipse(x, y, [cx, cy + 0.038 * z], [0.018 * z, 0.005 * z]),
            Material::Metal,
        );
        s.coat(
            [0.43; 3],
            rect(
                x,
                y,
                [
                    cx - 0.003 * z,
                    cy + 0.011 * z,
                    cx + 0.003 * z,
                    cy + 0.035 * z,
                ],
            ),
            Material::Metal,
        );
        s.coat(
            [0.70; 3],
            rect(
                x,
                y,
                [
                    cx - 0.007 * z,
                    cy - 0.045 * z,
                    cx + 0.007 * z,
                    cy + 0.011 * z,
                ],
            ),
            Material::Solid,
        );
    }
    s
}
fn supper_motion(mut s: Sample, x: f32, y: f32, t: f32) -> Sample {
    for (i, (cx, cy, z)) in [(0.40, 0.74, 1.), (0.65, 0.76, 1.1)].iter().enumerate() {
        let phase = t * (2.2 + i as f32 * 0.4) + i as f32 * 5.;
        let flick =
            0.70 + 0.14 * phase.sin() + 0.09 * (phase * 1.73).cos() + 0.06 * (phase * 3.7).sin();
        let bend = 0.0035 * (phase * 0.8).sin() + 0.0018 * (phase * 2.3).sin();
        if y > 0.64 && s.material == Material::Floor {
            let light = (-((x - cx) / (0.115 * z)).powi(2)
                - ((y - (cy + 0.035)) / (0.12 * z)).powi(2))
            .exp();
            for (c, a) in s.color.iter_mut().zip([0.38, 0.25, 0.10]) {
                *c += a * light * flick;
            }
            // Cast object shadows move with the source, surfaces stay put.
            let shadow = (-((x - (cx + 0.065 + bend * 5.)) / (0.016 * z)).powi(2)
                - ((y - (cy + 0.03)) / (0.075 * z)).powi(2))
            .exp();
            s.color = gain(s.color, 1. - shadow * 0.27 * flick);
        }
        if y < 0.64 && s.material == Material::Solid && x < 0.12 {
            let wall = (-((x - cx) / 0.42).powi(2) - ((y - cy) / 0.36).powi(2)).exp();
            for (c, a) in s.color.iter_mut().zip([0.23, 0.15, 0.06]) {
                *c += a * wall * flick;
            }
        }
        // Core and halo respond to the same flame, with two-frequency bending.
        let fy = cy - 0.056 * z;
        let outer = ellipse(
            x,
            y,
            [cx + bend, fy],
            [0.006 * z, 0.016 * z * (0.9 + 0.30 * flick)],
        );
        let core = ellipse(x, y, [cx + bend * 0.4, fy + 0.004], [0.003 * z, 0.009 * z]);
        let halo = (-((x - cx - bend) / (0.025 * z)).powi(2) - ((y - fy) / (0.037 * z)).powi(2))
            .exp()
            * 0.25
            * flick;
        for (c, a) in s.color.iter_mut().zip([1., 0.58, 0.12]) {
            *c += a * (halo + outer * 0.85 + core * 0.65);
        }
        if y < 0.61 && s.material == Material::Window {
            let refl = ellipse(x, y, [cx + bend * 0.5, 0.47], [0.0035 * z, 0.014 * z]);
            for (c, a) in s.color.iter_mut().zip([0.54, 0.36, 0.13]) {
                *c += a * refl * flick;
            }
        }
    }
    // Warm soup steam rises, expands, disperses. At each age the whole plume
    // occupies new cells, rather than a stationary patch bobbing up and down.
    if y > 0.57 && y < 0.85 && x > 0.43 && x < 0.61 {
        for i in 0..7 {
            let age = (t * 0.075 + i as f32 / 7.).fract();
            let cx = 0.52 + 0.014 * (age * 7. + i as f32 * 2.3).sin() + age * 0.025;
            let cy = 0.84 - age * 0.25;
            let alpha = ellipse(x, y, [cx, cy], [0.009 + age * 0.021, 0.015 + age * 0.014])
                * (PI * age).sin().powi(2)
                * 0.20;
            s.color = mix(s.color, [0.71; 3], alpha);
        }
    }
    // Slow rainfall on the exterior side of glass, clipped by the panes and
    // stopped by the sill. Drops travel downward with a finite lifetime.
    if s.material == Material::Window && y < 0.605 && y > 0.08 {
        for i in 0..20 {
            let cx = 0.15 + hash(i, 91) * 0.74;
            if (x - cx).abs() > 0.003 {
                continue;
            }
            let cy = 0.08 + (hash(i, 94) + t * (0.030 + hash(i, 92) * 0.015)).fract() * 0.52;
            let drop = line(x, y, [cx, cy], [cx + 0.001, cy + 0.018], 0.0008);
            s.color = mix(s.color, [0.69; 3], drop * 0.42);
        }
    }
    s
}

fn arcade(x: f32, y: f32) -> Sample {
    let mut s = Sample::new([0.; 3], Material::Air);
    // A sheltered pool opens toward a second, smaller courtyard. Architectural
    // planes get different exposure, with projecting cornices and deep reveals.
    if y > 0.62 {
        s = Sample::new([0.06, 0.22, 0.25], Material::Water);
    }
    let wall = rect(x, y, [0.12, 0.20, 0.94, 0.63]);
    let stone = 0.92 + 0.075 * fbm(x * 47., y * 60., 3) + 0.06 * (1. - y);
    s.coat([stone, stone * 0.98, stone * 0.92], wall, Material::Solid);
    for i in 0..5 {
        let cx = 0.22 + i as f32 * 0.145;
        let hw = 0.052;
        let spring = 0.405;
        let cap = spring - 0.10 * (1. - ((x - cx) / hw).powi(2)).max(0.).sqrt();
        let arch = (x - cx).abs() < hw && y > cap && y < 0.62;
        let outer = (x - cx).abs() < hw + 0.012 && y > cap - 0.014 && y < 0.62;
        if outer {
            s = Sample::new([0.62 - 0.10 * ((x - cx) / hw); 3], Material::Solid);
        }
        if arch {
            let depth = 0.12 + 0.17 * smooth(0.37, 0.62, y);
            s = Sample::new([depth; 3], Material::Solid);
            // A distant sunlit wall and a recessed green garden in alternate bays.
            if y > 0.47 && (x - cx).abs() < 0.039 {
                let v = 0.31 + 0.09 * fbm(x * 69., y * 47., 3);
                s.color = if i % 2 == 0 {
                    [v; 3]
                } else {
                    [v * 0.62, v * 0.78, v * 0.64]
                };
            }
            if y > 0.583 {
                s = Sample::new([0.19, 0.32, 0.34], Material::Solid);
            }
            if x < cx - hw + 0.009 {
                s.color = [0.18; 3];
            }
        }
        // Round columns: cylindrical value rolloff, capitals and bases.
        let col = cx + 0.073;
        if (x - col).abs() < 0.013 && y > 0.30 && y < 0.625 {
            let nx = (x - col) / 0.013;
            let value = 0.52 + 0.63 * (1. - nx * nx).max(0.).sqrt() + 0.10 * nx;
            s = Sample::new([value; 3], Material::Solid);
        }
        s.coat(
            [0.66; 3],
            rect(x, y, [col - 0.019, 0.302, col + 0.019, 0.318]),
            Material::Solid,
        );
        s.coat(
            [0.29; 3],
            rect(x, y, [col - 0.018, 0.318, col + 0.018, 0.325]),
            Material::Solid,
        );
        s.coat(
            [0.51; 3],
            rect(x, y, [col - 0.019, 0.603, col + 0.019, 0.627]),
            Material::Solid,
        );
    }
    for (yy, v) in [(0.20, 0.61), (0.214, 0.27), (0.267, 0.57), (0.279, 0.19)] {
        s.coat(
            [v; 3],
            rect(x, y, [0.10, yy, 0.96, yy + 0.009]),
            Material::Solid,
        );
    }
    // An exposed top parapet, chipped joints, projecting corner blockwork.
    if y > 0.223 && y < 0.263 && x > 0.12 && x < 0.94 {
        let joint = (x * 25.).fract() < 0.035;
        s.color = [if joint { 0.30 } else { 0.48 }; 3];
    }
    // Near left colonnade establishes a vanishing point, with visible soffits.
    for (cx, top, foot, w) in [(0.045, 0.02, 0.96, 0.048), (0.135, 0.13, 0.75, 0.024)] {
        if (x - cx).abs() < w && y > top && y < foot {
            let nx = (x - cx) / w;
            let n = (1. - nx * nx).max(0.).sqrt();
            let value = 0.46 + 0.65 * n + 0.11 * nx;
            s = Sample::new([value; 3], Material::Solid);
            let course = ((y - top) * 37.).fract() < 0.065;
            if course {
                s.color = gain(s.color, 0.76);
            }
        }
        s.coat(
            [0.56; 3],
            rect(x, y, [cx - w * 1.25, top, cx + w * 1.25, top + 0.02]),
            Material::Solid,
        );
        s.coat(
            [0.42; 3],
            rect(
                x,
                y,
                [cx - w * 1.25, foot - 0.018, cx + w * 1.25, foot + 0.01],
            ),
            Material::Solid,
        );
    }
    let walkway = polygon(x, y, &[[0., 1.], [0.20, 1.], [0.27, 0.63], [0.10, 0.63]]);
    if walkway > 0.5 {
        let joints = y > 0.74 && (3. / (y - 0.50)).fract() < 0.10;
        let value = if joints {
            0.23
        } else {
            0.48 + noise(x * 70., y * 65.) * 0.05
        };
        s = Sample::new([value; 3], Material::Floor);
    }
    let right = polygon(x, y, &[[0.98, 0.61], [0.88, 0.61], [0.78, 1.], [1., 1.]]);
    if right > 0.5 {
        let step = (9. / (1.23 - y)).fract();
        let value = if step < 0.14 {
            0.22
        } else if step > 0.88 {
            0.68
        } else {
            0.46
        };
        s = Sample::new([value; 3], Material::Floor);
    }
    s.coat(
        [0.63; 3],
        line(x, y, [0.20, 1.], [0.27, 0.63], 0.004),
        Material::Solid,
    );
    s.coat(
        [0.53; 3],
        line(x, y, [0.78, 1.], [0.88, 0.63], 0.003),
        Material::Solid,
    );
    s
}
fn pink_beads(s: &mut Sample, x: f32, y: f32, t: f32) {
    for i in 0..26 {
        let seed = i as f32;
        let depth = noise(seed * 7.13 + 1.7, 3.9);
        let py = 0.685 + depth * 0.27;
        let px = (noise(seed * 3.7, 8.1) + t * (0.0006 + depth * 0.0007)).rem_euclid(1.16) - 0.08;
        let bob = 0.0015 * (t * 0.82 + seed * 2.3).sin();
        let r = 0.0035 + depth * 0.0045;
        // A small reflection stays attached to its bead and distorts with the
        // water. The sphere itself has a lit upper edge and a shaded underside.
        if (x - px).abs() > r * 1.5 || (y - py).abs() > r * 4. {
            continue;
        }
        let dy = (y - py - bob) / r;
        let dx = (x - px) * 1.7 / r;
        let reflection = ellipse(
            x,
            y,
            [px + 0.001 * (t * 1.3 + seed).sin(), py + r * 1.7 + bob],
            [r * 0.48, r * 1.15],
        );
        s.color = mix(s.color, [0.96, 0.52, 0.73], reflection * 0.23);
        let coverage = 1. - smooth(0.72, 1.18, dx * dx + dy * dy);
        let light = (0.62 - dy * 0.30 - dx * 0.16).clamp(0., 1.);
        let color = mix([0.65, 0.24, 0.43], [1.04, 0.80, 0.91], light);
        s.coat(color, coverage, Material::Water);
    }
}

fn boat(s: &mut Sample, x: f32, y: f32, t: f32) {
    let bx = (t * 0.007 + 0.36).rem_euclid(1.35) - 0.15;
    let by = 0.83 + 0.003 * (t * 0.9).sin();
    let wake = line(
        x,
        y,
        [bx - 0.11, by + 0.032],
        [bx - 0.05, by + 0.012],
        0.0015,
    ) + line(
        x,
        y,
        [bx - 0.11, by - 0.006],
        [bx - 0.05, by + 0.012],
        0.0015,
    );
    s.coat([0.53, 0.62, 0.63], wake * 0.50, Material::Water);
    s.coat(
        [0.07; 3],
        ellipse(x, y, [bx + 0.003, by + 0.023], [0.063, 0.012]),
        Material::Water,
    );
    s.coat(
        [0.26, 0.16, 0.085],
        polygon(
            x,
            y,
            &[
                [bx - 0.065, by],
                [bx + 0.064, by],
                [bx + 0.041, by + 0.023],
                [bx - 0.041, by + 0.023],
            ],
        ),
        Material::Solid,
    );
    s.coat(
        [0.62, 0.39, 0.17],
        ellipse(x, y, [bx, by + 0.002], [0.058, 0.009]),
        Material::Solid,
    );
    s.coat(
        [0.16, 0.115, 0.07],
        ellipse(x, y, [bx, by + 0.002], [0.044, 0.005]),
        Material::Solid,
    );
    for dx in [-0.024, 0.021] {
        s.coat(
            [0.72, 0.50, 0.24],
            line(
                x,
                y,
                [bx + dx, by - 0.003],
                [bx + dx + 0.003, by + 0.008],
                0.002,
            ),
            Material::Solid,
        );
    }
}
