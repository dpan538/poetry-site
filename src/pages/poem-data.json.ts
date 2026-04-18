import type { APIRoute } from 'astro'
import { getCollection } from 'astro:content'
import { analyzePoemText, computeEmotionScore } from '../lib/poem-analyzer'
import { poemContentTypes } from '../lib/content-types'

export const GET: APIRoute = async () => {
  const worlds = ['their-world', 'bless-you', 'still-life'] as const
  const allPoems: Array<{
    id: string
    world: (typeof worlds)[number]
    title: string
    firstLine: string
    emotionScore: number
    contentTypes: string[]
  }> = []

  for (const world of worlds) {
    const poems = await getCollection(world)
    for (const poem of poems) {
      const body = poem.body ?? ''
      const analysis = analyzePoemText(body, poem.data.title)
      const emotionScore = computeEmotionScore(analysis.metrics)
      const contentTypes = poemContentTypes[poem.id] ?? []

      allPoems.push({
        id: poem.id,
        world,
        title: poem.data.title,
        firstLine: poem.data.firstLine ?? '',
        emotionScore: Math.round(emotionScore * 1000) / 1000,
        contentTypes,
      })
    }
  }

  return new Response(JSON.stringify(allPoems), {
    headers: { 'Content-Type': 'application/json' },
  })
}
