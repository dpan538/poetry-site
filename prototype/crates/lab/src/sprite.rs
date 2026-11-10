pub(super) struct Sprite {
    pub(super) left: i32,
    pub(super) top: i32,
    pub(super) width: i32,
    pub(super) height: i32,
    pub(super) pivot: [f32; 2],
    pub(super) pixels: &'static [u8],
}

impl Sprite {
    pub(super) fn near(&self, x: f32, y: f32) -> bool {
        (x - self.pivot[0]).abs() < 30.0 && (y - self.pivot[1]).abs() < 23.0
    }

    fn pixel(&self, x: i32, y: i32) -> [f32; 4] {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return [0.0; 4];
        }
        let start = ((y * self.width + x) * 4) as usize;
        std::array::from_fn(|i| f32::from(self.pixels[start + i]) / 255.0)
    }

    pub(super) fn sample(&self, x: f32, y: f32, angle: f32) -> [f32; 4] {
        let [cx, cy] = self.pivot;
        let (sin, cos) = angle.sin_cos();
        let x0 = x - cx;
        let y0 = y - cy;
        let x = cx + cos * x0 + sin * y0 - self.left as f32;
        let y = cy - sin * x0 + cos * y0 - self.top as f32;
        let (i, j) = (x.floor() as i32, y.floor() as i32);
        let (u, v) = (x - i as f32, y - j as f32);
        let [a, b, c, d] = [
            self.pixel(i, j),
            self.pixel(i + 1, j),
            self.pixel(i, j + 1),
            self.pixel(i + 1, j + 1),
        ];
        std::array::from_fn(|k| {
            (a[k] * (1.0 - u) + b[k] * u) * (1.0 - v) + (c[k] * (1.0 - u) + d[k] * u) * v
        })
    }
}
