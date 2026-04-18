export type ContentType =
  | 'love'
  | 'monologue'
  | 'narrative'
  | 'observation'
  | 'incantatory'
  | 'political'
  | 'spiritual'
  | 'fragmented';

export const poemContentTypes: Record<string, ContentType[]> = {
  breakfast: ['narrative', 'observation'],
  cottage: ['narrative', 'spiritual'],
  shadow: ['incantatory', 'spiritual', 'fragmented'],
  candle: ['monologue', 'spiritual'],
  'the-observer': ['observation', 'political'],
  'going-home': ['narrative', 'political'],
  side: ['political', 'fragmented'],
  'join-us': ['incantatory', 'political'],
  hope: ['incantatory', 'spiritual', 'political'],
  window: ['narrative', 'political'],
  manifesto: ['monologue', 'political', 'fragmented'],
  trash: ['monologue', 'political'],
  siren: ['incantatory', 'political', 'spiritual'],
  'the-age-of-secrets': ['political', 'fragmented'],
  'private-sky': ['narrative', 'political'],
  umbrella: ['monologue', 'love'],
  'the-barn': ['narrative', 'observation'],
  pockets: ['observation', 'political', 'fragmented'],
  room: ['monologue', 'fragmented'],
  'the-white-tree': ['narrative', 'spiritual'],
  farewell: ['narrative', 'spiritual'],

  river: ['love', 'fragmented'],
  ears: ['love', 'observation'],
  kite: ['love', 'monologue'],
  gentleness: ['love', 'spiritual'],
  telecine: ['fragmented', 'monologue'],
  beloved: ['love', 'spiritual'],
  'in-the-sunlight': ['love', 'narrative', 'observation'],
  'silver-tipped-shoes': ['love', 'observation', 'incantatory'],
  feet: ['observation', 'spiritual', 'fragmented'],
  missing: ['spiritual', 'fragmented'],
  unworthy: ['monologue', 'political'],
  wandering: ['monologue', 'fragmented'],
  how: ['love', 'monologue'],
  'milk-cow-cat': ['love', 'narrative'],
  symphony: ['spiritual', 'observation'],
  'behind-my-mind': ['love', 'spiritual'],
  'dancing-children': ['observation', 'spiritual'],
  today: ['fragmented', 'monologue'],
  'flying-away': ['narrative', 'observation', 'spiritual'],
  perplexity: ['narrative', 'spiritual'],
  'crown-antiques': ['observation', 'narrative'],
  'drowned-spirit': ['narrative', 'spiritual'],
  'young-blood': ['political', 'monologue'],
  'the-absurd': ['incantatory', 'political', 'fragmented'],
  'traces-of-healing': ['spiritual', 'love'],
  cube: ['monologue', 'political'],
  'the-only': ['observation', 'monologue'],

  tourist: ['narrative', 'observation'],
  dusk: ['love', 'observation'],
  'sleeping-in': ['narrative', 'observation'],
  roaring: ['observation', 'monologue'],
  'iron-nails': ['observation'],
  'urban-monologue': ['observation', 'political'],
  'just-a-sign': ['monologue', 'political'],
  thermos: ['monologue', 'observation'],
  winter: ['love', 'spiritual'],
  'snowy-county': ['observation', 'incantatory'],
  'after-snow': ['observation', 'fragmented'],
  'green-tea': ['narrative', 'observation'],
  'afternoon-nap': ['narrative', 'love'],
  'boring-clouds': ['monologue', 'observation'],
  platform: ['love', 'fragmented'],
  coin: ['love', 'monologue'],
  'stone-frog': ['love', 'observation'],
  yellow: ['observation', 'spiritual'],
  'early-spring': ['observation', 'fragmented'],
  washing: ['incantatory', 'monologue'],
  'third-three': ['monologue', 'fragmented'],
  'black-paint': ['political', 'fragmented', 'monologue'],
  'night-light': ['monologue', 'observation'],
  ghost: ['spiritual', 'fragmented'],
  photo: ['observation', 'narrative'],
  feather: ['monologue', 'spiritual'],
  'breathe-away': ['observation', 'spiritual'],
};

export function contentSimilarity(poemA: string, poemB: string): number {
  const typesA = poemContentTypes[poemA] ?? [];
  const typesB = poemContentTypes[poemB] ?? [];
  if (typesA.length === 0 || typesB.length === 0) return 0;
  const intersection = typesA.filter((t) => typesB.includes(t)).length;
  const union = new Set([...typesA, ...typesB]).size;
  return intersection / union;
}
