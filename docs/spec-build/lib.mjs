// Shared helpers for the design-fable pages.

export const MODS = [
  { n: 'Billing', c: 'BI', files: 96, lines: 7840, tests: 58, cov: 81, uses: 1, usedBy: 1, state: 'good', word: 'Good', level: 1,
    k: { Models: 6, Controllers: 5, Requests: 6, Resources: 4, Policies: 2, Events: 3, Listeners: 2, Jobs: 3, Migrations: 7, Views: 0, Tests: 58 } },
  { n: 'Catalog', c: 'CA', files: 121, lines: 9105, tests: 74, cov: 77, uses: 0, usedBy: 2, state: 'good', word: 'Good', level: 0,
    k: { Models: 8, Controllers: 6, Requests: 7, Resources: 7, Policies: 2, Events: 2, Listeners: 1, Jobs: 2, Migrations: 12, Views: 0, Tests: 74 } },
  { n: 'Checkout', c: 'CH', files: 84, lines: 6412, tests: 31, cov: 54, uses: 4, usedBy: 2, state: 'bad', word: '2 problems', level: 2,
    k: { Models: 4, Controllers: 6, Requests: 7, Resources: 0, Policies: 2, Events: 5, Listeners: 5, Jobs: 3, Migrations: 9, Views: 12, Tests: 31 } },
  { n: 'Inventory', c: 'IN', files: 62, lines: 4380, tests: 40, cov: 72, uses: 1, usedBy: 0, state: 'good', word: 'Good', level: 1,
    k: { Models: 4, Controllers: 3, Requests: 3, Resources: 3, Policies: 1, Events: 2, Listeners: 1, Jobs: 1, Migrations: 4, Views: 0, Tests: 40 } },
  { n: 'Notifications', c: 'NO', files: 38, lines: 2190, tests: 12, cov: 39, uses: 2, usedBy: 0, state: 'warn', word: 'Low coverage', level: 3,
    k: { Models: 2, Controllers: 1, Requests: 1, Resources: 0, Policies: 0, Events: 0, Listeners: 6, Jobs: 4, Migrations: 2, Views: 10, Tests: 12 } },
  { n: 'Shipping', c: 'SH', files: 71, lines: 5265, tests: 29, cov: 66, uses: 1, usedBy: 1, state: 'good', word: 'Good', level: 2,
    k: { Models: 5, Controllers: 4, Requests: 5, Resources: 4, Policies: 2, Events: 4, Listeners: 3, Jobs: 6, Migrations: 8, Views: 1, Tests: 29 } },
  { n: 'Users', c: 'US', files: 79, lines: 6020, tests: 63, cov: 88, uses: 0, usedBy: 3, state: 'good', word: 'Good', level: 0,
    k: { Models: 3, Controllers: 3, Requests: 4, Resources: 2, Policies: 2, Events: 0, Listeners: 0, Jobs: 0, Migrations: 2, Views: 0, Tests: 63 } },
];
export const KINDS = ['Models', 'Controllers', 'Requests', 'Resources', 'Policies', 'Events', 'Listeners', 'Jobs', 'Migrations', 'Views', 'Tests'];
// row uses column, number of references
export const USES = {
  Billing: { Users: 7 },
  Catalog: {},
  Checkout: { Catalog: 14, Billing: 6, Users: 3, Shipping: 2 },
  Inventory: { Catalog: 9 },
  Notifications: { Checkout: 2, Users: 5 },
  Shipping: { Checkout: 4 },
  Users: {},
};
export const cmdHtml = (c) => c.split(' ').map((w) => `<span style="white-space:nowrap">${w}</span>`).join(' ');
export const mod = (n) => MODS.find((m) => m.n === n);
export const num = (v) => v.toLocaleString('en-US');

export const CMD = 'php artisan module:make-model Coupon Checkout --migration --policy --fillable=code,percent_off,expires_at';
export const FILES = [
  ['Modules/Checkout/app/Models/', 'Coupon.php'],
  ['Modules/Checkout/app/Policies/', 'CouponPolicy.php'],
  ['Modules/Checkout/database/migrations/', '2026_11_22_101544_create_coupons_table.php'],
];
export const WHY = {
  a: [
    ['app/Services/ShippingQuote.php', 18, 'Shipping\\Contracts\\RateCalculator', 'contract'],
    ['app/Http/Controllers/CheckoutController.php', 54, 'Shipping\\Contracts\\RateCalculator', 'contract'],
  ],
  b: [
    ['app/Listeners/CreateShipment.php', 12, 'Checkout\\Events\\OrderPlaced', 'event'],
    ['app/Listeners/NotifyCarrier.php', 9, 'Checkout\\Events\\OrderPlaced', 'event'],
    ['app/Jobs/BookCarrier.php', 22, 'Checkout\\Contracts\\OrderReader', 'contract'],
    ['app/Services/LabelPrinter.php', 40, 'Checkout\\Contracts\\OrderReader', 'contract'],
  ],
};
export const FAIL_OUT = [
  ['c', '$ php artisan module:migrate Checkout'],
  ['', ''],
  ['', '  Running Migration Checkout Module'],
  ['', ''],
  ['', '   INFO  Running migrations.'],
  ['', ''],
  ['', '  2026_11_20_091200_add_coupon_id_to_orders_table .... 11.62ms FAIL'],
  ['', ''],
  ['e', '   Illuminate\\Database\\QueryException'],
  ['', ''],
  ['e', '  SQLSTATE[42S21]: Column already exists: 1060 Duplicate column name'],
  ['e', "  'coupon_id' (Connection: mysql, SQL: alter table `orders` add"],
  ['e', '  `coupon_id` bigint unsigned null)'],
];

// ---------- colour maths ----------
const lin = (c) => { c /= 255; return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4; };
export const lum = (hex) => { const h = hex.replace('#', ''); const [r, g, b] = [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16)); return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b); };
export const contrast = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); };

// tokens: { name: [light, dark, role] }
export function themeCss(cls, tokens) {
  const block = (i) => Object.entries(tokens).map(([k, v]) => `--${k}:${v[i]}`).join(';');
  return `.${cls}.t-light,body:not([data-mock="dark"]) .${cls}.t-auto{${block(0)}}\n.${cls}.t-dark,body[data-mock="dark"] .${cls}.t-auto{${block(1)}}\n`;
}
export function tokenTable(tokens) {
  const rows = Object.entries(tokens).map(([k, v]) => `<tr><td><code>${k}</code></td><td>${v[2]}</td>` +
    [0, 1].map((i) => `<td><span class="chip" style="background:${v[i]}"></span><code>${v[i]}</code></td>`).join('') + '</tr>').join('');
  return `<div class="scroll"><table class="spec"><tr><th>Token</th><th>Role</th><th>Light</th><th>Dark</th></tr>${rows}</table></div>`;
}
// pairs: [fgToken, bgToken, min]
export function contrastTable(name, tokens, pairs) {
  let bad = 0;
  const rows = pairs.map(([f, b, min = 4.5]) => {
    const cells = [0, 1].map((i) => { const r = contrast(tokens[f][i], tokens[b][i]); if (r < min) { bad++; console.error(`CONTRAST FAIL ${name} ${f} on ${b} ${i ? 'dark' : 'light'}: ${r.toFixed(2)} < ${min}`); } return `<td>${r.toFixed(1)} : 1</td>`; }).join('');
    return `<tr><td><code>${f}</code> on <code>${b}</code></td>${cells}<td>${min === 3 ? '3 : 1, marks and large text' : '4.5 : 1, text'}</td></tr>`;
  }).join('');
  if (!bad) console.log(`contrast ok: ${name} (${pairs.length} pairs x 2 themes)`);
  return `<div class="scroll"><table class="spec"><tr><th>Pair</th><th>Light</th><th>Dark</th><th>AA needs</th></tr>${rows}</table></div>`;
}

// ---------- state glyphs (shape carries the state, colour repeats it) ----------
export const G = {
  bad: `<svg class="gl g-bad" viewBox="0 0 14 14" width="14" height="14" aria-hidden="true"><path d="M7 .9 13.1 7 7 13.1.9 7z" fill="currentColor"/><path d="M7 3.8v3.9M7 9.5v1" stroke="var(--g-on)" stroke-width="1.5"/></svg>`,
  warn: `<svg class="gl g-warn" viewBox="0 0 14 14" width="14" height="14" aria-hidden="true"><path d="M7 1.6 12.9 12H1.1z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/><path d="M7 5.4v3.2M7 10v.9" stroke="currentColor" stroke-width="1.4"/></svg>`,
  good: `<svg class="gl g-good" viewBox="0 0 14 14" width="14" height="14" aria-hidden="true"><path d="M2.4 7.4 5.6 10.6 11.8 3.8" fill="none" stroke="currentColor" stroke-width="1.7"/></svg>`,
  off: `<svg class="gl g-off" viewBox="0 0 14 14" width="14" height="14" aria-hidden="true"><circle cx="7" cy="7" r="5.2" fill="none" stroke="currentColor" stroke-width="1.4" stroke-dasharray="2.2 2"/><path d="M4.4 7h5.2" stroke="currentColor" stroke-width="1.4"/></svg>`,
};
export const st = (state) => G[state] || '';

// ---------- document shell ----------
export const DOC_CSS = `
*{box-sizing:border-box}
html{background:#000}
body{margin:0;background:#000;color:#fff;font:16px/1.6 system-ui,-apple-system,"Segoe UI",Roboto,"Helvetica Neue",Arial,sans-serif}
.doc{max-width:1240px;margin:0 auto;padding:20px 20px 80px}
.doc h1{font-size:28px;line-height:1.2;font-weight:650;margin:18px 0 8px}
.doc h2{font-size:21px;line-height:1.3;font-weight:650;margin:52px 0 10px;padding-top:18px;border-top:1px solid #2a2d33}
.doc h3{font-size:17px;font-weight:650;margin:30px 0 6px}
.doc h4{font-size:15px;font-weight:650;margin:20px 0 4px}
.doc p,.doc li{max-width:78ch}
.doc p{margin:0 0 12px}
.doc ul,.doc ol{padding-left:22px;margin:0 0 12px}
.doc a{color:#fff}
.dim{color:#a8adb7}
.doc code{font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;font-size:.86em;background:#16181c;padding:1px 5px;border-radius:3px;white-space:nowrap}
.nav{display:flex;flex-wrap:wrap;gap:6px 18px;align-items:center;font-size:15px;padding:6px 0 12px;border-bottom:1px solid #2a2d33}
.nav b{font-weight:650}
.nav .sp{flex:1}
.nav button{font:inherit;font-size:14px;color:#fff;background:#16181c;border:1px solid #4a4e57;border-radius:4px;padding:4px 10px;cursor:pointer}
.nav button:focus-visible,.doc a:focus-visible{outline:2px solid #fff;outline-offset:2px}
.toc{display:flex;flex-wrap:wrap;gap:4px 18px;margin:10px 0 0;padding:0;list-style:none;font-size:15px}
table.spec{width:100%;border-collapse:collapse;font-size:15px;margin:8px 0 16px}
table.spec th,table.spec td{text-align:left;vertical-align:top;padding:7px 14px 7px 0;border-bottom:1px solid #2a2d33}
table.spec th{color:#a8adb7;font-weight:500;white-space:nowrap}
table.spec td:first-child{min-width:150px}
.chip{display:inline-block;width:14px;height:14px;border:1px solid #555;margin-right:6px;vertical-align:-2px}
.scroll{overflow-x:auto;-webkit-overflow-scrolling:touch}
.shot{margin:10px 0 6px}
.cap{color:#a8adb7;font-size:14px;margin:4px 0 22px;max-width:90ch}
.pair{display:flex;flex-wrap:wrap;gap:16px}
.pair>div{zoom:.5}
.pairlab{display:flex;gap:16px;color:#a8adb7;font-size:14px;margin-top:10px}
.pairlab span{width:590px}
.small2{display:flex;flex-wrap:wrap;gap:20px;align-items:flex-start;margin:10px 0 10px}
.flow{margin:0 0 14px}
.flow b{font-weight:650}
.flow ol{margin:4px 0 2px}
.flow .n{color:#a8adb7}
.kbd{display:inline-block;font:12px/1 ui-monospace,Menlo,Consolas,monospace;border:1px solid #555a64;border-bottom-width:2px;border-radius:4px;padding:3px 5px;margin:0 1px;white-space:nowrap}
.diagram{max-width:100%;height:auto;display:block;margin:8px 0 14px}
.diagram text{font-family:system-ui,-apple-system,"Segoe UI",Roboto,sans-serif}
.mk{text-align:left}
.mk *{box-sizing:border-box}
.gl{flex:none;vertical-align:-2px}
@media (max-width:700px){body{font-size:15px}.doc h1{font-size:24px}.pairlab{display:none}}
@media (prefers-reduced-motion:reduce){*{transition:none!important;animation:none!important}}
`;

export const THEME_JS = `<script>
function setMock(t){document.body.setAttribute('data-mock',t);var b=document.getElementById('tbtn');if(b){b.textContent='Mock theme: '+t+'. Switch to '+(t==='dark'?'light':'dark');b.setAttribute('aria-pressed',t==='dark')}}
function flipMock(){setMock(document.body.getAttribute('data-mock')==='dark'?'light':'dark')}
</script>`;

export function nav(cur) {
  const items = [['index.html', 'Overview'], ['f1.html', 'F1 Site plan'], ['f2.html', 'F2 Crosstab'], ['f3.html', 'F3 Logbook']];
  return `<div class="nav">${items.map(([h, t]) => (h === cur ? `<b>${t}</b>` : `<a href="${h}">${t}</a>`)).join('')}<span class="sp"></span><button type="button" id="tbtn" aria-pressed="false" onclick="flipMock()">Mock theme: light. Switch to dark</button></div>`;
}

export function page({ file, title, fonts, css, body }) {
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${title}</title>
${fonts.map((f) => `<link rel="stylesheet" href="${f}">`).join('\n')}
<style>${DOC_CSS}${css}</style>
</head>
<body data-mock="light">
<div class="doc">
${nav(file)}
${body}
</div>
${THEME_JS}
</body>
</html>
`;
}

export const shot = (html, cap) => `<div class="scroll shot">${html}</div>${cap ? `<p class="cap">${cap}</p>` : ''}`;
export const pair = (fn) => `<div class="scroll shot"><div style="min-width:1196px"><div class="pair"><div>${fn('t-light')}</div><div>${fn('t-dark')}</div></div><div class="pairlab"><span>Light</span><span>Dark</span></div></div></div>`;
export const flows = (list) => list.map((f) => `<div class="flow"><b>${f.t}</b> <span class="n">${f.n}</span><ol>${f.s.map((s) => `<li>${s}</li>`).join('')}</ol>${f.alt ? `<p class="dim" style="margin:0">${f.alt}</p>` : ''}</div>`).join('');
export const wb = (s) => s.replace(/([\\/])/g, '$1<wbr>');
export const k = (s) => `<span class="kbd">${s}</span>`;
export const table = (head, rows) => `<div class="scroll"><table class="spec"><tr>${head.map((h) => `<th>${h}</th>`).join('')}</tr>${rows.map((r) => `<tr>${r.map((c) => `<td>${c}</td>`).join('')}</tr>`).join('')}</table></div>`;

// small box-and-arrow diagram on the black page
export function diagram(w, h, boxes, arrows, label) {
  const b = boxes.map(([x, y, bw, bh, t, sub, strong]) => `<rect x="${x}" y="${y}" width="${bw}" height="${bh}" rx="4" fill="${strong ? '#1c1f24' : 'none'}" stroke="${strong ? '#fff' : '#6b7079'}" stroke-width="1.2"/><text x="${x + 10}" y="${y + 20}" fill="#fff" font-size="13" font-weight="600">${t}</text>${sub ? sub.split('|').map((s, i) => `<text x="${x + 10}" y="${y + 37 + i * 15}" fill="#a8adb7" font-size="11.5">${s}</text>`).join('') : ''}`).join('');
  const a = arrows.map(([x1, y1, x2, y2, t, dash]) => `<path d="M${x1},${y1} L${x2},${y2}" stroke="#a8adb7" stroke-width="1.2" fill="none" marker-end="url(#dg)"${dash ? ' stroke-dasharray="4 3"' : ''}/>${t ? (x1 === x2 ? `<text x="${x1 + 7}" y="${(y1 + y2) / 2 + 4}" fill="#a8adb7" font-size="11">${t}</text>` : `<text x="${(x1 + x2) / 2 - 3}" y="${(y1 + y2) / 2 - 7}" fill="#a8adb7" font-size="11" text-anchor="middle">${t}</text>`) : ''}`).join('');
  return `<svg class="diagram" viewBox="0 0 ${w} ${h}" width="${w}" role="img" aria-label="${label}"><defs><marker id="dg" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" orient="auto"><path d="M0,0 8,4 0,8z" fill="#a8adb7"/></marker></defs>${a}${b}</svg>`;
}
