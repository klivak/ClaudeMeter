/**
 * Design tokens taken off the shipped ClaudeMeter UI (screenshots/, Catppuccin
 * Mocha dark palette) — not from a design doc. Every colour below appears in
 * screenshots/dashboard-dark-v5.1.png or screenshots/theme-*.png.
 */
export const COLOR = {
  // stage (the only thing allowed to differ from the app)
  bg: '#07070d',
  bgSoft: '#0d0d17',
  // app surfaces
  surface: '#1e1e2e',
  surfaceAlt: '#282840',
  surfaceHi: '#31314c',
  line: '#3a3a55',
  // type
  ink: '#eef0fa',
  inkDim: '#b4bad6',
  muted: '#7b82a4',
  // status / accents (straight from the progress bars and badges)
  accent: '#00d6b8',
  accent2: '#a970ff',
  green: '#3fd97f',
  amber: '#f5b942',
  red: '#f5476b',
  codex: '#2ad4c0',
  claude: '#d97757',
  // theme swatches
  sunset: '#f08a6a',
  midnight: '#5b8def',
} as const;

export const FONT = {
  sans: 'Inter',
  mono: 'JetBrains Mono',
} as const;

export const BRAND = {
  name: 'ClaudeMeter',
  tagline: 'Real-time Claude usage, right in your tray',
  studio: 'by klivak',
  repo: 'github.com/klivak/claudemeter',
  site: 'klivak.github.io/claude-meter',
  version: '5.8.0',
  languages: 40,
  ram: '~2 MB',
  size: '~3 MB',
  themes: 4,
} as const;

/** Real pixel sizes of the shipped screenshots — never guess an aspect ratio. */
export const SCREENS = {
  dark: { src: 'screens/01-dashboard-dark.png', w: 377, h: 677 },
  light: { src: 'screens/02-dashboard-light.png', w: 381, h: 680 },
  sunset: { src: 'screens/03-theme-sunset.png', w: 381, h: 680 },
  midnight: { src: 'screens/04-theme-midnight.png', w: 377, h: 676 },
  settings: { src: 'screens/05-settings.png', w: 380, h: 742 },
  tooltip: { src: 'screens/06-tooltip.png', w: 246, h: 274 },
  notification: { src: 'screens/07-notification.png', w: 364, h: 127 },
  taskManager: { src: 'screens/08-task-manager.png', w: 888, h: 398 },
} as const;

export type ScreenKey = keyof typeof SCREENS;

const build = <T extends Record<string, number>>(durations: T) => {
  let from = 0;
  const out = {} as Record<keyof T, { from: number; duration: number }>;
  for (const [name, duration] of Object.entries(durations) as [keyof T, number][]) {
    out[name] = { from, duration };
    from += duration;
  }
  return { scene: out, total: from };
};

/** 16:9 long cut — 1680 frames @ 30 fps = 56 s. */
const DURATIONS = {
  hook: 150,
  brand: 120,
  tray: 180,
  dashboard: 240,
  examples: 390,
  settings: 150,
  languages: 150,
  stats: 150,
  cta: 150,
} as const;

const long = build(DURATIONS);
export const SCENE = long.scene;
export const TOTAL_FRAMES = long.total;

/** 9:16 Shorts cut — 825 frames @ 30 fps = 27.5 s. Fewer scenes, faster beats. */
const SHORTS_DURATIONS = {
  hook: 105,
  brand: 75,
  tray: 120,
  dashboard: 135,
  examples: 180,
  stats: 90,
  cta: 120,
} as const;

const short = build(SHORTS_DURATIONS);
export const SHORTS_SCENE = short.scene;
export const SHORTS_TOTAL_FRAMES = short.total;

export const FPS = 30;

/** Set to true once a music track exists at public/audio/music.mp3 (see MUSIC_PROMPT.md). */
export const WITH_MUSIC = false;
export const MUSIC_START_SECONDS = 0;
