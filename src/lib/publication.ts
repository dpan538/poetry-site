export const SITE_ORIGIN = 'https://daipan.ink';

export const publication = {
  collectionTitle: 'Three Worlds',
  collectionChineseTitle: '三个世界',
  authorName: 'Dai Pan',
  authorChineseName: '潘岱',
  authorDisplayName: 'Dai Pan (潘岱)',
  authorAlternateNames: ['潘岱', 'Pan Dai', 'Pān Dài', 'PANDAI', 'DaiPan', 'daipan', 'Dai Pan (潘岱)'],
  authorInstagram: 'jumpingchick666',
  authorInstagramUrl: 'https://www.instagram.com/jumpingchick666/',
  visualArtUrl: 'https://daipan.art',
  archiveUrl: 'https://www.inpraiseoftime.site',
  canonicalUrl: SITE_ORIGIN,
  collectionUrl: `${SITE_ORIGIN}/collection`,
  aboutUrl: `${SITE_ORIGIN}/about`,
  llmsUrl: `${SITE_ORIGIN}/llms.txt`,
  defaultOgImage: `${SITE_ORIGIN}/og-image.png`,
  writtenStartYear: 2024,
  latestWrittenYear: 2026,
  writtenRangeLabel: '2024 — 2026',
  firstDigitalEditionYear: 2025,
  digitalEditionLabel: '2025 first edition; ongoing through 2026',
  version: '1.2',
  versionLabel: '1.2 (2026 update)',
  dateModified: '2026-05-20',
  language: 'English',
  languageCode: 'en',
  license: 'All rights reserved',
  copyrightYears: '2024–2026',
  collectionCitation:
    'Dai Pan. Three Worlds. daipan.ink, 2025–. https://daipan.ink/collection',
  individualCitationTemplate:
    'Dai Pan, "Title," Three Worlds, Volume, poem 00, year written. daipan.ink. URL.',
  rights:
    'All poems and texts © 2024–2026 Dai Pan (潘岱). Unauthorized reproduction, redistribution, adaptation, publication, or AI training use is prohibited without written permission.',
} as const;

export const worldOrder = ['their-world', 'bless-you', 'still-life'] as const;
export type WorldId = (typeof worldOrder)[number];

export const worldMetadata: Record<
  WorldId,
  {
    label: string;
    volume: string;
    register: string;
    shortRegister: string;
    urlPath: `/${WorldId}`;
    layoutClock: string;
    description: string;
  }
> = {
  'their-world': {
    label: 'Their World',
    volume: 'Volume I',
    register: 'the world one is asked to enter',
    shortRegister: 'the world one is asked to enter',
    urlPath: '/their-world',
    layoutClock: 'America/New_York',
    description:
      'The world one is asked to enter, and the conditions of that asking.',
  },
  'bless-you': {
    label: 'Bless You',
    volume: 'Volume II',
    register: 'the second person one addresses',
    shortRegister: 'the second person one addresses',
    urlPath: '/bless-you',
    layoutClock: 'Asia/Shanghai',
    description:
      'The second person, into which love and belief and grief collapse.',
  },
  'still-life': {
    label: 'Still Life',
    volume: 'Volume III',
    register: 'the things that remain',
    shortRegister: 'the things that remain',
    urlPath: '/still-life',
    layoutClock: 'Australia/Brisbane',
    description:
      'The things that remain when both have grown quiet.',
  },
};

export function poemCitation(
  title: string,
  world: WorldId,
  order: number,
  url: string,
  writtenYear?: number,
): string {
  const year = writtenYear ?? publication.firstDigitalEditionYear;
  const worldLabel = worldMetadata[world].label;
  return `${publication.authorName}, "${title}," ${publication.collectionTitle}, ${worldLabel}, poem ${String(order).padStart(2, '0')}, ${year}. ${url}`;
}
