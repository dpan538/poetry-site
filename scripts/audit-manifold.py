#!/usr/bin/env python3
"""Audit catalogue registration, manuscript pins and generated MANIFOLD dialogs."""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]


def audit(build_dir=None, expected_originals=None):
    catalogue = json.loads((ROOT / 'site/data/manifold-catalog.json').read_text())
    manifest = json.loads((ROOT / 'site/data/manifold.json').read_text())
    counts = Counter(row['piece'] for row in catalogue)
    assert all(n == 1 for n in counts.values()), 'Duplicate catalogue identifiers'
    assert all(re.fullmatch('[a-z-]+', slug) for slug in counts), 'Invalid scene identifier'
    originals = {row['piece']: row['original'].get('renderer', 'originals')
                 for row in catalogue if 'original' in row}
    if expected_originals is not None:
        assert len(originals) == expected_originals, 'Unexpected number of independent originals'
    for renderer, crate in [('originals', 'originals'), ('atmospheres', 'studies')]:
        code = (ROOT / f'prototype/crates/{crate}/src/lib.rs').read_text()
        registry = code.split('fn published_kind(', 1)[1].split('#[wasm_bindgen]', 1)[0]
        exposed = set(re.findall(r'"([a-z-]+)"\s*=>\s*Some\(', registry))
        registered = {slug for slug, player in originals.items() if player == renderer}
        assert exposed == registered, f'{renderer}: catalogue and Rust mount registry disagree'
    assert set(originals.values()) <= {'originals', 'atmospheres'}, 'Unknown Rust renderer'
    volumes = defaultdict(list)
    routes = set()
    for path in (ROOT / 'src/content').glob('*/*.md'):
        head = path.read_text().split('---', 2)[1]
        route = f'{path.parent.name}/{path.stem}'
        routes.add(route)
        entry = manifest[route]
        assert entry['piece'] in counts, f'{route}: unknown scene'
        pin = re.search(r'^manifoldScene:\s*"([a-z-]+)"', head, re.M)
        if pin:
            assert entry['piece'] == pin[1], f'{route}: manuscript pin not applied'
        order = int(re.search(r'^order:\s*(\d+)', head, re.M)[1])
        volumes[path.parent.name].append((order, route))
        if build_dir:
            html = (build_dir / route / 'index.html').read_text()
            assert f'data-piece="{entry["piece"]}"' in html, f'{route}: stale generated dialog'
            if entry['piece'] in originals:
                renderer = originals[entry['piece']]
                marker = 'data-original' if renderer == 'originals' else 'data-atmosphere'
                assert marker in html, f'{route}: incorrect runtime marker'
    assert routes == set(manifest), 'Manuscript routes and MANIFOLD manifest disagree'
    use = Counter(row['piece'] for row in manifest.values())
    assert all(use[slug] == 1 for slug in originals), 'An independent original is unused or shared'
    adjacent = []
    for rows in volumes.values():
        rows.sort()
        for (_, a), (_, b) in zip(rows, rows[1:]):
            if manifest[a]['piece'] == manifest[b]['piece']:
                adjacent.append([a, b])
    if build_dir:
        for player in ['originals', 'atmospheres']:
            assert (build_dir / f'assets/manifold/{player}.js').is_file(), f'Missing {player} bindings'
            wasm = build_dir / f'assets/manifold/{player}_bg.wasm'
            assert wasm.read_bytes()[:4] == b'\0asm', f'Missing or invalid {player} WASM'
    return {'catalogue': len(catalogue), 'asciiRest': sum('painting' not in row and 'original' not in row for row in catalogue),
            'earlierRustScenes': sum('painting' in row for row in catalogue), 'newIndependentOriginals': len(originals),
            'works': len(manifest), 'usedScenes': len(use), 'adjacentDuplicates': adjacent,
            'originals': originals, 'generatedDialogsChecked': len(routes) if build_dir else 0}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build-dir', type=Path)
    parser.add_argument('--expected-originals', type=int)
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    result = audit(args.build_dir, args.expected_originals)
    output = json.dumps(result, indent=2) + '\n'
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(output)
    print(output, end='')
