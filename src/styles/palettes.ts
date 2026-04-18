export type WorldId = 'their-world' | 'bless-you' | 'still-life';

export type Palette = {
  background: string;
  primary: string;
  accent: string;
  meta: string;
};

export const palettes: Record<WorldId, Palette> = {
  'their-world': {
    background: '#F5F2ED',
    primary: '#1A1A1A',
    accent: '#C0392B',
    meta: '#1A1A1A',
  },
  'bless-you': {
    background: '#F2EDE4',
    primary: '#2C1810',
    accent: '#9B7B8B',
    meta: '#1A1A1A',
  },
  'still-life': {
    background: '#F8F7F4',
    primary: '#2A2A2A',
    accent: '#8A9E8A',
    meta: '#1A1A1A',
  },
};

export function applyPalette(world: string, element: HTMLElement): void {
  const palette = palettes[world as WorldId];
  if (!palette) return;

  element.style.setProperty('--color-bg', palette.background);
  element.style.setProperty('--color-primary', palette.primary);
  element.style.setProperty('--color-accent', palette.accent);
  element.style.setProperty('--color-meta', palette.meta);
}
