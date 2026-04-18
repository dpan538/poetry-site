import { analyzePoemText, computeEmotionScore } from './poem-analyzer';
import { contentSimilarity } from './content-types';

export interface PoemRecord {
  id: string;
  world: string;
  body: string;
  title: string;
}

export interface ScoredPoem {
  id: string;
  world: string;
  score: number;
}

export interface ReadingHistory {
  prev2: { id: string; emotionScore: number; world: string } | null;
  prev1: { id: string; emotionScore: number; world: string };
}

export function getNextPoem(
  current: PoemRecord,
  history: ReadingHistory,
  allPoems: PoemRecord[],
  seed: number,
): string {
  const currentAnalysis = analyzePoemText(current.body, current.title);
  const currentEmotion = computeEmotionScore(currentAnalysis.metrics);

  let targetDirection: 'up' | 'down' | 'neutral' = 'neutral';
  if (history.prev2) {
    const prev2Emotion = history.prev2.emotionScore;
    const prev1Emotion = history.prev1.emotionScore;
    const risingTwice =
      prev2Emotion < prev1Emotion && prev1Emotion < currentEmotion;
    const fallingTwice =
      prev2Emotion > prev1Emotion && prev1Emotion > currentEmotion;
    if (risingTwice) targetDirection = 'down';
    else if (fallingTwice) targetDirection = 'up';
    else targetDirection = prev1Emotion < currentEmotion ? 'up' : 'down';
  }

  const candidates: ScoredPoem[] = allPoems
    .filter((p) => p.id !== current.id)
    .filter((p) => p.id !== history.prev1.id)
    .filter((p) => (history.prev2 ? p.id !== history.prev2.id : true))
    .map((p) => {
      const analysis = analyzePoemText(p.body, p.title);
      const emotion = computeEmotionScore(analysis.metrics);

      let arcScore = 0;
      if (targetDirection === 'up') {
        arcScore = emotion > currentEmotion ? 1 : 0.3;
      } else if (targetDirection === 'down') {
        arcScore = emotion < currentEmotion ? 1 : 0.3;
      } else {
        arcScore = 1 - Math.abs(emotion - currentEmotion);
      }

      const contentScore = contentSimilarity(current.id, p.id);
      const worldScore = p.world !== current.world ? 1 : 0.3;
      const worldPenalty = p.world === history.prev1.world ? 0.5 : 1;

      const total =
        (arcScore * 0.4 + contentScore * 0.35 + worldScore * 0.15 + 0.1) * worldPenalty;

      return { id: p.id, world: p.world, score: total };
    });

  candidates.sort((a, b) => b.score - a.score);
  if (candidates.length === 0) return current.id;
  const top5 = candidates.slice(0, 5);
  const index = Math.floor(seed * top5.length);
  return top5[index]?.id ?? candidates[0].id;
}

export function getDailyPoemId(allPoems: PoemRecord[]): string {
  const today = new Date();
  const dateStr = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`;
  let hash = 5381;
  for (let i = 0; i < dateStr.length; i++) {
    hash = ((hash << 5) + hash) ^ dateStr.charCodeAt(i);
    hash |= 0;
  }
  const index = Math.abs(hash >>> 0) % allPoems.length;
  return allPoems[index].id;
}

/** Static build: pick next poem by content similarity only (no reading history). */
export function getStaticNextPoem(
  currentId: string,
  currentWorld: string,
  all: { id: string; world: string }[],
): { id: string; world: string } {
  const scored = all
    .filter((p) => p.id !== currentId)
    .map((p) => ({
      ...p,
      s: contentSimilarity(currentId, p.id),
    }))
    .sort((a, b) => b.s - a.s || a.id.localeCompare(b.id));

  const top5 = scored.slice(0, 5);
  if (top5.length === 0) {
    const fallback = all.find((p) => p.id !== currentId);
    return fallback ?? { id: currentId, world: currentWorld };
  }

  let hash = 5381;
  for (let i = 0; i < currentId.length; i++) {
    hash = ((hash << 5) + hash) ^ currentId.charCodeAt(i);
    hash |= 0;
  }
  const pick = top5[Math.abs(hash >>> 0) % top5.length];
  return { id: pick.id, world: pick.world };
}
