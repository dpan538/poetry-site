use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

fn files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            found.extend(files(&entry.path())?);
        } else {
            found.push(entry.path());
        }
    }
    found.sort();
    Ok(found)
}

fn digest(entries: &[(String, Vec<u8>)]) -> String {
    let mut hash = Sha256::new();
    for (path, bytes) in entries {
        hash.update((path.len() as u64).to_le_bytes());
        hash.update(path.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    format!("{:x}", hash.finalize())[..24].to_owned()
}

fn move_asset(output: &Path, source: &Path, url: &str) -> Result<()> {
    let target = output.join(url.trim_start_matches('/'));
    fs::create_dir_all(target.parent().context("Asset parent directory")?)?;
    fs::rename(source, target)?;
    Ok(())
}

/// Keep immutable URLs tied to their bytes, not a deploy timestamp. Large leaf
/// assets have independent hashes: changing the catalogue must not redownload
/// an unchanged reader WASM. JS modules remain together so relative imports and
/// wasm-bindgen's snippet hierarchy resolve inside the same module bundle.
pub fn fingerprint(output: &Path) -> Result<()> {
    let assets = output.join("assets");
    let source = files(&assets)?;
    let mut leaves = BTreeMap::new();
    let mut modules = Vec::new();
    for path in source {
        let relative = path.strip_prefix(&assets)?.to_string_lossy().into_owned();
        let bytes = fs::read(&path)?;
        if matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("wasm" | "css" | "json")
        ) {
            let hash = digest(&[(relative.clone(), bytes)]);
            let url = format!("/assets/immutable/{hash}/{relative}");
            move_asset(output, &path, &url)?;
            leaves.insert(relative, url);
        } else {
            modules.push((relative, bytes));
        }
    }
    for (relative, bytes) in &mut modules {
        if !relative.ends_with(".js") {
            continue;
        }
        let mut js = String::from_utf8(std::mem::take(bytes))?;
        for (leaf, url) in &leaves {
            let name = Path::new(leaf)
                .file_name()
                .context("Asset filename")?
                .to_string_lossy();
            // Explicit scene WASM URLs in the wasm-bindgen player snippet.
            js = js.replace(&format!("sceneAsset(\"{name}\")"), &format!("\"{url}\""));
            // Default wasm-bindgen URLs and the bootstrap catalogue/WASM URLs.
            for quote in ['\'', '"'] {
                for prefix in ["", "./"] {
                    js = js.replace(
                        &format!("{quote}{prefix}{name}{quote}"),
                        &format!("{quote}{url}{quote}"),
                    );
                }
            }
        }
        *bytes = js.into_bytes();
    }
    let hash = digest(&modules);
    let base = format!("/assets/immutable/{hash}");
    let mut replacements = BTreeMap::new();
    for (relative, bytes) in modules {
        let source = assets.join(&relative);
        fs::write(&source, bytes)?;
        let url = format!("{base}/{relative}");
        move_asset(output, &source, &url)?;
        replacements.insert(format!("/assets/{relative}"), url);
    }
    replacements.extend(
        leaves
            .into_iter()
            .map(|(path, url)| (format!("/assets/{path}"), url)),
    );
    for path in files(output)? {
        if path.extension().and_then(|s| s.to_str()) != Some("html") {
            continue;
        }
        let mut page = fs::read_to_string(&path)?;
        for (source, target) in &replacements {
            page = page.replace(source, target);
        }
        fs::write(path, page)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str, catalog: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("daipan-assets-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("assets/snippets/reader")).unwrap();
        for (path, bytes) in [
            ("assets/browser_bg.wasm", "unchanged WASM"),
            ("assets/base.css", "body { color: black; }"),
            ("assets/reading.json", catalog),
            (
                "assets/start.js",
                "import './browser.js'; fetch(new URL('./reading.json', import.meta.url));",
            ),
            (
                "assets/browser.js",
                "import './snippets/reader/player.js'; new URL('browser_bg.wasm', import.meta.url);",
            ),
            (
                "assets/snippets/reader/player.js",
                "export const version = 1;",
            ),
            (
                "index.html",
                "<link href=\"/assets/base.css\"><script src=\"/assets/start.js\"></script>",
            ),
        ] {
            fs::write(root.join(path), bytes).unwrap();
        }
        root
    }

    fn asset(root: &Path, name: &str) -> PathBuf {
        files(&root.join("assets/immutable"))
            .unwrap()
            .into_iter()
            .find(|path| path.file_name().and_then(|s| s.to_str()) == Some(name))
            .unwrap()
    }

    #[test]
    fn content_changes_invalidate_only_the_affected_leaf_assets() {
        let first = fixture("first", "{\"works\":[]}");
        let same = fixture("same", "{\"works\":[]}");
        let changed = fixture("changed", "{\"works\":[{}]}");
        for root in [&first, &same, &changed] {
            fingerprint(root).unwrap();
        }
        for name in ["browser_bg.wasm", "base.css"] {
            assert_eq!(
                asset(&first, name).strip_prefix(&first).unwrap(),
                asset(&changed, name).strip_prefix(&changed).unwrap()
            );
        }
        for name in ["reading.json", "start.js"] {
            assert_eq!(
                asset(&first, name).strip_prefix(&first).unwrap(),
                asset(&same, name).strip_prefix(&same).unwrap()
            );
            assert_ne!(
                asset(&first, name).strip_prefix(&first).unwrap(),
                asset(&changed, name).strip_prefix(&changed).unwrap()
            );
        }
        let js = fs::read_to_string(asset(&first, "start.js")).unwrap();
        assert!(js.contains("import './browser.js'"));
        assert!(js.contains("/assets/immutable/"));
        assert!(!js.contains("new URL('./reading.json'"));
        let browser = fs::read_to_string(asset(&first, "browser.js")).unwrap();
        assert!(browser.contains("import './snippets/reader/player.js'"));
        assert!(browser.contains("new URL('/assets/immutable/"));
        assert!(
            !fs::read_to_string(first.join("index.html"))
                .unwrap()
                .contains("/assets/base.css")
        );
        for root in [first, same, changed] {
            fs::remove_dir_all(root).unwrap();
        }
    }
}
