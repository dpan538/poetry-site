# Reader delivery and caching audit

Baseline: commit `416907c7c58965565e8165c7e6fe1d12947dd60f`. Scope: static delivery and reader initialization, preserving the publication's text, layout rules, scene assignments and recommendation weights.

## Findings and decisions

All three reports identify worthwhile pipeline improvements, with two qualifications:

- Each of the 83 work pages contained approximately 10.6 KB of whole-publication navigation JSON in `#reading-data`. It also included two complete, hidden Index lists for desktop and mobile. Those lists accounted for substantially more of the roughly 100 KB HTML than the JSON alone. Both are now shared rather than repeated.
- Production CSS and WASM returned `Cache-Control: public, max-age=0, must-revalidate`, despite `x-vercel-cache: HIT`. The CDN was caching them, but browsers still had to validate stale local copies. This is a browser-freshness problem, not an absence of CDN caching. Immutable, content-fingerprinted assets now have a one-year browser lifetime.
- The approximately 139 KB gzipped WASM was the complete Rust reader: layout, navigation, dialogs and DOM interaction. It was not downloaded for every existing soft page turn. A first uncached visit still needs the reader, but subsequent visits can reuse its bytes without validation. Removing the optional WASM name section also reduces this first-load cost. Desktop Index links now use the existing soft-navigation path.

There is no evidence in this audit supporting a conclusion about unrelated sites or an individual's deployment habits.

## Implementation

The Rust SSG emits one `reading.json` containing publication metadata and sparse per-work affinity maps. A work page retains only its current volume and slug in `#reading-data`. The browser fetches the catalogue alongside WASM initialization, parses it once and retains it across soft navigation. Opening an Index constructs only that overlay's list; ordinary volume links remain as a fallback if the catalogue is unavailable. Server-rendered previous and next links also work without the catalogue.

The build fingerprints each CSS, JSON and WASM file independently using SHA-256. JavaScript modules and their relative imports stay together in a fingerprinted module bundle. Catalogue edits therefore do not invalidate unchanged WASM or CSS. The output and Vercel configuration use `/assets/immutable/<content-hash>/…` with `public, max-age=31536000, immutable`. HTML and unversioned URLs retain revalidation so a deployment can be discovered. This follows [Vercel's Cache-Control documentation](https://vercel.com/docs/caching/cache-control-headers).

MANIFOLD modules remain loaded on demand. Their relative imports and explicit WASM URLs resolve to the fingerprinted output. If soft navigation discovers HTML referencing a different runtime bundle, it performs a normal navigation instead of combining a previous deployment's runtime with new data.

## Measurements

Sizes below are bytes, measured from before/after static build artifacts using the same Python gzip compression. They are reproducible payload comparisons, not claims about a particular CDN's compression settings or page latency.

| Artifact | Before | After | Change |
| --- | ---: | ---: | ---: |
| Mean work HTML, all 83 pages | 102,384 | 28,053 | −72.6% |
| Mean work HTML, gzip | 17,995 | 6,014 | −66.6% |
| Reader WASM | 400,163 | 304,046 | −24.0% |
| Reader WASM, gzip | 139,121 | 127,473 | −8.4% |
| Shared catalogue, gzip | Embedded repeatedly | 7,273 | One cacheable resource |

The 83 work pages together shed 6,169,505 uncompressed bytes. For example, The Absurd falls from 105,490 to 31,176 bytes, and Church from 147,397 to 73,105 bytes. Long prose still produces larger pages because its complete text remains server-rendered.

## Verification

- All 83 works preserve identical SEO article text, mobile reading article text, desktop body text and MANIFOLD markup compared with the baseline. The shared catalogue preserves all 83 × 83 affinity values, emotion scores, content types and scene assignments.
- The native fingerprint test verifies stable URLs, independent invalidation and preserved relative module imports. Five existing reader layout tests also pass.
- Chrome checks cover 12 routes at 1440 × 900 and 393 × 852: The Absurd, Crown Antiques, Going Home, Early Spring, Church, Dusk, Silver Tipped Shoes, The Observer, Sleeping In, Iron Nails, Platform and About. No horizontal overflow, runtime errors or failed responses were found.
- Browser network checks verify that subsequent hard navigations reuse cached reader WASM and catalogue without transferring response bytes or issuing 304 validations. Soft Index navigation requests neither the runtime nor the catalogue again.
- WebKit checks cover desktop and mobile Index construction, section switching, scene loading, About/Home navigation and reduced motion. Failure checks cover a permanent catalogue failure, a successful retry after a transient failure and the cross-deployment runtime safeguard.
- All three Rust MANIFOLD players load through their fingerprinted paths: Church (painting player), Dusk (original player) and Sleeping In (atmospheric player). Selecting the current work in the Index closes the overlay without a document reload on desktop and mobile.
- Settled desktop reading, MANIFOLD and mobile Index screenshots were inspected. The transient reveal animation was allowed to finish before assessment.

The cold-start reader and catalogue requests are intentional remaining costs. This change does not claim to eliminate WASM initialization or browser layout computation on a new document. A real iPhone's Safari toolbar and safe area were not reproduced by desktop WebKit emulation; this delivery-only change does not alter their styling.
