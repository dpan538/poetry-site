# Original MANIFOLD release verification

Verified locally on 11 November 2026 for the nine approved originals described in [the integration record](manifold-originals-integration.md).

| Boundary | Evidence |
| --- | --- |
| Manuscript → catalogue | Nine manuscript pins resolve to distinct registered originals; the Rust mount registries expose the same nine identifiers. |
| Catalogue → static pages | All 83 generated poem/prose dialogs match the manifest. Both original-renderer WASM files have valid headers. |
| Build | The Rust static build produces 91 pages, with 80 poems and three prose pieces. No inference model is needed for the publication build. |
| Source motion and rendering | 17 Rust tests pass: seven first-batch tests and ten atmospheric tests, including 128 upstream tone fixtures, static structure, spatial constraints, continuous motion and sparse pink beads over blue water. |
| Poem page → WASM → Canvas | Five atmospheric originals pass Chrome and WebKit at 1440 × 1000 and 393 × 852: 20 combinations. Modules stay lazy, frames change, closing stops updates and reduced motion stays still. |
| Existing player regression | Four earlier originals plus Church, The Barn and Platform open in Chrome at both viewport sizes: 14 further combinations. |
| Responsive composition | All new Canvas frames preserve 1.7:1 with no horizontal document overflow. Sleeping In continues playing after viewport changes. Desktop and mobile overlay screenshots were inspected. |
| Catalogue repetition | 49 registered scenes, 44 in use across 83 works; each of the nine originals is assigned once, with no adjacent duplicates inside a volume. |

The five newly bound works retain their established source excerpts and keywords. The other 78 assignments are unchanged from the preceding local integration. The full-corpus editorial ranking has been refreshed for all 49 catalogue entries.

Sleeping In was sampled at 4, 24 and 48 seconds on desktop and mobile. Water remains blue, stone remains white/ivory, and pink appears only as small drifting beads and their local reflections. The revised pool also has a continuous playback recording; the other five-scene recordings from the prior accepted geometry revision run beyond 48 seconds without a forced timeline restart.

Reproduce the catalogue/build boundary check with:

```sh
python3 scripts/audit-manifold.py --build-dir prototype/target/rust-preview --expected-originals 9
```

Detailed local outputs are under `prototype/target/scene-studies-review/library-integration/`, including `report.json` (34 browser combinations), `registration-report.json`, `catalogue-report.json`, pool samples and overlays. These generated artifacts are excluded from Git. Automated WebKit emulation does not establish physical iPhone Safari compatibility.
