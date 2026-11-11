# Nine approved original MANIFOLD scenes

Two reviewed batches contain **nine independent originals**. The second batch excludes its withdrawn study 03. Study numbers belong to their individual review galleries; publication IDs are stable and distinct.

| Batch | Work | Scene ID | Replaces |
| --- | --- | --- | --- |
| First · 02 | Private Sky | `private-blue` | `green-storm` |
| First · 03 | Dusk | `dusk-gauze` | `misty-forest` |
| First · 04 | Crown Antiques | `crown-tiles` | `marine-drive` |
| First · 06 | Drowned Spirit | `drowned-water` | `lantern-lake` |
| Second · 01 | Going Home | `homeward-bridge` | `desert-night` |
| Second · 02 | The Observer | `observer-supper` | `lava-wall` |
| Second · 04 | Silver Tipped Shoes | `silver-lake` | `green-storm` |
| Second · 05 | Sleeping In | `sleeping-courtyard` | `snow-train` |
| Second · 06 | Iron Nails | `iron-shrine` | `kyoto-dusk` |

The catalogue contains **49 scenes**: 34 ascii.rest scenes, six previously accepted Rust scenes and these nine originals. Each original serves one work. Across 83 works, 44 scenes are used and there are no adjacent duplicates within a volume. Church remains `winter-church`; The Barn remains `wivenhoe-park`.

The five new catalogue descriptions were evaluated against the full corpus using the pinned MiniLM model. The ranking report records 8,398 text and vocabulary inputs; 4,931 text spans were inferred in this run, while vocabulary vectors came from the existing cache. Manuscript `manifoldScene` metadata pins the five approved associations. This integration updates their scene IDs and descriptions while retaining established excerpts and keywords. The other 78 assignments are unchanged from the preceding local integration.

## Runtime

The first four use `data-original`, lazy `originals.js` and Rust WASM. They retain their approved 20-second motion sequences and their 170 × 100 desktop / 102 × 60 mobile grids.

The second five use `data-atmosphere`, lazy `atmospheres.js` and Rust WASM. The renderer resolves only approved slugs. Scenes 01/02/04/05 use 255 × 150 desktop and 136 × 80 mobile cells; 06 uses 204 × 120 and 119 × 70. All retain 1.7:1 framing. Clouds, water and local illumination continue beyond the gallery ruler without a forced restart. Buildings and terrain retain their geometry.

Sleeping In keeps blue water and white/ivory stone. Twenty-six small pink beads drift, bob gently and reflect locally; there is no whole-pool blue/pink transition. Reduced motion renders a settled frame. Closing releases the player; atmospheric scene resizing preserves elapsed time and hidden tabs pause playback.

The build compiles both modules and generates their bindings alongside the existing players. The review galleries remain at localhost:4380 and localhost:4381; poem pages use the normal publication at localhost:4373.

## Verification

```sh
python3 scripts/audit-manifold.py --build-dir prototype/target/rust-preview --expected-originals 9
cargo test --locked --manifest-path prototype/Cargo.toml -p daipan-originals -p daipan-studies --lib
sh scripts/site build prototype/target/rust-preview
```

The audit compares catalogue IDs, the Rust publication registries, manuscript pins, generated dialog markers, WASM files, scene use and adjacent repetition. The static compiler also checks every excerpt against its manuscript.

Local evidence for this integration is stored in `prototype/target/scene-studies-review/library-integration/`: catalogue and registration reports, pool frames at 4/24/48 seconds, browser-flow results, and poem-overlay screenshots. Earlier continuous-playback recordings longer than 48 seconds remain under `prototype/target/scene-studies-review/perspective-v6/`. The 17 Rust tests and 34 browser-flow combinations passed across 12 poem pages. Browser automation includes Chrome and WebKit at desktop and phone sizes; it does not certify physical iPhone Safari behaviour.

The adapted ascii.rest algorithms retain their MIT attribution. Literary text remains © Dai Pan, all rights reserved. See the repository rights notice.
