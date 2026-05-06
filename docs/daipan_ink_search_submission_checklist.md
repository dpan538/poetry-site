# daipan.ink Search Submission Checklist

Date prepared: 2026-05-06

## Google Search Console

1. Add a Domain property for `daipan.ink`, or a URL prefix property for `https://daipan.ink/`.
2. Verify ownership through DNS TXT or the deployment platform.
3. Submit sitemap: `https://daipan.ink/sitemap.xml`.
4. Use URL Inspection for `https://daipan.ink/`.
5. Request indexing for the homepage.
6. Inspect Coverage / Pages after Google crawls the site.
7. Check Google-selected canonical. It should resolve to `https://daipan.ink/` after deployment and domain settings are corrected.
8. Check mobile usability.
9. Check Rich Results / structured data for the homepage and About page.
10. Repeat manual search tests after indexing settles:
    - `site:daipan.ink`
    - `"daipan.ink"`
    - `"Dai Pan" "daipan.ink"`
    - `"Pan Dai" "daipan.ink"`
    - `"Dai Pan" artist designer`
    - `"潘岱" "daipan.ink"`
    - `"Dai Pan" portfolio`
    - `"Pan Dai" portfolio`

## Bing Webmaster Tools

1. Add `https://daipan.ink/`.
2. Verify ownership.
3. Submit sitemap: `https://daipan.ink/sitemap.xml`.
4. Run URL Inspection for the homepage, About, Contact, and a sample poem page.
5. Review crawl errors and canonical selection.
6. Consider IndexNow later if the project needs fast update notification.

## Baidu Search Resource Platform

1. Add site `daipan.ink`.
2. Complete ownership verification.
3. Submit sitemap: `https://daipan.ink/sitemap.xml`.
4. Manually submit the homepage URL.
5. Use crawl diagnosis on:
   - `https://daipan.ink/`
   - `https://daipan.ink/about`
   - `https://daipan.ink/contact`
   - `https://daipan.ink/robots.txt`
   - `https://daipan.ink/sitemap.xml`
6. Check whether Baiduspider can fetch HTML with status 200.
7. If available, use Baidu normal inclusion API after verification.
8. Wait for inclusion; avoid repeated daily resubmission of the same URL unless Baidu reports a crawl failure.

## Other Chinese Search

1. 360 Webmaster Platform: add `daipan.ink`, verify, submit sitemap.
2. Sogou Webmaster Platform if available: add site, verify, submit sitemap.
3. ByteDance / Toutiao search: no standard public webmaster workflow is guaranteed, but `Bytespider` is now allowed in `robots.txt`.

## Manual Search Tests

Run these on Google, Bing, Baidu, 360, Sogou, and AI search tools after deployment:

- `site:daipan.ink`
- `"daipan.ink"`
- `"Dai Pan" "daipan.ink"`
- `"Pan Dai" "daipan.ink"`
- `"Dai Pan" artist designer`
- `"潘岱" "daipan.ink"`
- `"Dai Pan" portfolio`
- `"Pan Dai" portfolio`
- `"daipan.ink" -saipan -taipan -tattoo -printer`

## Domain / Deployment Tasks

1. In Vercel or the deployment platform, set `https://daipan.ink/` as the primary domain if that is the chosen canonical.
2. Ensure `https://www.daipan.ink/` redirects permanently to `https://daipan.ink/`.
3. Re-run:
   - `curl -I https://daipan.ink`
   - `curl -L -I https://www.daipan.ink`
   - `curl -L https://daipan.ink/robots.txt`
   - `curl -L https://daipan.ink/sitemap.xml`
4. Confirm the deployed sitemap and canonical tags no longer disagree with the HTTP redirect target.
