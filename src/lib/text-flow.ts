// Runs in browser only — measures text and distributes lines across frames

export interface FrameDimensions {
  width: number; // px
  height: number; // px
}

export interface FlowResult {
  frames: string[][]; // lines assigned to each frame
  overflow: string[]; // lines that didn't fit anywhere
}

export function flowLinesIntoFrames(
  lines: string[],
  frameDimensions: FrameDimensions[],
  fontSize: number,
  lineHeight: number,
): FlowResult {
  const frames: string[][] = frameDimensions.map(() => []);
  let lineIndex = 0;

  for (let fi = 0; fi < frameDimensions.length; fi++) {
    const { height } = frameDimensions[fi];
    const maxLines = Math.floor(height / (fontSize * lineHeight));
    let linesInFrame = 0;

    while (lineIndex < lines.length && linesInFrame < maxLines) {
      const line = lines[lineIndex];
      // Empty line = stanza break — still counts as a line
      frames[fi].push(line);
      linesInFrame++;
      lineIndex++;
    }
  }

  return {
    frames,
    overflow: lines.slice(lineIndex),
  };
}

export function measureFrames(frameElements: HTMLElement[]): FrameDimensions[] {
  return frameElements.map((el) => ({
    width: el.clientWidth,
    height: el.clientHeight,
  }));
}
