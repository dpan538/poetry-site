# daipan.ink Search Visibility Audit

Date: 2026-05-06  
Target canonical proposed by owner: `https://daipan.ink/`

## 1. Executive Summary

Current stack: Astro 6 static site with `@astrojs/sitemap`, Tailwind/Vite, static output in `dist/`, and content collections for poem pages. This is not a pure SPA: pages are statically rendered and crawlers can read HTML without executing JavaScript. The site has client scripts for navigation and poem interactions, but the core text, metadata, JSON-LD, and poem fallbacks are present in the initial HTML.

Most severe issues found before repair:

- The live deployment currently redirects `https://daipan.ink` to `https://www.daipan.ink/`, while code canonical URLs pointed to `https://daipan.ink`. This is the highest-risk identity/canonical conflict.
- `https://daipan.ink/sitemap.xml` returned Vercel `NOT_FOUND`; the live robots file pointed to `https://daipan.ink/sitemap-index.xml`.
- `/contact` returned 404.
- Homepage title and description were readable but centered on `Three Worlds`, not clearly on `Dai Pan / Pan Dai official website`.
- Chinese identity text existed weakly or only in scattered places; Baidu and Chinese search engines had less semantic help.
- AI and search disambiguation text did not explicitly rule out Saipan, Taipan, tattoo studios, printer ink, or AI image platforms on the homepage.

Most likely cause of Google misidentification: low domain/entity confidence plus the live canonical conflict (`apex -> www` but canonical `apex`), missing root `sitemap.xml`, insufficient homepage disambiguation, and a domain string close to `saipan` / `taipan` / `ink`.

Most likely cause of Baidu non-inclusion: the site is new/low-authority, `sitemap.xml` was missing at the expected root path, Chinese identity text was thin, `/contact` was absent, and no Baidu platform submission has been confirmed. DNS itself is not the primary blocker.

Code-side fixes have been implemented locally. Deployment and webmaster submission are still required.

## 2. Evidence Table

| Area | Test | Command / Method | Result | Severity | Recommendation |
|---|---|---|---|---|---|
| Stack | Project scan | `package.json`, `astro.config.mjs`, `src/pages` | Astro 6 static site, not Next/Vite SPA | Info | Keep Astro SSG; no framework rewrite needed |
| SPA / no-JS | Live homepage HTML | `curl -L https://daipan.ink` | Initial HTML contains title, meta, JSON-LD, hidden SEO anchor, links, and poem list script data | Medium | Strengthen homepage entity text, done locally |
| HTTPS canonical | Apex | `curl -I https://daipan.ink` | `307` to `https://www.daipan.ink/` | High | Set deployment primary domain to apex or change all canonicals to www |
| HTTPS canonical | WWW | `curl -I https://www.daipan.ink` | `200` on www | High | If apex is desired, redirect www to apex after dashboard primary domain change |
| HTTP | Apex HTTP | `curl -I http://daipan.ink` | `308` to `https://daipan.ink/`, then live redirects to www | Medium | Final target should be one canonical host |
| robots | Live robots | `curl -L https://daipan.ink/robots.txt` | Allows Google/Bing/Baidu/Sogou; points to `sitemap-index.xml`; GPTBot disallowed | Medium | Updated locally to root `sitemap.xml`, added 360/Bytespider |
| sitemap | Root sitemap | `curl -L https://daipan.ink/sitemap.xml` | `NOT_FOUND` | High | Added `src/pages/sitemap.xml.ts` |
| sitemap | Existing index | `curl -L https://daipan.ink/sitemap-index.xml` | Exists and points to `sitemap-0.xml` | Low | Keep as secondary, but submit root `sitemap.xml` |
| Contact | Live contact | `curl -L -I https://daipan.ink/contact` | `307` then `404` on www | Medium | Added low-risk Contact page |
| DNS | Apex A | `dig daipan.ink @1.1.1.1`, `@8.8.8.8`, `@223.5.5.5`, `@114.114.114.114` | All resolve to `216.198.79.1` | Low | DNS resolution OK |
| DNS | WWW CNAME/A | `dig www.daipan.ink ...` | CNAME to `c9c8bdc64b361fc3.vercel-dns-017.com`; A answers vary within Vercel | Low | DNS OK; host routing is deployment config issue |
| DNS | NS | `dig daipan.ink NS` | `dns1.registrar-servers.com`, `dns2.registrar-servers.com` | Info | No action unless changing provider |
| DNS | AAAA | `dig daipan.ink AAAA`, `dig www.daipan.ink AAAA` | No IPv6 address answers | Low | Not required; optional if Vercel supports desired IPv6 |
| China compatibility | External resources | `rg "https?://|fonts.googleapis|analytics|youtube|instagram|cdn"` | No Google Fonts, analytics, YouTube, Vimeo, or external CDN scripts; only outbound daipan.art/Instagram links | Low | Good for mainland loading; keep critical assets self-hosted |
| Build | Local build | `npm run build` | Passed, 87 pages built | Pass | Deploy build output |
| Local sitemap | URL status | `CHECK_ORIGIN=http://127.0.0.1:4322 npm run check:sitemap -- http://127.0.0.1:4322/sitemap.xml` | 87/87 local sitemap URLs returned 200 | Pass | Re-run after deployment against production |

## 3. Domain & DNS Findings

Apex DNS:

- Default resolver: `daipan.ink A 216.198.79.1`, TTL about 1011 seconds.
- `1.1.1.1`: `216.198.79.1`, TTL 1799.
- `8.8.8.8`: `216.198.79.1`, TTL 1799.
- `223.5.5.5`: `216.198.79.1`, TTL 1799.
- `114.114.114.114`: `216.198.79.1`, TTL 1799.

WWW DNS:

- `www.daipan.ink CNAME c9c8bdc64b361fc3.vercel-dns-017.com.`
- Returned A records include Vercel addresses such as `64.29.17.65`, `216.198.79.65`, and from Google DNS `64.29.17.1`, `216.198.79.1`.
- China resolvers `223.5.5.5` and `114.114.114.114` resolved successfully, though `114.114.114.114` was slower.

NS:

- `dns1.registrar-servers.com`
- `dns2.registrar-servers.com`

AAAA:

- No apex AAAA record.
- WWW resolves CNAME but no final AAAA answer.

Judgment: DNS is globally configured and China public resolvers can resolve the domain. The main issue is not DNS absence; it is deployment host canonical behavior and search indexing.

## 4. HTTPS / Redirect / Canonical Findings

Live results before deployment of fixes:

| URL | Status | Final target |
|---|---:|---|
| `https://daipan.ink` | 307 | `https://www.daipan.ink/` |
| `https://www.daipan.ink` | 200 | `https://www.daipan.ink/` |
| `http://daipan.ink` | 308 | `https://daipan.ink/`, then `https://www.daipan.ink/` |
| `http://www.daipan.ink` | 308 | `https://www.daipan.ink/` |
| `https://daipan.ink/robots.txt` | 200 after redirect | Served from www |
| `https://daipan.ink/sitemap.xml` | 404 after redirect | Vercel `NOT_FOUND` |
| `https://daipan.ink/sitemap-index.xml` | 200 after redirect | Sitemap index exists |

This conflicts with the intended canonical `https://daipan.ink/`. Code now emits apex canonicals without trailing slashes except the homepage, but Vercel/domain settings must be changed so the live HTTP redirect target also uses apex.

I did not add a `vercel.json` host redirect because the Vercel dashboard currently appears to treat `www.daipan.ink` as primary. Adding a code-level `www -> apex` redirect before changing the platform primary domain could create a redirect loop.

## 5. Google Search Misidentification Findings

Manual Google UI access was not automated here. These queries still need to be checked manually in Google:

- `site:daipan.ink`
- `"daipan.ink"`
- `"Dai Pan" "daipan.ink"`
- `"Pan Dai" "daipan.ink"`
- `"Dai Pan" artist designer`
- `"Dai Pan" portfolio`
- `"Pan Dai" portfolio`
- `"潘岱" "daipan.ink"`

Likely reasons Google may show `Including results for saipan.ink` or unrelated `Saipan / Taipan / tattoo ink` content:

- Domain string is close to higher-confidence words/domains (`saipan`, `taipan`, generic `ink`).
- Live apex redirects to www while HTML canonical points to apex.
- Expected root `sitemap.xml` did not exist.
- Homepage title was previously `Three Worlds — Dai Pan`, which is literary and less explicit as a domain identity statement.
- Homepage did not directly say it is the official website of Dai Pan / Pan Dai and not Saipan/Taipan/ink products.
- External authority and search console submissions are likely low or absent.

Local fix: homepage title and meta now explicitly identify `Dai Pan / Pan Dai | Official Artist & Designer Website`, and the initial HTML contains direct disambiguation text.

## 6. Baidu / Chinese Search Visibility Findings

Baidu-specific risks:

- Live `robots.txt` allowed `Baiduspider`, which is good.
- Live `sitemap.xml` was missing, which is bad for submission and crawler discovery.
- Chinese identity text was too limited for a Chinese search engine to confidently map `daipan.ink` to `Dai Pan / Pan Dai / 潘岱`.
- The site is mostly English and art/poetry oriented, which is fine, but Baidu benefits from a concise Chinese identity sentence.
- No evidence of WAF/bot blocking was found from curl; Vercel served HTML publicly.
- China DNS public resolvers returned valid answers, so DNS is not the main Baidu blocker.

Local fix:

- Added Chinese meta description.
- Added Chinese semantic identity text on homepage, About, and Contact.
- Added `Baiduspider`, `Sogou web spider`, `360Spider`, and `Bytespider` allow rules in robots.
- Added root `sitemap.xml`.

Manual Baidu actions are required in `docs/daipan_ink_search_submission_checklist.md`.

## 7. AI Search Misinterpretation Findings

AI search tools likely misinterpreted the domain because crawler-visible identity signals were weaker than the domain ambiguity. If an AI search system cannot fetch the site, cannot find a sitemap, or sees a generic `.ink` domain with limited entity text, it may fill gaps with similar strings such as Saipan, Taipan, tattoo studios, printer ink, or AI image platforms.

Local fixes:

- Homepage HTML now includes `Dai Pan / Pan Dai official website`.
- Homepage, About, Contact, JSON-LD, and `llms.txt` now explicitly say the site is not related to Saipan, Taipan, tattoo studios, printer ink products, or AI image generation platforms.
- `WebSite` JSON-LD now names `Dai Pan / Pan Dai` and `daipan.ink`.
- `Person` JSON-LD uses `Artist and Designer` and includes alternate names.
- `/llms.txt` exists and now starts with a concise identity/disambiguation block.
- `/humans.txt` added.

Note: `GPTBot` remains disallowed in `robots.txt` because the site also states that AI training on the full poems is prohibited. Search-oriented AI crawlers such as `OAI-SearchBot`, `ChatGPT-User`, `PerplexityBot`, `ClaudeBot`, and `Claude-Web` are allowed.

## 8. Crawler Visibility / No-JS Findings

Current Astro pages are statically rendered. Without JavaScript, crawlers can see:

- Homepage title, meta description, canonical, OG tags, Twitter Card, JSON-LD.
- Homepage semantic identity anchor with Dai Pan / Pan Dai, official website, Chinese identity text, and disambiguation.
- About page readable body text and Chinese identity paragraph.
- Contact page readable body text.
- World index pages and poem pages.
- Poem pages include SSR-rendered poem text in `article.poem-seo-text`.

Crawlers cannot rely on:

- Desktop homepage random poem redirect, which depends on client JavaScript.
- Interactive poem canvas, atlas overlay, mobile dock behavior.

SSR/SSG rewrite is not needed because Astro already provides SSG. The low-risk fix is semantic fallback text, metadata, JSON-LD, sitemap, and robots, which has been implemented.

## 9. robots.txt Findings

Before:

- Root robots existed.
- It allowed major crawlers.
- It pointed to `https://daipan.ink/sitemap-index.xml`.
- It lacked explicit `360Spider` and `Bytespider`.

After local fix:

- `Sitemap: https://daipan.ink/sitemap.xml`
- Explicit allow rules for Googlebot, Bingbot, Baiduspider, Sogou web spider, 360Spider, Bytespider, YisouSpider, OAI-SearchBot, Claude, Perplexity, CCBot, ChatGPT-User, and cohere-ai.
- `GPTBot` remains `Disallow: /` for rights/training reasons.

## 10. sitemap.xml Findings

Before:

- Astro generated `sitemap-index.xml` and `sitemap-0.xml`.
- `https://daipan.ink/sitemap.xml` returned 404.
- `/contact` was absent from sitemap because the route did not exist.

After local fix:

- Added `src/pages/sitemap.xml.ts`.
- Root sitemap includes homepage, About, Contact, Collection, Preface, all three world index pages, and all poem pages.
- Uses `lastmod` `2026-05-06`.
- Local verification: 87/87 sitemap URLs returned 200 in preview using `scripts/check-sitemap.mjs`.

## 11. Metadata / Open Graph Findings

Before:

- Metadata existed and was better than a blank SPA.
- Homepage title emphasized `Three Worlds`, not the domain/person identity.
- Chinese description was not present as a dedicated meta description.
- Apple touch icon was missing.
- Canonical used apex while live redirect used www.

After local fix:

- Homepage title: `Dai Pan / Pan Dai | Official Artist & Designer Website`.
- Homepage description clearly identifies Dai Pan / Pan Dai and fields.
- Chinese meta description added.
- Canonicals normalized to apex without trailing slash except homepage.
- OG/Twitter inherit the improved title and descriptions.
- Apple touch icon added.

## 12. Structured Data Findings

Before:

- JSON-LD existed: `Person`, `WebSite`, `CreativeWorkSeries`, `Book`, volumes, poem schema, breadcrumbs.
- It contained detailed identity data, but `WebSite` was more collection-centered than official-site-centered.

After local fix:

- `WebSite` name is now `Dai Pan / Pan Dai`.
- `alternateName` includes `daipan.ink`, `Three Worlds — Dai Pan`, and `Dai Pan official website`.
- `Person` job title changed to `Artist and Designer`.
- Descriptions include official website, project archive, and disambiguation.

Caution:

- Existing `sameAs`, education, nationality, and location were already present before this audit. I did not add new unverified social links or awards.

## 13. China Mainland Compatibility Findings

External resource inventory:

| Resource | Type | Mainland risk | Blocks first paint? | Notes |
|---|---|---|---|---|
| `https://daipan.art` | Outbound link | Low/unknown | No | Companion site link only |
| `https://www.instagram.com/jumpingchick666/` | Outbound link | High in mainland | No | Link only, no embed |
| Google Fonts | External font | None found | No | Site uses local/system fonts |
| Google Analytics / gtag | Script | None found | No | No tracking added |
| YouTube/Vimeo | Embed | None found | No | No video embeds |
| External CDN scripts | Script | None found | No | JS/CSS are self-hosted in `/_astro` |

Asset sizes after build:

- `dist/_astro/AtlasOverlay...js`: 124 KB
- `dist/_astro/PoemCanvas...js`: 16 KB
- `dist/_astro/BaseLayout...css`: 28 KB
- `dist/og-image.png`: 64 KB
- `dist/apple-touch-icon.png`: 16 KB

No large public images over 500 KB were found. Mainland compatibility is generally good; the bigger China issue is indexing/submission, not blocked resources.

## 14. daipan.ink vs daipan.art Strategy

Current relationship in content:

- `daipan.ink` is the poetry/digital edition/project archive site.
- `daipan.art` is described as the companion visual art practice site.

Risk:

- If both sites contain near-duplicate identity pages without clear purpose separation, search engines may split entity signals.
- If `daipan.art` is stronger and `daipan.ink` is new, Google may treat `daipan.ink` as secondary unless cross-links and canonicals are clear.

Options:

- Option A: `daipan.ink` official site, `daipan.art` visual art portfolio. Keep both, cross-link both ways, do not canonical one to the other. Good if `daipan.ink` is the personal official domain.
- Option B: `daipan.art` main official site, `daipan.ink` poetry archive. Keep `daipan.ink` canonical to itself for poetry pages, but make About text say it is the poetry archive. Good if art portfolio is the main public identity.
- Option C: redirect one domain to the other. Only use if the content is mostly duplicate and one domain should disappear from search. This is not recommended without a deliberate branding decision.

Do not canonical `daipan.ink` pages to `daipan.art` unless you want `daipan.ink` not to rank independently. Do not redirect either domain until the owner decides the primary-domain strategy.

## 15. Recommended Fix Plan

P0: must fix immediately

- Deploy the local code changes.
- In Vercel/domain settings, make the live canonical host match the code. If `https://daipan.ink/` is primary, stop redirecting apex to www and redirect www to apex.
- Submit `https://daipan.ink/sitemap.xml` to Google, Bing, and Baidu after deployment.

P1: should fix

- Verify Baidu Search Resource Platform and use crawl diagnosis.
- Re-run production sitemap check after deploy.
- Add reciprocal link from `daipan.art` to `https://daipan.ink/` with clear text such as "Poetry and project archive: daipan.ink".
- Monitor Google-selected canonical in Search Console.

P2: nice to have

- Add a verified email/contact method only if the owner wants it public.
- Add `sameAs` only for confirmed public profiles.
- Consider adding `CreativeWork` schema for major project pages beyond poems if such pages are added later.
- Optional IPv6 if the deployment platform supports it cleanly.

Manual platform actions

- See `docs/daipan_ink_search_submission_checklist.md`.

## 16. Files Changed

- `public/robots.txt`: changed sitemap to root `sitemap.xml`, added `360Spider` and `Bytespider`.
- `public/humans.txt`: added concise human-readable identity record.
- `public/apple-touch-icon.png`: added touch icon from existing OG image.
- `src/components/SEO.astro`: strengthened title handling, Chinese meta description, keywords, WebSite JSON-LD, Person JSON-LD.
- `src/layouts/BaseLayout.astro`: normalized canonical paths, added apple touch icon, clarified footer official site.
- `src/pages/index.astro`: changed homepage title/description and added crawler-visible identity/disambiguation text.
- `src/pages/about.astro`: strengthened first identity paragraph and added Chinese identity text.
- `src/pages/contact.astro`: added low-risk contact/identity page.
- `src/pages/sitemap.xml.ts`: added root sitemap generator.
- `src/pages/llms.txt.ts`: added concise AI crawler identity/disambiguation block and root sitemap reference.
- `scripts/check-sitemap.mjs`: added sitemap status checker.
- `package.json`: added `check:sitemap` script.
- `docs/daipan_ink_search_submission_checklist.md`: added manual submission checklist.
- `docs/daipan_ink_search_visibility_audit.md`: this report.

## 17. Manual Checks Required

After deployment:

1. `curl -I https://daipan.ink` should be 200 or final 200 at `https://daipan.ink/`, not `https://www.daipan.ink/`.
2. `curl -L -I https://www.daipan.ink` should permanently redirect to `https://daipan.ink/`.
3. `curl -L https://daipan.ink/robots.txt` should contain `Sitemap: https://daipan.ink/sitemap.xml`.
4. `curl -L https://daipan.ink/sitemap.xml` should return XML, not `NOT_FOUND`.
5. `curl -L -I https://daipan.ink/contact` should return 200.
6. Submit sitemap to Google, Bing, and Baidu.
7. Run Google/Baidu manual search queries listed above.
8. Inspect Google-selected canonical in Search Console.
9. Use Baidu crawl diagnosis for homepage, robots, sitemap, About, and Contact.
