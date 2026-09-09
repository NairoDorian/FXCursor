export interface ThemeAccent {
  id: string;
  name: string;
  primary: string;
  glow: string;
  subtle: string;
}

export const THEME_ACCENTS: ThemeAccent[] = [
  {
    id: 'cyan',
    name: 'Master Cyan',
    primary: '#00f2fe',
    glow: 'rgba(0, 242, 254, 0.35)',
    subtle: 'rgba(0, 242, 254, 0.12)',
  },
  {
    id: 'violet',
    name: 'Cyberpunk Violet',
    primary: '#a855f7',
    glow: 'rgba(168, 85, 247, 0.35)',
    subtle: 'rgba(168, 85, 247, 0.12)',
  },
  {
    id: 'solar',
    name: 'Solar Flare',
    primary: '#f97316',
    glow: 'rgba(249, 115, 22, 0.35)',
    subtle: 'rgba(249, 115, 22, 0.12)',
  },
  {
    id: 'emerald',
    name: 'Emerald Matrix',
    primary: '#10b981',
    glow: 'rgba(16, 185, 129, 0.35)',
    subtle: 'rgba(16, 185, 129, 0.12)',
  },
  {
    id: 'razor',
    name: 'Razor Silver',
    primary: '#ffffff',
    glow: 'rgba(255, 255, 255, 0.35)',
    subtle: 'rgba(255, 255, 255, 0.12)',
  },
];

export function applyThemeAccent(accentId: string) {
  const accent = THEME_ACCENTS.find((a) => a.id === accentId) || THEME_ACCENTS[0];
  const root = document.documentElement;
  root.style.setProperty('--accent-primary', accent.primary);
  root.style.setProperty('--accent-glow', accent.glow);
  root.style.setProperty('--accent-subtle', accent.subtle);
}
