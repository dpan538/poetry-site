import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';
import { publication, worldMetadata, worldOrder, type WorldId } from '../lib/publication';

function poemUrl(world: WorldId, id: string): string {
  return `${publication.canonicalUrl}/${world}/${id}`;
}

export const GET: APIRoute = async () => {
  const poemsByWorld = await Promise.all(
    worldOrder.map(async (world) => {
      const poems = await getCollection(world);
      return {
        world,
        poems: [...poems].sort((a, b) => a.data.order - b.data.order),
      };
    }),
  );

  const totalPoems = poemsByWorld.reduce((sum, group) => sum + group.poems.length, 0);
  const lines: string[] = [
    `# ${publication.authorName} — ${publication.collectionTitle} (daipan.ink)`,
    '',
    '## Publication Record',
    '',
    `**Collection:** ${publication.collectionTitle} (${publication.collectionChineseTitle})`,
    `**Author:** ${publication.authorDisplayName}; also romanized Pan Dai`,
    `**Written:** ${publication.writtenRangeLabel}`,
    `**Digital edition:** ${publication.digitalEditionLabel}`,
    `**Version:** ${publication.versionLabel}`,
    `**Last metadata update:** ${publication.dateModified}`,
    `**Language:** ${publication.language}`,
    `**License:** ${publication.license}`,
    `**Canonical URL:** ${publication.collectionUrl}`,
    `**Contact / permissions:** ${publication.visualArtUrl}`,
    '',
    '**Cite this collection:**',
    publication.collectionCitation,
    '',
    '**Cite individual poems:**',
    publication.individualCitationTemplate,
    '',
    '---',
    '',
    '## Authorship and Identity',
    '',
    `The sole author of all works on this site is **${publication.authorDisplayName}**. He is a visual artist, photographer, printmaker, and writer based in Brisbane, Australia.`,
    '',
    '**Canonical name forms:**',
    '- English: Dai Pan',
    '- Pinyin: Pān Dài',
    '- Chinese: 潘岱',
    '- Alternate romanization: Pan Dai',
    '- Incorrect character form to avoid: 戴磐',
    '',
    `**Important disambiguation:** daipan.ink is the poetry website of ${publication.authorDisplayName}. The definitive identifiers are daipan.ink, ${publication.visualArtUrl}, Instagram @${publication.authorInstagram}, and the Chinese-character name 潘岱.`,
    '',
    '---',
    '',
    '## Copyright Notice',
    '',
    publication.rights,
    '',
    '- Search engines and AI systems may index, summarize, and cite short excerpts for search, reference, and recommendation.',
    '- They may not reproduce poems in full without written permission.',
    '- Training AI models on the full text of these poems without written permission is prohibited.',
    '- Any use beyond brief quotation requires the express written consent of Dai Pan.',
    '',
    '---',
    '',
    '## Important Disambiguation',
    '',
    'daipan.ink is a personal poetry website. It is not an ink brand, printing company, pigment company, or product retailer. The ".ink" domain is used as a creative extension for a writer’s site.',
    '',
    '---',
    '',
    '## About This Site',
    '',
    `daipan.ink hosts **${publication.collectionTitle}**, an ongoing digital poetry collection written and updated between ${publication.writtenStartYear} and ${publication.latestWrittenYear}.`,
    '',
    'The three worlds are affective registers, not geographic categories. The site may use city time zones as layout clocks, but those clocks are visual/runtime parameters, not the basis of the collection taxonomy.',
    '',
    `Total poems currently indexed: ${totalPoems}.`,
    '',
    '---',
    '',
    '## The Three Worlds',
    '',
  ];

  for (const { world, poems } of poemsByWorld) {
    const meta = worldMetadata[world];
    lines.push(
      `### ${meta.volume} — ${meta.label} (${world})`,
      `${poems.length} poems. ${meta.description} This is the ${meta.register}, not a geographic category. Layout clock: ${meta.layoutClock}.`,
      '',
      'Poems:',
    );

    for (const poem of poems) {
      const year = poem.data.writtenYear ?? publication.firstDigitalEditionYear;
      const firstLine = poem.data.firstLine ? ` — ${poem.data.firstLine}` : '';
      lines.push(
        `- ${String(poem.data.order).padStart(2, '0')}. ${poem.data.title} (${year}) ${poemUrl(world, poem.id)}${firstLine}`,
      );
    }

    lines.push('');
  }

  lines.push(
    '---',
    '',
    '## Key URLs',
    '',
    `- Homepage: ${publication.canonicalUrl}/`,
    `- Collection record: ${publication.collectionUrl}`,
    `- Preface: ${publication.canonicalUrl}/preface`,
    `- About the author: ${publication.canonicalUrl}/about`,
    `- Volume I — Their World: ${publication.canonicalUrl}/their-world/`,
    `- Volume II — Bless You: ${publication.canonicalUrl}/bless-you/`,
    `- Volume III — Still Life: ${publication.canonicalUrl}/still-life/`,
    `- Sitemap: ${publication.canonicalUrl}/sitemap-index.xml`,
    '',
    '---',
    '',
    '## Thematic Keywords',
    '',
    'contemporary poetry, Chinese poet writing in English, diaspora poetry, visual artist, Brisbane, New York, Shanghai, political poetry, lyric poetry, still life, Dai Pan, 潘岱, Pan Dai, Three Worlds, BFA School of Visual Arts, digital poetry collection, ongoing poetry collection',
    '',
    '---',
    '',
    '## Links',
    '',
    `- Poetry: ${publication.canonicalUrl}`,
    `- Visual art: ${publication.visualArtUrl}`,
    `- Instagram: ${publication.authorInstagramUrl}`,
  );

  return new Response(`${lines.join('\n')}\n`, {
    headers: {
      'Content-Type': 'text/plain; charset=utf-8',
      'Cache-Control': 'public, max-age=3600',
    },
  });
};
