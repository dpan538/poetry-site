import { getSeed } from './seed';
import { selectFrameLayout } from './frame-layouts';
import { analyzePoemText } from './poem-analyzer';

const analysisBreakfast = analyzePoemText('hello\n\nworld\nextra', 'Breakfast');
const inputsBreakfast = {
  worldId: 'their-world',
  poemId: 'breakfast',
  lineCount: analysisBreakfast.metrics.lineCount,
  wordCount: analysisBreakfast.metrics.wordCount,
  charCount: analysisBreakfast.metrics.charCount,
  stanzaCount: analysisBreakfast.metrics.stanzaCount,
};

const rand1 = getSeed(inputsBreakfast);
const layout1 = selectFrameLayout(analysisBreakfast.type, rand1);
console.log('Frame layout breakfast sample:', layout1.id);

const rand2 = getSeed(inputsBreakfast);
const layout2 = selectFrameLayout(analysisBreakfast.type, rand2);
console.log('Same seed, same frame layout:', layout1.id === layout2.id);

const analysisShadow = analyzePoemText('one two three\nfour', 'Shadow');
const inputsShadow = {
  worldId: 'their-world',
  poemId: 'shadow',
  lineCount: analysisShadow.metrics.lineCount,
  wordCount: analysisShadow.metrics.wordCount,
  charCount: analysisShadow.metrics.charCount,
  stanzaCount: analysisShadow.metrics.stanzaCount,
};
const rand3 = getSeed(inputsShadow);
const layout3 = selectFrameLayout(analysisShadow.type, rand3);
console.log('Frame layout shadow sample:', layout3.id);
console.log('Different poem, may differ:', layout1.id !== layout3.id);
