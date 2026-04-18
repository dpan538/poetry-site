// @ts-check
import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';

import tailwindcss from '@tailwindcss/vite';

// https://astro.build/config
export default defineConfig({
  site: 'https://daipan.ink',
  integrations: [sitemap()],
  /**
   * Dev Toolbar loads extra client chunks (audit, toolbar, …). When Vite
   * re-optimizes deps, the browser can still request old `?v=` URLs →
   * `net::ERR_ABORTED 504 (Outdated Optimize Dep)` and failed dynamic imports.
   * Disabling the toolbar removes those requests in dev.
   *
   * Per-user toggle instead of project-wide: `npx astro preferences disable devToolbar`
   */
  devToolbar: {
    enabled: false,
  },
  vite: {
    plugins: [tailwindcss()],
  },
});