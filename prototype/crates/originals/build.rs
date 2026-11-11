#[path = "../lab/palette.rs"]
mod palette;

use image::imageops::{FilterType, resize};
use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    for scene in ["dusk-gauze", "crown-tiles", "drowned-water"] {
        let path = format!("sources/{scene}.png");
        println!("cargo:rerun-if-changed={path}");
        let source = image::open(path)?.to_rgb8();
        let mut image = resize(&source, 340, 200, FilterType::Lanczos3);
        palette::reduce(&mut image, 40);
        fs::write(output.join(format!("{scene}.rgb")), image.as_raw())?;
    }
    Ok(())
}
