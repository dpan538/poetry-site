mod corpus;
mod render;

use anyhow::{Context, Result, bail, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tiny_http::{Header, Method, Response, Server, StatusCode};
use wasm_bindgen_cli_support::Bindgen;

fn copy_tree(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        let target = target.join(name);
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn build(root: &Path, output: &Path, wasm: &Path) -> Result<()> {
    fs::create_dir_all(output)?;
    let works = corpus::read(root)?;
    let nav = corpus::navigation(&works, root)?;
    copy_tree(&root.join("public"), output)?;
    let assets = output.join("assets");
    copy_tree(&root.join("site/assets"), &assets)?;
    Bindgen::new()
        .web(true)?
        .input_path(wasm)
        .out_name("browser")
        .typescript(false)
        .generate(&assets)
        .with_context(|| format!("Compile browser WASM first: {}", wasm.display()))?;
    build_paintings(&root.join("prototype/target"), &assets.join("manifold"))?;
    fs::write(
        assets.join("start.js"),
        r#"import init from './browser.js';

await init({ module_or_path: new URL('./browser_bg.wasm', import.meta.url) });
"#,
    )?;
    render::generate(root, output, &works, &nav)?;
    render::indexes(root, output, &works, &nav)?;
    fs::write(output.join(".daipan-build"), "")?;
    let (poems, prose) = corpus::counts(works.iter());
    println!(
        "Built {} pages ({poems} poems, {prose} prose pieces) in {}",
        works.len() + 8,
        output.display()
    );
    Ok(())
}

fn replace_build(root: &Path, output: &Path, wasm: &Path) -> Result<()> {
    let name = output
        .file_name()
        .and_then(|s| s.to_str())
        .context("Invalid output directory")?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    if output.exists() {
        let previous_build = output.join(".daipan-build").is_file()
            || (output.join("sitemap.xml").is_file() && output.join("_astro").is_dir());
        ensure!(
            previous_build || fs::read_dir(output)?.next().is_none(),
            "Output contains files outside a recognised site build"
        );
    }
    let id = std::process::id();
    let stage = parent.join(format!(".{name}.stage-{id}"));
    let backup = parent.join(format!(".{name}.backup-{id}"));
    fs::create_dir(&stage)?;
    if let Err(error) = build(root, &stage, wasm) {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    if output.exists() {
        fs::rename(output, &backup)?;
    }
    if let Err(error) = fs::rename(&stage, output) {
        if backup.exists() {
            fs::rename(&backup, output).context("Restore the previous output")?;
        }
        let _ = fs::remove_dir_all(&stage);
        return Err(error.into());
    }
    if backup.exists() {
        fs::remove_dir_all(backup)?;
    }
    Ok(())
}

fn file_for(output: &Path, url: &str) -> Option<PathBuf> {
    let route = url.split('?').next()?.trim_start_matches('/');
    if route
        .split('/')
        .any(|s| s.starts_with('.') || s.contains(['%', '\\']))
    {
        return None;
    }
    let path = output.join(route);
    let path = if path.is_dir() {
        path.join("index.html")
    } else {
        path
    };
    path.is_file().then_some(path)
}

fn media(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("xml") => "application/xml; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("wasm") => "application/wasm",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("webmanifest") => "application/manifest+json",
        _ => "application/octet-stream",
    }
}

fn serve(output: &Path, port: u16) -> Result<()> {
    let server = Server::http(("127.0.0.1", port)).map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("Serving Rust site at http://localhost:{port}/");
    for request in server.incoming_requests() {
        if !matches!(request.method(), Method::Get | Method::Head) {
            let _ = request.respond(Response::empty(StatusCode(405)));
            continue;
        }
        let path = file_for(output, request.url());
        let Some(path) = path else {
            let _ = request.respond(Response::from_string("Not found").with_status_code(404));
            continue;
        };
        if let Ok(file) = fs::File::open(&path) {
            let header =
                Header::from_bytes("Content-Type", media(&path)).expect("valid content type");
            let _ = request.respond(Response::from_file(file).with_header(header));
        } else {
            let _ = request
                .respond(Response::from_string("Unable to open resource").with_status_code(500));
        }
    }
    Ok(())
}

fn build_paintings(target: &Path, output: &Path) -> Result<()> {
    Bindgen::new()
        .web(true)?
        .input_path(target.join("wasm32-unknown-unknown/release/daipan_lab.wasm"))
        .out_name("paintings")
        .typescript(false)
        .generate(output)
        .context("Compile painting WASM first")
}

fn serve_lab(root: &Path, port: u16) -> Result<()> {
    let target = root.join("prototype/target");
    let output = target.join("manifold-lab");
    fs::create_dir_all(&output)?;
    fs::write(
        output.join("index.html"),
        include_str!("../../lab/index.html"),
    )?;
    build_paintings(&target, &output)?;
    serve(&output, port)
}

fn main() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()?;
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "build".into());
    if command == "lab" {
        return serve_lab(&root, args.next().as_deref().unwrap_or("4376").parse()?);
    }
    ensure!(
        matches!(command.as_str(), "build" | "serve"),
        "usage: daipan-site build|serve OUTPUT [PORT], or daipan-site lab [PORT]"
    );
    let output = PathBuf::from(args.next().unwrap_or_else(|| "dist".into()));
    if output.exists() && !output.is_dir() {
        bail!("Output is not a directory");
    }
    let wasm = root.join("prototype/target/wasm32-unknown-unknown/release/daipan_browser.wasm");
    replace_build(&root, &output, &wasm)?;
    if command == "serve" {
        serve(&output, args.next().as_deref().unwrap_or("4374").parse()?)?;
    }
    Ok(())
}
