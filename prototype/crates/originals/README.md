# Original MANIFOLD studies

Six studies pair one focused scene with a manuscript excerpt. The associations and their reasons are in [studies.json](studies.json). Studies 02, 03, 04 and 06 are approved and registered in the publication catalogue as `private-blue`, `dusk-gauze`, `crown-tiles` and `drowned-water`. Studies 01 and 05 remain local candidates.

The review gallery's words come from a reading of the manuscripts. Published excerpts and keywords are recalculated by the full-corpus semantic pipeline with the approved scene bindings; the gallery's hand-selected words are not copied into the publication manifest.

Run from the repository root.

```sh
sh scripts/site originals 4380
```

Open [localhost:4380](http://localhost:4380/). Select a work, play its twenty-second loop, scrub to a particular moment, and open the source-image comparison.

Three scenes use original procedural Rust drawing. Three use original source images generated with the built-in imagegen tool, with their prompts recorded in [prompts.json](prompts.json). The generated images are saved in [sources](sources). The build reduces those images to forty colors before low-resolution sampling. Desktop scenes use 170 × 100 cells and mobile scenes use 102 × 60 cells, both at a 1.7:1 aspect ratio.

The Rust WebAssembly renderer follows the layered-scene and ordered-dither approach demonstrated by [ascii.rest](https://ascii.rest/). Dot coverage is compensated against a tinted ground to preserve the source exposure. Buildings and other fixed objects retain their coordinates. Each scene defines its moving elements independently. The entrance settles over 1.2 seconds, and reduced-motion mode starts with a still frame.

The six-study gallery remains available for local review. The production mount API exposes only the four approved slugs, chooses the mobile or desktop grid, honours reduced motion and cancels its animation frame when unmounted. Their quotes remain original literary text by Dai Pan and are covered by the repository rights notice.

The second motion study keeps the approved compositions of 02, 03, 04 and 06. In Dusk, separate foliage and cloth masks remove the static originals before recomposition. A shared wind field drives branches, delayed curtain billow and moving floor shadows; its main gust repeats every five seconds within the continuous twenty-second loop. Cloud textures travel with their clouds, glass bounds occlude dust, and the river mask excludes the foreground bench.

Dusk also includes nine warm patches of projected light with independent short pulses and small movements. Crown Antiques uses 210 motes behind the glass and 120 in front, computed once per frame and assigned to spatial bins before cell shading. Drowned Spirit preserves the original lamps and reflections; fifty window sources and thirty shore lights turn on after the midpoint, with corresponding reflections. The ambient twilight grade uses soft masks to avoid rectangular color boundaries.

The clear-sky study keeps a large blue field, with one faint wisp moving only a short distance before dispersing. Static detail is in the shop glazing, weathered masonry, ironwork, brass lock and paving. Dusk adds sparse airborne glints alongside its projected floor light. Foreground motes can pass in front of the antique shop without moving its architecture.

The settling-dust revision gives each antique-shop mote a one-way fall, a surface contact and a short stationary resting phase. Only invisible particles respawn. Dusk retains its floor glints and airborne lights, reduces the curtain displacement, and adds subtle changing illumination at the bedside. Regression tests cover dust trajectories, settled positions, preservation of existing lamps, and distinct additional light sources.

Run the motion invariants from the repository root:

```sh
cargo test --locked --manifest-path prototype/Cargo.toml -p daipan-originals --lib
```

The blue-hour revision separates the warm sunset from a connected sky matte, advects that layer and fades it away while keeping trees, facades and the foreground fixed. Eighty additional shore/window sources are spatially indexed; their activation is computed once per frame. The original waterfront lighting remains present at the opening and throughout the sequence.

Water separates ambient sunset reflection from lamp emission: the broad sunset field cools with the sky, while existing lamp reflections are preserved only along their source columns. The regression suite checks both warm-water removal and lamp preservation.

Foreground protection applies to displacement, not frozen illumination: bench slats, gaps and paving cool with the ambient light. The reconstructed sky beneath sunset clouds keeps a blue chromaticity to prevent cyan patches. Water color reaches the shore independently of the wider ripple-displacement feather.
