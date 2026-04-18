/** Site origin for canonical URLs and SEO (matches `site` in astro.config). */
export const SITE_ORIGIN = 'https://daipan.ink';

/** First `maxWords` words, plus "..." when truncated (for poem meta descriptions). */
export function excerptWords(text: string, maxWords: number): string {
  const t = text.replace(/\s+/g, ' ').trim();
  if (!t) return '';
  const words = t.split(' ').filter(Boolean);
  if (words.length <= maxWords) return words.join(' ');
  return `${words.slice(0, maxWords).join(' ')}...`;
}

/** Truncate to `maxChars` with an ellipsis when needed (collection world blurbs). */
export function truncateChars(text: string, maxChars: number): string {
  const t = text.replace(/\s+/g, ' ').trim();
  if (t.length <= maxChars) return t;
  return `${t.slice(0, Math.max(0, maxChars - 1)).trimEnd()}…`;
}
