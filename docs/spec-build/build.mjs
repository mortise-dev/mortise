import fs from 'node:fs';
import path from 'node:path';
import { page } from './lib.mjs';
// The root comes from this file's own path. Pages go to docs/spec-build/out (ignored by git),
// never to docs/spec, which build.py writes.
const ROOT = fs.realpathSync(new URL('../..', import.meta.url));
const OUT = path.join(ROOT, 'docs', 'spec-build', 'out');
// A link at OUT could point anywhere, docs/spec included: refuse it. ROOT is resolved and OUT is not a
// link, so every write below stays inside the repository.
if (fs.lstatSync(OUT, { throwIfNoEntry: false })?.isSymbolicLink()) {
  console.error(`Write inside the repository: ${OUT} is a link.`);
  process.exit(1);
}
fs.mkdirSync(OUT, { recursive: true });
const only = process.argv[2];
const dirs = {};
for (const f of ['f1', 'f2', 'f3']) { if (fs.existsSync(new URL(`./${f}.mjs`, import.meta.url))) dirs[f] = await import(`./${f}.mjs`); }
const titles = { f1: 'F1 Site plan: Laravel Modules companion', f2: 'F2 Crosstab: Laravel Modules companion', f3: 'F3 Logbook: Laravel Modules companion' };
// Write one page to a new file beside it, then rename it over the page. The rename replaces a link
// (symbolic or hard) instead of writing through it, so no file outside OUT changes.
function write(name, content) {
  const temp = fs.mkdtempSync(path.join(OUT, '.page-'));
  try {
    fs.writeFileSync(path.join(temp, name), content);
    fs.renameSync(path.join(temp, name), path.join(OUT, name));
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
}
for (const [f, d] of Object.entries(dirs)) {
  if (only && only !== f) continue;
  write(`${f}.html`, page({ file: `${f}.html`, title: titles[f], fonts: d.fonts, css: d.css, body: d.body() }));
}
if (fs.existsSync(new URL('./index.mjs', import.meta.url)) && (!only || only === 'index')) {
  const ix = await import('./index.mjs');
  write('index.html', page({ file: 'index.html', title: 'Laravel Modules companion: three design directions', fonts: Object.values(dirs).flatMap((d) => d.fonts), css: Object.values(dirs).map((d) => d.css).join('\n'), body: ix.body(dirs) }));
}
for (const f of fs.readdirSync(OUT)) console.log(f, fs.statSync(path.join(OUT, f)).size);
