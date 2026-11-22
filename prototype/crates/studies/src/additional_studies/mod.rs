//! Original short studies composed directly at the character-grid resolution.
mod ears;
mod exteriors;
mod interiors;
mod missing;
mod objects;
mod paint;

use super::{Material, Sample};
use paint::Painting;

pub(crate) const COIN: u8 = 17;
pub(crate) const FLYING_AWAY: u8 = 18;
pub(crate) const STONE_FROG: u8 = 19;
pub(crate) const MILK_COW_CAT: u8 = 20;
pub(crate) const MISSING: u8 = 21;
pub(crate) const AFTER_SNOW: u8 = 22;
pub(crate) const JUST_A_SIGN: u8 = 23;
pub(crate) const EARS: u8 = 24;
pub(crate) const CANDLE: u8 = 25;

pub(super) fn duration(kind: u8) -> Option<f32> {
    match kind {
        COIN | STONE_FROG | JUST_A_SIGN | EARS => Some(12.),
        FLYING_AWAY | MILK_COW_CAT | CANDLE => Some(13.),
        MISSING | AFTER_SNOW => Some(14.),
        _ => None,
    }
}

pub(super) struct Artwork {
    kind: u8,
    base: Painting,
    frame: Painting,
}

impl Artwork {
    pub(super) fn new(kind: u8, cols: usize, rows: usize) -> Self {
        let mut base = Painting::new(cols, rows);
        match kind {
            COIN => objects::coin_background(&mut base),
            FLYING_AWAY => exteriors::sparrow_background(&mut base),
            STONE_FROG => exteriors::garden_background(&mut base),
            MILK_COW_CAT => interiors::cat_background(&mut base),
            MISSING => missing::background(&mut base),
            AFTER_SNOW => exteriors::snow_background(&mut base),
            JUST_A_SIGN => objects::sign_background(&mut base),
            EARS => ears::background(&mut base),
            CANDLE => interiors::candle_background(&mut base),
            _ => unreachable!("unknown additional study"),
        }
        base.wash(|[x, y], c| {
            let edges = (2. * x - 1.).powi(4) * 0.09 + (2. * y - 1.).powi(4) * 0.13;
            super::gain(c, 1. - edges)
        });
        let mut artwork = Self {
            kind,
            frame: base.clone(),
            base,
        };
        artwork.update(0.);
        artwork
    }

    pub(super) fn update(&mut self, seconds: f32) {
        self.frame.pixels.copy_from_slice(&self.base.pixels);
        let t = seconds.clamp(0., duration(self.kind).unwrap());
        match self.kind {
            COIN => objects::coin(&mut self.frame, t),
            FLYING_AWAY => exteriors::sparrow(&mut self.frame, t),
            STONE_FROG => exteriors::petals(&mut self.frame, t),
            MILK_COW_CAT => interiors::cat(&mut self.frame, t),
            MISSING => missing::light(&mut self.frame, t),
            AFTER_SNOW => exteriors::snow(&mut self.frame, t),
            JUST_A_SIGN => objects::traffic(&mut self.frame, t),
            EARS => ears::pulse(&mut self.frame, t),
            CANDLE => interiors::candle(&mut self.frame, t),
            _ => unreachable!("unknown additional study"),
        }
    }

    pub(super) fn sample(&self, x: usize, y: usize) -> Sample {
        let color = self.frame.pixels[y * self.frame.cols + x];
        let gamma = match self.kind {
            STONE_FROG => 0.85,
            CANDLE => 1.,
            _ => 0.93,
        };
        // Lift illustrated midtones before glyph coverage is chosen. Apply
        // one scale to all channels so the lift preserves the scene's hues.
        let peak = color.into_iter().fold(0., f32::max).max(0.001);
        Sample::new(super::gain(color, peak.powf(gamma - 1.)), Material::Solid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn night_lights_move_behind_the_sleeping_cat() {
        let mut artwork = Artwork::new(MILK_COW_CAT, 136, 80);
        let opening = artwork.frame.pixels.clone();
        artwork.update(6.7);
        let frame = &artwork.frame.pixels;
        for row in 55..58 {
            for col in 78..89 {
                let index = row * 136 + col;
                assert_eq!(frame[index], opening[index]);
            }
        }
        assert!(frame[136 * 11..136 * 46] != opening[136 * 11..136 * 46]);
    }

    #[test]
    fn every_preview_animates_and_seeks_independently_of_playback_order() {
        for kind in COIN..=CANDLE {
            let mut artwork = Artwork::new(kind, 102, 60);
            let opening = artwork.frame.pixels.clone();
            artwork.update(4.75);
            let middle = artwork.frame.pixels.clone();
            artwork.update(duration(kind).unwrap());
            assert_ne!(
                opening, artwork.frame.pixels,
                "kind {kind} ends differently"
            );
            artwork.update(4.75);
            assert_eq!(middle, artwork.frame.pixels, "kind {kind} seeks");
            artwork.update(0.);
            assert_eq!(opening, artwork.frame.pixels, "kind {kind} replays");
        }
    }
}
