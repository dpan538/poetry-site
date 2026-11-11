# Rust workspace

- `crates/ssg` compiles and serves the static publication.
- `crates/browser` runs the client interactions through Rust WASM and browser DOM interfaces.
- `crates/reader-core` computes typography, phrase breaks, stanza allocation, and collision avoidance.
- `crates/originals` renders the first four approved originals.
- `crates/studies` renders five approved atmospheric originals, published as `atmospheres.js` and WASM.
- `crates/site` runs the separate editorial pipeline for MANIFOLD.

```sh
cargo run --manifest-path prototype/Cargo.toml -p manifold-site
cargo test --manifest-path prototype/Cargo.toml -p reader-core
```

The editorial build reads the local Hugging Face cache for `sentence-transformers/all-MiniLM-L6-v2` revision `1110a243fdf4706b3f48f1d95db1a4f5529b4d41`. `MANIFOLD_MODEL_DIR` can point to a directory with that revision's model and tokenizer files.

Output lives in `prototype/target/manifold-evidence/`. The report records the scene scores, source excerpt and three keywords for each work. The embedding compares the entire corpus with the 49 scenes in `site/data/manifold-catalog.json`. The allocator honours explicit `manifoldScene` metadata, favours distinctive matches, lowers the priority of repeated scenes and separates neighbours in each volume. Pass `-- --preserve-scenes` to keep established associations while applying newly approved `manifoldScene` bindings and recalculating their excerpts and keywords. The site loads the matching Canvas player and scene only when the reader opens MANIFOLD.

Pass `-- --typst-plates` to the editorial command to render the earlier Typst-rs studies separately. The publication build uses the Canvas catalogue.
