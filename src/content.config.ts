import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

const poemSchema = z.object({
  title: z.string(),
  world: z.enum(['their-world', 'bless-you', 'still-life']),
  order: z.number(),
  firstLine: z.string().optional(),
});

export const collections = {
  'their-world': defineCollection({
    loader: glob({ pattern: '**/*.md', base: './src/content/their-world' }),
    schema: poemSchema,
  }),
  'bless-you': defineCollection({
    loader: glob({ pattern: '**/*.md', base: './src/content/bless-you' }),
    schema: poemSchema,
  }),
  'still-life': defineCollection({
    loader: glob({ pattern: '**/*.md', base: './src/content/still-life' }),
    schema: poemSchema,
  }),
};
