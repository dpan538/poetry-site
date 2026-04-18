export type PoemType =
  | 'dense-linear'
  | 'prose-lyric'
  | 'structured'
  | 'structured-repetitive'
  | 'incantatory'
  | 'lyric-fragmented'
  | 'lyric-dense'
  | 'minimal-fragment'
  | 'minimal-aphoristic'
  | 'meditative'
  | 'dramatic-monologue'
  | 'collage'
  | 'hymn-structured'
  | 'triptych'
  | 'hybrid'
  | 'still-observation'
  | 'dramatic-narrative';

export interface PoemMetrics {
  lineCount: number;
  wordCount: number;
  charCount: number;
  stanzaCount: number;
  avgLineLength: number;
  avgStanzaSize: number;
  stanzaVariance: number;
  punctuationDensity: number;
  uniqueWordRatio: number;
  lineEndPunctuation: number;
  repetitionScore: number;
  titleLength: number;
  hasDialogue: boolean;
  hasStageDirection: boolean;
  hasSectionNumbers: boolean;
}

export interface PoemAnalysis {
  metrics: PoemMetrics;
  type: PoemType;
}
