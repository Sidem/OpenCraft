import { defineConfig } from 'vite';

export default defineConfig({
  root: 'web',
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
