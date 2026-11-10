use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::Path,
};

pub const DOMAIN: &str = "https://daipan.ink";
pub const WORLDS: [(&str, &str, &str, &str); 3] = [
    ("their-world", "Their World", "I", "America/New_York"),
    ("bless-you", "Bless You", "II", "Asia/Shanghai"),
    ("still-life", "Still Life", "III", "Australia/Brisbane"),
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub title: String,
    pub world: String,
    pub order: usize,
    pub first_line: Option<String>,
    pub written_year: Option<u16>,
    pub source: Option<String>,
    pub column_alignment: Option<ColumnAlignment>,
    pub manifold_scene: Option<String>,
    #[serde(default)]
    pub kind: Kind,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnAlignment {
    pub left_line: String,
    pub right_stanza: String,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Poem,
    Prose,
}

pub struct Work {
    pub id: String,
    pub meta: Metadata,
    pub blocks: Vec<Vec<String>>,
}

impl Work {
    pub fn path(&self) -> String {
        format!("/{}/{}", self.meta.world, self.id)
    }
    pub fn poem(&self) -> bool {
        self.meta.kind == Kind::Poem
    }
    pub fn citation_year(&self) -> Option<u16> {
        self.meta
            .written_year
            .or_else(|| self.meta.source.is_none().then_some(2025))
    }
    pub fn opening(&self) -> &str {
        self.meta
            .first_line
            .as_deref()
            .or_else(|| {
                self.blocks
                    .iter()
                    .flatten()
                    .find(|l| !section(l))
                    .map(String::as_str)
            })
            .unwrap_or("")
    }
}

pub fn section(line: &str) -> bool {
    line == "(end)"
        || line
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
}

pub fn read(root: &Path) -> Result<Vec<Work>> {
    let mut works = Vec::new();
    for (world, ..) in WORLDS {
        let mut orders = HashSet::new();
        for file in fs::read_dir(root.join("src/content").join(world))? {
            let path = file?.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let raw = fs::read_to_string(&path)?.replace("\r\n", "\n");
            let raw = raw
                .strip_prefix("---\n")
                .with_context(|| format!("Missing frontmatter: {}", path.display()))?;
            let (header, body) = raw
                .split_once("\n---\n")
                .with_context(|| format!("Unclosed frontmatter: {}", path.display()))?;
            let meta: Metadata =
                serde_yaml::from_str(header).with_context(|| path.display().to_string())?;
            ensure!(meta.world == world, "Wrong volume: {}", path.display());
            ensure!(
                orders.insert(meta.order),
                "Repeated order in {world}: {}",
                meta.order
            );
            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .context("Invalid slug")?
                .to_owned();
            ensure!(
                !id.is_empty()
                    && id
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
                "Invalid slug: {id}"
            );
            let lines: Vec<_> = body.lines().map(str::trim_end).collect();
            let blocks = lines
                .split(|line| line.trim().is_empty())
                .filter(|s| !s.is_empty())
                .map(|lines| lines.iter().map(|line| line.to_string()).collect())
                .collect::<Vec<_>>();
            ensure!(!blocks.is_empty(), "Empty work: {id}");
            works.push(Work { id, meta, blocks });
        }
    }
    works.sort_by_key(|w| {
        (
            WORLDS
                .iter()
                .position(|(name, ..)| *name == w.meta.world)
                .unwrap(),
            w.meta.order,
        )
    });
    Ok(works)
}

pub fn counts<'a>(works: impl Iterator<Item = &'a Work>) -> (usize, usize) {
    works.fold(
        (0, 0),
        |(p, s), w| if w.poem() { (p + 1, s) } else { (p, s + 1) },
    )
}

pub fn navigation(works: &[Work], root: &Path) -> Result<Vec<Value>> {
    let baseline: Vec<Value> =
        serde_json::from_slice(&fs::read(root.join("site/data/navigation.json"))?)?;
    let positions: BTreeMap<_, _> = baseline
        .iter()
        .filter_map(|row| Some(((row["world"].as_str()?, row["id"].as_str()?), row)))
        .collect();
    let mut rows: Vec<_> = works.iter().map(|w| {
        let mut row = positions.get(&(w.meta.world.as_str(), w.id.as_str())).map(|r| (*r).clone())
            .unwrap_or_else(|| json!({"id":w.id,"world":w.meta.world,"emotionScore":0.5,"contentTypes":[],"atlasRelations":[]}));
        row["title"] = json!(w.meta.title);
        row["firstLine"] = json!(w.opening());
        row
    }).collect();
    // Preserve the stable order used to break ties in the published feed.
    rows.sort_by(|a, b| {
        let world = |v: &Value| {
            WORLDS
                .iter()
                .position(|(slug, ..)| Some(*slug) == v["world"].as_str())
        };
        world(a)
            .cmp(&world(b))
            .then_with(|| a["id"].as_str().cmp(&b["id"].as_str()))
    });
    Ok(rows)
}
