export const SITE_ORIGIN = 'https://daipan.ink';

export const publication = {
  collectionTitle: 'Three Worlds',
  collectionChineseTitle: '三个世界',
  authorName: 'Dai Pan',
  authorChineseName: '潘岱',
  authorDisplayName: 'Dai Pan (潘岱)',
  authorAlternateNames: ['潘岱', 'Pan Dai', 'Pān Dài', 'Dai Pan (潘岱)'],
  authorInstagram: 'jumpingchick666',
  authorInstagramUrl: 'https://www.instagram.com/jumpingchick666/',
  visualArtUrl: 'https://daipan.art',
  canonicalUrl: SITE_ORIGIN,
  collectionUrl: `${SITE_ORIGIN}/collection`,
  llmsUrl: `${SITE_ORIGIN}/llms.txt`,
  writtenStartYear: 2024,
  latestWrittenYear: 2026,
  writtenRangeLabel: '2024 — 2026',
  firstDigitalEditionYear: 2025,
  digitalEditionLabel: '2025 first edition; ongoing edition updated 2026',
  version: '1.1',
  versionLabel: '1.1 (2026 update)',
  dateModified: '2026-04-29',
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
    register: 'symbolic / exterior register',
    shortRegister: 'symbolic exterior',
    urlPath: '/their-world',
    layoutClock: 'America/New_York',
    description:
      'Enters the symbolic: the mechanical world that defines and disfigures us, a terrain of institutions, roles, and ambient control.',
  },
  'bless-you': {
    label: 'Bless You',
    volume: 'Volume II',
    register: 'address / attachment register',
    shortRegister: 'address and attachment',
    urlPath: '/bless-you',
    layoutClock: 'Asia/Shanghai',
    description:
      'Addresses a "you" that condenses attachment, devotion, intimacy, grief, and belief into a single unstable figure.',
  },
  'still-life': {
    label: 'Still Life',
    volume: 'Volume III',
    register: 'interior / perception register',
    shortRegister: 'interior perception',
    urlPath: '/still-life',
    layoutClock: 'Australia/Brisbane',
    description:
      'Turns toward the internal world: subjective overflow, sensual reflection, intuition, affect, and memory.',
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
