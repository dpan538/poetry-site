use anyhow::{Context, Result, ensure};
use std::{fs, path::Path};

pub struct Work {
    pub slug: String,
    pub blocks: Vec<String>,
    pub prose: bool,
    pub order: usize,
    pub scene: Option<String>,
}

pub fn read(root: &Path) -> Result<Vec<Work>> {
    let mut works = Vec::new();
    for world in ["their-world", "bless-you", "still-life"] {
        let directory = root.join("src/content").join(world);
        let mut paths = fs::read_dir(directory)?
            .map(|e| e.map(|e| e.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        paths.sort();
        for path in paths {
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let raw = fs::read_to_string(&path)?.replace("\r\n", "\n");
            let mut parts = raw.splitn(3, "---");
            ensure!(
                parts.next() == Some(""),
                "invalid frontmatter: {}",
                path.display()
            );
            let frontmatter = parts.next().context("missing frontmatter")?;
            let prose = frontmatter.lines().any(|line| line.trim() == "kind: prose");
            let scene = frontmatter
                .lines()
                .find_map(|line| line.strip_prefix("manifoldScene:"))
                .map(|value| value.trim().trim_matches(['\'', '"']).to_owned());
            let order = frontmatter
                .lines()
                .find_map(|line| line.strip_prefix("order:"))
                .and_then(|value| value.trim().parse().ok())
                .with_context(|| format!("missing work order: {}", path.display()))?;
            let body = parts.next().context("missing poem body")?;
            let lines: Vec<_> = body.lines().collect();
            let blocks = lines
                .split(|line| line.trim().is_empty())
                .filter(|lines| !lines.is_empty())
                .map(|lines| {
                    lines
                        .iter()
                        .map(|line| {
                            if prose {
                                line.strip_prefix('*')
                                    .and_then(|value| value.strip_suffix('*'))
                                    .unwrap_or(line)
                            } else {
                                line
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .collect::<Vec<_>>();
            ensure!(!blocks.is_empty(), "empty poem: {}", path.display());
            let slug = path
                .file_stem()
                .and_then(|s| s.to_str())
                .context("invalid poem filename")?;
            works.push(Work {
                slug: format!("{world}/{slug}"),
                blocks,
                prose,
                order,
                scene,
            });
        }
    }
    Ok(works)
}
