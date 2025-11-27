import type { Config } from 'tailwindcss';

const config: Config = {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        background: 'var(--background)',
        surface: 'var(--surface)',
        foreground: 'var(--foreground)',
        accent: 'var(--accent)',
        muted: 'var(--muted)',
        border: 'var(--border)'
      },
      borderRadius: {
        soft: '16px'
      },
      transitionDuration: {
        fast: '180ms',
        ui: '220ms'
      },
      boxShadow: {
        'soft-glow': '0 20px 40px rgba(0, 0, 0, 0.12)',
        'inner-glow': 'inset 0 1px 0 rgba(255,255,255,0.08)'
      },
      backdropBlur: {
        glass: '32px'
      },
      animation: {
        fadeIn: 'fadeIn 180ms ease-out'
      },
      keyframes: {
        fadeIn: {
          from: { opacity: '0' },
          to: { opacity: '1' }
        }
      }
    }
  },
  plugins: []
};

export default config;
