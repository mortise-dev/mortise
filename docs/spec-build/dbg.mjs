// usage: node dbg.mjs f1 home,detail light|dark
import fs from 'node:fs';
import { execFileSync } from 'node:child_process';
import { DOC_CSS } from './lib.mjs';
const [dir, screens, theme = 'light', w = '1200', h = '740'] = process.argv.slice(2);
const d = await import(`./${dir}.mjs`);
fs.mkdirSync('shots', { recursive: true });
for (const s of screens.split(',')) {
  const html = `<!doctype html><html><head><meta charset="utf-8">${d.fonts.map((f) => `<link rel="stylesheet" href="${f}">`).join('')}<style>${DOC_CSS}${d.css}body{padding:8px}</style></head><body data-mock="${theme}">${d[s]('t-auto')}</body></html>`;
  const f = `${process.cwd()}/shots/${dir}-${s}-${theme}.html`;
  fs.writeFileSync(f, html);
  execFileSync('/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', ['--headless=new', '--disable-gpu', '--hide-scrollbars', `--window-size=${w},${h}`, '--virtual-time-budget=5000', `--screenshot=${f.replace('.html', '.png')}`, `file://${f}`], { stdio: 'ignore' });
  console.log(f.replace('.html', '.png'));
}
