import { MODS, KINDS, USES, mod, num, st, wb, themeCss, cmdHtml, tokenTable, contrastTable, CMD, FILES, WHY, FAIL_OUT, shot, pair, flows, k, table, diagram } from './lib.mjs';

export const fonts = ['https://fonts.googleapis.com/css2?family=Archivo:wght@400;500;600;700&family=Archivo+Narrow:wght@400;500;600&family=Red+Hat+Mono:wght@400;500&display=swap'];

export const tokens = {
  page: ['#F3F4F6', '#0E1215', 'Window background, gaps between panes'],
  cell: ['#FFFFFF', '#161B1F', 'Empty cell, panes, fields'],
  head: ['#E9ECEF', '#1D2329', 'Column headers, totals, tab strip'],
  band: ['#DAEAEE', '#1B3038', 'Header of the row and column under the cursor'],
  ink: ['#13171B', '#E8ECEF', 'Text and numbers'],
  ink2: ['#4E5862', '#9CA8B2', 'Secondary text, header labels, zeros'],
  line: ['#CFD5DB', '#2E3840', 'Grid lines, 1 px'],
  h1: ['#E2EEF1', '#18292E', 'Cell fill, count 1 or 2'],
  h2: ['#BFDAE2', '#1D3C45', 'Cell fill, count 3 to 5'],
  h3: ['#8DBDCA', '#25596A', 'Cell fill, count 6 to 9'],
  h4: ['#2A7488', '#74C0D1', 'Cell fill, count 10 and more'],
  h4t: ['#FFFFFF', '#0E1215', 'Text on the darkest fill'],
  accent: ['#0B5567', '#7CC8D8', 'Cursor outline, primary button, planned cell'],
  onaccent: ['#FFFFFF', '#0E1215', 'Text on accent'],
  good: ['#1B7449', '#62C990', 'State: good'],
  warn: ['#855600', '#E5B34F', 'State: warning'],
  warnbg: ['#FAF0D8', '#30280F', 'Warning cell fill'],
  bad: ['#B3261E', '#FF8475', 'State: problem'],
  badbg: ['#FBE9E6', '#381C19', 'Problem cell fill'],
  off: ['#5C6670', '#8793A0', 'State: off'],
};
const pairs = [['ink', 'cell'], ['ink', 'page'], ['ink', 'head'], ['ink', 'band'], ['ink', 'h1'], ['ink', 'h2'], ['ink', 'h3'], ['h4t', 'h4'], ['ink2', 'cell'], ['ink2', 'page'], ['ink2', 'head'], ['accent', 'cell'], ['accent', 'page'], ['onaccent', 'accent'], ['good', 'cell'], ['warn', 'cell'], ['warn', 'warnbg'], ['ink', 'warnbg'], ['bad', 'cell'], ['bad', 'badbg'], ['bad', 'page'], ['ink', 'badbg'], ['off', 'cell'], ['off', 'head'], ['accent', 'line', 3]];

export const css = themeCss('f2', tokens) + `
.f2{--g-on:var(--cell)}
.f2 .win{width:1180px;height:720px;display:grid;grid-template-rows:38px 46px minmax(0,1fr) 30px;background:var(--page);color:var(--ink);font:13px/1.35 Archivo,system-ui,sans-serif;border:1px solid var(--line);border-radius:6px;overflow:hidden}
.f2 .nar{font-family:"Archivo Narrow",Archivo,sans-serif}
.f2 .mono{font-family:"Red Hat Mono",ui-monospace,monospace;font-size:12px}
.f2 .apps{display:flex;align-items:flex-end;gap:2px;padding:0 12px;background:var(--head);border-bottom:1px solid var(--line)}
.f2 .tab{padding:7px 12px 8px;border:1px solid transparent;border-bottom:0;border-radius:6px 6px 0 0;color:var(--ink2);display:flex;gap:7px;align-items:center}
.f2 .tab.on{background:var(--page);border-color:var(--line);color:var(--ink);font-weight:600;margin-bottom:-1px;padding-bottom:9px}
.f2 .apps .goto{margin:0 0 5px auto;width:270px;border:1px solid var(--line);border-radius:6px;background:var(--cell);padding:3px 8px;color:var(--ink2);display:flex;justify-content:space-between;align-items:center;white-space:nowrap}
.f2 .key{font:11px/1.5 "Red Hat Mono",monospace;border:1px solid var(--line);border-radius:3px;padding:0 4px;color:var(--ink2);background:var(--cell);white-space:nowrap}
.f2 .axis{display:flex;align-items:center;gap:10px;padding:0 12px;min-width:0}
.f2 .axis .lb{color:var(--ink2)}
.f2 .axis h4{margin:0;font-size:18px;font-weight:600}
.f2 .seg{display:flex;border:1px solid var(--line);border-radius:6px;overflow:hidden;background:var(--cell)}
.f2 .seg span{padding:4px 11px;border-right:1px solid var(--line);color:var(--ink2);white-space:nowrap}
.f2 .seg span:last-child{border-right:0}
.f2 .seg span.on{background:var(--accent);color:var(--onaccent);font-weight:600}
.f2 .seg span.on .key{background:transparent;color:var(--onaccent);border-color:var(--onaccent)}
.f2 .filter{margin-left:auto;width:170px;border:1px solid var(--line);border-radius:6px;background:var(--cell);padding:4px 8px;color:var(--ink2);display:flex;justify-content:space-between}
.f2 .btn{display:inline-flex;align-items:center;gap:7px;border:1px solid var(--ink2);border-radius:6px;padding:4px 10px;font-weight:600;background:var(--cell);color:var(--ink);white-space:nowrap}
.f2 .btn.pri{background:var(--accent);color:var(--onaccent);border-color:var(--accent)}
.f2 .btn.pri .key{background:transparent;color:var(--onaccent);border-color:var(--onaccent)}
.f2 .btn.q{border-color:var(--line);font-weight:500}
.f2 .body{padding:0 12px 10px;min-height:0;display:flex;flex-direction:column;gap:10px}
.f2 .cols{display:grid;gap:12px;min-height:0;flex:1}
.f2 .cellbar{display:flex;align-items:center;gap:10px;height:36px;flex:none;border:1px solid var(--line);border-radius:6px;background:var(--cell);padding:0 6px 0 10px;white-space:nowrap;overflow:hidden}
.f2 .cellbar .addr{font-weight:600;padding-right:10px;border-right:1px solid var(--line);align-self:stretch;display:flex;align-items:center}
.f2 .cellbar .r{margin-left:auto;display:flex;gap:6px}
.f2 table.x{border-collapse:collapse;background:var(--cell);width:100%;table-layout:fixed;flex:none}
.f2 table.x th,.f2 table.x td{border:1px solid var(--line);height:36px;padding:0 8px;overflow:hidden;white-space:nowrap}
.f2 table.x thead th{background:var(--head);font:500 12px/1.2 "Archivo Narrow",Archivo,sans-serif;color:var(--ink2);text-align:right;height:34px;padding:0 4px}
.f2 table.x thead th.l{text-align:left;padding:0 8px}
.f2 table.x th.m{text-align:left;font:500 13px Archivo,sans-serif;color:var(--ink);background:var(--cell)}
.f2 table.x th.m .cd{float:right;font:11px/1.6 "Red Hat Mono",monospace;color:var(--ink2)}
.f2 .onq{display:inline-block;width:8px;height:8px;background:var(--ink);margin-right:8px}
.f2 table.x td{text-align:right;font-variant-numeric:tabular-nums}
.f2 table.x td.l{text-align:left}
.f2 td.z,.f2 .c.z b{color:var(--ink2);font-weight:400}
.f2 .h1{background:var(--h1)}.f2 .h2{background:var(--h2)}.f2 .h3{background:var(--h3)}
.f2 .h4{background:var(--h4);color:var(--h4t);font-weight:600}
.f2 td.cur,.f2 .c.cur{outline:2px solid var(--accent);outline-offset:-2px}
.f2 td.mir{outline:2px dashed var(--accent);outline-offset:-2px}
.f2 table.x th.ch{background:var(--band);color:var(--ink)}
.f2 table.x tfoot th,.f2 table.x tfoot td{background:var(--head);font-weight:600}
.f2 table.x td.sb{background:var(--badbg);color:var(--bad);font-weight:600;text-align:left}
.f2 table.x td.sw{background:var(--warnbg);color:var(--warn);font-weight:600;text-align:left}
.f2 table.x td.sg{color:var(--good);text-align:left}
.f2 td.xb,.f2 .c.xb,.f2 .swt.xb{background:var(--badbg)}
.f2 td.xb b,.f2 .c.xb b{color:var(--bad)}
.f2 td.self,.f2 .c.self,.f2 .swt.self,.f2 table.x th.m.self{background-image:repeating-linear-gradient(135deg,transparent 0 5px,var(--line) 5px 6px);background-color:var(--head)}
.f2 td.up{background:var(--head)}
.f2 .c-bad{color:var(--bad)}.f2 .c-warn{color:var(--warn)}.f2 .c-good{color:var(--good)}.f2 .c-2{color:var(--ink2)}.f2 .c-acc{color:var(--accent)}
.f2 .mbar{display:inline-block;width:34px;height:6px;background:var(--head);border:1px solid var(--line);margin-right:6px;vertical-align:1px}
.f2 .mbar i{display:block;height:100%;background:var(--accent)}
.f2 .plus{display:inline-block;border:1.5px dashed var(--accent);color:var(--accent);background:var(--cell);border-radius:3px;font-size:11.5px;font-weight:600;padding:0 3px;margin-left:4px;line-height:1.3}
.f2 .wt{font-size:11.5px;font-weight:500;margin-right:4px}
.f2 .wtg{float:left;margin-top:1px}
.f2 .reader{display:grid;border:1px solid var(--line);border-radius:6px;background:var(--cell);min-height:0;flex:1}
.f2 .reader>div{padding:10px 14px;border-right:1px solid var(--line);min-width:0}
.f2 .reader>div:last-child{border-right:0}
.f2 h6{margin:0 0 6px;font-size:13px;font-weight:600}
.f2 h6.s{font-size:12px;color:var(--ink2);margin-bottom:4px}
.f2 .li{display:flex;gap:8px;padding:5px 0;border-top:1px solid var(--line);align-items:baseline}
.f2 .li .r{margin-left:auto;color:var(--ink2);white-space:nowrap}
.f2 .iss{display:grid;grid-template-columns:16px minmax(0,1fr);gap:8px;padding:6px 0;border-top:1px solid var(--line)}
.f2 .iss small{display:block;color:var(--ink2);font-size:12px}
.f2 p{margin:0 0 8px}
.f2 .acts{display:flex;flex-wrap:wrap;gap:6px}
.f2 .runs{display:flex;align-items:center;gap:10px;padding:0 12px;background:var(--head);border-top:1px solid var(--line);font-size:12px;color:var(--ink2)}
.f2 .runs b{color:var(--ink);font-weight:600}
.f2 .runs .r{margin-left:auto}
/* row sheet */
.f2 .strip{margin-bottom:12px}
.f2 .cells{display:flex;border:1px solid var(--line);border-right:0;background:var(--cell)}
.f2 .c{flex:1 1 0;min-width:0;border-right:1px solid var(--line);padding:5px 8px;height:50px;display:flex;flex-direction:column;justify-content:space-between;overflow:hidden;white-space:nowrap}
.f2 .c span{font:500 11.5px/1.2 "Archivo Narrow",Archivo,sans-serif;color:var(--ink2)}
.f2 .c b{font-size:15px;font-weight:600;text-align:right;display:flex;justify-content:flex-end;align-items:center;gap:5px}
.f2 .c.h2 span,.f2 .c.h3 span{color:var(--ink)}
.f2 .c.h4 span{color:var(--h4t)}
.f2 .c.t b{font-size:13px;justify-content:flex-start}
.f2 .pane{border:1px solid var(--line);border-radius:6px;background:var(--cell);padding:11px 14px;display:flex;flex-direction:column;gap:8px;min-width:0;min-height:0}
.f2 .ev{padding:6px 0;border-top:1px solid var(--line)}
.f2 .ev .mono{overflow-wrap:anywhere;display:block}
.f2 .ev div{display:flex;gap:8px;align-items:flex-start;color:var(--ink2);font-size:12px}
.f2 .ev div>span:first-child{min-width:0}
.f2 .tag{border:1px solid var(--line);border-radius:3px;padding:0 5px;font-size:11.5px;color:var(--ink2);margin-left:auto;white-space:nowrap}
.f2 .tag.b{border-color:var(--bad);color:var(--bad);font-weight:600}
.f2 .tog{display:inline-flex;align-items:center;gap:6px;font-weight:600}
.f2 .tog i{width:26px;height:14px;border:1.5px solid var(--ink);border-radius:7px;position:relative;display:block}
.f2 .tog i::after{content:"";position:absolute;right:1.5px;top:1.5px;width:8px;height:8px;border-radius:4px;background:var(--ink)}
/* create pane */
.f2 .lab{font-size:12px;font-weight:600;color:var(--ink2);margin-bottom:3px;display:flex;gap:6px}
.f2 .lab span{font-weight:400}
.f2 .field{border:1px solid var(--line);border-radius:6px;background:var(--cell);padding:5px 8px;min-height:30px;display:flex;align-items:center;gap:8px}
.f2 .field.foc{border-color:var(--accent);box-shadow:0 0 0 1px var(--accent)}
.f2 .field .r{margin-left:auto;color:var(--ink2)}
.f2 .chk{display:flex;gap:7px;align-items:center;padding:2px 0}
.f2 .chk>i{width:14px;height:14px;border:1.5px solid var(--ink2);border-radius:3px;background:var(--cell);display:block;flex:none;position:relative}
.f2 .chk>i.on{background:var(--accent);border-color:var(--accent)}
.f2 .chk>i.on::after{content:"";position:absolute;left:3.5px;top:.5px;width:4px;height:8px;border:solid var(--onaccent);border-width:0 1.8px 1.8px 0;transform:rotate(45deg)}
.f2 .chk span{color:var(--ink2);margin-left:auto;padding-right:14px}
.f2 .g2{display:grid;grid-template-columns:1fr 1fr;gap:0 10px}
.f2 .cmd{background:var(--ink);color:var(--cell);border-radius:6px;padding:8px 10px;font:12px/1.5 "Red Hat Mono",monospace;overflow-wrap:anywhere}
.f2 .fl{font:12px/1.6 "Red Hat Mono",monospace;color:var(--ink2);border:1.5px dashed var(--accent);border-radius:6px;padding:6px 10px;overflow-wrap:anywhere}
.f2 .fl b{color:var(--accent);font-weight:500}
.f2 .legend2{display:flex;gap:18px;flex-wrap:wrap;color:var(--ink2);font-size:12px;align-items:center}
.f2 .swt{display:inline-block;width:14px;height:14px;border:1px solid var(--line);vertical-align:-3px;margin-right:5px}
/* small */
.f2 .win.sm{width:580px;height:360px;grid-template-rows:38px minmax(0,1fr) 30px}
.f2 .win.sm2{width:580px;height:auto;grid-template-rows:38px auto}
.f2 .out{padding:10px 12px;display:flex;flex-direction:column;gap:8px}
.f2 .out pre{margin:0;font:11px/1.45 "Red Hat Mono",monospace;background:var(--cell);border:1px solid var(--line);border-radius:6px;padding:8px 10px;overflow:hidden;color:var(--ink)}
.f2 .out pre .pe{color:var(--bad)}.f2 .out pre .pc{color:var(--ink2)}
.f2 .fig{background:var(--page);border:1px solid var(--line);border-radius:6px;padding:12px;width:fit-content;color:var(--ink);font:13px/1.35 Archivo,sans-serif}
.f2 .fig svg{display:block}
.f2 .q0{fill:var(--cell)}.f2 .q1{fill:var(--h2)}.f2 .q2{fill:var(--h3)}.f2 .q3{fill:var(--h4)}.f2 .qb{fill:var(--bad)}.f2 .qd{fill:var(--line)}.f2 .qg{stroke:var(--line);stroke-width:.5;fill:none}
.f2 .legendrow{display:flex;gap:10px;flex-wrap:wrap;padding:12px;background:var(--page);border:1px solid var(--line);border-radius:6px;width:fit-content;color:var(--ink);font:13px/1.35 Archivo,sans-serif}
.f2 .legendrow table.x{width:auto}
`;

const heat = (v, tests) => { const c = tests ? [20, 40, 60] : [3, 6, 10]; return v === 0 ? 0 : 1 + c.filter((x) => v >= x).length; };
const ktot = Object.fromEntries(KINDS.map((kk) => [kk, MODS.reduce((a, m) => a + m.k[kk], 0)]));

const tabs = (empty) => `<div class="apps">${empty ? '' : `<span class="tab on"><span class="c-bad">${st('bad')}</span>shop-api</span><span class="tab"><span class="c-warn">${st('warn')}</span>crm</span><span class="tab"><span class="c-bad">${st('bad')}</span>legacy-portal</span>`}<span class="tab${empty ? ' on' : ''}">+ Add app</span><span class="goto"><span>Go to module, class, command</span><span class="key">Ctrl K</span></span></div>`;
const axis = (on) => `<div class="axis"><span class="lb">Columns</span><div class="seg">${[['Kinds', '1'], ['Uses', '2'], ['Checks', '3'], ['Activity', '4']].map(([l, kk]) => `<span${l === on ? ' class="on"' : ''}>${l} <span class="key">${kk}</span></span>`).join('')}</div><span class="filter"><span>Filter rows</span><span class="key">/</span></span><span class="btn">New module <span class="key">Shift N</span></span></div>`;
const runs = (txt) => `<div class="runs"><b>Runs</b>${txt}<span class="r">Show output <span class="key">\`</span></span></div>`;
const lastRun = `<span class="c-bad">${st('bad')}</span><span><span class="mono">module:migrate Checkout</span> failed at 10:02, exit code 1</span>`;

function kindCell(m, kk, cur, ghost) {
  const v = m.k[kk];
  const h = heat(v, kk === 'Tests');
  const wait = m.n === 'Checkout' && kk === 'Migrations';
  return `<td class="${v ? 'h' + h : 'z'}${cur ? ' cur' : ''}">${wait ? `<span class="wtg" title="1 waiting">${st('warn')}</span>` : ''}${v}${ghost ? '<span class="plus">+1</span>' : ''}</td>`;
}
const stateCell = (m) => (m.state === 'bad' ? `<td class="sb">${st('bad')} ${m.word}</td>` : m.state === 'warn' ? `<td class="sw">${st('warn')} ${m.word}</td>` : `<td class="sg">${st('good')} Good</td>`);

export function home(t) {
  const cols = `<col style="width:146px"><col style="width:116px"><col style="width:46px"><col style="width:58px"><col style="width:84px">${KINDS.map(() => '<col>').join('')}`;
  return `<div class="mk f2 ${t}"><div class="win">
${tabs()}${axis('Kinds')}
<div class="body">
<div class="cellbar"><span class="addr">Checkout, Models</span><span>4 classes: Cart, CartItem, Order, OrderLine</span><span class="r"><span class="btn pri">New model here <span class="key">N</span></span><span class="btn q">Open cell <span class="key">Enter</span></span></span></div>
<table class="x"><colgroup>${cols}</colgroup>
<thead><tr><th class="l">Module</th><th class="l">State</th><th>Files</th><th>Lines</th><th>Covered</th>${KINDS.map((kk) => `<th${kk === 'Models' ? ' class="ch"' : ''}>${kk}</th>`).join('')}</tr></thead>
<tbody>${MODS.map((m) => `<tr><th class="m${m.n === 'Checkout' ? ' ch' : ''}"><i class="onq" title="On"></i>${m.n}<span class="cd">${m.c}</span></th>${stateCell(m)}<td>${m.files}</td><td>${num(m.lines)}</td><td><span class="mbar"><i style="width:${m.cov}%"></i></span>${m.cov}%</td>${KINDS.map((kk) => kindCell(m, kk, m.n === 'Checkout' && kk === 'Models')).join('')}</tr>`).join('')}</tbody>
<tfoot><tr><th class="m" style="background:var(--head)">7 modules, 7 on</th><td class="l nar" style="font-weight:500">2 problems, 1 warning</td><td>551</td><td>41,212</td><td>71%</td>${KINDS.map((kk) => `<td>${ktot[kk]}</td>`).join('')}</tr></tfoot></table>
<div class="reader" style="grid-template-columns:1.15fr 1fr 1.1fr">
<div><h6>In this cell: 4 models</h6>
${[['Cart.php', 212, '2 days ago'], ['CartItem.php', 96, '3 weeks ago'], ['Order.php', 348, '2 days ago'], ['OrderLine.php', 121, '5 weeks ago']].map(([f, l, c]) => `<div class="li"><span class="mono">${f}</span><span class="r">${l} lines, changed ${c}</span></div>`).join('')}</div>
<div><h6>Create in this cell</h6><p class="c-2">The column picks the generator and the row picks the module, so the form opens with both set.</p><div class="acts"><span class="btn pri">New model in Checkout <span class="key">N</span></span><span class="btn q">Another generator <span class="key">G</span></span></div></div>
<div><h6>Checkout</h6>
<div class="iss"><span class="c-bad">${st('bad')}</span><div><b>Uses Shipping, and Shipping uses it</b><small>See column set Uses</small></div></div>
<div class="iss"><span class="c-bad">${st('bad')}</span><div><b>3 files read Billing\\Models\\Invoice</b></div></div>
<div class="iss"><span class="c-warn">${st('warn')}</span><div><b>1 migration waiting</b><small>add_coupon_id_to_orders_table</small></div></div></div>
</div>
</div>
${runs(lastRun)}
</div></div>`;
}

const kc = (label, v, cls = '', inner = '') => `<div class="c ${cls}"><span>${label}</span><b>${inner || v}</b></div>`;
const evRow = ([f, l, sym, kind]) => `<div class="ev"><span class="mono">${wb(f)}:${l}</span><div><span>uses ${wb(sym)}</span><span class="tag${kind === 'internal' ? ' b' : ''}">${kind === 'internal' ? 'internal class' : 'through a ' + kind}</span></div></div>`;

export function detail(t) {
  const m = mod('Checkout');
  const usesStrip = MODS.map((o) => { if (o.n === 'Checkout') return kc('Checkout', '', 'self', ' '); const v = USES.Checkout[o.n] || 0; const bad = o.n === 'Billing' || o.n === 'Shipping'; return kc(o.n, v, (v ? '' : 'z') + (bad ? ' xb' : '') + (o.n === 'Billing' ? ' cur' : ''), bad ? `${st('bad')}${v}` : ''); }).join('');
  const usedStrip = MODS.map((o) => { if (o.n === 'Checkout') return kc('Checkout', '', 'self', ' '); const v = (USES[o.n] || {}).Checkout || 0; const bad = o.n === 'Shipping'; return kc(o.n, v, (v ? '' : 'z') + (bad ? ' xb' : ''), bad ? `${st('bad')}${v}` : ''); }).join('');
  const act = [1, 0, 2, 1, 0, 1, 2, 3, 4, 2, 5, 3];
  return `<div class="mk f2 ${t}"><div class="win">
${tabs()}
<div class="axis"><span class="btn q">Back to all rows <span class="key">Esc</span></span><h4>Checkout</h4><span class="mono c-2">Modules/Checkout</span><span class="tog"><i></i>On</span><span style="margin-left:auto" class="acts"><span class="btn q">Run tests</span><span class="btn">Run migrate</span><span class="btn q">Seed</span><span class="btn q">Open in editor <span class="key">O</span></span><span class="btn pri">New in Checkout <span class="key">N</span></span></span></div>
<div class="body"><div class="cols" style="grid-template-columns:minmax(0,1fr) 336px">
<div>
<div class="strip"><h6 class="s">Inside: 84 files, 6,412 lines, 18 routes (12 API, 6 web)</h6><div class="cells">${KINDS.map((kk) => kc(kk, m.k[kk], m.k[kk] ? 'h' + heat(m.k[kk], kk === 'Tests') : 'z', kk === 'Migrations' ? `<span class="wt">1 waiting</span>9` : '')).join('')}</div></div>
<div class="strip"><h6 class="s">Checkout uses: 25 references to 4 modules</h6><div class="cells">${usesStrip}</div></div>
<div class="strip"><h6 class="s">Uses Checkout: 6 references from 2 modules</h6><div class="cells">${usedStrip}</div></div>
<div class="strip"><h6 class="s">Checks: 4 pass, 1 warning, 2 broken</h6><div class="cells">
${kc('module.json', '', 't', `<span class="c-good">${st('good')}</span>Valid`)}${kc('Namespace', '', 't', `<span class="c-good">${st('good')}</span>Matches`)}${kc('No circular use', '', 't xb', `${st('bad')}Broken`)}${kc('Contracts only', '', 't xb', `${st('bad')}Broken`)}${kc('Migrations', '', 't', `<span class="c-warn">${st('warn')}</span><span style="font:inherit;color:var(--warn)">1 waiting</span>`)}${kc('Coverage over 50%', '', 't', `<span class="c-good">${st('good')}</span>54%`)}${kc('Translations', '', 't', `<span class="c-good">${st('good')}</span>Complete`)}
</div></div>
<div class="strip" style="margin-bottom:0"><h6 class="s">Commits per week, last 12 weeks. 14 in the last 30 days, last change 2 days ago</h6><div class="cells">${act.map((v, i) => kc(i === 11 ? 'this week' : `${11 - i} wk ago`, v, v ? 'h' + heat(v) : 'z')).join('')}</div></div>
</div>
<div class="pane">
<div><h6>Checkout uses Billing</h6><p style="margin:0">6 references in 5 files. <b class="c-bad">3 files read Billing\\Models\\Invoice directly.</b> Billing offers no contract for invoices.</p></div>
<div>${[['app/Services/InvoiceSummary.php', 27, 'Billing\\Models\\Invoice', 'internal'], ['app/Http/Controllers/OrderController.php', 88, 'Billing\\Models\\Invoice', 'internal'], ['app/Listeners/MarkOrderPaid.php', 19, 'Billing\\Models\\Invoice', 'internal'], ['app/Services/PaymentStarter.php', 14, 'Billing\\Contracts\\PaymentGateway', 'contract'], ['app/Jobs/CapturePayment.php', 31, 'Billing\\Contracts\\PaymentGateway', 'contract']].map(evRow).join('')}</div>
<div class="acts" style="margin-top:auto"><span class="btn">Open the 3 files in editor</span><span class="btn q">Copy as text</span></div>
</div>
</div></div>
${runs(lastRun)}
</div></div>`;
}

export function gen(t) {
  const ks = KINDS.slice(0, 9);
  const ghost = { Models: 1, Policies: 1, Migrations: 1 };
  return `<div class="mk f2 ${t}"><div class="win">
${tabs()}${axis('Kinds')}
<div class="body"><div class="cols" style="grid-template-columns:minmax(0,1fr) 450px">
<div style="display:flex;flex-direction:column;gap:10px;min-width:0">
<div class="cellbar"><span class="addr">Checkout, Models</span><span class="c-acc"><b>Planned:</b> 3 files, nothing written yet</span></div>
<table class="x"><colgroup><col style="width:128px">${ks.map(() => '<col>').join('')}</colgroup>
<thead><tr><th class="l">Module</th>${ks.map((kk) => `<th${kk === 'Models' ? ' class="ch"' : ''}>${kk}</th>`).join('')}</tr></thead>
<tbody>${MODS.map((m) => `<tr><th class="m${m.n === 'Checkout' ? ' ch' : ''}"><i class="onq"></i>${m.n}</th>${ks.map((kk) => kindCell(m, kk, m.n === 'Checkout' && kk === 'Models', m.n === 'Checkout' && ghost[kk])).join('')}</tr>`).join('')}</tbody></table>
<div class="reader" style="grid-template-columns:1fr;flex:none"><div><h6>What changes in the Checkout row</h6>
<div class="li"><span>Models</span><span class="r">4 becomes 5</span></div><div class="li"><span>Policies</span><span class="r">2 becomes 3</span></div><div class="li"><span>Migrations</span><span class="r">9 becomes 10, and 2 will be waiting</span></div></div></div>
</div>
<div class="pane">
<h6 style="margin:0;display:flex;font-size:15px">New model in Checkout <span class="key" style="margin-left:auto">Esc</span></h6>
<div><div class="lab">Generator</div><div class="field"><b>Model</b><span class="mono c-2">module:make-model</span><span class="r">Change <span class="key">G</span></span></div></div>
<div><div class="lab">Model name</div><div class="field foc">Coupon</div></div>
<div><div class="lab">Fillable attributes <span>separated by commas</span></div><div class="field mono">code,percent_off,expires_at</div></div>
<div><div class="lab">Create with the model</div><div class="g2">
<div class="chk"><i class="on"></i>Migration<span class="mono">-m</span></div><div class="chk"><i></i>Controller<span class="mono">-c</span></div>
<div class="chk"><i class="on"></i>Policy<span class="mono">-p</span></div><div class="chk"><i></i>Request<span class="mono">-r</span></div>
<div class="chk"><i></i>Factory<span class="mono">-f</span></div><div class="chk"><i></i>Resource<span class="mono">-R</span></div>
<div class="chk"><i></i>Seeder<span class="mono">-s</span></div><div class="chk"><i></i>All of these<span class="mono">-a</span></div>
<div class="chk"><i></i>Use #[Fillable] attributes</div><div class="chk"><i></i>Overwrite existing files</div>
</div></div>
<div><div class="lab">Command</div><div class="cmd">${cmdHtml(CMD)}</div></div>
<div><div class="lab">Files this creates</div><div class="fl">${FILES.map(([d, f]) => `${wb(d)}<b>${f}</b>`).join('<br>')}</div></div>
<div class="acts" style="margin-top:auto"><span class="btn pri">Create 3 files <span class="key">Ctrl Enter</span></span><span class="btn q">Copy command</span></div>
</div>
</div></div>
${runs('<span>No run in progress</span>')}
</div></div>`;
}

const ORDER = ['Users', 'Catalog', 'Billing', 'Inventory', 'Shipping', 'Checkout', 'Notifications'];
function usesTable(cur = ['Shipping', 'Checkout']) {
  const head = ORDER.map((n) => `<th${n === cur[1] ? ' class="ch"' : ''}>${n}</th>`).join('');
  const rows = ORDER.map((r, i) => {
    const m = mod(r);
    const tds = ORDER.map((c, j) => {
      if (i === j) return '<td class="self"></td>';
      const v = (USES[r] || {})[c] || 0;
      const bad = (r === 'Shipping' && c === 'Checkout') || (r === 'Checkout' && (c === 'Shipping' || c === 'Billing'));
      const cls = [v ? (bad ? 'xb' : 'h' + heat(v)) : j > i ? 'up z' : 'z', r === cur[0] && c === cur[1] ? 'cur' : '', r === cur[1] && c === cur[0] ? 'mir' : ''].join(' ');
      return `<td class="${cls}">${bad ? `<b>${st('bad')} ${v}</b>` : v || ''}</td>`;
    }).join('');
    const tot = Object.values(USES[r] || {}).reduce((a, b) => a + b, 0);
    return `<tr><th class="m${r === cur[0] ? ' ch' : ''}"><i class="onq"></i>${r}</th><td>${m.level}</td>${tds}<td>${tot || ''}</td></tr>`;
  }).join('');
  return `<table class="x"><colgroup><col style="width:138px"><col style="width:50px">${ORDER.map(() => '<col>').join('')}<col style="width:62px"></colgroup>
<thead><tr><th class="l">Row uses column</th><th>Level</th>${head}<th>All uses</th></tr></thead><tbody>${rows}</tbody></table>`;
}

export function sig(t) {
  return `<div class="mk f2 ${t}"><div class="win">
${tabs()}${axis('Uses')}
<div class="body"><div class="cols" style="grid-template-columns:minmax(0,1fr) 400px">
<div style="display:flex;flex-direction:column;gap:10px;min-width:0">
<div class="cellbar"><span class="addr">Shipping uses Checkout</span><span><b class="c-bad">4 references, right of the diagonal.</b> Checkout also uses Shipping (2).</span></div>
${usesTable()}
<div class="reader" style="grid-template-columns:1.2fr 1fr">
<div><h6>How to read it</h6><p class="c-2">Rows are sorted by level, foundations first. A module may use what is listed before it, so every number belongs left of the hatched diagonal. A number to the right of it is a module using something that is built on top of it.</p>
<div class="legend2"><span><i class="swt h2"></i>few</span><span><i class="swt h4"></i>many references</span><span><i class="swt xb"></i>${st('bad').replace('class="gl', 'class="gl c-bad')} breaks a rule</span><span><i class="swt self"></i>itself</span></div></div>
<div><h6>Rules for shop-api</h6>
<div class="chk"><i class="on"></i>No circular use<span style="padding:0;font-weight:600;color:var(--bad)">1 broken</span></div>
<div class="chk"><i class="on"></i>Contracts and events only<span style="padding:0;font-weight:600;color:var(--bad)">1 broken</span></div>
<div class="chk"><i></i>Fail CI when a rule breaks</div></div>
</div>
</div>
<div class="pane">
<div><h6>Shipping and Checkout use each other</h6><p style="margin:0" class="c-2">Rule broken: no circular use. Both sides go through contracts and events, so access is fine. The direction is the problem.</p></div>
<div><h6 class="s">Shipping uses Checkout: 4 references in 4 files</h6>${WHY.b.map(evRow).join('')}</div>
<div><h6 class="s">Checkout uses Shipping: 2 references in 2 files</h6>${WHY.a.map(evRow).join('')}</div>
<div class="acts" style="margin-top:auto"><span class="btn">Open all 6 in editor</span><span class="btn q">Copy as text</span></div>
</div>
</div></div>
${runs(lastRun)}
</div></div>`;
}

export function empty(t) {
  return `<div class="mk f2 ${t}"><div class="win sm">
<div class="apps"><span class="tab on">+ Add app</span></div>
<div class="body" style="padding-top:12px">
<table class="x"><thead><tr><th class="l" style="width:150px">Module</th><th class="l">State</th><th>Files</th><th>Lines</th><th>Models</th><th>Tests</th></tr></thead>
<tbody><tr><td colspan="6" style="height:216px;text-align:center;white-space:normal;padding:0 70px">
<div style="font-size:16px;font-weight:600;margin-bottom:6px">No apps yet</div>
<div style="margin-bottom:12px">Add the folder of a Laravel app that uses nwidart/laravel-modules. The app becomes a tab and each module becomes a row. Nothing in the folder is changed.</div>
<span class="btn pri">Choose folder <span class="key">Ctrl O</span></span>
<div class="c-2" style="font-size:12px;margin-top:8px">or drop a folder on this window</div></td></tr></tbody></table>
</div>
<div class="runs"><b>Runs</b><span>Nothing has run</span></div>
</div></div>`;
}
export function fail(t) {
  return `<div class="mk f2 ${t}"><div class="win sm2">
<div class="apps" style="align-items:center;gap:8px"><span class="c-bad" style="display:flex">${st('bad')}</span><b>Migrate Checkout failed</b><span class="c-2">exit code 1, 0.8 s</span><span class="key" style="margin-left:auto">\`</span></div>
<div class="out">
<pre>${FAIL_OUT.filter(([, l]) => l).map(([c, l]) => `<span class="${c ? 'p' + c : ''}">${l.replace(/</g, '&lt;')}</span>`).join('\n')}</pre>
<div><b>The column orders.coupon_id already exists.</b> The migration stopped there and nothing after it ran. The cell Checkout, Migrations still shows 1 waiting.</div>
<div class="acts"><span class="btn pri">Open the migration</span><span class="btn">Run again</span><span class="btn q">Copy output</span></div>
</div></div></div>`;
}

// the Uses view with 60 modules, drawn from a fixed pseudo-random matrix
function mini60(t) {
  let seed = 7; const rnd = () => { seed = (seed * 1103515245 + 12345) % 2147483648; return seed / 2147483648; };
  const N = 60, S = 7; let cells = ''; let links = 0;
  const flagged = new Set(['12,31', '33,41', '50,52']);
  for (let i = 0; i < N; i++) for (let j = 0; j < N; j++) {
    let cls = '';
    if (i === j) cls = 'qd';
    else if (flagged.has(`${i},${j}`)) cls = 'qb';
    else if (j < i) { const p = j < 8 ? 0.34 : 0.05 + 0.12 * Math.exp(-(i - j) / 6); if (rnd() < p) cls = 'q' + (1 + Math.floor(rnd() * 3)); }
    if (cls && cls !== 'qd') links++;
    if (cls) cells += `<rect class="${cls}" x="${j * S}" y="${i * S}" width="${S - 1}" height="${S - 1}"/>`;
  }
  return `<div class="mk f2 ${t}"><div class="fig"><svg width="${N * S}" height="${N * S}" viewBox="0 0 ${N * S} ${N * S}" role="img" aria-label="Uses view with 60 modules"><rect class="q0" width="${N * S}" height="${N * S}"/>${cells}</svg></div></div>|${links}`;
}
const legend = (t) => `<div class="mk f2 ${t}"><div class="legendrow"><table class="x"><thead><tr><th class="l" style="width:150px">Module</th><th class="l" style="width:150px">State</th><th style="width:70px">Models</th></tr></thead><tbody>
<tr><th class="m"><i class="onq"></i>Billing</th><td class="sg">${st('good')} Good</td><td class="h3">6</td></tr>
<tr><th class="m"><i class="onq"></i>Notifications</th><td class="sw">${st('warn')} Low coverage</td><td class="h1">2</td></tr>
<tr><th class="m"><i class="onq"></i>Checkout</th><td class="sb">${st('bad')} 2 problems</td><td class="h2">4</td></tr>
<tr><th class="m self" style="color:var(--off)"><i class="onq" style="background:none;border:1.5px dashed var(--off)"></i>Reports</th><td class="l" style="color:var(--off)">${st('off')} Off</td><td class="z">3</td></tr>
<tr><th class="m"><i class="onq"></i>Checkout</th><td class="l c-acc">Planned</td><td class="h2">4<span class="plus">+1</span></td></tr>
</tbody></table></div></div>`;

export const concept = 'The app is one grid that keeps its rows. Rows are modules; you choose what the columns are: kinds of classes, the other modules, health checks, or weeks. Every cell is a place you can read, open, or create into, so "a model in Checkout" is one cell and one key.';

export function body() {
  return `
<h1>F2 Crosstab</h1>
<p class="dim">Modules down the side, a question across the top. Mock data is invented: app shop-api, 7 modules.</p>
<ul class="toc"><li><a href="#concept">Concept</a></li><li><a href="#ia">Information architecture</a></li><li><a href="#lang">Design language</a></li><li><a href="#mocks">Mock screens</a></li><li><a href="#egui">egui feasibility</a></li><li><a href="#weak">Weaknesses</a></li></ul>

<h2 id="concept">1. Concept</h2>
<p>${concept}</p>
<p>What is different to use: there is a cursor, as in a spreadsheet, and it is the only selection model. Statistics, the dependency view, health and history are not separate pages; they are four column sets over the same rows, switched with ${k('1')} to ${k('4')}. The dependency view is a matrix, not a node graph: it reads the same with 7 modules and with 60, and a circular use shows as a number on the wrong side of the diagonal. Creating never starts with "which module, which generator": the cell already says both.</p>

<h2 id="ia">2. Information architecture</h2>
<h3>Navigation model</h3>
${diagram(1040, 200, [
    [0, 30, 150, 70, 'App tabs', 'one tab per app|Apps grid past 6 apps'],
    [210, 10, 250, 110, 'Crosstab (home)', 'rows: modules|columns: Kinds, Uses, Checks,|Activity, later Live', true],
    [520, 10, 170, 50, 'Cell', 'cell bar, then reader pane'],
    [520, 70, 170, 50, 'Row sheet', 'one module, all column sets'],
    [750, 10, 170, 50, 'Create pane', 'form, command, files'],
    [750, 70, 170, 50, 'Run strip', 'runs and output'],
  ], [[150, 65, 210, 65, ''], [460, 35, 520, 35, 'Enter'], [460, 95, 520, 95, 'Enter on name'], [690, 35, 750, 35, 'N'], [690, 95, 750, 95, 'R']], 'F2 navigation model')}
<p>Two levels only: the grid and what the cursor is on. The cell bar above the grid always says what the current cell holds, in one line. ${k('Enter')} opens the cell in a reader pane on the right; ${k('Enter')} on a module name opens the row sheet. ${k('Esc')} always goes back to the grid.</p>

<h3>Screen inventory</h3>
${table(['Surface', 'Holds'], [
    ['App tabs', 'One tab per app with its state mark. "Add app" is the last tab. With more than about 6 apps the strip becomes an Apps grid: rows are apps, columns are Laravel, package, PHP runner, modules on, problems.'],
    ['Crosstab: Kinds, home', 'Pinned columns: module, state, files, lines, covered. Then one column per kind of class with the count. Fill follows the count in four steps. Totals row.'],
    ['Crosstab: Uses', 'Columns are the modules. A cell is how many times the row module references the column module. Sorted by level. Rule breaks are marked. Rules of the app below.'],
    ['Crosstab: Checks', 'Columns are the health checks: module.json valid, namespace matches folder, no circular use, contracts and events only, migrations ran, coverage over the threshold, translations complete. A cell is pass, warning or broken.'],
    ['Crosstab: Activity', 'Columns are the last 12 weeks. A cell is the number of commits that touched the module. Later a fifth set, Live: requests, latency, errors, jobs per module.'],
    ['Cell bar', 'One line for the current cell: what it holds and the two most likely actions.'],
    ['Reader pane', 'The opened cell. Kinds: the classes, with lines and last change. Uses: every reference as file, line, symbol. Checks: what failed and the fix.'],
    ['Row sheet', 'One module: its row from every column set stacked as strips, plus facts and actions (run tests, migrate, seed, turn on or off, open in editor).'],
    ['Create pane', 'The generator form for the current cell, the exact command, the files. Planned files show as "+1" in the grid.'],
    ['Run strip', 'Bottom line with the last run; expands to the run list and output.'],
    ['Rules, Settings', 'Boundary rules per app. PHP command per app, editor, theme, density.'],
  ])}

<h3>Flows</h3>
${flows([
    { t: '(a) Add an app.', n: '3 actions', s: ['Click the "Add app" tab, or press ' + k('Ctrl O') + '. The system folder dialog opens.', 'Choose the folder.', 'A new tab shows what was found: Laravel 13.2, package 13.3.0, 7 modules, PHP runs with Herd (changeable). Press ' + k('Enter') + ' on "Add shop-api".'], alt: 'Dropping a folder on the window: 2 actions.' },
    { t: '(b) Find the module with a problem and see what is wrong.', n: '0 actions to find, 1 to read', s: ['Open the app. The State column of Checkout reads "2 problems" on a red cell with a diamond. The tab carries the same mark.', 'Click that cell. The cell bar and the reader name both problems; each links to its cell in Uses.'] },
    { t: '(c) Create a model with a migration and a policy in an existing module.', n: '6 actions', s: ['Click the cell Checkout, Models.', 'Press ' + k('N') + '. The create pane opens with generator Model and module Checkout set.', 'Type the name, Coupon.', 'Tick Migration.', 'Tick Policy.', 'Press ' + k('Ctrl Enter') + ', "Create 3 files".'] },
    { t: '(d) Run migrations for one module and read the output.', n: '3 actions', s: ['Click the cell Checkout, Migrations. It reads "1 waiting 9"; the cell bar names the waiting migration and offers "Run migrate".', 'Click "Run migrate". A line shows <code>php artisan module:migrate Checkout</code>.', 'Press ' + k('Enter') + '. The run strip expands with the output.'] },
    { t: '(e) See why two modules depend on each other.', n: '2 actions', s: ['Press ' + k('2') + ' for the Uses columns.', 'Click the cell where the two modules cross. The mirrored cell is outlined and the reader lists both directions, file by file.'] },
  ])}

<h3>Keyboard model</h3>
${table(['Keys', 'Does'], [
    [k('Arrow keys') + ' or ' + k('H') + k('J') + k('K') + k('L'), 'Move the cursor one cell'],
    [k('Home') + ' ' + k('End') + ' ' + k('Page Up') + ' ' + k('Page Down'), 'Jump along the row or the column'],
    [k('1') + ' to ' + k('4'), 'Column set: Kinds, Uses, Checks, Activity'],
    [k('Enter') + ' ' + k('Esc'), 'Open the cell or the row sheet, go back to the grid'],
    [k('N') + ' ' + k('Shift N') + ' ' + k('G'), 'New in this cell, new module, pick another generator for this row'],
    [k('R'), 'Run menu for the row: migrate, seed, tests, turn on or off'],
    [k('/') + ' ' + k('S'), 'Filter rows by name, sort rows by the current column'],
    [k('Ctrl K'), 'Go to an app, module, class or command by name'],
    [k('O') + ' ' + k('T'), 'Open the cell or module in the editor, open a terminal there'],
    [k('`'), 'Show or hide the run output'],
    [k('Ctrl Enter'), 'Run the command in the create pane'],
  ])}
<p>The whole app works with the cursor keys and about ten letters. This is the strongest keyboard model of the three because every target has an address.</p>

<h3>How it scales</h3>
<ul>
<li><b>1 app, 5 modules.</b> One tab. Five rows leave room below the grid, so the reader stays open there and shows the classes of the current cell without a click.</li>
<li><b>10 apps, 60 modules each.</b> Tabs give way to the Apps grid as the first screen. Sixty rows scroll under a fixed header at 28 px per row; ${k('/')} filters and ${k('S')} sorts by any column, which answers "lowest coverage" or "most migrations" in one key. In Uses, cells shrink to squares and show fill only; the cell bar gives the number. The picture below is that view with 60 modules: the triangle is healthy, the three red cells right of the diagonal are the cycles.</li>
</ul>
${(() => { const [h, n] = mini60('t-auto').split('|'); return shot(h, `Uses with 60 modules, invented data: ${n} links. One square per pair of modules. A node graph of the same data would be ${n} lines crossing each other.`); })()}

<h2 id="lang">3. Design language</h2>
<p>The reference is a printed cross table and a type case: a tray of compartments, each holding one sort of thing. Cells are square-cornered and touch each other; only the containers around them are rounded. One hue, petrol, is used as a four-step fill that means "how many": 1 to 2, 3 to 5, 6 to 9, 10 and more (tests use steps of 20). Red and amber are kept for rule breaks and warnings, so a grid without problems is petrol and grey.</p>
<h3>Colour</h3>
${tokenTable(tokens)}
<h4>Contrast</h4>
${contrastTable('F2', tokens, pairs)}
<h3>Type</h3>
${table(['Role', 'Face', 'Size, weight'], [
    ['Interface text, numbers in cells', 'Archivo (SIL OFL), tabular figures', '13 px, 400. Numbers on the darkest fill 600'],
    ['Column headers, cell labels in strips', 'Archivo Narrow (SIL OFL)', '12 px and 11.5 px, 500'],
    ['Pane titles, module name on the row sheet', 'Archivo', '13 px, 15 px and 18 px, 600'],
    ['Commands, paths, short codes, key hints', 'Red Hat Mono (SIL OFL)', '12 px and 11 px, 400'],
  ])}
<p>Scale: 11, 11.5, 12, 13, 15, 18. Archivo has sturdy, wide numerals that line up in columns; its narrow cut lets eleven column headers fit without rotating text.</p>
<h3>Shape, space, motion</h3>
${table(['Token', 'Value'], [
    ['Spacing unit', '4 px. Cell padding 8 px. Gap between panes 10 or 12 px.'],
    ['Cell size', '36 px high by default, 28 px compact. Kind columns 64 px wide. Uses cells square from 20 modules up.'],
    ['Corner radius', '0 on cells. 6 px on panes, fields, buttons and the window. 3 px on check boxes and key hints.'],
    ['Stroke', 'Grid lines 1 px. Cursor 2 px solid accent, inside the cell. Mirrored cell 2 px dashed. Planned items 1.5 px dashed.'],
    ['Elevation', 'None in the grid. Menus and the Go to list: one shadow, 0 4 px 12 px black at 18%.'],
    ['Icons', '14 px, 1.5 px stroke, built from lines and one diamond. State marks only; actions are words.'],
    ['Density', 'High. 13 px text, 36 px rows. A density setting changes the row height only.'],
    ['Motion', 'The cursor moves without animation. Changing the column set fades the fills over 120 ms so the eye sees that the rows stayed. Nothing else moves.'],
  ])}
<h3>Module identity and state</h3>
<p>A module is its row: the name at the left edge, always visible while columns scroll, and a short code (BI, CA, CH) used where the module is a column. No colour per module. State is a shape and a word in the State column, and the fill of that cell repeats it:</p>
${shot(legend('t-auto'))}
${table(['State', 'Shape', 'Words'], [
    ['Good', 'Check mark', '"Good"'],
    ['Warning', 'Outlined triangle on an amber cell', 'The reason, for example "Low coverage"'],
    ['Problem', 'Filled diamond on a red cell. In Uses, the number sits right of the diagonal or carries the diamond.', 'The count, for example "2 problems"'],
    ['Off', 'Dashed circle, hollow dashed square before the name, hatched name cell', '"Off"'],
    ['Planned', 'Dashed "+1" inside the cell', '"Planned" in the cell bar'],
  ])}

<h2 id="mocks">4. Mock screens</h2>
<p class="dim">The switch at the top right of this page changes the theme of the large mocks. The pair below the home screen is fixed, one light and one dark.</p>
<h3>Home: shop-api by kinds of classes</h3>
${shot(home('t-auto'), 'The cursor is on Checkout, Models. The cell bar names the four models; the reader below lists them. Fill follows the count, so the shape of each module shows: Notifications is listeners and views, Users is mostly tests, and the empty Resources cell in Checkout is a visible gap.')}
<h4>Both themes</h4>
${pair(home)}
<h3>One module: the Checkout row sheet</h3>
${shot(detail('t-auto'), 'The same cells as the grid, one strip per column set. The cursor is on "Checkout uses Billing" and the reader shows the three files that reach into Billing\'s models.')}
<h3>Generator: module:make-model</h3>
${shot(gen('t-auto'), 'Opened with N on the cell Checkout, Models. The form lists the real options of the command. The three cells that will change show a dashed +1.')}
<h3>Signature: Uses, the dependency matrix</h3>
${shot(sig('t-auto'), 'Read a row: Checkout uses Users 3 times, Catalog 14 times, Billing 6 times, Shipping twice. The only number right of the diagonal is Shipping using Checkout: the cycle. Its mirror cell is outlined with dashes.')}
<h3>Empty and failure</h3>
<div class="small2">${empty('t-auto')}${fail('t-auto')}</div>
<p class="cap">Left: no apps yet. Right: the run strip expanded after <code>module:migrate Checkout</code> failed.</p>

<h2 id="egui">5. Feasibility in egui</h2>
${table(['Part', 'Cost or risk', 'Simpler fallback'], [
    ['The grid', 'Low. egui_extras has a table with fixed header, resizable columns and row virtualisation. Cell fills and the cursor outline are rectangles.', 'None needed.'],
    ['Pinned left columns with horizontal scroll', 'Medium. The stock table pins the header, not columns. Needs two synchronised tables or a hand-drawn grid.', 'No pinning: keep the module name as the only left column and move state and sizes into the row sheet.'],
    ['Cursor and keyboard grid navigation', 'Low to medium. Own focus state (row, column) and key handling; scroll the cursor into view.', 'None needed; this is the core of the direction.'],
    ['Uses matrix at 60 by 60', 'Low. 3,600 rectangles per frame is cheap for a GPU painter. Header labels become short codes; rotated text exists in egui but is fiddly.', 'Codes only, full names in the cell bar.'],
    ['Narrow headers', 'Low. Bundle Archivo Narrow as a second static font. Long kind names may still clip in compact mode.', 'Abbreviate in the header, full name in the cell bar.'],
    ['Reader and create pane', 'Low. Ordinary panels and custom-drawn inputs.', 'None needed.'],
    ['Hatch on the diagonal and on off rows', 'Low. Diagonal lines under a clip rectangle.', 'A flat grey fill.'],
    ['Fill fade when the column set changes', 'Low. One animated value.', 'Cut.'],
  ])}

<h2 id="weak">6. Honest weaknesses</h2>
<ul>
<li><b>It is still a table.</b> It is the least surprising of the three at first glance, and the closest to the baseline in look, even though it is used differently. It does not give the feeling of a codebase as a place.</li>
<li><b>The dependency matrix must be learned.</b> "Row uses column, numbers belong left of the diagonal" takes a minute; a drawn link between two boxes takes none. F1 explains a cycle faster to a newcomer.</li>
<li><b>Small apps look sparse.</b> Five rows fill a third of the window. The reader below helps but the grid earns its keep from about ten modules up.</li>
<li><b>Only kinds with a column can be created in one step.</b> The package has 41 generators; a grid shows about a dozen kinds before scrolling. The rest need ${k('G')} and a search.</li>
<li><b>Chains are hard to follow.</b> "What does Notifications reach through Checkout" needs two rows read in turn. F1 shows the path.</li>
<li><b>Dense.</b> It is comfortable for people who like spreadsheets and tiring for people who do not.</li>
</ul>`;
}
