mod palette;

use image::{
    RgbImage, Rgba, RgbaImage,
    imageops::{FilterType, crop_imm, resize},
};
use serde::Deserialize;
use std::{
    env,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
struct Scene {
    piece: String,
    painting: Option<Painting>,
}

#[derive(Deserialize)]
struct Painting {
    number: u8,
    label: String,
    credit: String,
    source: Option<String>,
    crop: Option<f64>,
    motion: String,
    #[serde(default)]
    soft: bool,
}

type Cutout = ([u32; 2], u32, &'static [[u32; 2]]);

fn wivenhoe(frame: &RgbImage, top: u32, output: &Path) -> std::io::Result<()> {
    if frame.dimensions() != (800, 400) || top != 16 {
        return Err(std::io::Error::other(
            "Wivenhoe cutouts require the original 800 × 433 source and y=16 crop",
        ));
    }
    // Coordinates on the NGA source. Each shape stops at the neck and stays above the fence.
    let animals: &[Cutout] = &[
        (
            [79, 347],
            59,
            &[
                [62, 350],
                [65, 345],
                [69, 344],
                [67, 340],
                [71, 339],
                [74, 343],
                [80, 342],
                [85, 348],
                [82, 355],
                [76, 357],
                [68, 357],
                [61, 355],
            ],
        ),
        (
            [209, 308],
            190,
            &[
                [198, 313],
                [203, 308],
                [207, 301],
                [212, 302],
                [216, 309],
                [211, 314],
                [205, 318],
                [201, 321],
                [197, 319],
            ],
        ),
        (
            [291, 312],
            271,
            &[
                [280, 316],
                [283, 312],
                [287, 312],
                [290, 307],
                [295, 308],
                [297, 313],
                [291, 319],
                [284, 322],
                [280, 321],
            ],
        ),
        (
            [334, 319],
            352,
            &[
                [329, 313],
                [336, 314],
                [341, 320],
                [346, 326],
                [348, 331],
                [343, 333],
                [337, 329],
                [332, 325],
                [329, 320],
            ],
        ),
    ];
    cutouts(frame, top, output, "wivenhoe", animals)
}

fn moonlit_bridge(frame: &RgbImage, top: u32, output: &Path) -> std::io::Result<()> {
    if frame.dimensions() != (800, 400) || top != 104 {
        return Err(std::io::Error::other(
            "Moonlit Bridge cutout requires the y=104 crop",
        ));
    }
    let figure: &[Cutout] = &[(
        [711, 425],
        700,
        &[
            [714, 397],
            [719, 397],
            [720, 402],
            [716, 405],
            [716, 409],
            [718, 421],
            [710, 426],
            [704, 425],
            [704, 418],
            [707, 408],
            [711, 405],
            [713, 401],
        ],
    )];
    cutouts(frame, top, output, "moonlit", figure)
}

fn cutouts(
    frame: &RgbImage,
    top: u32,
    output: &Path,
    scene: &str,
    parts: &[Cutout],
) -> std::io::Result<()> {
    let mut plate = frame.clone();
    let mut data = String::from("static SPRITES: &[Sprite] = &[\n");
    for (index, &(pivot, grass, polygon)) in parts.iter().enumerate() {
        let left = polygon.iter().map(|p| p[0]).min().unwrap() - 2;
        let right = polygon.iter().map(|p| p[0]).max().unwrap() + 3;
        let upper = polygon.iter().map(|p| p[1]).min().unwrap() - 2;
        let lower = polygon.iter().map(|p| p[1]).max().unwrap() + 3;
        let mut head = RgbaImage::new(right - left, lower - upper);
        for y in upper..lower {
            for x in left..right {
                let inside = polygon
                    .iter()
                    .zip(polygon.iter().cycle().skip(1))
                    .filter(|&([ax, ay], [bx, by])| {
                        (*ay > y) != (*by > y)
                            && (x as f32)
                                < *ax as f32
                                    + (*bx as f32 - *ax as f32) * (y as f32 - *ay as f32)
                                        / (*by as f32 - *ay as f32)
                    })
                    .count()
                    % 2
                    == 1;
                if inside {
                    let [r, g, b] = frame.get_pixel(x, y - top).0;
                    head.put_pixel(x - left, y - upper, Rgba([r, g, b, 255]));
                    plate.put_pixel(x, y - top, *frame.get_pixel(grass, y - top));
                }
            }
        }
        fs::write(output.join(format!("{scene}-{index}.rgba")), head.as_raw())?;
        writeln!(
            data,
            "Sprite {{ left: {left}, top: {}, width: {}, height: {}, pivot: [{:.1}, {:.1}], pixels: include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{scene}-{index}.rgba\")) }},",
            upper - top, head.width(), head.height(), pivot[0] as f32, (pivot[1] - top) as f32
        )
        .unwrap();
    }
    data.push_str("];\n");
    fs::write(output.join(format!("{scene}-sprites.rs")), data)?;
    fs::write(
        output.join(format!("{scene}-ground.rgb")),
        resize(&plate, 200, 100, FilterType::Lanczos3).as_raw(),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    const COLUMNS: u32 = 200;
    let catalog = "../../../site/data/manifold-catalog.json";
    println!("cargo:rerun-if-changed={catalog}");
    let scenes: Vec<Scene> = serde_json::from_slice(&fs::read(catalog)?)?;
    let mut studies = String::from("pub static PAINTINGS: &[Painting] = &[\n");
    for Scene {
        piece: name,
        painting,
    } in scenes
    {
        let Some(entry) = painting else { continue };
        let reference = entry.source.is_none();
        let motion = match entry.motion.as_str() {
            "flare" => "Motion::Flare".to_owned(),
            "winter" => "Motion::Winter".to_owned(),
            "screen" => "Motion::Screen".to_owned(),
            "moonlight" => "Motion::Moonlight".to_owned(),
            "wivenhoe" => "Motion::Wivenhoe".to_owned(),
            "clouds" => "Motion::Clouds".to_owned(),
            _ => return Err(format!("Invalid study motion for {name}").into()),
        };
        let source = if reference {
            format!("references/{name}.png")
        } else {
            format!("paintings/{name}.jpg")
        };
        println!("cargo:rerun-if-changed={source}");
        let painting = image::open(source)?.to_rgb8();
        let crop = entry.crop.unwrap_or(0.5);
        if !(0.0..=1.0).contains(&crop) {
            return Err(format!("Invalid source crop for {name}").into());
        }
        let aspect = if reference { 1.7 } else { 2.0 };
        let width = painting
            .width()
            .min((painting.height() as f32 * aspect) as u32);
        let height = (width as f32 / aspect).round() as u32;
        let left = (painting.width() - width) / 2;
        let top = ((painting.height() - height) as f64 * crop).round() as u32;
        let frame = crop_imm(&painting, left, top, width, height).to_image();
        match name.as_str() {
            "wivenhoe-park" => wivenhoe(&frame, top, &output)?,
            "moonlit-bridge" => moonlit_bridge(&frame, top, &output)?,
            _ => {}
        }
        let rows = (COLUMNS as f32 / aspect).round() as u32;
        let mut grid = resize(&frame, COLUMNS, rows, FilterType::Lanczos3);
        if reference {
            palette::reduce(&mut grid, 48);
        }
        let photo_y = [
            top as f32 / painting.height() as f32,
            height as f32 / painting.height() as f32,
        ];
        fs::write(output.join(format!("{name}.rgb")), grid.as_raw())?;
        writeln!(
            studies,
            "Painting {{ piece: {name:?}, number: {}, label: {:?}, caption: {:?}, source: {:?}, motion: {motion}, soft: {}, photo_y: {photo_y:?}, rgb: include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{name}.rgb\")) }},",
            entry.number,
            entry.label,
            entry.credit,
            entry.source.as_deref().unwrap_or_default(),
            entry.soft,
        )?;
    }
    studies.push_str("];\n");
    fs::write(output.join("paintings.rs"), studies)?;
    Ok(())
}
