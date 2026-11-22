// Render pages in headless Chrome at 1280 px, run the audit, save tiles. Usage: node shot.mjs <outdir> <file.html[#dark]>...
import { spawn } from 'node:child_process';
import { mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, basename } from 'node:path';
const [outdir, ...targets] = process.argv.slice(2);
mkdirSync(outdir, { recursive: true });
const profile = join(tmpdir(), 'mortise-chrome-' + process.pid);
const port = 9333 + (process.pid % 500);
const chrome = spawn('/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', ['--headless=new', '--remote-debugging-port=' + port, '--user-data-dir=' + profile, '--hide-scrollbars', '--no-first-run', '--disable-gpu', 'about:blank'], { stdio: 'ignore' });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let ws, id = 0; const pending = new Map();
async function connect() {
  for (let i = 0; i < 50; i++) {
    try { const r = await fetch(`http://127.0.0.1:${port}/json`); const t = (await r.json()).find((x) => x.type === 'page'); ws = new WebSocket(t.webSocketDebuggerUrl); break; } catch { await sleep(200); }
  }
  await new Promise((r) => (ws.onopen = r));
  ws.onmessage = (e) => { const m = JSON.parse(e.data); if (m.id && pending.has(m.id)) { pending.get(m.id)(m.result || m); pending.delete(m.id); } };
}
const send = (method, params = {}) => new Promise((r) => { const i = ++id; pending.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const AUDIT = `(() => {
  const lum = (c) => { const v = c.map((x) => { x /= 255; return x <= 0.03928 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4); }); return 0.2126 * v[0] + 0.7152 * v[1] + 0.0722 * v[2]; };
  const parse = (s) => { const m = s.match(/rgba?\\(([^)]+)\\)/); if (!m) return null; const p = m[1].split(',').map(Number); return { c: p.slice(0, 3), a: p.length > 3 ? p[3] : 1 }; };
  const bgOf = (el) => { let e = el; while (e) { const b = parse(getComputedStyle(e).backgroundColor); if (b && b.a > 0.5) return { c: b.c, el: e }; e = e.parentElement; } return { c: [0, 0, 0], el: null }; };
  const out = { contrast: [], ink3: [], clip: [], overflow: [], count: 0 };
  const seen = new Set();
  document.querySelectorAll('.m').forEach((root) => {
    const theme = root.getAttribute('data-theme');
    const cs = getComputedStyle(root);
    const tok = (n) => parse((() => { const d = document.createElement('i'); d.style.color = 'var(--' + n + ')'; root.appendChild(d); const v = getComputedStyle(d).color; d.remove(); return v; })());
    const ink3 = tok('ink-3').c.join(), frame = tok('frame').c.join(), well = tok('well').c.join();
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
    let n;
    while ((n = walker.nextNode())) {
      if (!n.nodeValue.trim()) continue;
      const el = n.parentElement; if (el.closest('svg')) continue;
      const st = getComputedStyle(el); if (st.visibility === 'hidden' || st.display === 'none') continue;
      const r = el.getBoundingClientRect(); if (!r.width || !r.height) continue;
      out.count++;
      const fg = parse(st.color); const bg = bgOf(el);
      const ratio = (Math.max(lum(fg.c), lum(bg.c)) + 0.05) / (Math.min(lum(fg.c), lum(bg.c)) + 0.05);
      const key = theme + '|' + fg.c.join() + '|' + bg.c.join();
      if (ratio < 4.5 && !seen.has(key)) { seen.add(key); out.contrast.push({ theme, fg: fg.c.join(), bg: bg.c.join(), ratio: ratio.toFixed(2), text: n.nodeValue.trim().slice(0, 40), cls: el.className }); }
      if (fg.c.join() === ink3 && (bg.c.join() === frame || bg.c.join() === well)) out.ink3.push({ theme, on: bg.c.join() === frame ? 'frame' : 'well', text: n.nodeValue.trim().slice(0, 40) });
      let a = el, hit = null;
      for (let d = 0; d < 4 && a && a !== root; d++, a = a.parentElement) { const s2 = getComputedStyle(a); if (s2.overflow !== 'visible' && s2.overflowX !== 'visible' && a.scrollWidth > a.clientWidth + 1 && a.clientWidth > 0 && !a.classList.contains('mwrap')) { hit = a; break; } }
      if (hit) out.clip.push({ text: n.nodeValue.trim().slice(0, 50), cls: hit.className, sw: hit.scrollWidth, cw: hit.clientWidth });
    }
    const w = root.querySelector('.win');
    if (w) { const wr = w.getBoundingClientRect(); w.querySelectorAll('*').forEach((e) => { if (e.closest('svg') && e.tagName !== 'svg') return; const r = e.getBoundingClientRect(); if (r.width && (r.right > wr.right + 1 || r.left < wr.left - 1) && !e.closest('.tip')) out.overflow.push({ cls: e.className && e.className.baseVal !== undefined ? e.className.baseVal : e.className, right: Math.round(r.right - wr.right), text: (e.textContent || '').trim().slice(0, 30) }); }); }
  });
  out.docOverflow = document.documentElement.scrollWidth - document.documentElement.clientWidth;
  out.clip = out.clip.slice(0, 40); out.overflow = out.overflow.slice(0, 25);
  return JSON.stringify(out);
})()`;
await connect();
await send('Page.enable'); await send('Runtime.enable');
const report = {};
for (const t of targets) {
  const [file, hash] = t.split('#');
  const name = basename(file, '.html') + (hash ? '-' + hash : '-light');
  await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 900, deviceScaleFactor: 1, mobile: false });
  await send('Page.navigate', { url: 'about:blank' }); await sleep(100);
  await send('Page.navigate', { url: 'file://' + file + (hash ? '#' + hash : '') });
  await sleep(900);
  await send('Runtime.evaluate', { expression: 'document.fonts.ready.then(()=>1)', awaitPromise: true });
  await sleep(300);
  const fonts = await send('Runtime.evaluate', { expression: "JSON.stringify([document.fonts.check('12px Asap'), document.fonts.check('12px Inconsolata')])", returnByValue: true });
  const a = await send('Runtime.evaluate', { expression: AUDIT, returnByValue: true });
  const h = (await send('Runtime.evaluate', { expression: 'document.documentElement.scrollHeight', returnByValue: true })).result.value;
  report[name] = { height: h, fonts: fonts.result.value, audit: JSON.parse(a.result.value) };
  let y0 = 0, y1 = h;
  if (process.env.FROM) y0 = (await send('Runtime.evaluate', { expression: `Math.max(0, Math.round(document.getElementById('${process.env.FROM}').getBoundingClientRect().top + scrollY - 10))`, returnByValue: true })).result.value;
  if (process.env.TO) y1 = (await send('Runtime.evaluate', { expression: `Math.round(document.getElementById('${process.env.TO}').getBoundingClientRect().top + scrollY - 10)`, returnByValue: true })).result.value;
  if (!process.env.NOSHOT) {
    const tile = Number(process.env.TILE || 1500);
    for (let y = y0, i = 0; y < y1; y += tile, i++) {
      const r = await send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true, clip: { x: 0, y, width: 1280, height: Math.min(tile, y1 - y), scale: 1 } });
      writeFileSync(join(outdir, `${name}-${String(i).padStart(2, '0')}.png`), Buffer.from(r.data, 'base64'));
    }
  }
}
writeFileSync(join(outdir, 'report.json'), JSON.stringify(report, null, 1));
for (const [k, v] of Object.entries(report)) console.log(k, 'h=' + v.height, 'fonts=' + v.fonts, 'texts=' + v.audit.count, 'lowContrast=' + v.audit.contrast.length, 'ink3=' + v.audit.ink3.length, 'clip=' + v.audit.clip.length, 'overflow=' + v.audit.overflow.length, 'docOverflow=' + v.audit.docOverflow);
ws.close(); chrome.kill(); await sleep(300); try { rmSync(profile, { recursive: true, force: true }); } catch {}
