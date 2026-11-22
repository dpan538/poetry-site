use super::super::{Color, mix, smooth, tint};

pub(super) type Point = [f32; 2];

#[derive(Clone)]
pub(super) struct Painting {
    pub(super) cols: usize,
    rows: usize,
    pub(super) pixels: Vec<Color>,
}

impl Painting {
    pub(super) fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            pixels: vec![[0.; 3]; cols * rows],
        }
    }

    pub(super) fn wash(&mut self, mut shade: impl FnMut(Point, Color) -> Color) {
        for (index, pixel) in self.pixels.iter_mut().enumerate() {
            let x = (index % self.cols) as f32 + 0.5;
            let y = (index / self.cols) as f32 + 0.5;
            *pixel = shade([x / self.cols as f32, y / self.rows as f32], *pixel);
        }
    }

    fn fill(&mut self, bounds: [f32; 4], color: u32, alpha: impl Fn(Point) -> f32) {
        let cols = self.cols as f32;
        let rows = self.rows as f32;
        let left = (bounds[0] * cols).floor().max(0.) as usize;
        let top = (bounds[1] * rows).floor().max(0.) as usize;
        let right = (bounds[2] * cols).ceil().min(cols) as usize;
        let bottom = (bounds[3] * rows).ceil().min(rows) as usize;
        let ink = tint(color);
        for row in top..bottom {
            for col in left..right {
                let i = row * self.cols + col;
                let coverage = alpha([(col as f32 + 0.5) / cols, (row as f32 + 0.5) / rows]);
                self.pixels[i] = mix(self.pixels[i], ink, coverage);
            }
        }
    }

    pub(super) fn oval(&mut self, [cx, cy]: Point, [rx, ry]: Point, color: u32) {
        self.ellipse([cx, cy], [rx, ry], color, 1., false);
    }

    pub(super) fn glow(&mut self, center: Point, radius: Point, color: u32, opacity: f32) {
        self.ellipse(center, radius, color, opacity, true);
    }

    fn ellipse(&mut self, [cx, cy]: Point, [rx, ry]: Point, color: u32, opacity: f32, soft: bool) {
        let aa = 0.55 / (rx * self.cols as f32).min(ry * self.rows as f32);
        self.fill(
            [
                cx - rx - 0.01,
                cy - ry - 0.01,
                cx + rx + 0.01,
                cy + ry + 0.01,
            ],
            color,
            |[x, y]| {
                let d = ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2);
                if soft {
                    (1. - d).max(0.).powi(2) * opacity
                } else {
                    ((1. + aa - d.sqrt()) / (2. * aa)).clamp(0., 1.) * opacity
                }
            },
        );
    }

    pub(super) fn rect(&mut self, [l, t, r, b]: [f32; 4], color: u32) {
        let dx = 1. / self.cols as f32;
        let dy = 1. / self.rows as f32;
        self.fill([l, t, r, b], color, |[x, y]| {
            let width = (r.min(x + dx * 0.5) - l.max(x - dx * 0.5)).clamp(0., dx);
            let height = (b.min(y + dy * 0.5) - t.max(y - dy * 0.5)).clamp(0., dy);
            width * height / (dx * dy)
        });
    }

    pub(super) fn polygon(&mut self, points: &[Point], color: u32) {
        self.translucent(points, color, 1.);
    }

    pub(super) fn translucent(&mut self, points: &[Point], color: u32, opacity: f32) {
        let bounds = points
            .iter()
            .fold([1., 1., 0., 0.], |[l, t, r, b], &[x, y]| {
                [x.min(l), y.min(t), x.max(r), y.max(b)]
            });
        let dx = 0.26 / self.cols as f32;
        let dy = 0.26 / self.rows as f32;
        self.fill(bounds, color, |[x, y]| {
            let mut hits = 0;
            for (sx, sy) in [
                (x - dx, y - dy),
                (x + dx, y - dy),
                (x - dx, y + dy),
                (x + dx, y + dy),
            ] {
                let mut inside = false;
                let mut previous = points[points.len() - 1];
                for &next in points {
                    if (next[1] > sy) != (previous[1] > sy)
                        && sx
                            < next[0]
                                + (sy - next[1]) * (previous[0] - next[0]) / (previous[1] - next[1])
                    {
                        inside = !inside;
                    }
                    previous = next;
                }
                hits += i32::from(inside);
            }
            hits as f32 * 0.25 * opacity
        });
    }

    pub(super) fn line(&mut self, a: Point, b: Point, radius: f32, color: u32) {
        let halo = radius + 0.009;
        let [vx, vy] = [(b[0] - a[0]) * 1.7, b[1] - a[1]];
        let length = (vx * vx + vy * vy).max(0.0000001);
        let aa = 0.62 / self.rows as f32;
        self.fill(
            [
                a[0].min(b[0]) - halo,
                a[1].min(b[1]) - halo,
                a[0].max(b[0]) + halo,
                a[1].max(b[1]) + halo,
            ],
            color,
            |[x, y]| {
                let [dx, dy] = [(x - a[0]) * 1.7, y - a[1]];
                let t = ((dx * vx + dy * vy) / length).clamp(0., 1.);
                1. - smooth(
                    (radius - aa).max(0.),
                    radius + aa,
                    ((dx - vx * t).powi(2) + (dy - vy * t).powi(2)).sqrt(),
                )
            },
        );
    }
}

pub(super) fn shifted([x, y]: Point, [tx, ty]: Point, angle: f32) -> Point {
    let (s, c) = angle.sin_cos();
    [x + tx * c - ty * s / 1.7, y + tx * s * 1.7 + ty * c]
}
