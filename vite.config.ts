import { defineConfig } from 'vite';
import pkg from './package.json' with { type: 'json' };

export default defineConfig({
  root: 'web',
  // The game's version (rule: docs/DECISIONS.md, "Version numbers"); read in TS as `VERSION` (ui/patch-notes.ts).
  define: { __APP_VERSION__: JSON.stringify(pkg.version) },
  // Relative asset URLs, so the build works from any sub-path (e.g. GitHub Pages' /<repo>/).
  base: './',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    target: 'es2022',
  },
  server: {
    port: 5173,
    strictPort: false,
  },
});
