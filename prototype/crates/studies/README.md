# MANIFOLD scene studies

Five approved originals, numbered **01, 02, 04, 05, 06**. The withdrawn 03 is excluded. These five join the preceding four originals, making nine independently made MANIFOLD scenes across the two batches. The local publication maps them to Going Home, The Observer, Silver Tipped Shoes, Sleeping In and Iron Nails.

```sh
sh scripts/site studies 4381
```

Open <http://localhost:4381/?review=library-v7>. Rebuild with `sh scripts/site studies --build`. The local gallery versions its JavaScript/WASM assets and reloads when a completed build publishes a new revision.

## Scene model

The previous two-dimensional cutouts and sine-shaped cloud ribbons have been replaced. The scene layer adapts techniques from the bundled **ascii.rest** MIT sources:

- **Alpine Dawn:** ray-marched heightfields, finite-difference normals, directional illumination, cast shadows, slope/altitude snow coverage and distance haze. Terrain and visibility are computed once on scene selection.
- **Alpine Dawn / Lake Pichola:** domain-warped, horizontally wrapping cloud density fields, independent illuminated edges and continuous bilinear advection. Every scene has a different seed and atmosphere.
- **Gangaur Ghat:** light/shadow hierarchy across architectural planes, recesses, cornices, local lamps and reflections. Buildings and human silhouettes do not deform.
- **Lake Pichola:** render current above-water radiance before reflecting it into water. Reflections include moving clouds and current light, then receive depth-scaled ripple distortion.

01 places a perspective bridge on a stone quay beside its far house, with a separate path connecting the landing to the entrance; it includes drifting water lamps. 02 uses four subcell samples per cell to refine window bevels, ceramic rims, cutlery and stemware. It separates a predominantly black exterior and black-coated figures from a warm dinner interior, with cream table linen, white plates, candlelight, cast shadows, glass reflections, rising steam and exterior rain. 04 and 06 use different terrain configurations. 04 shades reflected terrain along mirrored camera rays, uses angle-dependent water reflectance, and separates uneven shorelines from mountain-foot shading. 05 uses a bright blue sky, white/ivory limestone, grey recesses and a drifting empty boat. Its pool stays blue. Twenty-six small pink beads drift with the current, bob gently and carry local reflections; pink never recolours the full pool, sky or architecture. 06 uses one large foreground torii to frame a smaller, lower shrine hall. Shrine steps, stone approach and pillar feet share consistent ground contacts. Basalt post bases stop red strokes above ground. There are no foreground trees or freestanding lamps; mountain depth controls aerial perspective.

Time is **continuous**. The first-minute ruler is an inspection control, not a forced loop boundary. Clouds do not reverse or restart at 24 seconds; source textures wrap only beyond the visible region. The player targets 15 fps.

## Publication runtime

The static build generates `atmospheres.js` and `atmospheres_bg.wasm` from this crate. A catalogue entry uses `original.renderer: atmospheres`; the compiler emits `data-atmosphere` on the poem dialog. Opening MANIFOLD imports the module on demand. `mount_atmosphere` resolves only the five approved stable slugs and selects the desktop or mobile grid. The player preserves elapsed time while resizing, pauses in hidden tabs, renders a settled still for reduced motion, and releases the frame callback on close. The local gallery retains inspection controls without shipping them on poem pages.

## Character renderer

Rust ports Kyoto Dusk's coupled highlight shoulder → material occupancy → stable spatial dither → glyph choice → coverage-compensated ink → palette selection. Neutral black/grey/white inks remain available alongside scene-specific colours. Light radiance may exceed one before the shoulder.

The atlas now uses the upstream player's actual ` ·•●` monospace font masks and baseline, replacing custom circle approximations. 01/02/04/05 use **255 × 150** desktop and **136 × 80** mobile cells. The retained 06 stays at **204 × 120** and **119 × 70**. All remain **1.7:1**. The backing canvas follows actual width and DPR; changed cells are copied at whole-device-pixel boundaries. Rotation/resize preserves scene and time. Pause, visibility handling, reduced motion and disposal control the animation frame lifecycle.

```sh
cargo test --locked --manifest-path prototype/Cargo.toml -p daipan-studies --lib
```

Checks cover the 128 upstream transfer fixtures, neutral inks, continuous source changes, stationary structures, the torii/road constraint, bridge landing, the approved-slug boundary, and blue water with sparse local pink beads. Browser checks cover Chrome/WebKit, desktop/mobile, time seeking, continuous playback, resizing, console errors and character output. These are functional checks; they do not establish artistic acceptance.

Earlier 20-case gallery checks and recordings longer than 48 seconds remain under `prototype/target/scene-studies-review/perspective-v6/`. Integration evidence and the revised pool review are stored under `prototype/target/scene-studies-review/library-integration/`. Automated WebKit checks do not substitute for a physical iPhone Safari test.

The adapted scene algorithms, four-dot encoding and atlas workflow credit **ascii.rest**, Copyright © 2026 bas3line. The full [MIT notice](ascii.rest-LICENSE.txt) is retained. Literary excerpts © Dai Pan; see the repository's content rights notice.
