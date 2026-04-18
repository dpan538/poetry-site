export type PRNG = () => number;

function mulberry32(seed: number): PRNG {
  let s = seed >>> 0;
  return function () {
    s += 0x6d2b79f5;
    let z = s;
    z = Math.imul(z ^ (z >>> 15), z | 1);
    z ^= z + Math.imul(z ^ (z >>> 7), z | 61);
    return ((z ^ (z >>> 14)) >>> 0) / 0x100000000;
  };
}

function djb2(str: string): number {
  let hash = 5381;
  for (let i = 0; i < str.length; i++) {
    hash = ((hash << 5) + hash) ^ str.charCodeAt(i);
    hash |= 0;
  }
  return hash >>> 0;
}

const worldCities: Record<string, string> = {
  'their-world': 'America/New_York',
  'bless-you': 'Asia/Shanghai',
  'still-life': 'Australia/Brisbane',
};

function getWorldTimeFraction(worldId: string): number {
  const tz = worldCities[worldId] ?? 'UTC';
  try {
    const now = new Date();
    const timeStr = now.toLocaleTimeString('en-US', {
      timeZone: tz,
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    });
    const [h, m] = timeStr.split(':').map(Number);
    return (h * 60 + m) / 1440;
  } catch {
    return 0;
  }
}

function getLocalDateString(): string {
  const now = new Date();
  const y = now.getFullYear();
  const mo = String(now.getMonth() + 1).padStart(2, '0');
  const d = String(now.getDate()).padStart(2, '0');
  return `${y}-${mo}-${d}`;
}

export interface SeedInputs {
  worldId: string;
  poemId: string;
  lineCount: number;
  wordCount: number;
  charCount: number;
  stanzaCount: number;
}

const LAYOUT_TIME_KEY_PREFIX = 'tw-layout-time:';

/** Freeze world time slice for 15–30 min per poem so layout does not flicker on every navigation. */
function getCachedWorldTimeInt(worldId: string, poemId: string): number {
  const fresh = () => Math.floor(getWorldTimeFraction(worldId) * 10000);
  if (typeof localStorage === 'undefined') {
    return fresh();
  }
  const key = `${LAYOUT_TIME_KEY_PREFIX}${worldId}:${poemId}`;
  const now = Date.now();
  try {
    const raw = localStorage.getItem(key);
    if (raw) {
      const parsed = JSON.parse(raw) as { v: unknown; exp: unknown };
      const v = typeof parsed.v === 'number' ? parsed.v : null;
      const exp = typeof parsed.exp === 'number' ? parsed.exp : null;
      if (v !== null && exp !== null && now < exp) {
        return v;
      }
    }
  } catch {
    /* ignore */
  }
  const v = fresh();
  const ttlMs = (15 + Math.random() * 15) * 60 * 1000;
  try {
    localStorage.setItem(key, JSON.stringify({ v, exp: now + ttlMs }));
  } catch {
    /* ignore */
  }
  return v;
}

export function getSeed(inputs: SeedInputs): PRNG {
  const worldTimeInt = getCachedWorldTimeInt(inputs.worldId, inputs.poemId);

  const components = [
    inputs.worldId,
    inputs.poemId,
    getLocalDateString(),
    String(inputs.lineCount),
    String(inputs.wordCount),
    String(inputs.charCount),
    String(inputs.stanzaCount),
    String(worldTimeInt),
  ];

  const combined = components.join('|');
  const seed = djb2(combined);
  return mulberry32(seed);
}
