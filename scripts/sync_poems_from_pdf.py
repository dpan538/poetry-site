#!/usr/bin/env python3
"""Extract poem bodies from Poetry collection draft (2).pdf and update src/content/**/*.md."""

from __future__ import annotations

import re
import sys
from pathlib import Path

try:
    from pypdf import PdfReader
except ImportError:
    print("Install pypdf: pip3 install pypdf", file=sys.stderr)
    sys.exit(1)

ROOT = Path(__file__).resolve().parents[1]
PDF = ROOT / "Poetry collection draft (2).pdf"
CONTENT = ROOT / "src" / "content"


def load_pdf_text() -> str:
    r = PdfReader(str(PDF))
    return "\n".join(p.extract_text() or "" for p in r.pages)


def title_pattern(title: str) -> str:
    """Allow PDF line breaks between words (e.g. Going\\n \\nHome)."""
    parts = [p for p in title.split() if p]
    gap = r"(?:\s|\n)+"
    return gap.join(re.escape(p) for p in parts)


def parse_frontmatter(raw: str) -> tuple[dict[str, str], str]:
    if not raw.startswith("---"):
        return {}, raw
    parts = raw.split("---", 2)
    if len(parts) < 3:
        return {}, raw
    fm: dict[str, str] = {}
    for line in parts[1].splitlines():
        if ":" not in line:
            continue
        k, v = line.split(":", 1)
        fm[k.strip()] = v.strip().strip('"')
    return fm, parts[2].lstrip("\n")


def collect_poems() -> list[tuple[str, Path, int, str, str]]:
    """Return rows: world, path, order, title, slug — PDF order = world then order index."""
    rows: list[tuple[str, Path, int, str, str]] = []
    order_world = {"their-world": 0, "bless-you": 1, "still-life": 2}
    for world in ("their-world", "bless-you", "still-life"):
        paths = list((CONTENT / world).glob("*.md"))
        decorated: list[tuple[int, Path, dict[str, str]]] = []
        for path in paths:
            fm, _ = parse_frontmatter(path.read_text(encoding="utf-8"))
            if "order" not in fm or "title" not in fm:
                continue
            decorated.append((int(fm["order"]), path, fm))
        for ord_, path, fm in sorted(decorated, key=lambda x: x[0]):
            rows.append((world, path, ord_, fm["title"], path.stem))
    rows.sort(key=lambda x: (order_world[x[0]], x[2]))
    return rows


def title_anchor_regex(slug: str, title: str) -> re.Pattern[str]:
    """Match poem title line in PDF (optional blank lines before title)."""
    specials = {
        "side": r"(?:^|\n)\s*Side(?:\s+\([^)]+\))\s*\n",
        "join-us": r"(?:^|\n)\s*Join\s*\n?\s*us,?\s*\n",
        "silver-tipped-shoes": r"(?:^|\n)\s*Silver-Tipped\s*\n?\s*Shoes\s*\n",
        "urban-monologue": r"(?:^|\n)\s*Urban\s+Monologue\s+",
        "the-age-of-secrets": r"(?:^|\n)\s*The\s*\n?\s*Age\s*\n?\s*of\s*\n?\s*Secrets\s*\n",
        "room": r"(?:^|\n)\s*Room\s+\(1\)\s*\n",
        "in-the-sunlight": r"(?:^|\n)\s*In\s*\n?\s*The\s*\n?\s*Sunlight\s*\n",
        "traces-of-healing": r"(?:^|\n)\s*Traces\s*\n?\s*of\s*\n?\s*Healing\s*\n",
        "sleeping-in": r"(?:^|\n)\s*Sleeping\s*\n?\s*in\s*\n",
        "kite": r"(?:^|\n)\s*Kite\s*\n(?=\s*A\s)",
        # "River" appears inside other poems (e.g. "Allegheny River"); anchor first line of that poem.
        "ears": r"(?:^|\n)\s*Ears\s*\n(?=\s*Skin\s)",
        "river": r"(?:^|\n)\s*River\s*\n(?=\s*I\s)",
        "gentleness": r"(?:^|\n)\s*Gentleness\s*\n(?=\s*A\s)",
        "telecine": r"(?:^|\n)\s*Telecine\s*\n(?=\s*The\s)",
        "beloved": r"(?:^|\n)\s*Beloved\s*\n(?=\s*Within\s)",
        "tourist": r"(?:^|\n)\s*Tourist\s*\n(?=\s*In\s*t)",
    }
    flags = re.IGNORECASE | re.MULTILINE
    if slug in specials:
        return re.compile(specials[slug], flags)
    pat = title_pattern(title)
    return re.compile(r"(?:^|\n)\s*(" + pat + r")\s*\n", flags)


def extract_body(raw_slice: str, title: str) -> str:
    lines = raw_slice.splitlines()
    # drop leading empty
    while lines and not lines[0].strip():
        lines.pop(0)
    if lines and re.fullmatch(title_pattern(title), lines[0].strip(), flags=re.I):
        lines = lines[1:]
    raw = "\n".join(lines)
    raw = re.sub(r"\s*\n\s*", " ", raw)
    raw = re.sub(r" +", " ", raw).strip()
    if not raw:
        return ""
    parts = re.split(r"(?<=[.!?])\s+", raw)
    stanzas: list[str] = []
    buf: list[str] = []
    for p in parts:
        p = p.strip()
        if not p:
            continue
        buf.append(p)
        if len(buf) >= 3:
            stanzas.append(" ".join(buf))
            buf = []
    if buf:
        stanzas.append(" ".join(buf))
    return "\n\n".join(stanzas) if stanzas else raw


def yaml_escape(s: str) -> str:
    return s.replace("\\", "\\\\").replace('"', '\\"')


def find_next_match(rx: re.Pattern[str], text: str, cursor: int) -> re.Match[str] | None:
    """First regex match at or after cursor (PDF order can differ from poem book order)."""
    for m in rx.finditer(text):
        if m.start() >= cursor:
            return m
    return None


def first_line_from_body(body: str, title: str) -> str:
    flat = body.replace("\n\n", " ").replace("\n", " ").strip()
    if not flat:
        return title
    sent = re.split(r"(?<=[.!?])\s+", flat)[0]
    if len(sent) > 200:
        sent = sent[:197].rsplit(" ", 1)[0] + "…"
    return sent


def main() -> None:
    if not PDF.exists():
        print(f"Missing PDF: {PDF}", file=sys.stderr)
        sys.exit(1)
    text = load_pdf_text()
    poems = collect_poems()
    matches: list[dict[str, object]] = []
    cursor = 0
    for world, path, order, title, slug in poems:
        rx = title_anchor_regex(slug, title)
        m = find_next_match(rx, text, cursor)
        if not m:
            print(f"WARN: not found: {title!r} ({path.name})", file=sys.stderr)
            continue
        title_start = m.start()
        body_start = m.end()
        matches.append(
            {
                "world": world,
                "path": path,
                "title": title,
                "order": order,
                "title_start": title_start,
                "body_start": body_start,
            }
        )
        cursor = body_start

    updated = 0
    for i, m in enumerate(matches):
        end = matches[i + 1]["title_start"] if i + 1 < len(matches) else len(text)
        slice_ = text[int(m["body_start"]) : int(end)]
        body = extract_body(slice_, str(m["title"]))
        if not body:
            print(f"WARN: empty body: {m['path']}", file=sys.stderr)
            continue
        path: Path = m["path"]  # type: ignore[assignment]
        fm, _ = parse_frontmatter(path.read_text(encoding="utf-8"))
        first = first_line_from_body(body, str(m["title"]))
        lines = ["---"]
        lines.append(f'title: "{yaml_escape(fm.get("title", str(m["title"])))}"')
        lines.append(f'world: "{yaml_escape(fm.get("world", str(m["world"])))}"')
        lines.append(f'order: {fm.get("order", m["order"])}')
        lines.append(f'firstLine: "{yaml_escape(first)}"')
        lines.append("---\n")
        lines.append(body + "\n")
        path.write_text("\n".join(lines), encoding="utf-8")
        updated += 1
        print("OK", path.relative_to(ROOT))

    print(f"Updated {updated} / {len(poems)} poems")


if __name__ == "__main__":
    main()
