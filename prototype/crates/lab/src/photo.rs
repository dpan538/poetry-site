use super::{CELL, COLS, Renderer};
use wasm_bindgen::Clamped;
use web_sys::ImageData;

impl Renderer {
    pub(super) fn draw_photo(&mut self, cols: usize, rows: usize, seconds: f32) {
        let study = &self.study;
        let pixels = &mut self.photo_pixels;
        let grain = &self.grain;
        let entrance_motion = self.entrance_motion;
        let size = CELL as usize;
        let width = cols * size;
        let height = rows * size;
        pixels.resize(width * height * 4, 255);
        let exposure = study.exposure(seconds);
        let settle = if entrance_motion {
            let remaining = (1.0 - seconds / 1.5).max(0.0);
            remaining * remaining
        } else {
            0.0
        };
        let scale_x = COLS as f32 / cols as f32;
        let scale_y = study.painting().rows() as f32 / rows as f32;
        for y in 0..rows {
            for x in 0..cols {
                let seed = grain[y * cols + x];
                let color = study.photo_ink(
                    (x as f32 + 0.5) * scale_x - 0.5,
                    (y as f32 + 0.5) * scale_y - 0.5,
                    exposure,
                );
                let peak = color[0].max(color[1]).max(color[2]);
                let brightness = color[0] * 0.2126 + color[1] * 0.7152 + color[2] * 0.0722;
                let radius =
                    0.55 + (brightness.sqrt() + (seed - 0.5) * 0.12).clamp(0.0, 1.0) * 1.67;
                let coverage = std::f32::consts::PI * radius.powi(2) / (size * size) as f32;
                let contrast = ((1.0 / peak.max(0.02) - 1.0) / (1.0 - coverage)).clamp(0.12, 0.52);
                let phase = seed * std::f32::consts::TAU + seconds * 16.0;
                let center_x = 2.5 + phase.sin() * settle * 0.66;
                let center_y = 2.5 + (phase * 1.29).cos() * settle * 0.55;
                for py in 0..size {
                    for px in 0..size {
                        let distance = (px as f32 + 0.5 - center_x).powi(2)
                            + (py as f32 + 0.5 - center_y).powi(2);
                        let dot =
                            (((radius + 0.55).powi(2) - distance) / (2.2 * radius)).clamp(0.0, 1.0);
                        let gain = 1.0 + (dot - coverage) * contrast;
                        let start = ((y * size + py) * width + x * size + px) * 4;
                        for (index, pigment) in color.into_iter().enumerate() {
                            pixels[start + index] =
                                (255.0 * (pigment * gain).clamp(0.0, 1.0)).round() as u8;
                        }
                    }
                }
            }
        }
        if let Ok(frame) = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(pixels.as_slice()),
            width as u32,
            height as u32,
        ) {
            let _ = self.ctx.put_image_data(&frame, 0.0, 0.0);
        }
    }
}
