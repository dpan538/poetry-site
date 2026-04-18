import type { PoemType } from './poem-types';

export interface TextFrame {
  colStart: number; // 1-based grid column
  colSpan: number; // Fibonacci: 1,2,3,5,8,13,21
  rowStart: number; // 1-based grid row
  rowSpan: number; // how many rows this frame occupies
}

export interface FrameLayout {
  id: string;
  poemTypes: PoemType[];
  titleFrame: TextFrame;
  bodyFrames: TextFrame[]; // content flows through these in order
}

export const frameLayouts: FrameLayout[] = [
  // Dense-linear, Prose-lyric: title left, one wide body column right
  {
    id: 'FL-dense-A',
    poemTypes: ['dense-linear', 'prose-lyric'],
    titleFrame: { colStart: 1, colSpan: 8, rowStart: 1, rowSpan: 3 },
    bodyFrames: [{ colStart: 10, colSpan: 13, rowStart: 1, rowSpan: 9 }],
  },

  // Dense-linear alt: title right, body left
  {
    id: 'FL-dense-B',
    poemTypes: ['dense-linear', 'prose-lyric', 'lyric-dense'],
    titleFrame: { colStart: 16, colSpan: 8, rowStart: 1, rowSpan: 3 },
    bodyFrames: [{ colStart: 1, colSpan: 13, rowStart: 1, rowSpan: 9 }],
  },

  // Dense-linear: title top, body two columns below
  {
    id: 'FL-dense-C',
    poemTypes: ['dense-linear', 'prose-lyric', 'dramatic-narrative'],
    titleFrame: { colStart: 1, colSpan: 21, rowStart: 1, rowSpan: 2 },
    bodyFrames: [
      { colStart: 1, colSpan: 10, rowStart: 3, rowSpan: 7 },
      { colStart: 12, colSpan: 10, rowStart: 3, rowSpan: 7 },
    ],
  },

  // Structured: title narrow left, body right, wide breathing
  {
    id: 'FL-structured-A',
    poemTypes: ['structured', 'hymn-structured', 'structured-repetitive'],
    titleFrame: { colStart: 1, colSpan: 5, rowStart: 1, rowSpan: 2 },
    bodyFrames: [{ colStart: 7, colSpan: 13, rowStart: 1, rowSpan: 9 }],
  },

  // Structured alt: title top-right, body left
  {
    id: 'FL-structured-B',
    poemTypes: ['structured', 'hymn-structured', 'meditative'],
    titleFrame: { colStart: 14, colSpan: 8, rowStart: 1, rowSpan: 2 },
    bodyFrames: [{ colStart: 1, colSpan: 13, rowStart: 1, rowSpan: 9 }],
  },

  // Triptych: title top, three body columns
  {
    id: 'FL-triptych-A',
    poemTypes: ['triptych', 'collage'],
    titleFrame: { colStart: 1, colSpan: 13, rowStart: 1, rowSpan: 1 },
    bodyFrames: [
      { colStart: 1, colSpan: 7, rowStart: 2, rowSpan: 8 },
      { colStart: 9, colSpan: 7, rowStart: 2, rowSpan: 8 },
      { colStart: 17, colSpan: 7, rowStart: 2, rowSpan: 8 },
    ],
  },

  // Triptych alt: title right, two body columns left
  {
    id: 'FL-triptych-B',
    poemTypes: ['triptych', 'collage', 'hybrid'],
    titleFrame: { colStart: 17, colSpan: 7, rowStart: 1, rowSpan: 2 },
    bodyFrames: [
      { colStart: 1, colSpan: 7, rowStart: 1, rowSpan: 9 },
      { colStart: 9, colSpan: 7, rowStart: 1, rowSpan: 9 },
    ],
  },

  // Incantatory: title tall left column, body center
  {
    id: 'FL-incant-A',
    poemTypes: ['incantatory', 'structured-repetitive'],
    titleFrame: { colStart: 1, colSpan: 3, rowStart: 1, rowSpan: 5 },
    bodyFrames: [{ colStart: 5, colSpan: 13, rowStart: 1, rowSpan: 9 }],
  },

  // Incantatory alt: title top large, body narrow below
  {
    id: 'FL-incant-B',
    poemTypes: ['incantatory', 'minimal-aphoristic'],
    titleFrame: { colStart: 1, colSpan: 13, rowStart: 1, rowSpan: 3 },
    bodyFrames: [{ colStart: 3, colSpan: 8, rowStart: 5, rowSpan: 5 }],
  },

  // Minimal fragment: diagonal, large whitespace
  {
    id: 'FL-minimal-A',
    poemTypes: ['minimal-fragment', 'minimal-aphoristic'],
    titleFrame: { colStart: 1, colSpan: 8, rowStart: 1, rowSpan: 2 },
    bodyFrames: [{ colStart: 14, colSpan: 8, rowStart: 6, rowSpan: 4 }],
  },

  // Minimal alt: centered island
  {
    id: 'FL-minimal-B',
    poemTypes: ['minimal-fragment', 'still-observation', 'minimal-aphoristic'],
    titleFrame: { colStart: 6, colSpan: 13, rowStart: 3, rowSpan: 2 },
    bodyFrames: [{ colStart: 9, colSpan: 8, rowStart: 6, rowSpan: 4 }],
  },

  // Lyric fragmented: scattered frames
  {
    id: 'FL-lyric-A',
    poemTypes: ['lyric-fragmented', 'lyric-dense', 'hybrid'],
    titleFrame: { colStart: 2, colSpan: 8, rowStart: 2, rowSpan: 2 },
    bodyFrames: [
      { colStart: 12, colSpan: 8, rowStart: 1, rowSpan: 4 },
      { colStart: 2, colSpan: 8, rowStart: 5, rowSpan: 5 },
    ],
  },

  // Lyric alt: title mid-left, two frames right and bottom
  {
    id: 'FL-lyric-B',
    poemTypes: ['lyric-fragmented', 'lyric-dense'],
    titleFrame: { colStart: 1, colSpan: 5, rowStart: 4, rowSpan: 3 },
    bodyFrames: [
      { colStart: 8, colSpan: 13, rowStart: 1, rowSpan: 5 },
      { colStart: 8, colSpan: 13, rowStart: 7, rowSpan: 3 },
    ],
  },

  // Meditative: centered, breathing room
  {
    id: 'FL-meditative-A',
    poemTypes: ['meditative', 'still-observation'],
    titleFrame: { colStart: 3, colSpan: 8, rowStart: 2, rowSpan: 2 },
    bodyFrames: [{ colStart: 5, colSpan: 8, rowStart: 5, rowSpan: 5 }],
  },

  // Dramatic monologue: title top, full-width body
  {
    id: 'FL-dramatic-A',
    poemTypes: ['dramatic-monologue', 'dramatic-narrative', 'collage'],
    titleFrame: { colStart: 1, colSpan: 13, rowStart: 1, rowSpan: 2 },
    bodyFrames: [{ colStart: 1, colSpan: 21, rowStart: 4, rowSpan: 6 }],
  },

  // Dramatic alt: title right, body left full height
  {
    id: 'FL-dramatic-B',
    poemTypes: ['dramatic-monologue', 'dramatic-narrative'],
    titleFrame: { colStart: 16, colSpan: 8, rowStart: 1, rowSpan: 2 },
    bodyFrames: [{ colStart: 1, colSpan: 13, rowStart: 1, rowSpan: 9 }],
  },

  // Still observation: right float
  {
    id: 'FL-still-A',
    poemTypes: ['still-observation', 'meditative'],
    titleFrame: { colStart: 14, colSpan: 8, rowStart: 3, rowSpan: 2 },
    bodyFrames: [{ colStart: 16, colSpan: 8, rowStart: 6, rowSpan: 4 }],
  },
];

// Select frame layout based on poem type and seed
export function selectFrameLayout(poemType: PoemType, rand: () => number): FrameLayout {
  const valid = frameLayouts.filter((fl) => fl.poemTypes.includes(poemType));
  if (valid.length === 0) {
    // Fallback: use FL-dense-A
    return frameLayouts[0];
  }
  const idx = Math.floor(rand() * valid.length);
  return valid[idx];
}
