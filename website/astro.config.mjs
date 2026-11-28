// The Mortise documentation site. Deployed to GitHub Pages by .github/workflows/docs.yml.
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { readdir, readFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { downloads } from './src/downloads.mjs';

// Screen.astro can reach every snapshot of the image tests, so the build copies all of them.
// The pages use only the resized copies, so this removes the original PNGs no page names.
const dropUnusedSnapshots = {
  name: 'drop-unused-snapshots',
  hooks: {
    'astro:build:done': async ({ dir }) => {
      const root = fileURLToPath(dir);
      const files = await readdir(root, { recursive: true });
      const pages = await Promise.all(
        files.filter((f) => f.endsWith('.html')).map((f) => readFile(join(root, f), 'utf8')),
      );
      const used = pages.join('\n');
      const assets = await readdir(join(root, '_astro'));
      for (const name of assets.filter((n) => /^s\d[^/]*\.png$/.test(n) && !used.includes(n))) {
        await rm(join(root, '_astro', name));
      }
    },
  },
};

export default defineConfig({
  site: 'https://mortise-dev.github.io',
  base: '/mortise',
  redirects: {
    // Pages that moved keep their old address.
    '/how-to/install': '/mortise/start-here/installation/',
    // Short links to share: /mortise/download/macos and the like serve the latest installer.
    '/download': '/mortise/start-here/installation/',
    ...Object.fromEntries(downloads.map((d) => [`/download/${d.slug}`, d.url])),
  },
  // The screenshots, the fonts and the logo come from outside this folder.
  vite: { server: { fs: { allow: ['..'] } } },
  integrations: [
    dropUnusedSnapshots,
    starlight({
      title: 'Mortise',
      description: 'A desktop app that shows what is inside the modules of a Laravel app.',
      logo: { src: '../assets/icon/mortise.svg' },
      favicon: '/favicon.svg',
      social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/mortise-dev/mortise' }],
      editLink: { baseUrl: 'https://github.com/mortise-dev/mortise/edit/main/website/' },
      customCss: ['./src/styles/mortise.css'],
      expressiveCode: {
        themes: ['github-dark-default', 'github-light-default'],
        styleOverrides: {
          borderRadius: '6px',
          codeFontFamily: 'Inconsolata, ui-monospace, Menlo, Consolas, monospace',
          codeFontSize: '0.9rem',
          uiFontFamily: 'Asap, system-ui, sans-serif',
        },
      },
      sidebar: [
        { label: 'Start here', items: [{ autogenerate: { directory: 'start-here' } }] },
        { label: 'Tutorial', items: [{ autogenerate: { directory: 'tutorial' } }] },
        { label: 'How-to guides', items: [{ autogenerate: { directory: 'how-to' } }] },
        { label: 'Reference', items: [{ autogenerate: { directory: 'reference' } }] },
        { label: 'Explanation', items: [{ autogenerate: { directory: 'explanation' } }] },
      ],
    }),
  ],
});
