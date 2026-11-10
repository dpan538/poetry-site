mod art;
mod content;
mod embedding;
mod semantic;

use anyhow::Result;
use serde::Serialize;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Serialize)]
struct Entry<'a> {
    piece: &'a str,
    description: &'a str,
    excerpt: &'a str,
    keywords: &'a [String],
}

fn main() -> Result<()> {
    let prototype = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = prototype.join("..");
    let evidence = prototype.join("target/manifold-evidence");
    fs::create_dir_all(&evidence)?;

    let works = content::read(&root)?;
    let catalog: Vec<semantic::Scene> =
        serde_json::from_slice(&fs::read(root.join("site/data/manifold-catalog.json"))?)?;
    #[derive(serde::Deserialize)]
    struct Existing {
        piece: String,
    }
    let existing = if std::env::args().any(|arg| arg == "--preserve-scenes") {
        let manifest: BTreeMap<String, Existing> =
            serde_json::from_slice(&fs::read(root.join("site/data/manifold.json"))?)?;
        Some(
            manifest
                .into_iter()
                .map(|(slug, entry)| (slug, entry.piece))
                .collect(),
        )
    } else {
        None
    };
    let study = semantic::rank(
        &works,
        &catalog,
        &prototype.join("target/semantic-cache"),
        existing.as_ref(),
    )?;
    fs::write(
        evidence.join("ranking.json"),
        serde_json::to_vec_pretty(&study)?,
    )?;

    if std::env::args().any(|arg| arg == "--typst-plates") {
        let output = evidence.join("typst-plates");
        fs::create_dir_all(&output)?;
        for index in 0..8 {
            art::build(index, &output.join(format!("{index}.png")))?;
        }
    }

    let entries: BTreeMap<_, _> = study
        .picks
        .iter()
        .map(|(slug, choice)| {
            (
                slug,
                Entry {
                    piece: &catalog[choice.scene].piece,
                    description: &catalog[choice.scene].description,
                    excerpt: &choice.passage,
                    keywords: &choice.keywords,
                },
            )
        })
        .collect();
    fs::write(
        evidence.join("manifold.json"),
        serde_json::to_string_pretty(&entries)? + "\n",
    )?;
    eprintln!("Generated MANIFOLD scenes for {} works", entries.len());
    Ok(())
}
