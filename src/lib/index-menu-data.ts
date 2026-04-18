import { getCollection } from 'astro:content';

export type IndexMenuPoemRow = {
  id: string;
  title: string;
  firstLine: string;
  order: number;
};

const worlds = ['their-world', 'bless-you', 'still-life'] as const;

export type IndexWorld = (typeof worlds)[number];

export async function loadPoemsByWorldForIndexMenu(): Promise<
  Record<IndexWorld, IndexMenuPoemRow[]>
> {
  const out: Record<IndexWorld, IndexMenuPoemRow[]> = {
    'their-world': [],
    'bless-you': [],
    'still-life': [],
  };

  for (const w of worlds) {
    const list = await getCollection(w);
    out[w] = [...list]
      .sort((a, b) => a.data.order - b.data.order)
      .map((p) => ({
        id: p.id,
        title: p.data.title,
        firstLine: p.data.firstLine ?? '',
        order: p.data.order,
      }));
  }

  return out;
}
