import type { ClientPoemRecord } from './poem-data-store'

export interface RecommendationContext {
  currentId: string
  currentWorld: string
  currentEmotionScore: number
  prev1: { id: string; world: string; emotionScore: number } | null
  prev2: { id: string; world: string; emotionScore: number } | null
}

function contentSimilarity(typesA: string[], typesB: string[]): number {
  if (!typesA.length || !typesB.length) return 0
  const intersection = typesA.filter(t => typesB.includes(t)).length
  const union = new Set([...typesA, ...typesB]).size
  return intersection / union
}

function deterministicFloat(seed: string): number {
  let hash = 5381
  for (let i = 0; i < seed.length; i++) {
    hash = ((hash << 5) + hash) ^ seed.charCodeAt(i)
  }
  return (hash >>> 0) / 0xffffffff
}

export function getNextPoemId(
  ctx: RecommendationContext,
  allPoems: ClientPoemRecord[],
): string {
  const current = allPoems.find(p => p.id === ctx.currentId)
  if (!current) return allPoems[0]?.id ?? ctx.currentId

  let targetDirection: 'up' | 'down' | 'neutral' = 'neutral'
  if (ctx.prev2 && ctx.prev1) {
    const risingTwice =
      ctx.prev2.emotionScore < ctx.prev1.emotionScore &&
      ctx.prev1.emotionScore < ctx.currentEmotionScore
    const fallingTwice =
      ctx.prev2.emotionScore > ctx.prev1.emotionScore &&
      ctx.prev1.emotionScore > ctx.currentEmotionScore
    if (risingTwice) targetDirection = 'down'
    else if (fallingTwice) targetDirection = 'up'
    else targetDirection =
      ctx.prev1.emotionScore < ctx.currentEmotionScore ? 'up' : 'down'
  }

  const excludeIds = new Set(
    [ctx.currentId, ctx.prev1?.id, ctx.prev2?.id].filter(Boolean) as string[],
  )

  const candidates = allPoems
    .filter(p => !excludeIds.has(p.id))
    .map(p => {
      let arcScore = 0.5
      if (targetDirection === 'up') {
        arcScore = p.emotionScore > ctx.currentEmotionScore ? 1 : 0.2
      } else if (targetDirection === 'down') {
        arcScore = p.emotionScore < ctx.currentEmotionScore ? 1 : 0.2
      } else {
        arcScore = 1 - Math.abs(p.emotionScore - ctx.currentEmotionScore)
      }

      const contentScore = contentSimilarity(current.contentTypes, p.contentTypes)
      const worldScore = p.world !== ctx.currentWorld ? 1 : 0.3
      const worldPenalty = ctx.prev1 && p.world === ctx.prev1.world ? 0.6 : 1

      const total =
        (arcScore * 0.4 + contentScore * 0.35 + worldScore * 0.15 + 0.1) *
        worldPenalty

      return { poem: p, score: total }
    })

  candidates.sort((a, b) => b.score - a.score)

  const top5 = candidates.slice(0, 5)
  const seedStr = `${ctx.currentId}-${new Date().toDateString()}`
  const idx = Math.floor(deterministicFloat(seedStr) * top5.length)
  return top5[idx]?.poem.id ?? candidates[0]?.poem.id ?? ctx.currentId
}

export function getPrevPoemId(ctx: RecommendationContext): string | null {
  return ctx.prev1?.id ?? null
}
