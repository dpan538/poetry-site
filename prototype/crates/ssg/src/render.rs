use crate::corpus::{DOMAIN, WORLDS, Work, counts, section};
use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fmt::Write, fs, path::Path};

const POEM_SKIN: &str = "data-skin-cmukzv4u";
const MOBILE_SKIN: &str = "data-skin-iwd45nhr";
const WORLD_SKIN: &str = "data-skin-zkyqgzqi";
const DOCK_SKIN: &str = "data-skin-htlfczsh";
const SITE_INDEX_SKIN: &str = "data-skin-ewxlwce5";

pub fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn safe_json(value: &Value) -> String {
    value
        .to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
}

fn template(input: &str, slots: &BTreeMap<String, String>) -> Result<String> {
    let mut output = String::new();
    let mut rest = input;
    while let Some((before, after)) = rest.split_once("{{") {
        let (key, suffix) = after.split_once("}}").context("Unclosed template marker")?;
        output.push_str(before);
        output.push_str(
            slots
                .get(key)
                .with_context(|| format!("Missing template slot {key}"))?,
        );
        rest = suffix;
    }
    output.push_str(rest);
    Ok(output)
}

fn list_label((poems, prose): (usize, usize)) -> String {
    if prose > 0 {
        format!("{poems} poems · {prose} prose pieces")
    } else {
        format!("{poems} poems")
    }
}

fn set_counts(html: &str, works: &[Work]) -> String {
    let (p, s) = counts(works.iter());
    let (still_p, still_s) = counts(works.iter().filter(|w| w.meta.world == "still-life"));
    let full = if s == 0 {
        format!("{p} poems")
    } else {
        format!("{p} poems and {s} prose pieces")
    };
    let zh = if s == 0 {
        format!("{p} 首诗")
    } else {
        format!("{p} 首诗及 {s} 篇散文")
    };
    html.replace("79 poems", &full)
        .replace("79 首诗", &zh)
        .replace("27 poems", &list_label((still_p, still_s)))
        .replace(
            "Still Life (27)",
            &format!("Still Life ({})", still_p + still_s),
        )
}

fn first_line(work: &Work, class: &str, skin: &str) -> String {
    let line = work.opening();
    if line.is_empty() {
        return String::new();
    }
    format!("<span class=\"{class}\" {skin}>{}</span>", escape(line))
}

fn menus(works: &[Work], current: Option<&Work>) -> BTreeMap<String, String> {
    let mut slots = BTreeMap::new();
    let skin = if current.is_some() {
        DOCK_SKIN
    } else {
        SITE_INDEX_SKIN
    };
    for (i, (world, ..)) in WORLDS.iter().enumerate() {
        let mut mobile = String::new();
        let mut desktop = String::new();
        for (idx, w) in works.iter().filter(|w| w.meta.world == *world).enumerate() {
            let path = w.path();
            let title = escape(&w.meta.title);
            let order = format!("{:02}", w.meta.order);
            let active = current.is_some_and(|v| v.id == w.id && v.meta.world == w.meta.world);
            let class = if active { " is-current" } else { "" };
            let selected = if active {
                r#" aria-current="page""#
            } else {
                ""
            };
            let _ = write!(
                mobile,
                r#"<li class="mobile-index-item{class}" {skin}><a class="mobile-index-link" {skin} href="{path}"{selected}><span class="mobile-index-order" {skin}>{order}</span> <span class="mobile-index-copy" {skin}><span class="mobile-index-title" {skin}>{title}</span> {}</span></a></li>"#,
                first_line(w, "mobile-index-line", skin)
            );
            let _ = write!(
                desktop,
                r#"<li class="poem-item{class}" style="--item-i: {idx}"><a class="poem-link" href="{path}"{selected}><span class="poem-order">{order}</span> <span class="poem-title">{title}</span> {}</a></li>"#,
                first_line(w, "poem-first-line", "")
            );
        }
        slots.insert(format!("MOBILE_{i}"), mobile);
        slots.insert(format!("DESKTOP_{i}"), desktop);
    }
    slots
}

fn world_rows(works: &[Work], world: &str) -> String {
    let mut output = String::new();
    for w in works.iter().filter(|w| w.meta.world == world) {
        let title = escape(&w.meta.title);
        let _ = writeln!(
            output,
            r#"<li class="poem-index-item" {WORLD_SKIN}><a class="poem-index-link" {WORLD_SKIN} href="{}"><span class="poem-index-num" {WORLD_SKIN}>{:02}</span> <span class="poem-index-text" {WORLD_SKIN}><span class="poem-index-title" {WORLD_SKIN}>{title}</span> {}</span></a></li>"#,
            w.path(),
            w.meta.order,
            first_line(w, "poem-index-first-line", WORLD_SKIN)
        );
    }
    output
}

fn prose_emphasis(line: &str) -> Option<&str> {
    line.strip_prefix('*')?.strip_suffix('*')
}

fn public_block(work: &Work, block: &[String], scope: &str, class: &str) -> String {
    if !work.poem() && block.len() == 1 && section(&block[0]) {
        return format!(
            "<h2 class=\"prose-section\" {scope}>{}</h2>",
            escape(&block[0])
        );
    }
    let br = format!("<br {scope}>");
    let lines = block
        .iter()
        .map(|line| {
            if !work.poem()
                && let Some(text) = prose_emphasis(line)
            {
                format!("<em>{}</em>", escape(text))
            } else {
                escape(line)
            }
        })
        .collect::<Vec<_>>()
        .join(&br);
    format!("<p class=\"{class}\" {scope}>{lines}</p>")
}

fn mobile_text(work: &Work) -> String {
    let mut body = String::new();
    for block in &work.blocks {
        if !work.poem() {
            body.push_str(&public_block(
                work,
                block,
                MOBILE_SKIN,
                "mobile-poem-stanza prose-paragraph",
            ));
            continue;
        }
        let _ = write!(
            body,
            "<p class=\"mobile-poem-stanza mobile-poem-stanza--lineated\" {MOBILE_SKIN}>"
        );
        for line in block {
            let _ = write!(
                body,
                "<span class=\"mobile-poem-line\" {MOBILE_SKIN} data-line>{}</span>",
                escape(line)
            );
        }
        body.push_str("</p>");
    }
    body
}

fn desktop_text(work: &Work) -> String {
    let mut body =
        String::from("<div class=\"body-frame\" id=\"body-frame-0\" aria-hidden=\"true\">");
    for block in &work.blocks {
        if !work.poem() {
            body.push_str(&public_block(
                work,
                block,
                "",
                "poem-stanza prose-paragraph",
            ));
            continue;
        }
        body.push_str("<div class=\"poem-stanza\">");
        for line in block {
            let variant = if section(line.trim()) {
                " poem-line--section"
            } else if line.trim().starts_with('(') && line.trim().ends_with(')') {
                " poem-line--stage"
            } else {
                ""
            };
            let _ = write!(
                body,
                "<div class=\"poem-line{variant}\">{}</div>",
                escape(line)
            );
        }
        body.push_str("</div>");
    }
    body.push_str("</div>");
    body
}

#[derive(Deserialize)]
pub struct Scene {
    piece: String,
    description: String,
    excerpt: String,
    keywords: [String; 3],
}

#[derive(Deserialize)]
struct CatalogEntry {
    piece: String,
    painting: Option<PaintingCredit>,
}

#[derive(Deserialize)]
struct PaintingCredit {
    credit: String,
    source: Option<String>,
}

fn manifold(scene: &Scene, title: &str, painting: Option<&PaintingCredit>) -> String {
    let piece = escape(&scene.piece);
    let (desc, marker) = match painting {
        Some(painting) => (
            format!(
                "{} — {}",
                escape(&scene.description),
                escape(&painting.credit)
            ),
            if painting.source.is_none() {
                " data-painting data-photo"
            } else {
                " data-painting"
            },
        ),
        None => (escape(&scene.description), ""),
    };
    let excerpt = escape(&scene.excerpt);
    let keywords = scene
        .keywords
        .iter()
        .map(|word| escape(word))
        .collect::<Vec<_>>()
        .join(" <span aria-hidden=\"true\">·</span> ");
    format!(
        r#"<dialog id="manifold" class="manifold" aria-label="Manifold — {title}" data-piece="{piece}"{marker}>
  <form class="manifold-exit" method="dialog"><button class="manifold-close" type="submit" autofocus aria-label="Close Manifold">×</button></form>
  <figure class="manifold-center">
    <div class="manifold-plate">
      <canvas role="img" aria-label="{desc}"></canvas>
      <button class="manifold-playback" data-manifold-playback type="button" aria-label="Pause animation" hidden></button>
      <button class="manifold-retry" data-manifold-retry type="button">Retry animation</button>
    </div>
    <figcaption><p class="manifold-keywords">{keywords}</p><blockquote>“{excerpt}”</blockquote></figcaption>
  </figure>
</dialog>"#
    )
}

const LOADER: &str = r#"<link rel="stylesheet" href="/assets/site.css"><script type="module" src="/assets/start.js"></script>"#;
const WORK_LOADER: &str = r#"<link rel="stylesheet" href="/assets/site.css"><script type="module" blocking="render" src="/assets/start.js"></script>"#;

fn render_work(
    root: &Path,
    works: &[Work],
    work: &Work,
    scenes: &BTreeMap<String, Scene>,
    painting: Option<&PaintingCredit>,
    nav: &[Value],
    shell: &str,
) -> Result<String> {
    let (_, volume, _, _) = WORLDS
        .iter()
        .find(|(w, ..)| *w == work.meta.world)
        .context("Unknown volume")?;
    let world = &work.meta.world;
    let path = work.path();
    let scene = &scenes[&path[1..]];
    let url = format!("{DOMAIN}{path}");
    let title = escape(&work.meta.title);
    let kind = if work.poem() { "poem" } else { "prose" };
    let order = format!("{:02}", work.meta.order);
    let opening: String = work.opening().chars().take(150).collect();
    let type_label = if work.poem() { "Poem" } else { "Prose" };
    let year = work
        .citation_year()
        .map(|year| format!(", {year}"))
        .unwrap_or_default();
    let citation = format!(
        "Dai Pan, “{title},” Three Worlds, {volume}, {} {order}{year}. {url}",
        if work.poem() { "poem" } else { "text" }
    );
    let mut seo = format!(
        "<article class=\"poem-seo-text\" {POEM_SKIN} aria-label=\"{title} — {kind} in {volume}\"><h1 {POEM_SKIN}>{title}</h1><p class=\"poem-seo-meta\" {POEM_SKIN}>From <em {POEM_SKIN}>{volume}</em>, <em {POEM_SKIN}>Three Worlds</em> by Dai Pan (潘岱). {type_label} {order}.</p>"
    );
    for block in &work.blocks {
        seo.push_str(&public_block(work, block, POEM_SKIN, "poem-seo-stanza"));
    }
    let _ = write!(
        seo,
        "<p class=\"poem-seo-citation\" {POEM_SKIN}>Cite as: {citation}</p></article>"
    );
    let mut schema: Value =
        serde_json::from_slice(&fs::read(root.join("site/data/work-schema.json"))?)?;
    schema["@type"] = json!(if work.poem() { "Poem" } else { "CreativeWork" });
    for key in ["@id", "url", "mainEntityOfPage"] {
        schema[key] = json!(url);
    }
    schema["name"] = json!(work.meta.title);
    schema["isPartOf"] = json!({"@id":format!("{DOMAIN}/{world}#volume")});
    schema["position"] = json!(work.meta.order);
    schema["abstract"] = json!(opening);
    if !work.poem() {
        schema["genre"] = json!("Literary prose");
    }
    if let Some(year) = work.meta.written_year {
        schema["dateCreated"] = json!(year.to_string());
    }
    let crumbs = json!({"@context":"https://schema.org","@type":"BreadcrumbList","itemListElement":[
        {"@type":"ListItem","position":1,"name":"Three Worlds","item":format!("{DOMAIN}/")},
        {"@type":"ListItem","position":2,"name":volume,"item":format!("{DOMAIN}/{world}")},
        {"@type":"ListItem","position":3,"name":work.meta.title,"item":url}
    ]});
    let current = nav
        .iter()
        .find(|row| row["world"].as_str() == Some(world) && row["id"].as_str() == Some(&work.id));
    let candidates: Vec<_> = nav.iter().map(|row| {
        let relation = |source: &Value, target: &Value| source["atlasRelations"].as_array().and_then(|r|
            r.iter().find(|p| p["id"]==target["id"])).and_then(|r| r["weight"].as_f64()).unwrap_or(0.0);
        let weight = current.map(|cur| relation(cur,row).max(0.82*relation(row,cur))).unwrap_or(0.0);
        let key = format!("{}/{}", row["world"].as_str().unwrap_or_default(), row["id"].as_str().unwrap_or_default());
        json!({"id":row["id"],"world":row["world"],"score":row["emotionScore"],"types":row["contentTypes"],"affinity":weight,"scene":scenes.get(&key).map(|scene| scene.piece.as_str())})
    }).collect();
    let data = format!(
        r#"<script type="application/json" id="reading-data">{}</script>"#,
        safe_json(&json!({"current":work.id,"world":world,"works":candidates}))
    );
    let column_alignment = if let Some(alignment) = &work.meta.column_alignment {
        let right = work
            .blocks
            .iter()
            .position(|block| block.first() == Some(&alignment.right_stanza))
            .with_context(|| format!("Unknown right column stanza for {path}"))?;
        let (block, line) = work.blocks[..right]
            .iter()
            .enumerate()
            .find_map(|(block, lines)| {
                lines
                    .iter()
                    .position(|line| line == &alignment.left_line)
                    .map(|line| (block, line))
            })
            .with_context(|| format!("Unknown left column line for {path}"))?;
        ensure!(work.poem(), "Column alignment requires verse in {path}");
        format!(r#"data-column-alignment="[{block},{line},{right}]""#)
    } else {
        String::new()
    };
    let mut slots = menus(works, Some(work));
    for (key,value) in [
        ("LOADER",format!("{WORK_LOADER}{data}")),("WORLD",world.to_owned()),
        ("PAGE_TITLE",format!("{title} — Dai Pan")),
        ("CANONICAL",url),
        ("DESCRIPTION",format!("{title} — {} by Dai Pan (潘岱), from {volume}, Three Worlds.",if work.poem() {"a poem"} else {"a prose piece"})),
        ("KEYWORDS",format!("Dai Pan, Pan Dai, 潘岱, Three Worlds, {volume}, {title}, {kind}")),
        ("SOCIAL_DESC",format!("From {volume}, Three Worlds by Dai Pan (潘岱). {}",escape(&opening))),
        ("IMAGE_ALT",format!("{title} — {kind} by Dai Pan (潘岱)")),
        ("WORK_SCHEMA",safe_json(&schema)),("BREADCRUMBS",safe_json(&crumbs)),("SEO_TEXT",seo),
        ("TITLE",title.clone()),("VOLUME",(*volume).to_string()),("ORDER",order),
        ("KIND",kind.to_string()),("WORK_ID",work.id.clone()),("WORK_META",format!("{} — {}",world.replace('-'," "),work.meta.order)),
        ("COLUMN_ALIGNMENT",column_alignment),("DESKTOP_TEXT",desktop_text(work)),("MOBILE_TEXT",mobile_text(work)),
        ("MANIFOLD_TRIGGER",r#"<button class="manifold-trigger" type="button" data-manifold-open aria-label="Open Manifold" aria-haspopup="dialog" aria-controls="manifold">Manifold</button>"#.to_owned()),
        ("MANIFOLD",manifold(scene,&title,painting)),
    ] { slots.insert(key.to_owned(),value); }
    template(&set_counts(shell, works), &slots)
}

pub fn generate(root: &Path, output: &Path, works: &[Work], nav: &[Value]) -> Result<()> {
    let templates = root.join("site/templates");
    let collection_record = set_counts(
        &fs::read_to_string(templates.join("collection-record.html"))?,
        works,
    );
    for route in [
        "home",
        "collection",
        "about",
        "contact",
        "preface",
        "their-world",
        "bless-you",
        "still-life",
    ] {
        let input = fs::read_to_string(templates.join(format!("{route}.html")))?;
        let mut slots = menus(works, None);
        let mut loader = LOADER.to_string();
        if route == "home" {
            let entries: Vec<_> = works
                .iter()
                .map(|w| json!({"id":w.id,"world":w.meta.world}))
                .collect();
            let _ = write!(
                loader,
                r#"<script type="application/json" id="entry-data">{}</script>"#,
                safe_json(&json!(entries))
            );
        }
        slots.insert("LOADER".into(), loader);
        if matches!(route, "collection" | "about") {
            let (title, subheading) = if route == "collection" {
                ("h1", "h2")
            } else {
                ("h2", "h3")
            };
            slots.insert(
                "COLLECTION_RECORD".into(),
                template(
                    &collection_record,
                    &BTreeMap::from([
                        ("RECORD_TITLE".into(), title.into()),
                        ("RECORD_SUBHEADING".into(), subheading.into()),
                    ]),
                )?,
            );
        }
        if WORLDS.iter().any(|(world, ..)| *world == route) {
            slots.insert("WORLD_ROWS".into(), world_rows(works, route));
            slots.insert(
                "VOLUME_COUNT".into(),
                list_label(counts(works.iter().filter(|w| w.meta.world == route))),
            );
        }
        let page = template(&set_counts(&input, works), &slots)?;
        write_page(output, if route == "home" { "" } else { route }, &page)?;
    }
    let shell = fs::read_to_string(templates.join("work.html"))?;
    let scenes: BTreeMap<String, Scene> =
        serde_json::from_slice(&fs::read(root.join("site/data/manifold.json"))?)?;
    let catalog: Vec<CatalogEntry> =
        serde_json::from_slice(&fs::read(root.join("site/data/manifold-catalog.json"))?)?;
    let catalog: BTreeMap<_, _> = catalog
        .into_iter()
        .map(|entry| (entry.piece, entry.painting))
        .collect();
    for scene in scenes.values() {
        let slug = &scene.piece;
        ensure!(
            !slug.is_empty() && slug.bytes().all(|b| b.is_ascii_lowercase() || b == b'-'),
            "Invalid MANIFOLD scene slug"
        );
        let painting = catalog
            .get(slug)
            .with_context(|| format!("Unknown MANIFOLD scene {slug}"))?;
        if painting.is_none() {
            ensure!(
                root.join(format!("site/assets/manifold/pieces/{slug}.js"))
                    .is_file(),
                "Missing ascii.rest scene {slug}"
            );
        }
    }
    for work in works {
        let path = work.path();
        let scene = scenes.get(&path[1..]).with_context(|| {
            format!("Missing MANIFOLD scene for {path}. Rebuild editorial assets.")
        })?;
        if let Some(piece) = &work.meta.manifold_scene {
            ensure!(
                piece == &scene.piece,
                "MANIFOLD scene for {path} must match {piece}. Rebuild editorial assets."
            );
        }
        ensure!(
            work.blocks.iter().any(|block| {
                block
                    .iter()
                    .map(|line| {
                        if work.poem() {
                            line.as_str()
                        } else {
                            prose_emphasis(line).unwrap_or(line)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
                    .contains(&scene.excerpt)
            }),
            "Scene excerpt is out of date for {path}. Rebuild editorial assets."
        );
        let page = render_work(
            root,
            works,
            work,
            &scenes,
            catalog[&scene.piece].as_ref(),
            nav,
            &shell,
        )?;
        write_page(output, &path[1..], &page)?;
    }
    Ok(())
}

fn write_page(output: &Path, path: &str, html: &str) -> Result<()> {
    let directory = output.join(path);
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("index.html"), html)?;
    Ok(())
}

pub fn indexes(root: &Path, output: &Path, works: &[Work], nav: &[Value]) -> Result<()> {
    let mut sitemap = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    let mut add = |path: &str| {
        let _ = writeln!(sitemap, "  <url><loc>{DOMAIN}{path}</loc></url>");
    };
    for path in ["/", "/about", "/contact", "/collection", "/preface"] {
        add(path);
    }
    for (world, ..) in WORLDS {
        add(&format!("/{world}"));
        for work in works.iter().filter(|w| w.meta.world == world) {
            add(&work.path());
        }
    }
    sitemap.push_str("</urlset>\n");
    fs::write(output.join("sitemap.xml"), &sitemap)?;
    fs::write(output.join("sitemap-0.xml"), &sitemap)?;
    fs::write(
        output.join("sitemap-index.xml"),
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<sitemapindex xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"><sitemap><loc>{DOMAIN}/sitemap-0.xml</loc></sitemap></sitemapindex>\n"
        ),
    )?;
    let original = fs::read_to_string(root.join("site/data/llms-base.txt"))?;
    let Some((front, rest)) = original.split_once("## Poem Index") else {
        bail!("Missing index marker in llms source");
    };
    let tail = rest
        .split_once("## Key URLs")
        .context("Missing URLs marker in llms source")?
        .1;
    let mut llms = set_counts(front, works);
    llms.push_str("## Works Index\n");
    for (world, label, roman, clock) in WORLDS {
        let list: Vec<_> = works.iter().filter(|w| w.meta.world == world).collect();
        let _ = write!(
            llms,
            "\n### Volume {roman} — {label} ({world})\n{} · Layout clock {clock}\n\n",
            list_label(counts(list.iter().copied()))
        );
        for w in list {
            let _ = writeln!(
                llms,
                "- {:02}. {}{} {DOMAIN}{} — {}",
                w.meta.order,
                w.meta.title,
                w.citation_year()
                    .map(|year| format!(" ({year})"))
                    .unwrap_or_default(),
                w.path(),
                w.opening()
            );
        }
    }
    llms.push_str("\n---\n\n## Key URLs");
    llms.push_str(tail);
    fs::write(output.join("llms.txt"), llms)?;
    fs::write(output.join("poem-data.json"), serde_json::to_vec(nav)?)?;
    Ok(())
}
