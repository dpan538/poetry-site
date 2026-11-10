# daipan.ink

[![Read Three Worlds](https://img.shields.io/badge/Read-daipan.ink-285e72?style=flat-square)](https://daipan.ink/)
[![Rust](https://img.shields.io/badge/Rust-2024-b2492e?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-reader-526353?style=flat-square&logo=webassembly&logoColor=white)](https://webassembly.org/)

Three Worlds by Dai Pan. The Rust compiler renders the publication to static files, and Rust WASM runs the reading grid, navigation and overlays in the browser.

## Technology

The Cargo workspace uses Rust 2024. The static compiler turns Markdown manuscripts and HTML templates into pages, a sitemap and discovery metadata. The reader uses WebAssembly with `wasm-bindgen` and `web-sys` for DOM and Canvas access, and CSS for the responsive presentation. The separate editorial compiler uses Candle and MiniLM to rank scene matches, with Typst available for generating visual studies.

## Run locally

From the repository root:

```sh
rustup target add wasm32-unknown-unknown
sh scripts/site serve prototype/target/rust-preview 4373
```

Open [localhost:4373](http://localhost:4373/). To build the static output for a host:

```sh
sh scripts/site build dist
```

Cargo generates the WebAssembly bindings and module entry point. MANIFOLD loads either the ascii.rest player or the Rust painting renderer when opened.

Vercel builds production from `main` using `sh scripts/build-vercel`, which installs the pinned Rust toolchain when needed and publishes `dist/`. The production build uses the checked-in scene manifest and requires no model weights or service credentials.

Run `sh scripts/site lab 4376` to preview the six custom MANIFOLD animations and their source credits at [localhost:4376](http://localhost:4376/).

## Edit the publication

- `src/content/` contains the manuscript Markdown. A work has a stable volume, slug and order, with `kind: prose` for prose and poem as the default.
- `site/templates/` and `site/assets/` carry forward the existing page structure, typography, colors and CSS. The Rust compiler fills the templates from the manuscript, builds the catalogue and emits the canonical URLs and metadata.
- The Rust browser crate measures the active typeface, arranges verse into reading spreads, and sets prose in one continuous column. It also handles navigation and dialogs.
- `site/data/manifold.json` maps each work to its ranked Canvas scene, source excerpt and three keywords. Both scene players render a still frame when reduced motion is enabled. [ascii.rest scene documentation](https://ascii.rest/docs/pieces/)

The scene manifest is a checked-in editorial asset. To update semantic rankings after editing manuscript text, run the [editorial compiler](prototype/README.md), review its choices, and copy the generated manifest into `site/data/manifold.json`. The site compiler checks that each excerpt matches its work. Model weights stay in the editor's local cache.

## Credits and rights

Thank you to [ascii.rest](https://ascii.rest/) and its open-source contributors. MANIFOLD includes 34 of its scenes alongside six Rust-rendered loops. The bundled ascii.rest player and scenes retain their [MIT license](site/assets/manifold/ascii.rest-LICENSE.txt).

The poems, prose and other original literary text are by Dai Pan (潘岱). All rights reserved. See the [repository rights notice](LICENSE.md) for copying and reuse terms.
