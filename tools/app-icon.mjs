#!/usr/bin/env node
// Makes the app icon files of every platform from the two drawings in assets/icon:
//
//   mortise.svg     the icon, for 24 px and up
//   mortise-16.svg  the icon drawn in whole pixels, for 16 px
//
// Run from the root of the repository after a drawing changed:
//
//   node tools/app-icon.mjs
//
// It writes:
//
//   assets/icon/macos/Mortise.icns          the bundle icon of macOS
//   assets/icon/macos/mortise-512.png       the Dock icon the app sets when it runs
//   assets/icon/windows/mortise.ico         the icon of the .exe and the taskbar
//   assets/icon/linux/hicolor/NxN/apps/mortise.png
//                                           the hicolor theme sizes; mortise.svg is
//                                           the scalable one
//
// It needs macOS (`iconutil` packs the .icns) and Google Chrome, which draws the SVG.
// Node 22 or later, no packages.

import { spawn, execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync, readFileSync, rmSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const ICON = resolve('assets/icon');
const MASTER = join(ICON, 'mortise.svg');
const SMALL = join(ICON, 'mortise-16.svg');
const CHROME = process.env.CHROME || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';

// The block of mortise.svg is 928 px of 1024. On macOS the shape is 824 px of 1024,
// the grid Apple gives for app icons, so the drawing is scaled down to that.
const MAC = 824 / 928;

const work = mkdtempSync(join(tmpdir(), 'mortise-icon-'));
const port = 9333 + (process.pid % 500);
const chrome = spawn(CHROME, ['--headless=new', '--remote-debugging-port=' + port, '--user-data-dir=' + join(work, 'profile'), '--hide-scrollbars', '--no-first-run', '--disable-gpu', 'about:blank'], { stdio: 'ignore' });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

let ws;
let id = 0;
const pending = new Map();
async function connect() {
  for (let i = 0; i < 50 && !ws; i++) {
    try {
      const page = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((t) => t.type === 'page');
      ws = new WebSocket(page.webSocketDebuggerUrl);
    } catch {
      await sleep(200);
    }
  }
  if (!ws) throw new Error('Chrome did not start: ' + CHROME);
  await new Promise((r) => (ws.onopen = r));
  ws.onmessage = (e) => {
    const m = JSON.parse(e.data);
    if (m.id && pending.has(m.id)) {
      pending.get(m.id)(m);
      pending.delete(m.id);
    }
  };
}
async function send(method, params = {}) {
  const m = await new Promise((r) => {
    const i = ++id;
    pending.set(i, r);
    ws.send(JSON.stringify({ id: i, method, params }));
  });
  if (m.error) throw new Error(`${method}: ${m.error.message}`);
  return m.result;
}

// Draws `svg` `size` px wide, scaled by `scale` about the centre, and returns the PNG.
async function draw(svg, size, scale = 1) {
  const w = size * scale;
  const at = (size - w) / 2;
  const html = `<!doctype html><html><body style="margin:0;background:transparent">` +
    `<img src="file://${svg}" style="position:absolute;left:${at}px;top:${at}px;width:${w}px;height:${w}px"></body></html>`;
  const page = join(work, 'page.html');
  writeFileSync(page, html);
  await send('Page.navigate', { url: 'file://' + page });
  await send('Runtime.evaluate', { expression: 'document.images[0].decode().then(() => 1)', awaitPromise: true });
  const shot = await send('Page.captureScreenshot', { format: 'png', clip: { x: 0, y: 0, width: size, height: size, scale: 1 } });
  return Buffer.from(shot.data, 'base64');
}

// The drawing for a size: whole pixels at 16, the full drawing above.
const source = (size) => (size <= 16 ? SMALL : MASTER);

// An .ico of PNG images, which Windows reads since Vista.
function ico(images) {
  const head = Buffer.alloc(6 + 16 * images.length);
  head.writeUInt16LE(0, 0);
  head.writeUInt16LE(1, 2);
  head.writeUInt16LE(images.length, 4);
  let offset = head.length;
  images.forEach(({ size, png }, i) => {
    const e = 6 + 16 * i;
    head.writeUInt8(size >= 256 ? 0 : size, e);
    head.writeUInt8(size >= 256 ? 0 : size, e + 1);
    head.writeUInt8(0, e + 2);
    head.writeUInt8(0, e + 3);
    head.writeUInt16LE(1, e + 4);
    head.writeUInt16LE(32, e + 6);
    head.writeUInt32LE(png.length, e + 8);
    head.writeUInt32LE(offset, e + 12);
    offset += png.length;
  });
  return Buffer.concat([head, ...images.map((i) => i.png)]);
}

try {
  await connect();
  await send('Page.enable');
  await send('Emulation.setDeviceMetricsOverride', { width: 1024, height: 1024, deviceScaleFactor: 1, mobile: false });
  await send('Emulation.setDefaultBackgroundColorOverride', { color: { r: 0, g: 0, b: 0, a: 0 } });

  // macOS: an .iconset folder, packed by iconutil.
  const iconset = join(work, 'Mortise.iconset');
  mkdirSync(iconset);
  for (const points of [16, 32, 128, 256, 512]) {
    for (const factor of [1, 2]) {
      const size = points * factor;
      const name = `icon_${points}x${points}${factor === 2 ? '@2x' : ''}.png`;
      // The 16 px drawing has its own margin and is used as it is.
      writeFileSync(join(iconset, name), await draw(source(size), size, size <= 16 ? 1 : MAC));
    }
  }
  mkdirSync(join(ICON, 'macos'), { recursive: true });
  execFileSync('iconutil', ['-c', 'icns', '-o', join(ICON, 'macos', 'Mortise.icns'), iconset]);
  writeFileSync(join(ICON, 'macos', 'mortise-512.png'), readFileSync(join(iconset, 'icon_512x512.png')));

  // Windows.
  const windows = [];
  for (const size of [16, 24, 32, 48, 64, 128, 256]) windows.push({ size, png: await draw(source(size), size) });
  mkdirSync(join(ICON, 'windows'), { recursive: true });
  writeFileSync(join(ICON, 'windows', 'mortise.ico'), ico(windows));

  // Linux: the hicolor theme.
  rmSync(join(ICON, 'linux'), { recursive: true, force: true });
  for (const size of [16, 24, 32, 48, 64, 128, 256, 512]) {
    const dir = join(ICON, 'linux', 'hicolor', `${size}x${size}`, 'apps');
    mkdirSync(dir, { recursive: true });
    writeFileSync(join(dir, 'mortise.png'), await draw(source(size), size));
  }
} finally {
  chrome.kill();
  await sleep(300);
  rmSync(work, { recursive: true, force: true });
}
