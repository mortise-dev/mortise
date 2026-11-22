import fs from 'node:fs';
import path from 'node:path';
import { page } from './lib.mjs';
const OUT = '/Users/nicolaswidart/Sites/php/packages/LaravelModules/docs/companion-app/design-fable';
const only = process.argv[2];
const dirs = {};
for (const f of ['f1', 'f2', 'f3']) { if (fs.existsSync(new URL(`./${f}.mjs`, import.meta.url))) dirs[f] = await import(`./${f}.mjs`); }
const titles = { f1: 'F1 Site plan: Laravel Modules companion', f2: 'F2 Crosstab: Laravel Modules companion', f3: 'F3 Logbook: Laravel Modules companion' };
for (const [f, d] of Object.entries(dirs)) {
  if (only && only !== f) continue;
  fs.writeFileSync(path.join(OUT, `${f}.html`), page({ file: `${f}.html`, title: titles[f], fonts: d.fonts, css: d.css, body: d.body() }));
}
if (fs.existsSync(new URL('./index.mjs', import.meta.url)) && (!only || only === 'index')) {
  const ix = await import('./index.mjs');
  fs.writeFileSync(path.join(OUT, 'index.html'), page({ file: 'index.html', title: 'Laravel Modules companion: three design directions', fonts: Object.values(dirs).flatMap((d) => d.fonts), css: Object.values(dirs).map((d) => d.css).join('\n'), body: ix.body(dirs) }));
}
for (const f of fs.readdirSync(OUT)) console.log(f, fs.statSync(path.join(OUT, f)).size);
