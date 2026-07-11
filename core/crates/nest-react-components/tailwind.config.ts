import type { Config } from 'tailwindcss';
import nestPreset from './nest-tailwind-preset.json';

export default {
  content: ['./src/**/*.{ts,tsx}'],
  presets: [nestPreset as Config],
  theme: {
    extend: {
      fontFamily: {
        body: ['Inter', 'system-ui', 'sans-serif'],
        heading: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
    },
  },
  plugins: [],
} satisfies Config;
