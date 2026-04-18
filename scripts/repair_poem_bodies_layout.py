#!/usr/bin/env python3
"""Replace poem markdown bodies with PDF layout text (pypdf extraction_mode='layout').

Preserves frontmatter YAML exactly. Does not alter firstLine or other keys.
"""

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
WORLDS = ("their-world", "bless-you", "still-life")


def norm_title(s: str) -> str:
    s = s.strip().lower().replace("-", " ")
    return re.sub(r"\s+", " ", s)


def load_pdf_lines() -> list[str]:
    r = PdfReader(str(PDF))
    lines: list[str] = []
    for page in r.pages:
        txt = page.extract_text(extraction_mode="layout") or ""
        lines.extend(txt.splitlines())
    return lines


def collect_titles() -> tuple[list[str], dict[str, str]]:
    """Return (ordered unique display titles, norm_key -> display title)."""
    titles: list[str] = []
    seen: set[str] = set()
    key_to_title: dict[str, str] = {}
    for world in WORLDS:
        for path in sorted((CONTENT / world).glob("*.md")):
            raw = path.read_text(encoding="utf-8")
            m = re.search(r'^title:\s*"([^"]+)"', raw, re.M)
            if not m:
                continue
            t = m.group(1)
            k = norm_title(t)
            if k not in seen:
                seen.add(k)
                titles.append(t)
            key_to_title[k] = t
    return titles, key_to_title


def title_line_indices(lines: list[str], key_to_title: dict[str, str]) -> dict[str, int]:
    """Map normalized title key -> PDF line index (first match)."""
    keys = set(key_to_title.keys())
    out: dict[str, int] = {}
    for i, ln in enumerate(lines):
        k = norm_title(ln)
        if k in keys and k not in out:
            out[k] = i
    return out


def extract_body(lines: list[str], display_title: str, idx: dict[str, int], all_keys: set[str]) -> str:
    k = norm_title(display_title)
    if k not in idx:
        raise KeyError(f"Title not found in PDF: {display_title!r} (key {k!r})")
    a = idx[k] + 1
    while a < len(lines) and not lines[a].strip():
        a += 1
    b = a
    while b < len(lines):
        s = lines[b].strip()
        if s and norm_title(s) in all_keys and norm_title(s) != k:
            break
        b += 1
    chunk = lines[a:b]
    while chunk and not chunk[-1].strip():
        chunk.pop()
    return "\n".join(chunk)


def split_frontmatter(raw: str) -> tuple[str, str]:
    if not raw.startswith("---"):
        return "", raw
    parts = raw.split("---", 2)
    if len(parts) < 3:
        return "", raw
    fm = parts[1]
    body = parts[2].lstrip("\n")
    return fm, body


def main() -> None:
    if not PDF.exists():
        print(f"Missing PDF: {PDF}", file=sys.stderr)
        sys.exit(1)

    lines = load_pdf_lines()
    titles, key_to_title = collect_titles()
    all_keys = {norm_title(t) for t in titles}
    idx = title_line_indices(lines, key_to_title)

    missing = [t for t in titles if norm_title(t) not in idx]
    if missing:
        print("ERROR: titles not found in PDF:", file=sys.stderr)
        for t in missing:
            print(f"  {t!r}", file=sys.stderr)
        sys.exit(1)

    updated = 0
    for world in WORLDS:
        for path in sorted((CONTENT / world).glob("*.md")):
            raw = path.read_text(encoding="utf-8")
            m = re.search(r'^title:\s*"([^"]+)"', raw, re.M)
            if not m:
                continue
            title = m.group(1)
            fm, _old_body = split_frontmatter(raw)
            if not fm:
                print(f"WARN: no frontmatter {path}", file=sys.stderr)
                continue
            body = extract_body(lines, title, idx, all_keys)
            new_raw = f"---{fm}---\n{body}\n"
            path.write_text(new_raw, encoding="utf-8")
            updated += 1
            print("OK", path.relative_to(ROOT))

    print(f"Updated {updated} files")


if __name__ == "__main__":
    main()
