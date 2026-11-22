import { MODS, num, st, wb, themeCss, cmdHtml, tokenTable, contrastTable, CMD, FILES, WHY, FAIL_OUT, shot, pair, flows, k, table, diagram } from './lib.mjs';

export const fonts = ['https://fonts.googleapis.com/css2?family=Recursive:slnt,wght,CASL,CRSV,MONO@-15..0,300..1000,0..1,0..1,0..1&display=swap'];

export const tokens = {
  page: ['#FFFFFF', '#1A1C33', 'The log: where entries are written'],
  rail: ['#EEEEF7', '#121326', 'Thread list on the left'],
  band: ['#F5F5FB', '#20223D', 'Margin column, standing block, drafts'],
  ink: ['#1A1C3A', '#ECEDF8', 'Text. Blue-black, not neutral black'],
  ink2: ['#52557A', '#A8ABCB', 'Secondary text, times, kinds'],
  line: ['#D6D7E8', '#34375C', 'Rules between entries, field borders'],
  accent: ['#2B2F9E', '#B4B9FF', 'Links, primary button, drafts, caret'],
  onaccent: ['#FFFFFF', '#15162A', 'Text on accent'],
  mark: ['#FFE96A', '#5A531C', 'Highlighter: where you are. Never a state'],
  cmdbg: ['#1A1C3A', '#0D0E1E', 'Command and output blocks'],
  cmdfg: ['#ECEDF8', '#ECEDF8', 'Text in command blocks'],
  good: ['#17754A', '#6AD3A0', 'State: good'],
  warn: ['#8A5300', '#EFB65C', 'State: warning'],
  bad: ['#B3261E', '#FF8C82', 'State: problem'],
  badbg: ['#FCEAE8', '#3B1F2A', 'Problem fill'],
  off: ['#5F627D', '#8C90B0', 'State: off'],
};
const pairs = [['ink', 'page'], ['ink', 'rail'], ['ink', 'band'], ['ink', 'mark'], ['ink2', 'page'], ['ink2', 'rail'], ['ink2', 'band'], ['accent', 'page'], ['accent', 'band'], ['accent', 'rail'], ['onaccent', 'accent'], ['cmdfg', 'cmdbg'], ['good', 'page'], ['good', 'rail'], ['good', 'band'], ['warn', 'page'], ['warn', 'rail'], ['warn', 'band'], ['bad', 'page'], ['bad', 'rail'], ['bad', 'band'], ['bad', 'badbg'], ['off', 'page'], ['off', 'rail']];

export const css = themeCss('f3', tokens) + `
.f3{--g-on:var(--page)}
.f3 .win{width:1180px;height:720px;display:grid;grid-template-columns:232px minmax(0,1fr);background:var(--page);color:var(--ink);font:13px/1.45 Recursive,system-ui,sans-serif;font-variation-settings:"MONO" 0,"CASL" 0,"CRSV" 0;border:1px solid var(--line);border-radius:6px;overflow:hidden}
.f3 .mono{font-variation-settings:"MONO" 1,"CASL" 0,"CRSV" 0}
.f3 b{font-weight:650}
.f3 .rail{background:var(--rail);border-right:1px solid var(--line);padding:12px 8px;display:flex;flex-direction:column;gap:1px;min-width:0}
.f3 .rail .app{padding:2px 8px 12px}
.f3 .rail .app b{display:block;font-size:15px}
.f3 .rail .app small{color:var(--ink2);font-size:12px}
.f3 .rail h6{margin:12px 8px 3px;font-size:12px;font-weight:650;color:var(--ink2)}
.f3 .th{display:flex;align-items:center;gap:8px;padding:5px 8px;border-radius:4px}
.f3 .th.on{background:var(--mark);font-weight:650}
.f3 .th .ct{margin-left:auto;font-size:12px;color:var(--ink2);font-weight:400}
.f3 .th.on .ct{color:var(--ink)}
.f3 .main{display:grid;grid-template-rows:44px minmax(0,1fr) auto;min-width:0;min-height:0;position:relative}
.f3 .hd{display:flex;align-items:center;gap:12px;padding:0 16px;border-bottom:1px solid var(--line)}
.f3 .hd h4{margin:0;font-size:18px;font-weight:650}
.f3 .seg{display:flex;gap:2px;margin-left:auto}
.f3 .seg span{padding:3px 9px;border-radius:4px;color:var(--ink2)}
.f3 .seg span.on{background:var(--mark);color:var(--ink);font-weight:650}
.f3 .log{overflow:hidden;display:flex;flex-direction:column;justify-content:flex-start;background:linear-gradient(to right,var(--band) 0 92px,var(--line) 92px 93px,transparent 93px)}
.f3 .en{display:grid;grid-template-columns:93px minmax(0,1fr);border-top:1px solid var(--line)}
.f3 .en:first-child{border-top:0}
.f3 .mg{padding:9px 8px 9px 16px;color:var(--ink2);font-size:12px;line-height:1.5}
.f3 .mg b{display:block;color:var(--ink);font-size:13px;font-variant-numeric:tabular-nums}
.f3 .bd{padding:9px 16px;min-width:0}
.f3 .en.stand .bd{background:var(--band)}
.f3 .en.day{border-top:1px solid var(--line)}
.f3 .en.day .mg{padding-top:4px;padding-bottom:4px;color:var(--ink);font-weight:650}
.f3 .en.day .bd{padding:0}
.f3 .t{display:flex;gap:10px;align-items:baseline}
.f3 .t .r{margin-left:auto;display:flex;gap:12px;white-space:nowrap}
.f3 .d{color:var(--ink2);margin-top:2px}
.f3 .lnk{color:var(--accent);font-weight:650;text-decoration:underline;text-underline-offset:3px;text-decoration-thickness:1px;white-space:nowrap}
.f3 .c-bad{color:var(--bad)}.f3 .c-warn{color:var(--warn)}.f3 .c-good{color:var(--good)}.f3 .c-2{color:var(--ink2)}.f3 .c-acc{color:var(--accent)}.f3 .c-off{color:var(--off)}
.f3 table.t{border-collapse:collapse;width:100%;margin-top:8px}
.f3 table.t th,.f3 table.t td{padding:3px 12px 3px 0;border-top:1px solid var(--line);text-align:right;font-variant-numeric:tabular-nums;white-space:nowrap}
.f3 table.t th{font-weight:500;color:var(--ink2);font-size:12px;border-top:0}
.f3 table.t .l{text-align:left}
.f3 table.t td.l b{font-weight:650}
.f3 .mbar{display:inline-block;width:40px;height:5px;background:var(--line);margin-right:6px;vertical-align:1px}
.f3 .mbar i{display:block;height:100%;background:var(--ink)}
.f3 .prompt{border-top:2px solid var(--ink);padding:10px 16px 11px;background:var(--page)}
.f3 .pl{display:flex;gap:9px;align-items:center;font-size:15px;min-height:24px;white-space:nowrap}
.f3 .pc{color:var(--ink2);font-size:13px}
.f3 .ph{color:var(--ink2)}
.f3 .caret{display:inline-block;width:2px;height:19px;background:var(--accent);margin:0 -4px 0 -4px}
.f3 .hints{display:flex;gap:6px;margin-top:8px;color:var(--ink2);font-size:12px;align-items:center}
.f3 .hint{border:1px solid var(--line);border-radius:4px;padding:1px 8px;color:var(--ink)}
.f3 .hints .r{margin-left:auto}
.f3 .key{font-size:11px;line-height:1.5;font-variation-settings:"MONO" 1,"CASL" 0,"CRSV" 0;border:1px solid var(--line);border-radius:3px;padding:0 4px;color:var(--ink2);white-space:nowrap}
.f3 .btn{display:inline-flex;align-items:center;gap:7px;border:1px solid var(--ink);border-radius:4px;padding:4px 10px;font-weight:650;color:var(--ink);white-space:nowrap}
.f3 .btn.pri{background:var(--accent);color:var(--onaccent);border-color:var(--accent)}
.f3 .btn.pri .key{color:var(--onaccent);border-color:var(--onaccent)}
.f3 .btn.q{border-color:var(--line);font-weight:500}
.f3 .acts{display:flex;flex-wrap:wrap;gap:6px}
.f3 .tog{display:inline-flex;align-items:center;gap:6px;font-weight:650}
.f3 .tog i{width:26px;height:14px;border:1.5px solid var(--ink);border-radius:3px;position:relative;display:block}
.f3 .tog i::after{content:"";position:absolute;right:1.5px;top:1.5px;width:8px;height:8px;border-radius:1px;background:var(--ink)}
.f3 .g3{display:grid;grid-template-columns:1.1fr 1.25fr 1.2fr;gap:22px;margin-top:8px}
.f3 h6{margin:0 0 3px;font-size:12px;font-weight:650;color:var(--ink2)}
.f3 .kl{display:grid;grid-template-columns:1fr auto 1fr auto;gap:1px 10px}
.f3 .kl span:nth-child(even){text-align:right;font-variant-numeric:tabular-nums;padding-right:10px}
.f3 .row{display:flex;gap:8px;align-items:baseline;padding:1px 0}
.f3 .row .n{margin-left:auto;font-variant-numeric:tabular-nums;white-space:nowrap}
.f3 .kv{display:grid;grid-template-columns:78px minmax(0,1fr);gap:1px 8px}
.f3 .kv>span:nth-child(odd){color:var(--ink2)}
/* draft */
.f3 .en.draft{border-top:1.5px dashed var(--accent);margin-top:auto}
.f3 .en.draft .bd{background:var(--band);display:grid;grid-template-columns:400px minmax(0,1fr);gap:22px;padding:12px 16px}
.f3 .en.draft .mg b{color:var(--accent)}
.f3 .lab{font-size:12px;font-weight:650;color:var(--ink2);margin:0 0 3px;display:flex;gap:6px}
.f3 .lab span{font-weight:400}
.f3 .field{border:1px solid var(--line);border-radius:4px;background:var(--page);padding:5px 8px;min-height:30px;display:flex;align-items:center;gap:8px;margin-bottom:8px}
.f3 .field.foc{border-color:var(--accent);box-shadow:0 0 0 1px var(--accent)}
.f3 .field .r{margin-left:auto;color:var(--ink2)}
.f3 .chk{display:flex;gap:7px;align-items:center;padding:2px 0}
.f3 .chk>i{width:14px;height:14px;border:1.5px solid var(--ink2);border-radius:2px;background:var(--page);display:block;flex:none;position:relative}
.f3 .chk>i.on{background:var(--accent);border-color:var(--accent)}
.f3 .chk>i.on::after{content:"";position:absolute;left:3.5px;top:.5px;width:4px;height:8px;border:solid var(--onaccent);border-width:0 1.8px 1.8px 0;transform:rotate(45deg)}
.f3 .chk span{color:var(--ink2);margin-left:auto;padding-right:12px}
.f3 .g2{display:grid;grid-template-columns:1fr 1fr;gap:0 10px}
.f3 .cmd{background:var(--cmdbg);color:var(--cmdfg);border-radius:4px;padding:8px 10px;font-size:12px;line-height:1.5;font-variation-settings:"MONO" 1,"CASL" 0,"CRSV" 0;margin-bottom:8px}
.f3 .fl{font-size:12px;line-height:1.6;font-variation-settings:"MONO" 1,"CASL" 0,"CRSV" 0;color:var(--ink2);border:1.5px dashed var(--accent);border-radius:4px;padding:6px 10px;overflow-wrap:anywhere;margin-bottom:10px;background:var(--page)}
.f3 .fl b{color:var(--accent);font-weight:500}
.f3 .tk{background:var(--mark);padding:1px 3px;margin:0 -3px}
.f3 .ev{padding:5px 0;border-top:1px solid var(--line)}
.f3 .ev .mono{overflow-wrap:anywhere;display:block;font-size:12px}
.f3 .ev div{display:flex;gap:8px;align-items:flex-start;color:var(--ink2);font-size:12px}
.f3 .ev div>span:first-child{min-width:0}
.f3 .tag{border:1px solid var(--line);border-radius:3px;padding:0 5px;font-size:11.5px;color:var(--ink2);margin-left:auto;white-space:nowrap}
.f3 .pop{position:absolute;left:109px;bottom:86px;width:760px;background:var(--page);border:1px solid var(--ink);border-radius:4px;box-shadow:0 6px 18px rgba(0,0,0,.2);padding:4px}
.f3 .pop .o{display:grid;grid-template-columns:230px 200px 1fr;gap:10px;padding:4px 8px;border-radius:3px;white-space:nowrap}
.f3 .pop .o.on{background:var(--mark)}
.f3 .pop .o span:nth-child(2),.f3 .pop .o span:nth-child(3){color:var(--ink2)}
.f3 .pop .o.on span{color:var(--ink)}
.f3 .pop .ft{padding:5px 8px 3px;border-top:1px solid var(--line);margin-top:4px;color:var(--ink2);font-size:12px}
.f3 pre{margin:8px 0;font-size:11px;line-height:1.45;font-family:inherit;font-variation-settings:"MONO" 1,"CASL" 0,"CRSV" 0;background:var(--cmdbg);color:var(--cmdfg);border-radius:4px;padding:8px 10px;overflow:hidden}
.f3 pre .e{color:#FF8C82}.f3 pre .c{color:#A8ABCB}
.f3 .win.sm{width:580px;height:360px;grid-template-columns:minmax(0,1fr)}
.f3 .win.sm2{width:580px;height:auto;grid-template-columns:minmax(0,1fr)}
.f3 .win.sm .main,.f3 .win.sm2 .main{grid-template-rows:minmax(0,1fr) auto}
.f3 .win.sm2 .main{grid-template-rows:auto}
.f3 .win.sm .log,.f3 .win.sm2 .log{background:linear-gradient(to right,var(--band) 0 72px,var(--line) 72px 73px,transparent 73px)}
.f3 .win.sm .en,.f3 .win.sm2 .en{grid-template-columns:73px minmax(0,1fr)}
.f3 .win.sm .mg,.f3 .win.sm2 .mg{padding-left:12px}
.f3 .legend{display:flex;flex-direction:column;gap:0;width:520px;background:var(--page);color:var(--ink);font:13px/1.45 Recursive,sans-serif;font-variation-settings:"MONO" 0,"CASL" 0,"CRSV" 0;border:1px solid var(--line);border-radius:6px;overflow:hidden}
.f3 .legend .th{border-radius:0;border-top:1px solid var(--line);padding:6px 12px}
.f3 .legend .th:first-child{border-top:0}
`;

const rail = (cur) => `<div class="rail">
<div class="app"><b>shop-api</b><small>Laravel 13.2, package 13.3.0, Herd</small></div>
<div class="th${cur === 'all' ? ' on' : ''}">All of shop-api<span class="ct">3 new</span></div>
<h6>Needs attention</h6>
<div class="th${cur === 'Checkout' ? ' on' : ''}"><span class="c-bad">${st('bad')}</span>Checkout<span class="ct">2 problems</span></div>
<div class="th"><span class="c-warn">${st('warn')}</span>Notifications<span class="ct">39% covered</span></div>
<h6>Modules</h6>
${['Billing', 'Catalog', 'Inventory', 'Shipping', 'Users'].map((n) => `<div class="th"><span class="c-good">${st('good')}</span>${n}</div>`).join('')}
<div class="th c-acc" style="font-weight:650">+ New module</div>
<h6 style="margin-top:auto">Other apps</h6>
<div class="th"><span class="c-warn">${st('warn')}</span>crm<span class="ct">update to 13.3.0</span></div>
<div class="th"><span class="c-bad">${st('bad')}</span>legacy-portal<span class="ct">2 problems</span></div>
<div class="th c-acc" style="font-weight:650">+ Add app</div>
</div>`;
const hd = (title, extra = '', on = 'All') => `<div class="hd"><h4>${title}</h4>${extra}<div class="seg">${['All', 'Runs', 'Created', 'Checks', 'Changes'].map((f) => `<span${f === on ? ' class="on"' : ''}>${f}</span>`).join('')}</div></div>`;
const en = (time, kind, body, cls = '') => `<div class="en ${cls}"><div class="mg"><b>${time}</b>${kind}</div><div class="bd">${body}</div></div>`;
const day = (d) => `<div class="en day"><div class="mg">${d}</div><div class="bd"></div></div>`;
const prompt = (scope, line, hints) => `<div class="prompt"><div class="pl mono"><span class="pc">${scope}</span>${line}</div><div class="hints">${hints}</div></div>`;
const chips = `${['make', 'migrate', 'test', 'why', 'stats', 'health', 'open', 'add'].map((h) => `<span class="hint mono">${h}</span>`).join('')}<span class="r"><span class="key">Tab</span> lists commands, <span class="key">Up</span> earlier lines, <span class="key">Ctrl K</span> go to</span>`;

const eTests = en('15:58', 'run', `<div class="t"><span><span class="c-good">${st('good')}</span> <span class="mono">php artisan test Modules/Checkout/tests</span></span><span class="c-2">31 passed in 4.2 s</span><span class="r"><span class="lnk">Run again</span></span></div>`);
const eReq = en('16:31', 'created', `<div class="t"><span><span class="c-good">${st('good')}</span> <span class="mono">module:make-request ApplyCouponRequest Checkout</span></span><span class="c-2">1 file</span><span class="r"><span class="lnk">Open file</span></span></div>`);
const eCheck = en('09:12', 'check', `<div class="t"><span><span class="c-bad">${st('bad')}</span> Health check: <b>2 problems in Checkout</b>, 1 warning in Notifications</span><span class="r"><span class="lnk">Show</span></span></div>`);
const eChange = en('09:40', 'change', `<div class="t"><span>3 files changed in <b>Checkout</b>, not committed, branch <span class="mono">feature/coupons</span></span><span class="r"><span class="lnk">Show files</span></span></div>`);
const eFail = en('10:02', 'run', `<div class="t"><span><span class="c-bad">${st('bad')}</span> <span class="mono">module:migrate Checkout</span></span><span class="c-bad"><b>failed</b>, exit code 1, 0.8 s</span><span class="r"><span class="lnk">Show output</span><span class="lnk">Run again</span></span></div><div class="d">The column orders.coupon_id already exists. 1 migration is still waiting.</div>`);

export function home(t) {
  const rows = MODS.map((m) => `<tr><td class="l"><b>${m.n}</b></td><td class="l ${m.state === 'bad' ? 'c-bad' : m.state === 'warn' ? 'c-warn' : 'c-good'}">${st(m.state)} ${m.word}</td><td>${m.files}</td><td>${num(m.lines)}</td><td>${m.tests}</td><td><span class="mbar"><i style="width:${m.cov}%"></i></span>${m.cov}%</td><td>${m.uses}</td><td>${m.usedBy}</td></tr>`).join('');
  return `<div class="mk f3 ${t}"><div class="win">
${rail('all')}
<div class="main">
${hd('All of shop-api')}
<div class="log">
${en('Now', 'live', `<div><b>7 modules, all on.</b> <b class="c-bad">2 problems in Checkout.</b> <span class="c-warn"><b>1 warning in Notifications.</b></span> 1 migration waiting. <span class="c-2">551 files, 41,212 lines, 307 tests, 71% covered.</span></div>
<table class="t"><tr><th class="l">Module</th><th class="l">State</th><th>Files</th><th>Lines</th><th>Tests</th><th>Covered</th><th>Uses</th><th>Used by</th></tr>${rows}</table>`, 'stand')}
${day('Yesterday')}${eReq}${day('Today')}${eCheck}${eChange}${eFail}
</div>
${prompt('shop-api', '<i class="caret"></i><span class="ph">Type a command, or a module name to open it</span>', chips)}
</div></div></div>`;
}

export function detail(t) {
  const kinds = [['Tests', 31], ['Views', 12], ['Migrations', 9], ['Requests', 7], ['Controllers', 6], ['Events', 5], ['Listeners', 5], ['Models', 4], ['Jobs', 3], ['Policies', 2]];
  return `<div class="mk f3 ${t}"><div class="win">
${rail('Checkout')}
<div class="main">
${hd('Checkout', '<span class="mono c-2">Modules/Checkout</span><span class="tog"><i></i>On</span>')}
<div class="log">
${en('Now', 'live', `<div class="row"><span class="c-bad">${st('bad')}</span><span><b>Checkout and Shipping use each other.</b> Rule: no circular use.</span><span class="n lnk">why</span></div>
<div class="row"><span class="c-bad">${st('bad')}</span><span><b>3 files read Billing\\Models\\Invoice directly.</b> Rule: contracts and events only.</span><span class="n lnk">why</span></div>
<div class="g3">
<div><h6>Inside: 84 files, 6,412 lines</h6><div class="kl">${kinds.map(([n, v]) => `<span>${n}</span><span>${v}</span>`).join('')}</div></div>
<div><h6>Uses</h6>
<div class="row"><span>Catalog</span><span class="n">14</span></div>
<div class="row"><span>Billing <span class="c-bad">${st('bad')} 3 read a model</span></span><span class="n">6</span></div>
<div class="row"><span>Users</span><span class="n">3</span></div>
<div class="row"><span>Shipping <span class="c-bad">${st('bad')} circular</span></span><span class="n">2</span></div>
<h6 style="margin-top:6px">Used by</h6>
<div class="row"><span>Shipping <span class="c-bad">${st('bad')} circular</span></span><span class="n">4</span></div>
<div class="row"><span>Notifications</span><span class="n">2</span></div></div>
<div><h6>Facts</h6><div class="kv"><span>Tests</span><span>31, 54% covered</span><span>Routes</span><span>18: 12 API, 6 web</span><span>Migrations</span><span>9 ran, <b class="c-warn">1 waiting</b></span><span>Last change</span><span>2 days ago, 14 commits in 30 days</span></div>
<div class="acts" style="margin-top:8px"><span class="btn">Run migrate</span><span class="btn q">Run tests</span><span class="btn q">Open in editor</span></div></div>
</div>`, 'stand')}
${day('Yesterday')}${eTests}${eReq}${day('Today')}
${en('09:12', 'check', `<div class="t"><span><span class="c-bad">${st('bad')}</span> Health check: <b>2 problems</b>, both listed above</span><span class="r"><span class="lnk">Show</span></span></div>`)}
${eChange}${eFail}
</div>
${prompt('shop-api, in Checkout', '<i class="caret"></i><span class="ph">Commands here apply to Checkout: make model Coupon -m -p</span>', chips)}
</div></div></div>`;
}

export function gen(t) {
  return `<div class="mk f3 ${t}"><div class="win">
${rail('Checkout')}
<div class="main">
${hd('Checkout', '<span class="mono c-2">Modules/Checkout</span><span class="tog"><i></i>On</span>')}
<div class="log">
${en('Now', 'live', `<div class="t"><span><b class="c-bad">${st('bad')} 2 problems.</b> 84 files, 31 tests, 1 migration waiting.</span><span class="r"><span class="lnk">Show all</span></span></div>`, 'stand')}
${day('Today')}${eChange}${eFail}
<div class="en draft"><div class="mg"><b>Draft</b>not run yet</div><div class="bd">
<div>
<div class="lab">Generator</div><div class="field"><b>Model</b><span class="mono c-2">module:make-model</span><span class="r">in Checkout</span></div>
<div class="lab">Model name</div><div class="field foc">Coupon</div>
<div class="lab">Fillable attributes <span>separated by commas</span></div><div class="field mono">code,percent_off,expires_at</div>
<div class="lab">Create with the model</div><div class="g2">
<div class="chk"><i class="on"></i>Migration<span class="mono">-m</span></div><div class="chk"><i></i>Controller<span class="mono">-c</span></div>
<div class="chk"><i class="on"></i>Policy<span class="mono">-p</span></div><div class="chk"><i></i>Request<span class="mono">-r</span></div>
<div class="chk"><i></i>Factory<span class="mono">-f</span></div><div class="chk"><i></i>Resource<span class="mono">-R</span></div>
<div class="chk"><i></i>Seeder<span class="mono">-s</span></div><div class="chk"><i></i>All of these<span class="mono">-a</span></div>
<div class="chk"><i></i>Use #[Fillable] attributes</div><div class="chk"><i></i>Overwrite existing files</div>
</div></div>
<div>
<div class="lab">Command that will run</div><div class="cmd">${cmdHtml(CMD)}</div>
<div class="lab">Files this creates</div><div class="fl">${FILES.map(([d, f]) => `${wb(d)}<b>${f}</b>`).join('<br>')}</div>
<div class="acts"><span class="btn pri">Create 3 files <span class="key">Enter</span></span><span class="btn q">Copy command</span><span class="btn q">Discard <span class="key">Esc</span></span></div>
<div class="d" style="margin-top:10px">The line below and this form are the same thing. Change either one and the other follows.</div>
</div>
</div></div>
</div>
${prompt('shop-api, in Checkout', '<span><b>make model</b> <span class="tk">Coupon</span> --migration --policy --fillable=code,percent_off,expires_at</span><i class="caret"></i>', `<span><span class="key">Enter</span> creates 3 files</span><span><span class="key">Tab</span> moves into the form</span><span><span class="key">Esc</span> discards the draft</span><span class="r">Short flags work too: <span class="mono">make model Coupon -m -p</span></span>`)}
</div></div></div>`;
}

const evRow = ([f, l, sym, kind]) => `<div class="ev"><span class="mono">${wb(f)}:${l}</span><div><span>uses ${wb(sym)}</span><span class="tag">through a ${kind}</span></div></div>`;
export function sig(t) {
  const opts = [['<b>mig</b>rate &lt;module&gt;', 'module:migrate', 'Run the waiting migrations of a module'], ['<b>mig</b>rate status &lt;module&gt;', 'module:migrate-status', 'Show which migrations ran'], ['<b>mig</b>rate rollback &lt;module&gt;', 'module:migrate-rollback', 'Undo the last batch'], ['<b>mig</b>rate refresh &lt;module&gt;', 'module:migrate-refresh', 'Roll back, then migrate again'], ['<b>mig</b>rate fresh', 'module:migrate-fresh', 'Drop all tables, then migrate'], ['make <b>mig</b>ration &lt;name&gt;', 'module:make-migration', 'Create a migration file']];
  return `<div class="mk f3 ${t}"><div class="win">
${rail('all')}
<div class="main">
${hd('All of shop-api')}
<div class="log">
${en('Now', 'live', `<div class="t"><span><b>7 modules, all on.</b> <b class="c-bad">2 problems in Checkout.</b> <b class="c-warn">1 warning in Notifications.</b> 1 migration waiting.</span><span class="r"><span class="lnk">Show all</span></span></div>`, 'stand')}
${day('Today')}
${en('10:21', 'answer', `<div class="t"><span class="mono"><b>why checkout shipping</b></span><span class="r"><span class="lnk">Open all 6 in editor</span><span class="lnk">Copy as text</span><span class="lnk">Pin</span></span></div>
<div style="margin:4px 0 8px"><b class="c-bad">${st('bad')} Checkout and Shipping use each other.</b> Rule broken: no circular use. Both sides go through contracts and events, so access is fine. The direction is the problem.</div>
<div class="g2" style="gap:0 26px"><div><h6>Checkout uses Shipping: 2 references in 2 files</h6>${WHY.a.map(evRow).join('')}</div><div><h6>Shipping uses Checkout: 4 references in 4 files</h6>${WHY.b.map(evRow).join('')}</div></div>`)}
</div>
<div class="pop">${opts.map(([a, b, c], i) => `<div class="o${i ? '' : ' on'}"><span class="mono">${a}</span><span class="mono">${b}</span><span>${c}</span></div>`).join('')}<div class="ft"><span class="key">Tab</span> completes, <span class="key">Enter</span> runs, <span class="key">Esc</span> closes. Every command shows the artisan command it runs.</div></div>
${prompt('shop-api', '<span>mig</span><i class="caret"></i>', chips)}
</div></div></div>`;
}

export function empty(t) {
  return `<div class="mk f3 ${t}"><div class="win sm"><div class="main">
<div class="log" style="justify-content:flex-start">
${en('Now', 'start', `<div style="font-size:16px;font-weight:650;margin-bottom:4px">No apps yet</div><div style="margin-bottom:12px">Add the folder of a Laravel app that uses nwidart/laravel-modules. Its modules become threads on the left and everything you run is kept here. Nothing in the folder is changed.</div><div class="acts"><span class="btn pri">Choose folder <span class="key">Ctrl O</span></span></div><div class="d" style="margin-top:8px">or drop a folder on this window, or type the path below</div>`)}
</div>
${prompt('', '<span><b>add</b> </span><i class="caret"></i><span class="ph" style="margin-left:6px">~/code/shop-api</span>', '<span><span class="key">Enter</span> adds the app</span>')}
</div></div></div>`;
}
export function fail(t) {
  return `<div class="mk f3 ${t}"><div class="win sm2"><div class="main">
<div class="log">
${en('10:02', 'run', `<div class="t"><span><span class="c-bad">${st('bad')}</span> <b>Migrate Checkout failed</b></span><span class="c-2">exit code 1, 0.8 s</span></div>
<pre>${FAIL_OUT.filter(([, l]) => l).map(([c, l]) => `<span class="${c}">${l.replace(/</g, '&lt;')}</span>`).join('\n')}</pre>
<div style="margin-bottom:8px"><b>The column orders.coupon_id already exists.</b> The migration stopped there and nothing after it ran. 9 migrations ran earlier, 1 is still waiting.</div>
<div class="acts" style="margin-bottom:4px"><span class="btn pri">Open the migration</span><span class="btn">Run again</span><span class="btn q">Copy output</span></div>`)}
</div></div></div></div>`;
}
const legend = (t) => `<div class="mk f3 ${t}"><div class="legend">
<div class="th"><span class="c-good">${st('good')}</span>Billing<span class="ct">Good: check mark</span></div>
<div class="th"><span class="c-warn">${st('warn')}</span>Notifications<span class="ct">Warning: triangle and the reason</span></div>
<div class="th"><span class="c-bad">${st('bad')}</span>Checkout<span class="ct">Problem: diamond and the count</span></div>
<div class="th c-off"><span>${st('off')}</span><s>Reports</s><span class="ct">Off: dashed circle, struck name, "off"</span></div>
<div class="th on">Checkout<span class="ct">Highlighter: the thread you are in</span></div>
</div></div>`;

export const concept = 'The app is a logbook you can talk to. Each app has a log and each module a thread; you type a short command or pick one, and the answer is written into the log as a block that stays, can be run again, and can be pinned. The app writes entries too, when files, git or health change.';

export function body() {
  return `
<h1>F3 Logbook</h1>
<p class="dim">A command line that remembers and answers in structure. Mock data is invented: app shop-api, 7 modules.</p>
<ul class="toc"><li><a href="#concept">Concept</a></li><li><a href="#ia">Information architecture</a></li><li><a href="#lang">Design language</a></li><li><a href="#mocks">Mock screens</a></li><li><a href="#egui">egui feasibility</a></li><li><a href="#weak">Weaknesses</a></li></ul>

<h2 id="concept">1. Concept</h2>
<p>${concept}</p>
<p>What is different to use: there are no screens to go to. A question (<code>why checkout shipping</code>), an action (<code>migrate checkout</code>) and a creation (<code>make model Coupon -m -p</code>) all happen in the same place and leave the same kind of trace. The line you type and the form you click are two views of one draft, so a keyboard user never opens a form and a mouse user never types a flag. It is built for people who already live in a terminal: the words are the artisan words, and every entry shows the exact command that ran.</p>

<h2 id="ia">2. Information architecture</h2>
<h3>Navigation model</h3>
${diagram(1040, 200, [
    [0, 20, 170, 150, 'Rail', 'apps|threads of the app:|all, needs attention,|one per module'],
    [230, 20, 330, 100, 'Log of the thread', 'standing block: live summary, stays on top|entries by time: run, created, check, change,|answer', true],
    [230, 140, 330, 48, 'Command line', 'verb, thing, name, options. Tab lists'],
    [620, 20, 190, 46, 'Draft', 'form and line, before it runs'],
    [620, 80, 190, 46, 'Entry, expanded', 'output, evidence, table'],
    [870, 20, 150, 46, 'Pinned', 'entries kept on top'],
  ], [[170, 70, 230, 70, 'pick'], [395, 140, 395, 120, ''], [560, 43, 620, 43, 'typing'], [560, 103, 620, 103, 'Enter'], [810, 103, 870, 50, 'pin']], 'F3 navigation model')}
<p>Two things move: which thread you are in (rail, or type a module name) and where you are in its log (scroll, or ${k('Alt Up')} and ${k('Alt Down')}). Everything else is an entry. The standing block at the top of each thread is live and never scrolls away: it is the app summary on "All", and the module summary on a module thread. Entries below it are snapshots with their time.</p>

<h3>Screen inventory</h3>
${table(['Surface', 'Holds'], [
    ['Rail', 'The current app with versions and PHP runner. Threads: "All", then modules that need attention with the reason, then the other modules with their state mark. Other apps with their state. "New module" and "Add app".'],
    ['Log, "All" thread, home', 'Standing block: one sentence of state, app totals, the module table. Below: entries of every module by time. Filters: All, Runs, Created, Checks, Changes.'],
    ['Log, module thread', 'Standing block: the problems in words, what is inside, uses and used by, facts, the three most likely actions. Below: the entries of that module.'],
    ['Entry: run', 'The command, result, time taken. Expanded: the output, a one-sentence cause on failure, "Run again".'],
    ['Entry: created', 'The generator command and the files it wrote, each a link to the editor.'],
    ['Entry: check', 'Health results. Expanded: every check with pass, warning or broken.'],
    ['Entry: change', 'Files that changed on disk or in git, grouped by module.'],
    ['Entry: answer', 'The reply to a question: <code>why a b</code> (evidence both ways), <code>stats</code> (table), <code>graph</code> (a small layered drawing), <code>routes checkout</code>, <code>rules</code>.'],
    ['Draft', 'A command being composed: the form on the left, the exact artisan command and the files on the right. Becomes an entry when run.'],
    ['Command line', 'One line. Completion lists verbs, then things, then modules and options, each with the artisan command it maps to.'],
    ['Settings', 'Typed too: <code>set editor</code>, <code>set php</code>, <code>theme dark</code>; also reachable as a normal panel.'],
  ])}

<h3>Flows</h3>
${flows([
    { t: '(a) Add an app.', n: '3 actions', s: ['Click "Add app" in the rail, or type <code>add</code> and ' + k('Enter') + '. The system folder dialog opens.', 'Choose the folder.', 'A draft shows what was found: Laravel 13.2, package 13.3.0, 7 modules, PHP runs with Herd (changeable). Press ' + k('Enter') + '.'], alt: 'Typing <code>add ~/code/shop-api</code> and Enter twice, or dropping a folder: 2 actions.' },
    { t: '(b) Find the module with a problem and see what is wrong.', n: '0 actions to find, 1 to read', s: ['Open the app. The standing block says "2 problems in Checkout" and the rail lists Checkout under "Needs attention".', 'Click Checkout. Its standing block states both problems in words; "why" on either line writes the evidence into the log.'] },
    { t: '(c) Create a model with a migration and a policy in an existing module.', n: '2 actions typed, 7 with the mouse', s: ['Type <code>make model Coupon in checkout -m -p</code> (35 characters, with completion about 20 key presses). The draft fills in as you type and shows the command and the 3 files.', 'Press ' + k('Enter') + '.'], alt: 'With the mouse: click "make", choose Model, choose Checkout, type the name, tick Migration, tick Policy, click "Create 3 files". 7 actions, 6 inside the Checkout thread.' },
    { t: '(d) Run migrations for one module and read the output.', n: '2 actions', s: ['Type <code>migrate checkout</code>, or click "Run migrate" in the Checkout standing block.', 'Press ' + k('Enter') + '. The entry appears and the output streams into it.'] },
    { t: '(e) See why two modules depend on each other.', n: '2 actions', s: ['Type <code>why checkout shipping</code>.', 'Press ' + k('Enter') + '. The answer lists both directions, file by file.'], alt: 'When the link breaks a rule, the "why" link on the problem line does it in 1.' },
  ])}

<h3>Keyboard model</h3>
${table(['Keys', 'Does'], [
    ['Typing', 'Always goes to the command line; it keeps the focus'],
    [k('Tab') + ' ' + k('Shift Tab'), 'Complete the word, cycle the suggestions; inside a draft, move through the form'],
    [k('Enter') + ' ' + k('Esc'), 'Run the line or the draft, discard the draft or close the list'],
    [k('Up') + ' ' + k('Down'), 'Earlier and later lines you typed'],
    [k('Alt Up') + ' ' + k('Alt Down'), 'Move the selection through the entries'],
    [k('Space') + ' on an entry', 'Expand or collapse'],
    [k('Ctrl R') + ' ' + k('Ctrl C') + ' ' + k('Ctrl P'), 'On the selected entry: run again, copy the command, pin'],
    [k('Ctrl 1') + ' to ' + k('Ctrl 9'), 'Switch to the first nine threads'],
    [k('Ctrl K'), 'Go to an app, module or class by name'],
    [k('Ctrl O'), 'Add an app'],
  ])}
<p>Nothing in this direction needs the pointer. It is the only one of the three where the fastest path and the keyboard path are the same path.</p>

<h3>How it scales</h3>
<ul>
<li><b>1 app, 5 modules.</b> The rail is short and the log is quiet: a handful of entries a day. The standing block with the module table is most of what you see.</li>
<li><b>10 apps, 60 modules each.</b> The rail shows the current app only; other apps collapse to one line each with their state. Within the app, "Needs attention" stays on top and the other 55 modules sit behind a filter field. The module table in the standing block shows the rows that need attention and "Show all 60". Changes are grouped ("14 files changed in 3 modules") so the log does not flood. Reaching a module is typing three letters of its name. What does not scale is reading: nobody scrolls a day of entries for 60 modules, so the filters and the standing block must carry the load.</li>
</ul>

<h2 id="lang">3. Design language</h2>
<p>The reference is a ship's log or a lab notebook: the time in the margin, entries in blue-black ink, ruled lines between them, and a yellow highlighter for where you are. Ink is blue-black in light and the page is deep indigo in dark; there is no neutral grey anywhere. The highlighter has one meaning, "you are here" (current thread, current filter, current suggestion, the name being typed), and never means a state.</p>
<h3>Colour</h3>
${tokenTable(tokens)}
<h4>Contrast</h4>
${contrastTable('F3', tokens, pairs)}
<h3>Type</h3>
${table(['Role', 'Face', 'Size, weight'], [
    ['Interface text, entries', 'Recursive Sans Linear (SIL OFL)', '13 px, 400. Emphasis 650'],
    ['Commands, paths, output, key hints', 'Recursive Mono Linear, same family', '13 px in entries, 15 px in the command line, 11 and 12 px in output'],
    ['Thread title', 'Recursive Sans Linear', '18 px, 650'],
    ['Margin: time and kind', 'Recursive Sans Linear, tabular figures', '13 px 650 and 12 px 400'],
  ])}
<p>Scale: 11, 12, 13, 15, 18. One family in two cuts: a command inside a sentence keeps the same height and weight as the words around it, which matters when half of every line is a command. Ship the static Sans Linear and Mono Linear files.</p>
<h3>Shape, space, motion</h3>
${table(['Token', 'Value'], [
    ['Spacing unit', '4 px. Entry padding 9 px by 16 px. Margin column 92 px.'],
    ['Corner radius', '0 on entries (they are ruled lines, not cards). 4 px on fields, buttons, thread rows and command blocks. 6 px on the window.'],
    ['Stroke', 'Rules 1 px. The command line sits under a 2 px ink rule. A draft starts with a 1.5 px dashed accent rule.'],
    ['Elevation', 'Flat, with one exception: the completion list, 0 6 px 18 px black at 20%.'],
    ['Icons', 'The four state marks and the caret. Actions are words.'],
    ['Density', 'Medium. 13 px text; a collapsed entry is one line, 38 px.'],
    ['Motion', 'A new entry appears at the bottom and the log scrolls to it in 120 ms. Output streams line by line. The caret blinks once a second and stops under reduced motion. Nothing else moves.'],
  ])}
<h3>Module identity and state</h3>
<p>A module is its name: typed in lower case it is a command, in the rail it is a thread, in an entry it is bold. No colour per module. State is a mark and words, in the rail and in every entry:</p>
${shot(legend('t-auto'))}

<h2 id="mocks">4. Mock screens</h2>
<p class="dim">The switch at the top right of this page changes the theme of the large mocks. The pair below the home screen is fixed, one light and one dark.</p>
<h3>Home: the log of shop-api</h3>
${shot(home('t-auto'), 'The standing block answers "how is the app" in one sentence and lists the modules. Below it, what happened: one entry from yesterday, three from today, the last one a failed run. Typing goes to the line at the bottom.')}
<h4>Both themes</h4>
${pair(home)}
<h3>One module: the Checkout thread</h3>
${shot(detail('t-auto'), 'The standing block is the module page: problems in words, what is inside, uses and used by, facts and actions. The entries below are only those of Checkout. Commands typed here apply to Checkout.')}
<h3>Generator: module:make-model</h3>
${shot(gen('t-auto'), 'Typed in the Checkout thread. The draft shows the real options of the command; the line and the form stay in step. Enter turns the draft into a "created" entry with the three files.')}
<h3>Signature: ask, and the answer stays</h3>
${shot(sig('t-auto'), 'The answer to "why checkout shipping" is an entry like any other. Below, a new line is being typed: three letters list every migrate command with the artisan command it runs.')}
<h3>Empty and failure</h3>
<div class="small2">${empty('t-auto')}${fail('t-auto')}</div>
<p class="cap">Left: no apps yet. Right: the run entry expanded after <code>module:migrate Checkout</code> failed.</p>

<h2 id="egui">5. Feasibility in egui</h2>
${table(['Part', 'Cost or risk', 'Simpler fallback'], [
    ['Command line with completion', 'Low to medium. A single-line TextEdit plus a floating Area for the list. Highlighting parts of the line needs a custom text layouter, which TextEdit supports.', 'No highlighting in the line; the draft shows what was understood.'],
    ['Line and form in step', 'Medium, and it is logic, not drawing: a parser, a serialiser and care with the text cursor when the form rewrites the line.', 'One way only: the line fills the form; once the form is touched the line becomes read-only.'],
    ['Log with entries of different heights', 'Medium. egui virtualises rows of equal height; unequal heights need a cache of measured heights.', 'Keep the last 200 entries in the list, the rest behind "Earlier".'],
    ['Streaming output', 'Medium. Read the child process on a thread, strip or parse ANSI colour codes, append lines.', 'Strip colours; show plain text.'],
    ['Margin column and rules', 'Low. Two rectangles and lines.', 'None needed.'],
    ['Highlighter fill behind text', 'Low. A rectangle under the laid-out text.', 'A bold weight instead of a fill.'],
    ['Font', 'Low. Recursive is a variable font and egui does not read variable axes; bundle the static Sans Linear and Mono Linear files in Regular and Bold.', 'None needed.'],
    ['The small graph answer', 'Medium, the same layered layout as F1 in small.', 'Leave it out; answer <code>graph</code> with a table of links.'],
  ])}

<h2 id="weak">6. Honest weaknesses</h2>
<ul>
<li><b>No picture of the structure.</b> The app tells you there is a cycle; it does not show you the shape of the codebase. For "understand an app" it is the weakest of the three.</li>
<li><b>It asks, the others show.</b> You must know that <code>why</code> exists. Suggestion chips and ${k('Tab')} help, but recall is slower than recognition for someone who opens the app once a week.</li>
<li><b>Slowest with a mouse.</b> Creating a model is 7 clicks against 6 in F1 and F2.</li>
<li><b>Entries go stale.</b> An answer from this morning shows this morning's numbers. Only the standing block is live, and that distinction has to be learned.</li>
<li><b>The log gets loud</b> on a busy day in a large app. Grouping and filters contain it but do not remove it.</li>
<li><b>Comparing modules</b> means asking for a table every time, or keeping one pinned.</li>
<li><b>Charts fit poorly.</b> Live metrics later would need a place outside the log.</li>
<li><b>Familiar look.</b> A rail and a stream of entries reads as a chat app. It will not be mistaken for an admin panel, but it is not a look of its own the way F1 is.</li>
</ul>`;
}
