const sitemapUrl = process.argv[2] ?? 'http://localhost:4321/sitemap.xml';
const checkOrigin = process.env.CHECK_ORIGIN;

async function request(url, method = 'HEAD') {
  const response = await fetch(url, { method, redirect: 'follow' });
  return {
    url,
    status: response.status,
    finalUrl: response.url,
  };
}

function extractLocs(xml) {
  return [...xml.matchAll(/<loc>([^<]+)<\/loc>/g)].map((match) =>
    match[1]
      .replace(/&amp;/g, '&')
      .replace(/&lt;/g, '<')
      .replace(/&gt;/g, '>')
      .replace(/&quot;/g, '"')
      .replace(/&apos;/g, "'"),
  );
}

const sitemapResponse = await fetch(sitemapUrl, { redirect: 'follow' });
if (!sitemapResponse.ok) {
  throw new Error(`Sitemap returned ${sitemapResponse.status}: ${sitemapResponse.url}`);
}

const sitemapXml = await sitemapResponse.text();
const urls = extractLocs(sitemapXml);

if (urls.length === 0) {
  throw new Error(`No <loc> URLs found in ${sitemapUrl}`);
}

let failures = 0;

for (const url of urls) {
  const checkUrl = checkOrigin ? new URL(new URL(url).pathname, checkOrigin).href : url;
  let result = await request(checkUrl, 'HEAD');
  if (result.status === 405 || result.status === 403) {
    result = await request(checkUrl, 'GET');
  }

  const ok = result.status >= 200 && result.status < 400;
  const marker = ok ? 'OK' : 'FAIL';
  const checked = checkUrl === url ? url : `${url} as ${checkUrl}`;
  console.log(`${marker} ${result.status} ${checked}${result.finalUrl !== checkUrl ? ` -> ${result.finalUrl}` : ''}`);

  if (!ok) failures += 1;
}

if (failures > 0) {
  throw new Error(`${failures} sitemap URL(s) failed`);
}

console.log(`Checked ${urls.length} sitemap URL(s) from ${sitemapUrl}`);
