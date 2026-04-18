import type { PRNG } from './seed';
import type { PoemAnalysis } from './poem-types';
import { getValidModes, layoutModes, type LayoutMode } from './layout-modes';

export interface LayoutResult {
  mode: LayoutMode;
  titleColStart: number;
  titleColSpan: number;
  titleRowStart: number;
  titleRowSpan: number;
  titleSize: string;
  titleAlign: string;
  bodyColStart: number;
  bodyColSpan: number;
  bodyRowStart: number;
  metaCorner: string;
  accentTarget: string;
}

const GRID_COLS = 24;
const GRID_ROWS = 10;

function clampCol(start: number, span: number): number {
  const maxStart = GRID_COLS - span + 1;
  return Math.max(1, Math.min(start, maxStart));
}

function clampRow(row: number): number {
  return Math.max(1, Math.min(row, GRID_ROWS));
}

export function computeLayout(rand: PRNG, analysis: PoemAnalysis): LayoutResult {
  let validModes = getValidModes(analysis.type);

  if (validModes.length === 0) {
    validModes = getValidModes('prose-lyric');
  }
  if (validModes.length === 0) {
    validModes = [...layoutModes];
  }

  const weighted: LayoutMode[] = [];
  validModes.forEach((m) => {
    const w = m.id.startsWith('EX-') ? 1 : 3;
    for (let i = 0; i < w; i++) weighted.push(m);
  });

  const modeIndex = Math.floor(rand() * weighted.length);
  const mode = weighted[modeIndex];

  let titleColStart = mode.titleColStart;
  let titleRowStart = mode.titleRowStart;
  let bodyRowStart = mode.bodyRowStart;

  if (mode.seedAdjustable.includes('titleColStart')) {
    titleColStart = mode.titleColStart + (rand() > 0.5 ? 1 : 0);
  }
  if (mode.seedAdjustable.includes('titleRowStart')) {
    titleRowStart = Math.max(1, mode.titleRowStart + (rand() > 0.5 ? 1 : 0));
  }
  if (mode.seedAdjustable.includes('bodyRowStart')) {
    bodyRowStart = Math.max(
      titleRowStart + mode.titleRowSpan + 1,
      mode.bodyRowStart + (rand() > 0.5 ? 1 : 0),
    );
  }

  titleColStart = clampCol(titleColStart, mode.titleColSpan);
  titleRowStart = clampRow(titleRowStart);
  bodyRowStart = clampRow(bodyRowStart);

  return {
    mode,
    titleColStart,
    titleColSpan: mode.titleColSpan,
    titleRowStart,
    titleRowSpan: mode.titleRowSpan,
    titleSize: mode.titleSize,
    titleAlign: mode.titleAlign,
    bodyColStart: clampCol(mode.bodyColStart, mode.bodyColSpan),
    bodyColSpan: mode.bodyColSpan,
    bodyRowStart,
    metaCorner: mode.metaCorner,
    accentTarget: mode.accentTarget,
  };
}
