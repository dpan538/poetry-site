import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';
import { publication, worldOrder, type WorldId } from '../lib/publication';

const LASTMOD = '2026-05-06';

function escapeXml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

function entry(path: string, priority: string): string {
  const loc = path === '/' ? `${publication.canonicalUrl}/` : `${publication.canonicalUrl}${path}`;
  return [
    '  <url>',
    `    <loc>${escapeXml(loc)}</loc>`,
    `    <lastmod>${LASTMOD}</lastmod>`,
    '    <changefreq>weekly</changefreq>',
    `    <priority>${priority}</priority>`,
    '  </url>',
  ].join('\n');
}

export const GET: APIRoute = async () => {
  const urls: string[] = [
    entry('/', '1.0'),
    entry('/about', '0.8'),
    entry('/contact', '0.7'),
    entry('/collection', '0.8'),
    entry('/preface', '0.6'),
  ];

  for (const world of worldOrder) {
    urls.push(entry(`/${world}`, '0.7'));
    const poems = [...(await getCollection(world as WorldId))].sort(
      (a, b) => a.data.order - b.data.order,
    );
    for (const poem of poems) {
      urls.push(entry(`/${world}/${poem.id}`, '0.5'));
    }
  }

  const body = [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">',
    ...urls,
    '</urlset>',
    '',
  ].join('\n');

  return new Response(body, {
    headers: {
      'Content-Type': 'application/xml; charset=utf-8',
      'Cache-Control': 'public, max-age=3600',
    },
  });
};
