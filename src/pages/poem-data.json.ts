import type { APIRoute } from 'astro'
import { getCollection } from 'astro:content'
import { analyzePoemText, computeEmotionScore } from '../lib/poem-analyzer'
import { poemContentTypes } from '../lib/content-types'
import atlas from '../data/atlas.json'

type AtlasRelation = {
  id: string
  weight: number
  relation?: string
  reason?: string[]
}

const atlasRelationMap = new Map<string, AtlasRelation[]>(
  atlas.poems.map((poem) => [
    poem.id,
    (poem.related_poems ?? []).map((relation) => ({
      id: relation.id,
      weight: typeof relation.weight === 'number' ? relation.weight : 0,
      relation: relation.relation,
      reason: relation.reason,
    })),
  ]),
)

export const GET: APIRoute = async () => {
  const worlds = ['their-world', 'bless-you', 'still-life'] as const
  const allPoems: Array<{
    id: string
    world: (typeof worlds)[number]
    title: string
    firstLine: string
    emotionScore: number
    contentTypes: string[]
    atlasRelations: AtlasRelation[]
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
        atlasRelations: atlasRelationMap.get(poem.id) ?? [],
      })
    }
  }

  return new Response(JSON.stringify(allPoems), {
    headers: { 'Content-Type': 'application/json' },
  })
}
