import { MODS, mod, num, st, wb, themeCss, cmdHtml, tokenTable, contrastTable, CMD, FILES, WHY, FAIL_OUT, shot, pair, flows, k, table, diagram } from './lib.mjs';

export const fonts = ['https://fonts.googleapis.com/css2?family=Albert+Sans:wght@400;500;600;700&family=Spline+Sans+Mono:wght@400;500&display=swap'];

export const tokens = {
  ground: ['#E8EDEB', '#131917', 'The table the plan lies on: canvas background'],
  dot: ['#C3CEC9', '#2B3632', 'Grid dots on the ground, 16 px apart'],
  plot: ['#FCFDFC', '#1D2522', 'Inside of a plot, fields, cards'],
  room: ['#EEF2F0', '#27312D', 'Rooms inside a plot, bar tracks'],
  panel: ['#F5F7F6', '#181F1D', 'Top bar, inspector, drawer, dock'],
  ink: ['#1A2623', '#E9EFEC', 'Text, walls in light, primary button fill'],
  ink2: ['#4A5954', '#A7B4AF', 'Secondary text, quiet links'],
  line: ['#C0CAC6', '#3A4743', 'Hairlines, partitions, field borders'],
  wall: ['#1A2623', '#C9D3CF', 'Plot wall, 2 px'],
  draft: ['#1B5AC8', '#8CBAFF', 'Things that do not exist yet, focus ring, selection'],
  draftbg: ['#E5EDFB', '#1A2941', 'Fill of a planned item'],
  good: ['#1C7449', '#63CC92', 'State: good'],
  warn: ['#855600', '#E6B450', 'State: warning'],
  warnbg: ['#FAF0D8', '#332A12', 'Warning fill'],
  bad: ['#B3261E', '#FF8576', 'State: problem'],
  badbg: ['#FBE9E6', '#3A1D1A', 'Problem fill'],
  off: ['#5F6C68', '#8A9792', 'State: off'],
};
const pairs = [['ink', 'plot'], ['ink', 'ground'], ['ink', 'panel'], ['ink', 'room'], ['ink2', 'plot'], ['ink2', 'ground'], ['ink2', 'panel'], ['ink2', 'room'], ['plot', 'ink'], ['draft', 'plot'], ['draft', 'draftbg'], ['draft', 'ground'], ['good', 'plot'], ['good', 'panel'], ['warn', 'plot'], ['warn', 'panel'], ['warn', 'warnbg'], ['bad', 'plot'], ['bad', 'panel'], ['bad', 'badbg'], ['bad', 'ground'], ['off', 'plot'], ['off', 'panel'], ['wall', 'ground', 3], ['line', 'plot', 1]];

export const css = themeCss('f1', tokens) + `
.f1{--g-on:var(--plot)}
.f1 .win{width:1180px;height:720px;display:grid;grid-template-rows:44px minmax(0,1fr) 30px;background:var(--ground);color:var(--ink);font:13px/1.4 "Albert Sans",system-ui,sans-serif;border:1px solid var(--line);border-radius:6px;overflow:hidden}
.f1 .mono{font-family:"Spline Sans Mono",ui-monospace,monospace;font-size:12px}
.f1 .top{display:flex;align-items:center;gap:12px;padding:0 12px;background:var(--panel);border-bottom:1px solid var(--line)}
.f1 .path{display:flex;align-items:center;gap:7px;font-size:14px;color:var(--ink2)}
.f1 .path b{color:var(--ink);font-weight:600}
.f1 .seg{display:flex;border:1px solid var(--line);border-radius:4px;overflow:hidden;background:var(--plot)}
.f1 .seg span{padding:4px 10px;border-right:1px solid var(--line);color:var(--ink2)}
.f1 .seg span:last-child{border-right:0}
.f1 .seg span.on{background:var(--ink);color:var(--plot);font-weight:600}
.f1 .goto{margin-left:auto;width:270px;white-space:nowrap;border:1px solid var(--line);border-radius:4px;background:var(--plot);padding:4px 8px;color:var(--ink2);display:flex;justify-content:space-between;align-items:center}
.f1 .key{font:11px/1.5 "Spline Sans Mono",monospace;border:1px solid var(--line);border-radius:3px;padding:0 4px;color:var(--ink2);background:var(--plot);white-space:nowrap}
.f1 .btn{display:inline-flex;align-items:center;gap:7px;border:1px solid var(--wall);border-radius:4px;padding:4px 10px;font-weight:600;background:var(--plot);color:var(--ink);white-space:nowrap}
.f1 .btn.pri{background:var(--ink);color:var(--plot);border-color:var(--ink)}
.f1 .btn.pri .key{background:transparent;color:var(--plot);border-color:var(--ink2)}
.f1 .btn.q{border-color:var(--line);font-weight:500}
.f1 .stage{display:grid;grid-template-columns:minmax(0,1fr) 300px;min-height:0}
.f1 .stage.wide{grid-template-columns:minmax(0,1fr) 350px}
.f1 .stage.build{grid-template-columns:minmax(0,1fr) 430px}
.f1 .canvas{position:relative;overflow:hidden;background-image:radial-gradient(var(--dot) 1px,transparent 1.3px);background-size:16px 16px}
.f1 .lvl{position:absolute;left:14px;width:70px;color:var(--ink2);font-size:11.5px;line-height:1.3}
.f1 .lvl b{display:block;font-size:13px;color:var(--ink);font-weight:600}
.f1 .note{position:absolute;color:var(--ink2);font-size:12px}
.f1 .plot{position:absolute;background:var(--plot);border:2px solid var(--wall);border-radius:2px;padding:7px 10px;display:flex;flex-direction:column;gap:1px}
.f1 .plot h5{margin:0;font-size:15px;line-height:1.35;font-weight:600;display:flex;align-items:center;gap:6px}
.f1 .plot h5 .gl{margin-left:auto}
.f1 .plot .m{color:var(--ink2);font-size:12px;white-space:nowrap}
.f1 .plot .sw{font-size:12px;font-weight:600;display:flex;gap:5px;align-items:center}
.f1 .plot .cov{margin-top:auto;display:flex;align-items:center;gap:8px;font-size:12px;color:var(--ink2);white-space:nowrap}
.f1 .bar{flex:1;height:6px;background:var(--room);border:1px solid var(--line);min-width:20px}
.f1 .bar i{display:block;height:100%;background:var(--ink)}
.f1 .bar.w i{background:var(--warn)}
.f1 .plot.sel{outline:2px solid var(--draft);outline-offset:3px}
.f1 .plot.off{border-style:dashed;border-color:var(--off);background-image:repeating-linear-gradient(135deg,transparent 0 7px,var(--line) 7px 8px)}
.f1 .lot{position:absolute;border:1.5px dashed var(--draft);color:var(--draft);background:var(--draftbg);border-radius:2px;display:flex;align-items:center;justify-content:center;gap:8px;font-weight:600}
.f1 .lot .key{color:var(--draft);border-color:var(--draft);background:transparent}
.f1 svg.links,.f1 svg.marks{position:absolute;inset:0;width:100%;height:100%;pointer-events:none}
.f1 .lk{stroke:var(--ink2);fill:none;stroke-width:1.4}
.f1 .lk.b{stroke:var(--bad);stroke-width:2}
.f1 .ah{fill:var(--ink2)}.f1 .ah.b{fill:var(--bad)}
.f1 .door{fill:var(--plot);stroke:var(--ink2);stroke-width:1.4}
.f1 .brk{stroke:var(--bad);stroke-width:2}
.f1 .brkbg{fill:var(--ground)}
.f1 .ltx{position:absolute;font-size:12px;white-space:nowrap;padding:0 4px;background:var(--ground)}
.f1 .c-bad{color:var(--bad)}.f1 .c-warn{color:var(--warn)}.f1 .c-good{color:var(--good)}.f1 .c-off{color:var(--off)}.f1 .c-2{color:var(--ink2)}.f1 .c-draft{color:var(--draft)}
.f1 .insp{background:var(--panel);border-left:1px solid var(--line);padding:14px;display:flex;flex-direction:column;gap:14px;min-width:0;overflow:hidden}
.f1 .insp h4{margin:0;font-size:16px;line-height:1.3;font-weight:600}
.f1 .insp h6{margin:0 0 5px;font-size:12px;font-weight:600;color:var(--ink2)}
.f1 .insp p{margin:0}
.f1 .kv{display:grid;grid-template-columns:92px minmax(0,1fr);gap:4px 8px}
.f1 .kv>span:nth-child(odd){color:var(--ink2)}
.f1 .iss{display:grid;grid-template-columns:16px minmax(0,1fr);gap:8px;padding:7px 0;border-top:1px solid var(--line)}
.f1 .iss small{display:block;color:var(--ink2);font-size:12px}
.f1 .acts{display:flex;flex-wrap:wrap;gap:6px}
.f1 .dock{display:flex;align-items:center;gap:10px;padding:0 12px;background:var(--panel);border-top:1px solid var(--line);font-size:12px;color:var(--ink2)}
.f1 .dock b{color:var(--ink);font-weight:600}
.f1 .dock .r{margin-left:auto}
.f1 .tog{display:inline-flex;align-items:center;gap:6px;font-weight:600}
.f1 .tog i{width:26px;height:14px;border:1.5px solid var(--ink);border-radius:2px;position:relative;display:block}
.f1 .tog i::after{content:"";position:absolute;right:1px;top:1px;width:9px;height:9px;background:var(--ink)}
/* module interior */
.f1 .big{padding:0;gap:0}
.f1 .big .bh{display:flex;align-items:center;gap:10px;padding:8px 12px;border-bottom:1px solid var(--line)}
.f1 .big .bh b{font-size:18px;font-weight:600}
.f1 .rooms{flex:1;display:flex;gap:4px;padding:6px;min-height:0}
.f1 .rooms .col{display:flex;flex-direction:column;gap:4px;min-width:0}
.f1 .rooms .rw{display:flex;gap:4px;min-height:0}
.f1 .room{background:var(--room);border:1px solid var(--line);padding:6px 8px;display:flex;flex-direction:column;min-width:0;min-height:0;overflow:hidden}
.f1 .room b{font-weight:600;font-size:13px}
.f1 .room i{font-style:normal;font-size:20px;line-height:1.2;font-weight:500;margin-top:auto}
.f1 .room small{font-size:12px;display:flex;gap:4px;align-items:center;white-space:nowrap}
.f1 .ghost{align-self:flex-start;border:1.5px dashed var(--draft);color:var(--draft);background:var(--draftbg);font-size:12px;font-weight:600;padding:1px 6px;border-radius:2px;margin-top:4px;max-width:100%;line-height:1.3}
.f1 .gate{position:absolute;top:-9px;width:40px;height:9px;border-left:2px solid var(--wall);border-right:2px solid var(--wall);background:var(--plot)}
.f1 .gate span{position:absolute;bottom:13px;left:50%;transform:translateX(-50%);white-space:nowrap;font-size:12px;color:var(--ink2)}
.f1 .gate span b{color:var(--ink);font-weight:600}
.f1 .stub{padding:5px 8px}
.f1 .stub h5{font-size:13px}
.f1 .stub .m{white-space:normal;line-height:1.3}
/* build drawer */
.f1 .drawer{background:var(--panel);border-left:1px solid var(--line);padding:12px 14px;display:flex;flex-direction:column;gap:9px;min-width:0}
.f1 .drawer h4{margin:0;font-size:16px;font-weight:600;display:flex;align-items:center}
.f1 .drawer h4 .key{margin-left:auto}
.f1 .lab{font-size:12px;font-weight:600;color:var(--ink2);margin-bottom:3px;display:flex;gap:6px}
.f1 .lab span{font-weight:400}
.f1 .field{border:1px solid var(--line);border-radius:4px;background:var(--plot);padding:5px 8px;min-height:30px;display:flex;align-items:center;gap:8px}
.f1 .field.foc{border-color:var(--draft);box-shadow:0 0 0 1px var(--draft)}
.f1 .field .r{margin-left:auto;color:var(--ink2)}
.f1 .chk{display:flex;gap:7px;align-items:center;padding:2px 0}
.f1 .chk>i{width:14px;height:14px;border:1.5px solid var(--ink2);border-radius:2px;background:var(--plot);display:block;flex:none;position:relative}
.f1 .chk>i.on{background:var(--ink);border-color:var(--ink)}
.f1 .chk>i.on::after{content:"";position:absolute;left:3.5px;top:.5px;width:4px;height:8px;border:solid var(--plot);border-width:0 1.8px 1.8px 0;transform:rotate(45deg)}
.f1 .chk span{color:var(--ink2);margin-left:auto;padding-right:14px}
.f1 .g2{display:grid;grid-template-columns:1fr 1fr;gap:0 10px}
.f1 .cmd{background:var(--ink);color:var(--plot);border-radius:4px;padding:8px 10px;font:12px/1.5 "Spline Sans Mono",monospace;overflow-wrap:anywhere}
.f1 .fl{font:12px/1.6 "Spline Sans Mono",monospace;color:var(--ink2);border:1.5px dashed var(--draft);background:var(--draftbg);border-radius:2px;padding:6px 10px;overflow-wrap:anywhere}
.f1 .fl b{color:var(--draft);font-weight:500}
/* why sheet */
.f1 .ev{padding:6px 0;border-top:1px solid var(--line)}
.f1 .ev div{display:flex;gap:8px;align-items:flex-start;color:var(--ink2);font-size:12px}
.f1 .ev .mono{overflow-wrap:anywhere;display:block}
.f1 .ev div>span:first-child{min-width:0}
.f1 .tag{border:1px solid var(--line);border-radius:2px;padding:0 5px;font-size:11.5px;color:var(--ink2);margin-left:auto;white-space:nowrap}
.f1 .tag.b{border-color:var(--bad);color:var(--bad)}
/* small states */
.f1 .win.sm{width:580px;height:360px}
.f1 .win.sm2{width:580px;height:360px;grid-template-rows:44px minmax(0,1fr)}
.f1 .out{background:var(--panel);padding:10px 14px;display:flex;flex-direction:column;gap:8px;min-height:0}
.f1 .out pre{margin:0;font:11px/1.45 "Spline Sans Mono",monospace;background:var(--plot);border:1px solid var(--line);padding:8px 10px;overflow:hidden;color:var(--ink)}
.f1 .out pre .e{color:var(--bad)}
.f1 .out pre .c{color:var(--ink2)}
.f1 .legend{display:flex;gap:14px;flex-wrap:wrap;padding:14px;background:var(--ground);color:var(--ink);font:13px/1.4 "Albert Sans",sans-serif;border:1px solid var(--line);border-radius:6px;width:fit-content}
.f1 .legend .plot{position:relative;width:176px;height:76px}
`;

// ---- drawing helpers ----
const head = (x, y, dir, cls = '') => {
  const s = 5;
  const p = { r: `${x},${y} ${x - 8},${y - s} ${x - 8},${y + s}`, l: `${x},${y} ${x + 8},${y - s} ${x + 8},${y + s}`, d: `${x},${y} ${x - s},${y - 8} ${x + s},${y - 8}`, u: `${x},${y} ${x - s},${y + 8} ${x + s},${y + 8}` }[dir];
  return `<polygon class="ah ${cls}" points="${p}"/>`;
};
const link = (d, hx, hy, dir, cls = '') => `<path class="lk ${cls}" d="${d}"/>${head(hx, hy, dir, cls)}`;
const door = (x, y) => `<rect class="door" x="${x - 6}" y="${y - 4}" width="12" height="8"/>`;
const breach = (x, y) => `<rect class="brkbg" x="${x - 8}" y="${y - 3}" width="16" height="6"/><path class="brk" d="M${x - 5},${y - 5} L${x + 5},${y + 5} M${x + 5},${y - 5} L${x - 5},${y + 5}"/>`;

const P = { // plan positions: x, y, w
  Notifications: [290, 24, 159], Checkout: [250, 172, 235], Shipping: [590, 172, 215],
  Inventory: [96, 320, 199], Billing: [440, 320, 261], Catalog: [96, 468, 284], Users: [440, 468, 228],
};
const PH = 112;
function plot(m, extra = '') {
  const [x, y, w] = P[m.n];
  const stateLine = m.state === 'bad' ? `<div class="sw c-bad">${st('bad')}${m.word}</div>` : m.state === 'warn' ? `<div class="sw c-warn">${st('warn')}${m.word}</div>` : '';
  return `<div class="plot ${extra}" style="left:${x}px;top:${y}px;width:${w}px;height:${PH}px"><h5>${m.n}</h5>${stateLine}<div class="m">${m.files} files, ${num(m.lines)} lines</div><div class="m">uses ${m.uses}, used by ${m.usedBy}</div><div class="cov"><span class="bar${m.state === 'warn' ? ' w' : ''}"><i style="width:${m.cov}%"></i></span>${m.cov}% covered</div></div>`;
}
const levels = `<div class="lvl" style="top:28px"><b>Level 3</b></div><div class="lvl" style="top:176px"><b>Level 2</b></div><div class="lvl" style="top:324px"><b>Level 1</b></div><div class="lvl" style="top:472px"><b>Level 0</b>uses no other module</div>`;
const problemLinks = `
${link('M485,214 H590', 590, 214, 'r', 'b')}${link('M590,236 H485', 485, 236, 'l', 'b')}
${link('M462,284 V320', 462, 312, 'd', 'b')}`;
const problemMarks = `${breach(462, 321)}`;
const problemLabels = `<div class="ltx c-bad" style="left:493px;top:186px"><b>use each other</b></div><div class="ltx c-bad" style="left:474px;top:291px"><b>reads a Billing model</b></div>`;

const top = (pathHtml, lens = 'Structure', extra = '') => `<div class="top"><div class="path">${pathHtml}</div>${lens ? `<div class="seg">${['Structure', 'Health', 'Tests', 'Changes'].map((l) => `<span${l === lens ? ' class="on"' : ''}>${l}</span>`).join('')}</div>` : ''}${extra}<div class="goto"><span>Go to module, class, command</span><span class="key">Ctrl K</span></div><span class="btn pri">Build <span class="key">B</span></span></div>`;
const dock = (txt) => `<div class="dock"><b>Runs</b>${txt}<span class="r">Show output <span class="key">\`</span></span></div>`;
const lastRun = `<span class="c-bad">${st('bad')}</span><span><span class="mono">module:migrate Checkout</span> failed at 10:02, exit code 1</span>`;

export function home(t) {
  return `<div class="mk f1 ${t}"><div class="win">
${top('<b>shop-api</b><span>7 modules</span>')}
<div class="stage"><div class="canvas">
${levels}
<svg class="links">${problemLinks}</svg>
${MODS.map((m) => plot(m)).join('')}
<svg class="marks">${problemMarks}</svg>
${problemLabels}
<div class="lot" style="left:590px;top:24px;width:170px;height:${PH}px">New module <span class="key">N</span></div>
<div class="note" style="left:96px;top:600px">A module sits above everything it uses. Only links that break a rule are drawn until you select a plot.</div>
</div>
<div class="insp">
<div><h4>shop-api</h4><span class="mono c-2">~/code/shop-api</span></div>
<div class="kv"><span>Laravel</span><span>13.2</span><span>Package</span><span>13.3.0, up to date</span><span>PHP runs with</span><span>Herd</span><span>Modules</span><span>7 on, 0 off</span><span>Files</span><span>551, 41,212 lines</span><span>Tests</span><span>307, 71% covered</span><span>Links</span><span>9, of which 1 pair circular</span></div>
<div><h6>Needs attention</h6>
<div class="iss"><span class="c-bad">${st('bad')}</span><div><b>Checkout and Shipping use each other</b><small>2 files in Checkout, 4 files in Shipping</small></div></div>
<div class="iss"><span class="c-bad">${st('bad')}</span><div><b>Checkout reads a Billing model directly</b><small>3 files use Billing\\Models\\Invoice</small></div></div>
<div class="iss"><span class="c-warn">${st('warn')}</span><div><b>Notifications is 39% covered</b><small>The app average is 71%</small></div></div>
<div class="iss"><span class="c-warn">${st('warn')}</span><div><b>Checkout has 1 migration waiting</b><small>add_coupon_id_to_orders_table</small></div></div>
</div>
<div class="acts" style="margin-top:auto"><span class="btn q">Run all tests</span><span class="btn q">Rules</span><span class="btn q">Module index</span></div>
</div></div>
${dock(lastRun)}
</div></div>`;
}

const room = (name, n, flex, extra = '', ghost = '') => `<div class="room" style="flex:${flex} 1 0"><b>${name}</b>${ghost}<i>${n}</i>${extra}</div>`;
function rooms(g = {}) {
  const gh = (kk) => (g[kk] ? `<span class="ghost">+ ${g[kk]}</span>` : '');
  return `<div class="rooms">
<div class="col" style="flex:31 1 0">${room('Tests', 31, 1, '<small class="c-2">54% covered</small>')}</div>
<div class="col" style="flex:53 1 0">
<div class="rw" style="flex:28 1 0">${room('Views', 12, 12)}${room('Migrations', 9, 10, `<small class="c-warn">${st('warn')}1 waiting</small>`, gh('Migrations'))}${room('Requests', 7, 7)}</div>
<div class="rw" style="flex:16 1 0">${room('Controllers', 6, 6)}${room('Events', 5, 5)}${room('Listeners', 5, 5)}</div>
<div class="rw" style="flex:13 1 0">${room('Models', 4, 4, '', gh('Models'))}${room('Jobs', 3, 3)}${room('Policies', 2, 3, '', gh('Policies'))}</div>
</div></div>`;
}
const stub = (x, y, name, note, cls = '', w = 118, h = 62) => `<div class="plot stub" style="left:${x}px;top:${y}px;width:${w}px;height:${h}px"><h5>${name}</h5><div class="m ${cls}">${note}</div></div>`;

export function detail(t) {
  return `<div class="mk f1 ${t}"><div class="win">
${top('<span>shop-api</span><span>/</span><b>Checkout</b>')}
<div class="stage"><div class="canvas">
<div class="note" style="left:14px;top:92px"><b>Used by</b></div><div class="note" style="left:748px;top:66px"><b>Checkout uses</b></div>
<svg class="links">
${link('M126,151 H160', 160, 151, 'r', 'b')}
${link('M126,241 H160', 160, 241, 'r')}
${link('M712,123 H748', 748, 123, 'r')}
${link('M712,203 H748', 748, 203, 'r')}
${link('M712,293 H748', 740, 293, 'r', 'b')}
${link('M712,393 H748', 748, 393, 'r', 'b')}
</svg>
${stub(14, 120, 'Shipping', '4 references, circular', 'c-bad', 112)}
${stub(14, 210, 'Notifications', '2 references', '', 112)}
${stub(749, 92, 'Catalog', '14 references')}
${stub(749, 172, 'Users', '3 references')}
${stub(749, 252, 'Billing', '6 references, 3 into Models', 'c-bad', 118, 82)}
${stub(749, 362, 'Shipping', '2 references, circular', 'c-bad')}
<svg class="marks" style="z-index:2">${door(161, 151)}${door(161, 241)}${door(749, 123)}${door(749, 203)}${breach(749, 293)}${door(749, 393)}</svg>
<div class="plot big" style="left:160px;top:92px;width:552px;height:470px">
<div class="gate" style="left:70px"><span><b>Contracts</b> 2</span></div><div class="gate" style="left:240px"><span><b>Events</b> 5</span></div><div class="gate" style="left:410px"><span><b>Routes</b> 18</span></div>
<div class="bh"><b>Checkout</b><span class="mono c-2">Modules/Checkout</span><span class="sw c-bad" style="margin-left:auto;display:flex;gap:5px;align-items:center;font-weight:600">${st('bad')}2 problems</span></div>
${rooms()}
</div>
<div class="note" style="left:160px;top:584px">Gates on the top wall are what Checkout offers. A link that ends in a cross goes through a wall instead of a gate.</div>
</div>
<div class="insp">
<div style="display:flex;align-items:center"><h4>Checkout</h4><span class="tog" style="margin-left:auto"><i></i>On</span></div>
<div class="kv"><span>Size</span><span>84 files, 6,412 lines</span><span>Tests</span><span>31, 54% covered</span><span>Routes</span><span>18: 12 API, 6 web</span><span>Migrations</span><span>9 ran, <b class="c-warn">1 waiting</b></span><span>Last change</span><span>2 days ago, 14 commits in 30 days</span></div>
<div><h6>Problems</h6>
<div class="iss"><span class="c-bad">${st('bad')}</span><div><b>Checkout and Shipping use each other</b><small>Rule: no circular use. <u>Show why</u></small></div></div>
<div class="iss"><span class="c-bad">${st('bad')}</span><div><b>3 files read Billing\\Models\\Invoice</b><small>Rule: other modules use contracts and events only. <u>Show why</u></small></div></div>
</div>
<div style="margin-top:auto"><h6>Actions</h6><div class="acts"><span class="btn pri">Build in Checkout <span class="key">B</span></span><span class="btn">Run migrate</span><span class="btn q">Run tests</span><span class="btn q">Seed</span><span class="btn q">Turn off</span><span class="btn q">Open in editor <span class="key">O</span></span><span class="btn q">Terminal here</span></div></div>
</div></div>
${dock(lastRun)}
</div></div>`;
}

export function gen(t) {
  return `<div class="mk f1 ${t}"><div class="win">
${top('<span>shop-api</span><span>/</span><span>Checkout</span><span>/</span><b>Build</b>', '')}
<div class="stage build"><div class="canvas">
<div class="plot big" style="left:24px;top:40px;width:700px;height:540px">
<div class="bh"><b>Checkout</b><span class="c-draft" style="margin-left:auto;font-weight:600">3 files planned, nothing written yet</span></div>
${rooms({ Models: 'Coupon', Policies: 'CouponPolicy', Migrations: 'create_coupons_table' })}
</div>
<div class="note" style="left:24px;top:598px">Dashed blue marks where the new files will land. They turn solid when the command has run.</div>
</div>
<div class="drawer">
<h4>Build in Checkout <span class="key">Esc</span></h4>
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
</div></div>
${dock('<span>No run in progress</span>')}
</div></div>`;
}

const evRow = ([f, l, sym, kind], modName) => `<div class="ev"><span class="mono">${wb(f)}:${l}</span><div><span>uses ${wb(sym)}</span><span class="tag${kind === 'internal' ? ' b' : ''}">${kind === 'internal' ? 'internal class' : 'through a ' + kind}</span></div></div>`;
export function sig(t) {
  return `<div class="mk f1 ${t}"><div class="win">
${top('<b>shop-api</b><span>Checkout selected</span>')}
<div class="stage wide"><div class="canvas">
${levels}
<svg class="links">
${link('M370,136 V172', 370, 164, 'd')}
${link('M320,284 V468', 320, 460, 'd')}
${link('M400,284 V448 H470 V468', 470, 460, 'd')}
${problemLinks}
</svg>
${MODS.map((m) => plot(m, m.n === 'Checkout' ? 'sel' : '')).join('')}
<svg class="marks">${door(370, 172)}${door(320, 468)}${door(470, 468)}${problemMarks}</svg>
${problemLabels}
<div class="ltx c-2" style="left:378px;top:142px">2</div><div class="ltx c-2" style="left:300px;top:386px">14</div><div class="ltx c-2" style="left:408px;top:386px">3</div>
<div class="ltx c-bad" style="left:526px;top:218px;background:none">2</div><div class="ltx c-bad" style="left:526px;top:240px;background:none">4</div>
<div class="note" style="left:96px;top:600px">Numbers on a link count references. A small box on a wall is a gate: the link goes through a contract, an event or a route.</div>
</div>
<div class="insp">
<div><h4>Checkout and Shipping use each other</h4><p class="c-2" style="margin-top:4px">Rule broken: no circular use. Both sides go through gates, so access is fine. The direction is the problem.</p></div>
<div><h6>Checkout uses Shipping: 2 references in 2 files</h6>${WHY.a.map((r) => evRow(r)).join('')}</div>
<div><h6>Shipping uses Checkout: 4 references in 4 files</h6>${WHY.b.map((r) => evRow(r)).join('')}</div>
<div class="acts" style="margin-top:auto"><span class="btn">Open all 6 in editor</span><span class="btn q">Copy as text</span><span class="btn q">Next problem <span class="key">J</span></span></div>
</div></div>
${dock(lastRun)}
</div></div>`;
}

export function empty(t) {
  return `<div class="mk f1 ${t}"><div class="win sm">
<div class="top"><div class="path"><b>No apps yet</b></div></div>
<div class="canvas"><div class="lot" style="left:90px;top:38px;width:400px;height:200px;flex-direction:column;gap:10px;padding:0 28px;text-align:center">
<div style="font-size:16px">Add your first app</div>
<div style="font-weight:400;color:var(--ink)">Choose the folder of a Laravel app that uses nwidart/laravel-modules. Nothing in the folder is changed.</div>
<span class="btn pri">Choose folder <span class="key">Ctrl O</span></span>
<div style="font-weight:400;color:var(--ink2);font-size:12px">or drop a folder on this window</div></div></div>
<div class="dock"><b>Runs</b><span>Nothing has run</span></div>
</div></div>`;
}
export function fail(t) {
  return `<div class="mk f1 ${t}"><div class="win sm2">
<div class="top"><span class="c-bad" style="display:flex">${st('bad')}</span><div class="path"><b>Migrate Checkout failed</b><span>exit code 1, 0.8 s</span></div><span class="key" style="margin-left:auto">\`</span></div>
<div class="out">
<pre>${FAIL_OUT.filter(([, l]) => l).map(([c, l]) => `<span class="${c}">${l.replace(/</g, '&lt;')}</span>`).join('\n')}</pre>
<div><b>The column orders.coupon_id already exists.</b> The migration stopped there and nothing after it ran. 9 migrations ran earlier, 1 is still waiting.</div>
<div class="acts"><span class="btn pri">Open the migration</span><span class="btn">Run again</span><span class="btn q">Copy output</span></div>
</div></div></div>`;
}
const legend = (t) => `<div class="mk f1 ${t}"><div class="legend">
<div class="plot"><h5>Good</h5><div class="m">plain wall, no mark</div></div>
<div class="plot"><h5>Warning</h5><div class="sw c-warn">${st('warn')}Low coverage</div><div class="m">triangle and words</div></div>
<div class="plot"><h5>Problem</h5><div class="sw c-bad">${st('bad')}2 problems</div><div class="m">diamond, words, red link</div></div>
<div class="plot off"><h5>Off ${st('off')}</h5><div class="m" style="background:var(--plot);align-self:flex-start;padding:0 3px">dashed wall, hatched, "off"</div></div>
<div class="lot" style="position:relative;width:150px;height:76px">Planned</div>
</div></div>`;

export const concept = 'The app is drawn as a site plan. Every module is a walled plot, placed above everything it uses, and you move by zooming: all apps, one app, one module, one kind of class. A link that breaks a rule is drawn as a line through a wall, so the home screen answers "what is here, what is wrong, where do I build" before any click.';

export function body() {
  return `
<h1>F1 Site plan</h1>
<p class="dim">A codebase as a place. Mock data is invented: app shop-api, 7 modules.</p>
<ul class="toc"><li><a href="#concept">Concept</a></li><li><a href="#ia">Information architecture</a></li><li><a href="#lang">Design language</a></li><li><a href="#mocks">Mock screens</a></li><li><a href="#egui">egui feasibility</a></li><li><a href="#weak">Weaknesses</a></li></ul>

<h2 id="concept">1. Concept</h2>
<p>${concept}</p>
<p>What is different to use: there are no sections such as Modules, Graph, Health or Generate. There is one plan, and those are things you see on it or do to it. Statistics are lenses that change what a plot shows, not pages. Creating is "build here": select a plot or a room, press ${k('B')}, and the planned files appear on the plan as dashed outlines before anything is written.</p>

<h2 id="ia">2. Information architecture</h2>
<h3>Navigation model</h3>
${diagram(1040, 200, [
    [0, 30, 170, 70, 'All apps', 'one small plan per app|only with 2 or more apps'],
    [230, 30, 190, 70, 'App plan (home)', 'plots in levels|lens: Structure, Health, ...', true],
    [480, 30, 180, 70, 'Module plot', 'rooms, gates, neighbours'],
    [720, 30, 170, 70, 'Room', 'classes of one kind'],
    [230, 140, 190, 48, 'Inspector', 'facts and actions of the selection'],
    [480, 140, 180, 48, 'Build drawer', 'generator form, preview'],
    [720, 140, 170, 48, 'Output dock', 'runs and their output'],
  ], [[170, 65, 230, 65, 'Enter'], [420, 65, 480, 65, 'Enter'], [660, 65, 720, 65, 'Enter'], [325, 100, 325, 140, '', true], [570, 100, 570, 140, 'B', true], [805, 100, 805, 140, '', true]], 'F1 navigation model')}
<p>Four zoom stops on one canvas. ${k('Enter')} zooms in on the selection, ${k('Esc')} zooms out. The path in the top bar shows the stops and is clickable. Three surfaces stay in place at every stop: the inspector on the right (what is selected), the build drawer (replaces the inspector while building), and the output dock at the bottom. ${k('Ctrl K')} jumps to any app, module, class or command by name.</p>

<h3>Screen inventory</h3>
${table(['Surface', 'Holds'], [
    ['All apps', 'One miniature plan per app with name, Laravel and package version, PHP runner, modules on, state. An empty lot adds an app. Skipped when only one app is registered.'],
    ['App plan, home', 'All modules as plots in levels. Problem links. Lens switch: Structure (size, links), Health (checks), Tests (coverage fill), Changes (git activity, 30 days), later Live (requests, latency, errors). Empty lot for a new module. With nothing selected the inspector shows app facts, totals and "Needs attention".'],
    ['Module plot', 'Rooms sized by file count, one per kind of class. Gates on the top wall: contracts, events, routes. Neighbours left (used by) and right (uses) with reference counts. Inspector: facts, problems, actions.'],
    ['Room', 'The classes of one kind as a list: name, lines, last change, covered or not. Open in editor. Build one more of this kind.'],
    ['Why sheet', 'Opens in the inspector when a link or a problem is selected. Both directions, every reference as file, line, symbol, and whether it goes through a gate.'],
    ['Build drawer', 'Generator search (41 generators and module:make), the form for the chosen one, the exact command, the files. Planned files show on the plan.'],
    ['Output dock', 'List of runs (command, module, time, result) and the output of the selected run. Opens by itself when a run starts or fails.'],
    ['Module index', 'A plain sortable list of modules in the inspector, for sorting by a number. The escape hatch from the map.'],
    ['Rules', 'The boundary rules of the app: no circular use; contracts and events only; fail CI when a rule breaks.'],
    ['Settings', 'Apps and their PHP command, editor, terminal, theme, density.'],
  ])}

<h3>Flows</h3>
${flows([
    { t: '(a) Add an app.', n: '3 actions', s: ['Click the empty lot "Add app" on the All apps stop, or press ' + k('Ctrl O') + '. The system folder dialog opens.', 'Choose the folder.', 'The lot fills in with what was found: Laravel 13.2, package 13.3.0, 7 modules, PHP runs with Herd (changeable). Press ' + k('Enter') + ' on "Add shop-api".'], alt: 'Dropping a folder on the window replaces steps 1 and 2: 2 actions.' },
    { t: '(b) Find the module with a problem and see what is wrong.', n: '0 actions to find, 1 to read, 2 for the evidence', s: ['Open the app. Checkout carries the problem mark and two red links leave it. "Needs attention" names both problems in words.', 'Click a problem line (or the red link). The Why sheet lists the files.'], alt: 'Keyboard: ' + k('J') + ' selects the next problem from anywhere.' },
    { t: '(c) Create a model with a migration and a policy in an existing module.', n: '6 actions from the room, 8 from the plan', s: ['Select Checkout and press ' + k('Enter') + ', then select the Models room.', 'Press ' + k('B') + '. The drawer opens with module Checkout and generator Model already set.', 'Type the name, Coupon.', 'Tick Migration.', 'Tick Policy.', 'Press ' + k('Ctrl Enter') + ', "Create 3 files".'], alt: 'From the plan: select Checkout, ' + k('B') + ', type "mod", ' + k('Enter') + ', then the last four steps. 8 actions.' },
    { t: '(d) Run migrations for one module and read the output.', n: '3 actions', s: ['Select Checkout. The inspector says "1 waiting" next to Migrations.', 'Click "Run migrate". A line shows the exact command, <code>php artisan module:migrate Checkout</code>.', 'Press ' + k('Enter') + '. The dock opens and the output streams in.'] },
    { t: '(e) See why two modules depend on each other.', n: '1 or 2 actions', s: ['Click the link between the two plots. Or select one plot and Shift-click the other, then ' + k('W') + '.', 'Read the Why sheet: both directions, file by file.'] },
  ])}

<h3>Keyboard model</h3>
${table(['Keys', 'Does'], [
    [k('Arrow keys'), 'Move the selection to the nearest plot or room in that direction'],
    [k('Enter') + ' ' + k('Esc'), 'Zoom in on the selection, zoom out one stop'],
    [k('Ctrl K'), 'Go to an app, module, class or command by name'],
    [k('B') + ' ' + k('N'), 'Build in the selection, new module'],
    [k('1') + ' to ' + k('5'), 'Switch lens'],
    [k('J') + ' ' + k('Shift J'), 'Next and previous problem'],
    [k('W'), 'Why: explain the selected link, or the link between two selected plots'],
    [k('R'), 'Run menu for the selection: migrate, seed, tests, turn on or off'],
    [k('O') + ' ' + k('T'), 'Open in editor, open a terminal in the module folder'],
    [k('`'), 'Show or hide the output dock'],
    [k('Tab'), 'Move between canvas, inspector and dock. Inside a form, between fields.'],
    [k('Ctrl Enter'), 'Run the command in the build drawer'],
  ])}
<p>Everything on the canvas is also reachable through ${k('Ctrl K')} and the module index, so nothing needs the pointer.</p>

<h3>How it scales</h3>
<ul>
<li><b>1 app, 5 modules.</b> The app opens straight on its plan; the All apps stop does not exist. Plots are large and show every number. The plan usually has two or three levels and no scrolling.</li>
<li><b>10 apps, 60 modules each.</b> All apps shows ten miniature plans sorted by number of problems. On an app plan, plots drop to a compact size (name, state mark, one bar) and levels wrap into rows. Links stay hidden except for broken rules and the selection, so sixty plots stay readable. Typing in ${k('Ctrl K')} dims every plot that does not match. Modules that share a folder or a name prefix are drawn inside one outlined block. Sorting sixty modules by a number is a job for the module index, not the map.</li>
</ul>

<h2 id="lang">3. Design language</h2>
<p>The reference is a drawn site plan on a work table: ground, walls, gates, and blue construction lines for what is planned. The one rule that carries the look: <b>colour means state</b>. Structure is ink on paper. Green, amber and red appear only where the app has something to say, and blue only for things that do not exist yet. A plan with no problems is black and white.</p>
<h3>Colour</h3>
${tokenTable(tokens)}
<h4>Contrast</h4>
${contrastTable('F1', tokens, pairs.filter((p) => p[2] !== 1))}
<h3>Type</h3>
${table(['Role', 'Face', 'Size, weight'], [
    ['Interface text', 'Albert Sans (SIL OFL)', '13 px, 400. Line height 1.4'],
    ['Names of plots, panel titles', 'Albert Sans', '15 px and 16 px, 600'],
    ['Module name at the module stop, room numerals', 'Albert Sans', '18 px, 600 and 20 px, 500'],
    ['Secondary lines, notes', 'Albert Sans', '12 px, 400'],
    ['Commands, paths, key hints', 'Spline Sans Mono (SIL OFL)', '12 px and 11 px, 400'],
  ])}
<p>Scale: 11, 12, 13, 15, 16, 18, 20. Albert Sans was drawn with Scandinavian architecture lettering in mind and stays legible at 12 px without hinting. The monospace face is used only for text a terminal would also show.</p>
<h3>Shape, space, motion</h3>
${table(['Token', 'Value'], [
    ['Spacing unit', '4 px. The plan snaps to a 16 px grid, the same grid as the dots.'],
    ['Corner radius', '2 px on plots, rooms and planned items. 4 px on controls. 6 px on the window. Nothing rounder.'],
    ['Stroke', 'Wall 2 px. Partition and field 1 px. Broken-rule link 2 px, other links 1.4 px. Planned items 1.5 px dashed.'],
    ['Elevation', 'None on the plan. Panels are separated by a 1 px line. Menus and the Go to list use one shadow: 0 6 px 16 px black at 18%.'],
    ['Icons', '16 px grid, 1.5 px stroke, square caps, built from lines and rectangles only.'],
    ['Density', 'Rows 28 px. Plot 112 px high at the app stop, 44 px in compact mode.'],
    ['Motion', 'One motion: the camera. Moving between zoom stops takes 160 ms, ease out. Nothing else animates and nothing moves on its own. With reduced motion the camera cuts.'],
  ])}
<h3>Module identity and state</h3>
<p>A module is known by its name and its place: a plot keeps its position between sessions, and its width follows its lines of code (with a minimum so a small module stays readable). There is no colour per module, because sixty colours cannot be told apart. State never relies on colour alone:</p>
${shot(legend('t-auto'))}
${table(['State', 'Shape', 'Words'], [
    ['Good', 'Plain wall, no mark on the plan. A check in lists.', '"Good" in lists'],
    ['Warning', 'Outlined triangle', 'The reason, for example "Low coverage"'],
    ['Problem', 'Filled diamond, plus a 2 px link and, for access to internals, a cross on the wall', 'The count and the sentence, for example "2 problems"'],
    ['Off', 'Dashed wall, hatched fill, dashed circle', '"Off"'],
    ['Planned', 'Dashed blue outline', 'What will be created'],
  ])}

<h2 id="mocks">4. Mock screens</h2>
<p class="dim">The switch at the top right of this page changes the theme of the large mocks. The pair below the home screen is fixed, one light and one dark.</p>
<h3>Home: the plan of shop-api</h3>
${shot(home('t-auto'), 'Nothing is selected. Levels read from the bottom: Catalog and Users use nothing; Checkout and Shipping share a level because they use each other. The inspector holds app statistics and the list of things that need attention.')}
<h4>Both themes</h4>
${pair(home)}
<h3>One module: Checkout</h3>
${shot(detail('t-auto'), 'The plot from the inside. Room size follows the number of files. Neighbours are stubs at the edges. The link to Billing ends in a cross because 3 files reach into Billing\'s Models room.')}
<h3>Generator: module:make-model</h3>
${shot(gen('t-auto'), 'Opened with B from the Models room, so module and generator are already set. The form lists the real options of the command. The three planned files show in their rooms.')}
<h3>Signature: select a plot, read its links, ask why</h3>
${shot(sig('t-auto'), 'Checkout is selected, so all its links are drawn. The red pair is selected and the inspector has become the Why sheet.')}
<h3>Empty and failure</h3>
<div class="small2">${empty('t-auto')}${fail('t-auto')}</div>
<p class="cap">Left: no apps yet. Right: the output dock after <code>module:migrate Checkout</code> failed, with the cause in one sentence above the actions.</p>

<h2 id="egui">5. Feasibility in egui</h2>
${table(['Part', 'Cost or risk', 'Simpler fallback'], [
    ['Canvas with zoom stops', 'Medium. egui has a pan and zoom container (Scene), and a Painter draws rectangles, lines and text. Four fixed stops avoid continuous semantic zoom and blurry scaled text.', 'No camera animation: cut between stops.'],
    ['Level layout', 'Medium. Needs own code: find cycles (strongly connected components), put each group on the longest path from the foundation, pack rows. About 200 lines, no dependency.', 'Fixed rows by level, plots in alphabetical order, equal width.'],
    ['Link routing', 'High if all links are drawn at once. Orthogonal routes through the gutters cross each other with many links.', 'Draw only broken rules and the links of the selection (the design already does this). Straight lines with one elbow.'],
    ['Rooms inside a plot', 'Low. A slice-and-dice treemap is about 60 lines.', 'A fixed grid of equal rooms with a bar for the count.'],
    ['Dashed walls, hatch, dot grid', 'Low. Dashed lines exist in egui. Hatch is diagonal lines under a clip rectangle. Dots: draw only the visible ones.', 'Drop the dots; use a flat ground.'],
    ['Hit testing on links', 'Low to medium. Distance from pointer to each segment.', 'Links are not clickable; the Why sheet opens from the problem list.'],
    ['Screen reader and keyboard on a canvas', 'Medium. Canvas items need their own focus handling and AccessKit nodes.', 'The module index mirrors the plan as a plain list.'],
    ['Fonts', 'Low. Bundle static Regular, Medium and SemiBold of Albert Sans and Regular of Spline Sans Mono. egui does not read variable font axes.', 'None needed.'],
  ])}

<h2 id="weak">6. Honest weaknesses</h2>
<ul>
<li><b>Comparing numbers is slower than in F2.</b> A map cannot be sorted. "Which module has the lowest coverage" needs the Tests lens or the module index.</li>
<li><b>Sixty modules need zooming and scrolling.</b> The plan stays readable but no longer fits one screen at full detail. F2 shows sixty rows with less effort.</li>
<li><b>Most custom code of the three.</b> Layout, routing and canvas focus are all yours to write and test.</li>
<li><b>Creating is one step slower than F3</b> for someone who already knows the command by heart.</li>
<li><b>Levels must be learned.</b> "Above means uses" is one sentence, but it is a sentence. A cycle forces two modules onto one level, which hides which one should be lower.</li>
<li><b>Run history is a dock, not a place.</b> Someone who mostly runs commands will find F3 more direct.</li>
</ul>`;
}
