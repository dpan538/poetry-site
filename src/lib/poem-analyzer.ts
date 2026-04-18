import type { PoemMetrics, PoemAnalysis, PoemType } from './poem-types';

export function analyzePoemText(rawText: string, title: string): PoemAnalysis {
  const metrics = computeMetrics(rawText, title);
  const type = classifyPoem(metrics);
  return { metrics, type };
}

function computeMetrics(text: string, title: string): PoemMetrics {
  const stanzas = text
    .split(/\n\s*\n/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0);

  const lines = text
    .split('\n')
    .map((l) => l.trim())
    .filter((l) => l.length > 0);

  const words = text
    .replace(/[^a-zA-Z\s]/g, ' ')
    .split(/\s+/)
    .filter((w) => w.length > 0);

  const stanzaSizes = stanzas.map((s) =>
    s.split('\n').filter((l) => l.trim().length > 0).length,
  );

  const avgStanzaSize =
    stanzaSizes.length > 0
      ? stanzaSizes.reduce((a, b) => a + b, 0) / stanzaSizes.length
      : 0;

  const stanzaVariance =
    stanzaSizes.length > 1
      ? stanzaSizes.reduce((acc, s) => acc + Math.pow(s - avgStanzaSize, 2), 0) /
        stanzaSizes.length
      : 0;

  const punctuation = (text.match(/[.,;:!?—–]/g) || []).length;

  const uniqueWords = new Set(words.map((w) => w.toLowerCase()));

  const linesEndingWithPunct = lines.filter((l) => /[.,;:!?—–]$/.test(l)).length;

  const stopWords = new Set([
    'the',
    'a',
    'an',
    'and',
    'or',
    'but',
    'in',
    'on',
    'at',
    'to',
    'for',
    'of',
    'with',
    'is',
    'was',
    'i',
    'my',
    'your',
    'their',
    'this',
    'that',
    'it',
    'as',
    'by',
    'from',
    'into',
    'not',
    'be',
    'have',
    'had',
    'are',
    'were',
    'they',
    'we',
    'you',
    'he',
    'she',
    'his',
    'her',
    'its',
    'our',
  ]);
  const wordFreq: Record<string, number> = {};
  words.forEach((w) => {
    const lower = w.toLowerCase();
    if (!stopWords.has(lower)) wordFreq[lower] = (wordFreq[lower] || 0) + 1;
  });
  const repeatedWords = Object.values(wordFreq).filter((count) => count > 3).length;
  const repetitionScore = Math.min(repeatedWords / 5, 1);

  return {
    lineCount: lines.length,
    wordCount: words.length,
    charCount: text.length,
    stanzaCount: stanzas.length,
    avgLineLength: lines.length > 0 ? words.length / lines.length : 0,
    avgStanzaSize,
    stanzaVariance,
    punctuationDensity: words.length > 0 ? punctuation / words.length : 0,
    uniqueWordRatio: words.length > 0 ? uniqueWords.size / words.length : 1,
    lineEndPunctuation: lines.length > 0 ? linesEndingWithPunct / lines.length : 0,
    repetitionScore,
    titleLength: title.length,
    hasDialogue: /[\u201c\u201d"]/.test(text),
    hasStageDirection: /\([^)]{2,}\)/.test(text),
    hasSectionNumbers: /\(\d+\)/.test(text),
  };
}

function classifyPoem(m: PoemMetrics): PoemType {
  if (m.hasStageDirection && m.stanzaCount > 3) return 'collage';

  if (m.hasSectionNumbers) return 'triptych';

  if (m.hasDialogue && m.lineCount > 15 && m.avgLineLength > 5) return 'dramatic-narrative';

  if (m.hasDialogue || m.hasStageDirection) return 'dramatic-monologue';

  if (m.lineEndPunctuation > 0.6 && m.stanzaVariance < 2 && m.avgStanzaSize >= 3)
    return 'hymn-structured';

  if (m.repetitionScore > 0.5 && m.avgLineLength < 5) return 'incantatory';

  if (m.repetitionScore > 0.4 && m.stanzaVariance < 2) return 'structured-repetitive';

  if (m.lineCount <= 8 && m.avgLineLength < 5 && m.uniqueWordRatio > 0.8)
    return 'minimal-aphoristic';

  if (m.lineCount <= 12 && m.avgLineLength < 5) return 'minimal-fragment';

  if (m.lineCount <= 20 && m.punctuationDensity < 0.1 && m.uniqueWordRatio > 0.7)
    return 'still-observation';

  if (m.avgLineLength > 7 && m.lineCount > 15) return 'prose-lyric';

  if (m.avgLineLength > 6 && m.stanzaCount <= 3) return 'dense-linear';

  if (m.punctuationDensity > 0.15 && m.lineCount > 20) return 'lyric-dense';

  if (m.uniqueWordRatio > 0.75 && m.stanzaVariance > 3) return 'lyric-fragmented';

  if (
    m.lineCount > 8 &&
    m.lineCount <= 25 &&
    m.avgLineLength >= 4 &&
    m.avgLineLength <= 7
  )
    return 'meditative';

  if (m.stanzaVariance > 5) return 'hybrid';

  if (m.stanzaVariance < 3 && m.avgStanzaSize >= 3) return 'structured';

  return 'prose-lyric';
}

export function computeEmotionScore(metrics: PoemMetrics): number {
  const normalizedLineLength = Math.min(metrics.avgLineLength / 15, 1);
  const normalizedVariance = Math.min(metrics.stanzaVariance / 10, 1);

  return (
    metrics.punctuationDensity * 0.25 +
    metrics.repetitionScore * 0.2 +
    (1 - metrics.uniqueWordRatio) * 0.15 +
    normalizedVariance * 0.2 +
    normalizedLineLength * 0.1 +
    metrics.lineEndPunctuation * 0.1
  );
}
