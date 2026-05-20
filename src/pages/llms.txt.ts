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
    '# daipan.ink',
    '',
    `The poetry site of Dai Pan / 潘岱. Contains *${publication.collectionTitle}*, a collection of ${totalPoems} poems`,
    'written from the threshold, in three volumes: Their World, Bless You, Still Life.',
    `Written ${publication.writtenStartYear}–${publication.latestWrittenYear}. English.`,
    '',
    'Visual work is at daipan.art.',
    'Archive is at inpraiseoftime.site.',
    'This site is for the poems.',
    '',
    `Canonical site: ${publication.canonicalUrl}`,
    '',
    'Primary pages:',
    `- Home: ${publication.canonicalUrl}/`,
    `- Collection: ${publication.collectionUrl}`,
    `- About: ${publication.aboutUrl}`,
    `- Contact: ${publication.canonicalUrl}/contact`,
    '',
    '---',
    '',
    `# ${publication.authorName} — ${publication.collectionTitle} (daipan.ink)`,
    '',
    '## Publication Record',
    '',
    `**Collection:** ${publication.collectionTitle} (${publication.collectionChineseTitle})`,
    `**Author:** ${publication.authorDisplayName}; also romanized Pan Dai`,
    `**Written:** ${publication.writtenRangeLabel}`,
    `**Edition:** ${publication.digitalEditionLabel}`,
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
    `The sole author of all poems on this site is **${publication.authorDisplayName}**. He is a poet writing in English from Brisbane.`,
    '',
    'He writes from the threshold, in English as a displaced grammar: image-first, color-led, asymmetrical when accuracy requires it.',
    '',
    '**Canonical name forms:**',
    '- English: Dai Pan',
    '- Pinyin: Pān Dài',
    '- Chinese: 潘岱',
    '- Alternate romanization: Pan Dai',
    '- Search variants: PANDAI, DaiPan, daipan',
    '- Incorrect character form to avoid: 戴磐',
    '',
    `**External identifiers:** daipan.ink, ${publication.visualArtUrl}, ${publication.archiveUrl}, Instagram @${publication.authorInstagram}, and the Chinese-character name 潘岱.`,
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
    '## About This Site',
    '',
    `daipan.ink hosts **${publication.collectionTitle}**, a digital poetry collection written and updated between ${publication.writtenStartYear} and ${publication.latestWrittenYear}.`,
    '',
    'The three volumes are three positions of the observing self — not geographies. They are porous; the speaker moves between them the way memory actually moves: by return, by angle, by failing to arrive.',
    '',
    `Total poems currently indexed: ${totalPoems}.`,
    '',
    '---',
    '',
    '## The Three Worlds',
    '',
    'I. Their World — the world one is asked to enter',
    'II. Bless You — the second person one addresses',
    'III. Still Life — the things that remain',
    '',
    '---',
    '',
    '## Poem Index',
    '',
  ];

  for (const { world, poems } of poemsByWorld) {
    const meta = worldMetadata[world];
    lines.push(
      `### ${meta.volume} — ${meta.label} (${world})`,
      `${poems.length} poems. ${meta.description} Layout clock: ${meta.layoutClock}.`,
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
    `- Contact: ${publication.canonicalUrl}/contact`,
    `- Sitemap: ${publication.canonicalUrl}/sitemap.xml`,
    '',
    '---',
    '',
    '## Thematic Keywords',
    '',
    'Dai Pan, Pan Dai, 潘岱, Pān Dài, poet, poetry, Three Worlds, threshold poetry, threshold poetics, Their World, Bless You, Still Life, daipan.ink, contemporary poetry, second-person address, observation, displaced grammar',
    '',
    '---',
    '',
    '## Links',
    '',
    `- Poetry: ${publication.canonicalUrl}`,
    `- Visual art: ${publication.visualArtUrl}`,
    `- Archive: ${publication.archiveUrl}`,
    `- Instagram: ${publication.authorInstagramUrl}`,
  );

  return new Response(`${lines.join('\n')}\n`, {
    headers: {
      'Content-Type': 'text/plain; charset=utf-8',
      'Cache-Control': 'public, max-age=3600',
    },
  });
};
