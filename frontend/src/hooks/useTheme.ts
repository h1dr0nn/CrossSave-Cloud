import { useEffect, useState } from 'react';
import { DESIGN_TOKENS, ThemeMode } from '../utils/designTokens';

const prefersDark = () =>
  typeof window !== 'undefined' &&
  window.matchMedia &&
  window.matchMedia('(prefers-color-scheme: dark)').matches;

export function useTheme(initialMode?: ThemeMode) {
  const [mode, setMode] = useState<ThemeMode>(initialMode ?? (prefersDark() ? 'dark' : 'light'));

  useEffect(() => {
    const root = document.documentElement;
    root.setAttribute('data-theme', mode);
    root.style.setProperty('--background', mode === 'dark' ? DESIGN_TOKENS.backgroundDark : DESIGN_TOKENS.backgroundLight);
    root.style.setProperty('--surface', mode === 'dark' ? 'rgba(44, 44, 46, 0.7)' : 'rgba(255, 255, 255, 0.7)');
    root.style.setProperty('--foreground', mode === 'dark' ? '#F5F5F7' : '#1C1C1E');
    root.style.setProperty('--muted', mode === 'dark' ? '#8E8E93' : '#6B7280');
    root.style.setProperty('--border', mode === 'dark' ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.06)');
    root.style.setProperty('--accent', DESIGN_TOKENS.accent);
  }, [mode]);

  return {
    mode,
    toggle: () => setMode((prev) => (prev === 'dark' ? 'light' : 'dark')),
    setMode
  } as const;
}
