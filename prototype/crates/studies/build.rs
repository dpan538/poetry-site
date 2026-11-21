fn main() -> Result<(), Box<dyn std::error::Error>> {
    use image::imageops::{FilterType, crop_imm, resize};
    use std::{env, fs, path::PathBuf};

    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    for name in [
        "shadow-landing",
        "thermos-table",
        "manifesto-crt",
        "white-tree-trunk",
        "dancing-mural",
        "dancing-facade",
        "feather-ground",
        "black-paint-canvas",
    ] {
        let path = format!("sources/{name}.png");
        println!("cargo:rerun-if-changed={path}");
        let source = image::open(path)?.into_rgb8();
        let height = source.height();
        let width = (height * 17 / 10).min(source.width());
        let left = (source.width() - width) / 2;
        let crop = crop_imm(&source, left, 0, width, height).to_image();
        let plate = resize(&crop, 340, 200, FilterType::Triangle);
        fs::write(output.join(format!("{name}.rgb")), plate.as_raw())?;
    }

    for (name, width, height) in [("thermos-bottle", 80, 241), ("feather-cutout", 128, 76)] {
        let path = format!("sources/{name}.png");
        println!("cargo:rerun-if-changed={path}");
        let source = image::open(path)?.into_rgba8();
        let (mut left, mut top, mut right, mut bottom) = (source.width(), source.height(), 0, 0);
        for (x, y, pixel) in source.enumerate_pixels() {
            if pixel.0[3] > 32 {
                left = left.min(x);
                top = top.min(y);
                right = right.max(x + 1);
                bottom = bottom.max(y + 1);
            }
        }
        if left >= right || top >= bottom {
            return Err(format!("{name} has no opaque pixels").into());
        }
        let crop = crop_imm(&source, left, top, right - left, bottom - top).to_image();
        let sprite = resize(&crop, width, height, FilterType::Triangle);
        fs::write(output.join(format!("{name}.rgba")), sprite.as_raw())?;
    }
    Ok(())
}
