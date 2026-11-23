# Shared helpers for the Mortise spec pages. Everything a page needs is inlined at build time.
import html, json, os, re

# The built pages go to docs/spec, beside this folder.
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'spec')

# ---------------------------------------------------------------- tokens
def _rgb(h):
    h = h.lstrip('#'); return [int(h[i:i+2], 16) for i in (0, 2, 4)]
def mix(a, b, t):
    ra, rb = _rgb(a), _rgb(b)
    return '#%02X%02X%02X' % tuple(round(ra[i] + (rb[i] - ra[i]) * t) for i in range(3))
def lum(h):
    c = [x / 255 for x in _rgb(h)]
    c = [x / 12.92 if x <= 0.03928 else ((x + 0.055) / 1.055) ** 2.4 for x in c]
    return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
def cr(a, b):
    la, lb = lum(a), lum(b); return (max(la, lb) + 0.05) / (min(la, lb) + 0.05)

COLOR = {  # name: (light, dark, role)
 'frame':   ('#D1D8E2', '#0F1216', 'Title bar and app rail'),
 'panel':   ('#E5EAF0', '#15191E', 'Module index, workspace header strip, palette and dialog footer'),
 'surface': ('#F9FAFC', '#1D2229', 'Workspace, selected row, inputs, overlays'),
 'well':    ('#DBE1E9', '#11151B', 'Bar tracks, keycaps, hover fill, selected list row, segmented track'),
 'ink':     ('#1A1E25', '#E5EBF4', 'Primary text, bars, peg, focus ring'),
 'ink-2':   ('#48515C', '#A1AEBE', 'Secondary text, code stamps, outlines on frame'),
 'ink-3':   ('#5B6570', '#8895A5', 'Placeholders, tab key hints, disabled labels. Only on surface and panel'),
 'line':    ('#BCC6D3', '#2E353E', 'Hairlines inside a panel, pressed fill'),
 'line-2':  ('#747F8C', '#657180', 'Panel joints, control outlines on panel, surface and well'),
 'key':     ('#1A1E25', '#E5EBF4', 'Primary button fill, tooltip fill'),
 'key-fg':  ('#F9FAFC', '#15191E', 'Label on key'),
 'good':    ('#2C6A37', '#84CC90', 'Good finding state, runtime Up'),
 'warn':    ('#855600', '#E5B152', 'Warning state, runtime Out of service'),
 'bad':     ('#B02A20', '#F28B7F', 'Problem state, runtime Down, destructive button'),
 'bad-fg':  ('#F9FAFC', '#15191E', 'Label on a filled destructive button'),
 'off':     ('#5B6570', '#8895A5', 'Module turned off, runtime Unknown. Same value as ink-3, same limits'),
 'bad-soft':('#F8E2DD', '#3A201B', 'Problem alert background'),
 'code-bg': ('#1A1E25', '#0B0E12', 'Runner drawer and command preview'),
 'code-fg': ('#E5EBF4', '#E5EBF4', 'Runner text'),
 'code-dim':('#93A0B0', '#93A0B0', 'Runner secondary text'),
 'code-ok': ('#8FD49B', '#8FD49B', 'Runner success'),
 'code-err':('#F59A8E', '#F59A8E', 'Runner error'),
}
for _n, _t in (('key-hover', .16), ('key-press', .30)):
    COLOR[_n] = (mix(COLOR['key'][0], COLOR['surface'][0], _t), mix(COLOR['key'][1], COLOR['surface'][1], _t),
                 'Primary button, %s' % ('hover' if 'hover' in _n else 'pressed'))
ALPHA = {  # colours with alpha, kept apart because egui wants them premultiplied
 'scrim':  ('rgba(26,30,37,.42)', 'rgba(6,6,8,.6)', 'Layer under palette and dialog'),
 'shadow': ('rgba(10,13,18,.30)', 'rgba(0,0,0,.50)', 'Overlay shadow colour'),
}
TYPE = [  # name, px size, px line, weight, family, use
 ('title', 18, 22, 700, 'sans', 'Workspace title'),
 ('figure', 19, 26, 600, 'sans', 'Stat figure'),
 ('palette-input', 15, 20, 400, 'sans', 'Command palette query'),
 ('overlay-title', 15, 20, 700, 'sans', 'Dialog title, runner title'),
 ('row-name', 13, 17, 600, 'sans', 'Module name in the index, brand in the title bar'),
 ('body', 12.5, 18, 400, 'sans', 'Body text, table cells, list lines'),
 ('tab', 12.5, 18, 500, 'sans', 'Tab label, 600 when selected'),
 ('control', 12, 12, 600, 'sans', 'Button label'),
 ('label', 11.5, 15, 600, 'sans', 'Field label, state word, table header'),
 ('secondary', 11.5, 15, 400, 'sans', 'Secondary text, hints'),
 ('code', 12, 19, 400, 'mono', 'Command preview, file tree'),
 ('runner', 11.5, 18, 400, 'mono', 'Runner output'),
 ('stamp', 10.5, 16, 600, 'mono', 'Module code stamp, letter spacing 0.03 em'),
 ('keycap', 10.5, 16, 500, 'mono', 'Key hint'),
]
SPACE = [2, 4, 6, 8, 12, 16, 24, 28]
SIZE = {
 'titlebar-h': 34, 'rail-w': 52, 'rail-tile': 34, 'index-w': 264, 'index-w-min': 220, 'index-w-max': 320,
 'index-strip-w': 76, 'index-row-h': 40, 'index-row-compact-h': 26, 'control-h': 26, 'control-sm-h': 22,
 'field-h': 28, 'tab-h': 31, 'table-head-h': 26, 'table-row-h': 28, 'list-line-h': 27, 'palette-w': 620,
 'palette-input-h': 44, 'palette-row-h': 32, 'dialog-w': 440, 'toast-w': 320, 'tooltip-max-w': 240,
 'drawer-h-default': 200, 'drawer-h-min': 120, 'drawer-collapsed-h': 28, 'scrollbar-w': 10, 'scrollbar-thumb-w': 6,
 'scrollbar-thumb-min': 24, 'state-mark': 12, 'stamp-w': 34, 'stamp-h': 18, 'checkbox': 14, 'icon': 16,
 'icon-in-control': 13, 'peg': 5, 'bar-h': 6, 'window-min-w': 960, 'window-min-h': 600, 'hit-min': 24,
}
RADIUS = {'joint': 0, 'stamp': 2, 'control': 3, 'overlay': 4, 'window': 8}
STROKE = {'hairline': 1, 'joint': 1, 'icon': 1.5, 'checkbox': 1.5, 'focus-ring': 2, 'focus-offset': 1, 'problem-link': 2}
ELEV = {
 'none': 'Workbench, panels, boxes. No shadow.',
 'overlay': 'Palette and dialog. 1 px ink outline, shadow x 0, y 14, blur 34, colour shadow.',
 'toast': 'Toast. 1 px ink outline, shadow x 0, y 6, blur 16, colour shadow.',
 'tooltip': 'Tooltip. Key fill, no outline, no shadow.',
}
MOTION = {
 'selection': '0 ms. Row, tab, palette row and theme change redraw in the same frame.',
 'overlay-open': '0 ms. Palette, dialog and tooltip appear and go without a transition.',
 'drawer': '120 ms, ease out. Height of the runner drawer.',
 'created-file': '600 ms, linear. Well fill behind a new file fades to none.',
 'toast-in': '120 ms, ease out. Slides 8 px up.',
 'toast-out': '160 ms, ease in. Fades.',
 'toast-dwell': '4000 ms for done and note. A failed toast stays until closed.',
 'tooltip-delay': '500 ms after the pointer rests. 0 ms on keyboard focus.',
 'reduced': 'With reduced motion every duration above is 0 ms, except toast-dwell and tooltip-delay.',
}
def tokens_json():
    return {
     'name': 'Mortise, Slate', 'version': 'phase-0',
     'color': {k: {'light': v[0], 'dark': v[1]} for k, v in COLOR.items()},
     'colorAlpha': {k: {'light': v[0], 'dark': v[1]} for k, v in ALPHA.items()},
     'font': {'sans': 'Asap', 'mono': 'Inconsolata', 'weights': {'sans': [400, 500, 600, 700], 'mono': [400, 500, 600]}},
     'type': {n: {'size': s, 'line': l, 'weight': w, 'family': f} for n, s, l, w, f, _ in TYPE},
     'space': SPACE, 'size': SIZE, 'radius': RADIUS, 'stroke': STROKE,
     'elevation': {'overlay': {'x': 0, 'y': 14, 'blur': 34, 'color': 'shadow', 'outline': 'ink'},
                   'toast': {'x': 0, 'y': 6, 'blur': 16, 'color': 'shadow', 'outline': 'ink'}},
     'motionMs': {'selection': 0, 'overlayOpen': 0, 'drawer': 120, 'createdFile': 600, 'toastIn': 120,
                  'toastOut': 160, 'toastDwell': 4000, 'tooltipDelay': 500},
     'rules': ['ink-3 and off: only on surface and panel, never on frame or well',
               'warn as text: never on frame', 'line-2 as a control outline: never on frame, use ink-2 there',
               'colour only for state: good, warn, bad, off'],
    }

def css_vars(i):
    v = ';'.join('--%s:%s' % (k, c[i]) for k, c in COLOR.items())
    v += ';' + ';'.join('--%s:%s' % (k, c[i]) for k, c in ALPHA.items())
    return v

# ---------------------------------------------------------------- stylesheet
DOC_CSS = r'''
:root{color-scheme:dark}
*{box-sizing:border-box}
html,body{margin:0;padding:0;background:#000;color:#fff}
body{font:15px/1.6 system-ui,-apple-system,"Segoe UI",Roboto,Helvetica,Arial,sans-serif;-webkit-font-smoothing:antialiased}
.doc{max-width:1320px;margin:0 auto;padding:20px 20px 80px}
nav.top{display:flex;flex-wrap:wrap;gap:6px 18px;padding-bottom:12px;border-bottom:1px solid #2a2a2a;margin-bottom:22px;font-size:14px}
nav.top a{color:#a8a8a8;text-decoration:none}
nav.top a:hover{color:#fff;text-decoration:underline}
nav.top a.cur{color:#fff;font-weight:600}
.doc h1{font-size:26px;line-height:1.2;font-weight:650;margin:0 0 8px}
.doc h2{font-size:19px;line-height:1.3;font-weight:650;margin:40px 0 10px;padding-top:16px;border-top:1px solid #2a2a2a}
.doc h3{font-size:15px;line-height:1.35;font-weight:650;margin:26px 0 8px}
.doc h3 .sid,.doc a.sid{font:600 12.5px ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;color:#a8a8a8;text-decoration:none}
.doc h3 .sid{margin-right:8px}
.doc a.sid{text-decoration:underline;text-decoration-color:#555;text-underline-offset:3px}
.doc p,.doc li{max-width:86ch}
.doc p{margin:0 0 10px}
.doc ul,.doc ol{padding-left:20px;margin:0 0 12px}
.doc li{margin-bottom:4px}
.doc a{color:#fff}
.dim{color:#a8a8a8}
.doc code{font:13px ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;background:#151515;border:1px solid #262626;border-radius:3px;padding:0 4px}
.doc pre.json{font:12px/1.5 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;background:#0d0d0d;border:1px solid #262626;padding:12px;overflow:auto;max-height:420px;margin:8px 0 16px}
table.spec{width:100%;border-collapse:collapse;font-size:14px;margin:8px 0 16px}
table.spec th,table.spec td{text-align:left;vertical-align:top;padding:7px 14px 7px 0;border-bottom:1px solid #2a2a2a}
table.spec th{color:#a8a8a8;font-weight:500}
table.spec td.num{font-variant-numeric:tabular-nums;white-space:nowrap}
table.spec code{white-space:nowrap}
table.spec td.fail{color:#ff8a80;font-weight:600}
.tscroll{overflow-x:auto}
.tscroll table.spec{min-width:620px}
.dot{display:inline-block;width:14px;height:14px;border:1px solid rgba(255,255,255,.3);vertical-align:-2px;margin-right:6px}
.cap{font-size:13px;color:#a8a8a8;margin:0 0 8px}
.notes{font-size:13px;line-height:1.5;color:#d0d0d0;margin:0 0 22px;border-left:2px solid #333;padding:2px 0 2px 12px}
.notes div{margin:2px 0;max-width:110ch}
.notes b{color:#fff;font-weight:600}
.pair{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:14px;margin:8px 0 12px}
.trio{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:14px;margin:8px 0 12px}
.mwrap{overflow-x:auto;margin:8px 0 12px}
.pair .mwrap,.trio .mwrap{margin:0}
.themebar{position:sticky;top:0;z-index:20;display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:8px;padding:8px 0;background:#000;border-bottom:1px solid #2a2a2a;margin:14px 0 14px;font-size:14px}
.tt{display:inline-flex;border:1px solid #444}
.tt button{background:#000;color:#a8a8a8;border:0;padding:5px 14px;font:500 13px/1.3 system-ui,sans-serif;cursor:pointer}
.tt button.on{background:#fff;color:#000}
.tt button:focus-visible{outline:2px solid #fff;outline-offset:2px}
.toc{columns:2 320px;column-gap:28px;font-size:14px;margin:8px 0 16px;padding:0;list-style:none}
.toc li{margin:0 0 3px;break-inside:avoid}
.flowgrid{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:0 28px}
.strip{display:flex;flex-wrap:wrap;gap:14px;align-items:flex-start;margin:8px 0 12px}
.strip figure{margin:0}
.strip figcaption,.cellcap{font-size:12.5px;color:#a8a8a8;margin:6px 0 0}
.chips{display:grid;grid-template-columns:repeat(auto-fill,minmax(150px,1fr));gap:1px;margin:8px 0 14px}
.chip{padding:30px 10px 8px;font:12px/1.35 ui-monospace,Menlo,monospace}
.chip b{display:block;font:600 12px/1.35 system-ui,sans-serif}
svg.diagram{display:block;max-width:100%;height:auto;margin:8px 0 16px}
@media (max-width:1180px){.pair,.trio,.flowgrid{grid-template-columns:minmax(0,1fr)}}
'''

# The Mortise stylesheet from the identity page, unchanged, followed by the Slate tokens and the additions this spec defines.
FA_CSS = r'''
.m{--rail-w:52px;--idx-w:264px;--shell-h:548px;font:400 12.5px/1.45 var(--sans);color:var(--ink);text-align:left;-webkit-font-smoothing:antialiased;font-variant-numeric:tabular-nums}
.m *,.m *::before,.m *::after{box-sizing:border-box}
.m svg{display:block;flex:none}
.m .mono{font-family:var(--mono);font-size:.95em}
.m .dim{color:var(--ink-2)}
.m b,.m strong{font-weight:700}
.win{position:relative;background:var(--frame);border:1px solid var(--win-line);border-radius:var(--r-win);overflow:hidden;min-width:1080px}
.win.sm{min-width:0}
.tbar{display:grid;grid-template-columns:1fr auto 1fr;align-items:center;height:34px;padding:0 12px;gap:12px}
.tb-brand{display:flex;align-items:center;gap:8px;font-weight:700;font-size:13px}
.tb-find{display:flex;align-items:center;gap:8px;width:320px;height:22px;padding:0 8px;font-size:11.5px}
.tb-find .kbd{margin-left:auto}
.tb-right{justify-self:end;font-size:11.5px}
.shell{display:grid;grid-template-columns:var(--rail-w) var(--idx-w) minmax(0,1fr);min-height:var(--shell-h)}
.rail{display:flex;flex-direction:column;align-items:center;gap:8px;padding:10px 0}
.tile{position:relative;width:34px;height:34px;display:flex;align-items:center;justify-content:center;font:600 11px/1 var(--sans);letter-spacing:.02em}
.tile .pip{position:absolute;top:-4px;right:-4px;padding:1.5px;border-radius:50%;background:var(--pip-bg);color:var(--bad)}
.rail .sp{flex:1}
.idx{display:flex;flex-direction:column;min-width:0}
.idx-head{padding:12px 12px 10px}
.app-name{display:flex;align-items:baseline;justify-content:space-between;gap:8px;font-weight:700;font-size:14px;line-height:20px}
.app-name span{font-weight:400;font-size:11.5px;color:var(--ink-2)}
.app-sub{font-size:11.5px;color:var(--ink-2);margin-top:1px}
.filter{display:flex;align-items:center;gap:7px;margin-top:9px;height:26px;padding:0 8px;font-size:12px;color:var(--ink-3)}
.filter .kbd{margin-left:auto}
.mods{flex:1;display:flex;flex-direction:column}
.row{position:relative;display:grid;grid-template-columns:var(--g-w) minmax(0,1fr) auto;align-items:center;column-gap:9px;min-height:40px;padding:3px 12px}
.row .n{font-weight:600;font-size:13px;line-height:17px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.row .meta{font-size:11.5px;line-height:15px;color:var(--ink-2);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.idx-foot{display:flex;align-items:center;justify-content:space-between;gap:8px;padding:8px 12px;font-size:11.5px;color:var(--ink-2)}
.work{display:flex;flex-direction:column;min-width:0}
.whead{display:flex;align-items:center;justify-content:space-between;gap:12px;padding:12px 16px 8px;min-height:42px}
.wtitle{display:flex;align-items:baseline;gap:10px;min-width:0;white-space:nowrap}
.wtitle h4{margin:0;font-size:18px;line-height:22px;font-weight:700}
.wtitle .path{font-size:11.5px;color:var(--ink-2)}
.acts{display:flex;gap:6px;align-items:center}
.tabs{display:flex;gap:2px;padding:0 16px}
.tab{display:flex;align-items:baseline;gap:6px;padding:6px 10px;font-size:12.5px;font-weight:500;color:var(--ink-2);white-space:nowrap}
.tab i{font-style:normal;font-family:var(--mono);font-size:10.5px;color:var(--ink-3)}
.wbody{flex:1;padding:14px 16px 16px;display:flex;flex-direction:column;gap:12px;min-width:0}
.cols{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:14px}
.stack{display:flex;flex-direction:column;gap:12px;min-width:0}
.btn{display:inline-flex;align-items:center;gap:6px;height:26px;padding:0 10px;font-weight:600;font-size:12px;line-height:1;white-space:nowrap}
.btn.sm{height:22px;padding:0 8px;font-size:11.5px}
.btns{display:flex;gap:8px;flex-wrap:wrap;align-items:center}
.kbd{display:inline-flex;align-items:center;height:16px;padding:0 4px;font:500 10.5px/1 var(--mono);white-space:nowrap}
.st{display:inline-flex;align-items:center;gap:5px;font-size:11.5px;font-weight:600;line-height:1.2;white-space:nowrap}
.st.good{color:var(--good)}.st.warn{color:var(--warn)}.st.bad{color:var(--bad)}.st.off{color:var(--off)}
.stats{display:grid;grid-template-columns:repeat(4,minmax(0,1fr))}
.stat .k{font-size:11.5px;color:var(--ink-2)}
.stat .v{display:flex;align-items:center;gap:8px;flex-wrap:wrap;min-height:26px}
.stat .v .st{font-size:12px}
.box h5{display:flex;align-items:baseline;justify-content:space-between;gap:8px;margin:0;font-size:12.5px;line-height:1.4;font-weight:700}
.box h5 > span:last-child:not(.st){font-weight:400;font-size:11.5px;color:var(--ink-2)}
.kv{display:flex;align-items:center;justify-content:space-between;gap:10px;min-height:27px;font-size:12.5px}
.kv > span:first-child{display:flex;align-items:center;gap:7px;min-width:0}
.bar{display:grid;grid-template-columns:88px minmax(0,1fr) 34px;align-items:center;gap:10px;min-height:21px;font-size:12px}
.bar .tr{display:block;height:6px;background:var(--track)}
.bar .fl{display:block;height:100%;background:var(--bar)}
.bar .val{text-align:right;font-family:var(--mono);font-size:11.5px}
.bar.low .fl{background:var(--warn)}
.bar.low .val{color:var(--warn);font-weight:600}
.alert{display:flex;gap:9px;align-items:flex-start;padding:9px 12px;font-size:12.5px;line-height:1.5}
.alert .st{margin-top:2px}
.code{font:400 12px/1.6 var(--mono);white-space:pre-wrap;word-break:break-word;padding:10px 12px;background:var(--code-bg);color:var(--code-fg)}
.code .c-dim{color:var(--code-dim)}.code .c-ok,.drawer .c-ok{color:var(--code-ok)}.code .c-err,.drawer .c-err{color:var(--code-err)}
.tree{margin:0;font:400 12px/1.7 var(--mono);white-space:pre;color:var(--ink-2);overflow:hidden}
.tree b{font-weight:600;color:var(--ink)}
.lbl{font-size:11.5px;font-weight:600;margin:0 0 4px}
.lbl span{font-weight:400;color:var(--ink-2)}
.inp{display:flex;align-items:center;justify-content:space-between;gap:8px;min-height:28px;padding:0 9px;font-size:12.5px}
.field{margin-bottom:12px}
.chks{display:grid;grid-template-columns:1fr 1fr;gap:1px 12px}
.chk{display:flex;align-items:center;gap:8px;min-height:24px;font-size:12.5px}
.sq{width:14px;height:14px;display:inline-flex;align-items:center;justify-content:center;flex:none}
.drawer{background:var(--code-bg);color:var(--code-fg);padding:8px 16px 12px;font:400 11.5px/1.6 var(--mono)}
.dbar{display:flex;justify-content:space-between;align-items:center;gap:10px;margin-bottom:5px;font-family:var(--sans);font-size:12px;color:var(--code-dim)}
.dbar > span{display:flex;align-items:center;gap:8px}
.dbar b{color:var(--code-fg)}
.dbar .st.good{color:var(--code-ok)}.dbar .st.bad{color:var(--code-err)}
.m .dbar .kbd,.m .drawer .kbd{background:transparent;box-shadow:none;border:1px solid var(--code-dim);color:var(--code-dim)}
.stage{position:relative}
.scrim{position:absolute;inset:0;background:var(--scrim)}
.pal{position:absolute;top:58px;left:50%;transform:translateX(-50%);width:620px;background:var(--surface);overflow:hidden}
.pal-in{display:flex;align-items:center;gap:10px;height:44px;padding:0 14px;font-size:15px}
.pal-in .q{flex:1}
.caret{display:inline-block;width:1.5px;height:17px;background:var(--ink);vertical-align:-3px;margin-left:1px}
.pal-g{padding:9px 14px 3px;font-size:11.5px;font-weight:600;color:var(--ink-2)}
.pal-r{position:relative;display:flex;align-items:center;justify-content:space-between;gap:12px;min-height:32px;padding:0 14px;font-size:12.5px}
.pal-r .l{display:block;min-width:0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.pal-r .l svg{display:inline-block;vertical-align:-2px}
.pal-r .l .st{margin-left:8px;vertical-align:1px}
.code .tk{white-space:nowrap}
.pal-r .hint{display:flex;align-items:center;gap:6px;font-size:11.5px;color:var(--ink-2);white-space:nowrap}
.pal-f{display:flex;justify-content:space-between;gap:10px;flex-wrap:wrap;padding:8px 14px;margin-top:6px;font-size:11.5px;color:var(--ink-2)}
.empty{display:flex;flex-direction:column;justify-content:center;gap:12px;padding:26px 28px;min-height:262px}
.empty h4{margin:0;font-size:18px;line-height:1.25;font-weight:700}
.empty p{margin:0;max-width:46ch;color:var(--ink-2)}
.fail{display:flex;flex-direction:column;gap:10px;padding:12px 14px 14px;min-height:220px}
.fail p{margin:0;color:var(--ink-2)}
.m.cmp{--rail-w:40px;--idx-w:190px;--shell-h:452px;font-size:12px}
.m.cmp .win{min-width:570px}
.m.cmp .tbar{grid-template-columns:1fr auto;height:30px;padding:0 10px}
.m.cmp .tb-find{display:none}
.m.cmp .tile{width:28px;height:28px;font-size:10px}
.m.cmp .idx-head{padding:10px 9px 8px}
.m.cmp .row{padding:2px 8px;min-height:37px;column-gap:7px}
.m.cmp .row .n{font-size:12.5px}
.m.cmp .row .meta{font-size:11px}
.m.cmp .idx-foot{padding:7px 9px;white-space:nowrap}
.m.cmp .idx-foot .btn svg{display:none}
.m.cmp .whead{padding:10px 12px 6px;min-height:36px}
.m.cmp .wtitle h4{font-size:15px;line-height:20px}
.m.cmp .tabs{padding:0 8px;gap:0}
.m.cmp .tab{padding:5px 7px;font-size:11.5px}
.m.cmp .tab i{display:none}
.m.cmp .wbody{padding:10px 12px 12px;gap:10px}
.m.cmp .stats{grid-template-columns:1fr 1fr}
.m.cmp .bar{grid-template-columns:80px minmax(0,1fr) 30px;gap:8px;min-height:19px;font-size:11.5px}
.m.cmp .alert{padding:8px 10px;font-size:12px}
.fa{__LIGHT__;--sans:"Asap",system-ui,-apple-system,"Segoe UI",sans-serif;--mono:"Inconsolata",ui-monospace,Menlo,Consolas,monospace}
.fa[data-theme="dark"]{__DARK__}
.fa{--r-win:8px;--g-w:36px;--win-line:var(--line-2);--track:var(--well);--bar:var(--ink);--pip-bg:var(--frame)}
.fa .bm{color:var(--ink)}
.fa .bm .t2{fill:var(--line-2)}
.fa .tbar{background:var(--frame);border-bottom:1px solid var(--line-2);color:var(--ink)}
.fa .tb-find{background:var(--panel);border:1px solid var(--ink-2);border-radius:3px;color:var(--ink-2)}
.fa .tb-right{color:var(--ink-2)}
.fa .rail{background:var(--frame);border-right:1px solid var(--line-2)}
.fa .tile{border:1px solid var(--ink-2);border-radius:3px;background:var(--panel);color:var(--ink-2)}
.fa .tile.on{background:var(--key);border-color:var(--key);color:var(--key-fg)}
.fa .tile.add{border-style:dashed;background:transparent;font-size:15px;font-weight:400}
.fa .tile.ico{border:0;background:transparent}
.fa .tile.ico.on{background:var(--key);color:var(--key-fg)}
.fa .idx{background:var(--panel);border-right:1px solid var(--line-2)}
.fa .idx-head{border-bottom:1px solid var(--line)}
.fa .filter{background:var(--surface);border:1px solid var(--line-2);border-radius:3px}
.fa .mods{padding:8px 0 8px 8px}
.fa .row,.fa.cmp .row{border:1px solid transparent;border-right:0;padding-left:14px}
.fa .row.on{background:var(--surface);border-color:var(--line-2);margin-right:-1px}
.fa .row.on::before,.fa .pal-r.on::before,.fa .trow.on::before,.fa .lrow.on::before{content:"";position:absolute;left:4px;top:50%;margin-top:-2.5px;width:5px;height:5px;background:var(--ink)}
.fa .stamp{display:inline-block;width:34px;text-align:center;font:600 10.5px/16px var(--mono);letter-spacing:.03em;color:var(--ink-2);border:1px solid var(--line-2);border-radius:2px;vertical-align:1px}
.fa .row.on .stamp{color:var(--ink);border-color:var(--ink)}
.fa .idx-foot{border-top:1px solid var(--line)}
.fa .work{background:var(--surface)}
.fa .wtop{background:var(--panel);border-bottom:1px solid var(--line-2)}
.fa .tabs{gap:0;margin-bottom:-1px}
.fa .tab{border:1px solid transparent;border-bottom:0;padding:6px 12px 7px}
.fa .tab.on{background:var(--surface);border-color:var(--line-2);color:var(--ink);font-weight:600}
.fa .btn{border:1px solid var(--line-2);border-radius:3px;background:var(--surface);color:var(--ink)}
.fa .btn.pri{background:var(--key);border-color:var(--key);color:var(--key-fg)}
.fa .kbd{border:1px solid var(--line-2);border-radius:2px;background:var(--well);color:var(--ink-2)}
.fa .btn .kbd{margin-left:2px}
.fa .stats{border:1px solid var(--line-2);border-radius:3px}
.fa .stat{padding:9px 12px 8px;border-left:1px solid var(--line)}
.fa .stat:first-child{border-left:0}
.fa .stat .big{font-size:19px;font-weight:600;line-height:26px}
.fa.cmp .stat:nth-child(odd){border-left:0}
.fa.cmp .stat:nth-child(n+3){border-top:1px solid var(--line)}
.fa.cmp .stat .big{font-size:17px}
.fa .box{border:1px solid var(--line-2);border-radius:3px;padding:0 12px 8px}
.fa .box h5{padding:8px 0 7px;border-bottom:1px solid var(--line);margin-bottom:6px}
.fa .kv{border-bottom:1px solid var(--line)}
.fa .kv:last-child{border-bottom:0}
.fa .alert{background:var(--bad-soft);border:1px solid var(--bad);border-radius:3px}
.fa .code{border-radius:3px}
.fa .inp{background:var(--surface);border:1px solid var(--line-2);border-radius:3px}
.fa .field.opt{padding-top:8px;border-top:1px solid var(--line)}
.fa .sq{border:1.5px solid var(--line-2);border-radius:2px;background:var(--surface)}
.fa .sq.on{background:var(--key);border-color:var(--key);color:var(--key-fg)}
.fa .drawer{border-top:1px solid var(--line-2)}
.fa .pal{border:1px solid var(--ink);border-radius:4px;box-shadow:0 14px 34px var(--shadow)}
.fa .pal-in{border-bottom:1px solid var(--line-2)}
.fa .pal-r.on{background:var(--well);box-shadow:inset 0 1px 0 var(--line-2),inset 0 -1px 0 var(--line-2)}
.fa .pal-f{border-top:1px solid var(--line);background:var(--panel)}
.fa .wtitle h4{letter-spacing:-.005em}
'''
# Additions defined by this spec: states, new components, runtime marks. Nothing above is restyled.
ADD_CSS = r'''
.m svg.ic{display:inline-block;vertical-align:-2px}
.fa .is-focus{outline:2px solid var(--ink);outline-offset:1px}
.fa .btn.is-hover,.fa .tile.is-hover{background:var(--well)}
.fa .btn.is-press{background:var(--line)}
.fa .btn.pri.is-hover{background:var(--key-hover);border-color:var(--key-hover)}
.fa .btn.pri.is-press{background:var(--key-press);border-color:var(--key-press)}
.fa .btn.is-dis{background:transparent;border-color:var(--line);color:var(--ink-3)}
.fa .btn.pri.is-dis{background:var(--line);border-color:var(--line);color:var(--ink-2)}
.fa .btn.danger{border-color:var(--bad);color:var(--bad)}
.fa .btn.danger.fill{background:var(--bad);color:var(--bad-fg)}
.fa .btn.danger.is-hover{background:var(--bad-soft)}
.fa .btn.danger.fill.is-hover{background:var(--bad);box-shadow:inset 0 0 0 1px var(--bad-fg)}
.fa .btn.danger.is-press{background:var(--bad-soft);box-shadow:inset 0 0 0 1px var(--bad)}
.fa .btn.danger.fill.is-press{background:var(--bad);box-shadow:inset 0 0 0 2px var(--bad-fg)}
.fa .btn.danger.is-dis,.fa .btn.danger.fill.is-dis{border-color:var(--line);color:var(--ink-3);background:transparent;box-shadow:none}
.fa .btn.ghost{border-color:transparent;background:transparent}
.fa .btn.ghost.is-hover{background:var(--well)}
.fa .btn.icon{width:26px;padding:0;justify-content:center}
.fa .btn.icon.sm{width:22px}
.fa .inp.is-hover,.fa .filter.is-hover{border-color:var(--ink-2)}
.fa .inp.is-focus,.fa .filter.is-focus{border-color:var(--ink);color:var(--ink)}
.fa .inp.is-dis{background:var(--well);border-color:var(--line);color:var(--ink-2)}
.fa .inp.is-err{border-color:var(--bad)}
.fa .inp .ph,.fa .ph{color:var(--ink-3)}
.fa .errmsg{display:flex;gap:6px;align-items:flex-start;margin-top:4px;font-size:11.5px;line-height:1.4;color:var(--bad)}
.fa .errmsg .st{margin-top:1px}
.fa .hint2{margin-top:4px;font-size:11.5px;line-height:1.4;color:var(--ink-2)}
.fa .filter.has{color:var(--ink)}
.fa .sq.is-hover{border-color:var(--ink)}
.fa .sq.is-dis{background:var(--well);border-color:var(--line)}
.fa .sq.on.is-dis{background:var(--line);border-color:var(--line);color:var(--ink-2)}
.fa .chk.is-dis{color:var(--ink-3)}
.fa .seg{display:inline-flex;padding:2px;gap:2px;background:var(--well);border:1px solid var(--line-2);border-radius:3px}
.fa .seg span{display:inline-flex;align-items:center;height:20px;padding:0 10px;font-size:12px;font-weight:600;line-height:1;color:var(--ink-2);border:1px solid transparent;border-radius:2px;white-space:nowrap}
.fa .seg span.on{background:var(--surface);border-color:var(--ink);color:var(--ink)}
.fa .seg span.is-hover{color:var(--ink)}
.fa .seg span.is-dis{color:var(--ink-2);text-decoration:line-through}
.fa .tab.is-hover{color:var(--ink)}
.fa .row.is-hover,.fa .trow.is-hover,.fa .lrow.is-hover,.fa .pal-r.is-hover{background:var(--well)}
.fa .row.off .n{color:var(--ink-2);font-weight:500}
.fa .row.off .stamp{border-style:dashed}
.fa .row .marks{display:flex;align-items:center;gap:8px}
.fa .row.c1{min-height:26px;padding-top:1px;padding-bottom:1px}
.fa .row.c1 .meta{display:none}
.fa .grp{padding:9px 12px 3px 14px;font-size:11.5px;font-weight:600;color:var(--ink-2);display:flex;justify-content:space-between}
.fa .grp span{font-weight:400}
.fa .idx-rt{display:flex;align-items:center;gap:6px;margin-top:3px;font-size:11.5px;color:var(--ink-2)}
.st.up{color:var(--good)}.st.down{color:var(--bad)}.st.oos{color:var(--warn)}.st.unk{color:var(--off)}
.fa .tbl{width:100%;font-size:12.5px}
.fa .thead,.fa .trow{position:relative;display:grid;align-items:center;column-gap:12px;padding:0 10px 0 14px}
.fa .thead{min-height:26px;font-size:11.5px;font-weight:600;color:var(--ink-2);border-bottom:1px solid var(--line-2)}
.fa .thead .srt{color:var(--ink)}
.fa .trow{min-height:28px;border-bottom:1px solid var(--line)}
.fa .trow:last-child{border-bottom:0}
.fa .trow.on,.fa .lrow.on{background:var(--well);box-shadow:inset 0 1px 0 var(--line-2),inset 0 -1px 0 var(--line-2)}
.fa .trow.is-dis{color:var(--ink-3)}
.fa .trow > *,.fa .thead > *{min-width:0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.fa .r{text-align:right}
.fa .lrow{position:relative;display:flex;align-items:center;justify-content:space-between;gap:10px;min-height:30px;padding:0 10px 0 14px;font-size:12.5px;border-bottom:1px solid var(--line)}
.fa .lrow:last-child{border-bottom:0}
.fa .box.flush{padding:0 0 0}
.fa .lrow.dgrp{background:var(--panel);font-weight:600;border-top:1px solid var(--line-2)}
.fa .lrow.dact{padding-left:26px}
.fa .lrow.dact.on::before{left:14px}
.fa .box.flush h5{padding:8px 12px 7px;margin-bottom:0}
.fa .alert.warn{background:var(--surface);border-color:var(--warn)}
.fa .alert.note{background:var(--panel);border-color:var(--line-2)}
.fa .tip{display:inline-block;max-width:240px;padding:4px 8px;background:var(--key);color:var(--key-fg);font-size:11.5px;line-height:1.4;border-radius:3px}
.fa .tip .kbd{background:transparent;border-color:var(--key-fg);color:var(--key-fg);margin-left:4px}
.fa .toast{display:flex;gap:9px;align-items:flex-start;width:320px;padding:9px 10px 9px 12px;background:var(--surface);border:1px solid var(--ink);border-radius:4px;box-shadow:0 6px 16px var(--shadow);font-size:12.5px;line-height:1.45}
.fa .toast .st{margin-top:2px}
.fa .toast .tx{flex:1;min-width:0}
.fa .toast .tx .btns{margin-top:6px}
.fa .dlg{width:440px;background:var(--surface);border:1px solid var(--ink);border-radius:4px;box-shadow:0 14px 34px var(--shadow);overflow:hidden}
.fa .dlg-b{padding:14px 16px 12px;display:flex;flex-direction:column;gap:8px}
.fa .dlg-b h4{margin:0;font-size:15px;line-height:20px;font-weight:700}
.fa .dlg-b p{margin:0;color:var(--ink-2);line-height:1.5}
.fa .dlg-f{display:flex;justify-content:flex-end;gap:8px;padding:10px 16px;background:var(--panel);border-top:1px solid var(--line)}
.fa .dlg-f .lft{margin-right:auto}
.fa .over{position:absolute;inset:0;display:flex;align-items:flex-start;justify-content:center;padding-top:96px}
.fa .scrollv{position:relative;width:10px;align-self:stretch;flex:none}
.fa .scrollv i{position:absolute;left:2px;width:6px;border-radius:3px;background:var(--line-2)}
.fa .scrollv i.is-hover{background:var(--ink-2)}
.fa .scrollv i.is-press{background:var(--ink)}
.fa .prog{display:block;height:6px;background:var(--well)}
.fa .prog i{display:block;height:100%;background:var(--ink)}
.fa .busy{display:inline-flex;align-items:center;gap:6px;font-size:11.5px;font-weight:600;color:var(--ink)}
.fa .handle{height:7px;display:flex;align-items:center;justify-content:center;background:var(--surface);border-top:1px solid var(--line-2);cursor:ns-resize}
.fa .handle i{display:block;width:28px;height:1px;background:var(--line-2)}
.fa .handle.is-hover{border-top-color:var(--ink)}
.fa .handle.is-hover i,.fa .handle.is-press i{background:var(--ink);height:2px}
.fa .handle.is-press{border-top:2px solid var(--ink)}
.fa .dcol{display:flex;align-items:center;justify-content:space-between;gap:10px;height:28px;padding:0 16px;background:var(--code-bg);color:var(--code-dim);font-size:12px;border-top:1px solid var(--line-2)}
.fa .dcol b{color:var(--code-fg)}
.fa .dcol > span{display:flex;align-items:center;gap:8px}
.fa .dcol .st.good,.fa .drawer .st.good{color:var(--code-ok)}.fa .dcol .st.bad,.fa .drawer .st.bad{color:var(--code-err)}
.fa .drawer .hl{background:rgba(147,160,176,.18)}
.fa .dhist{display:grid;grid-template-columns:420px minmax(0,1fr);gap:14px}
.dbar > span{white-space:nowrap}
.fa .dhist .hr{display:flex;justify-content:space-between;gap:8px;padding:1px 6px;font-family:var(--sans);font-size:11.5px;color:var(--code-dim);white-space:nowrap}
.fa .dhist .hr.on{background:rgba(147,160,176,.18);color:var(--code-fg)}
.fa .dhist .hr span:first-child{overflow:hidden;text-overflow:ellipsis}
.fa .gen{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:0 14px}
.fa .gen .gh{grid-column:1/-1;padding:8px 0 2px;font-size:11.5px;font-weight:600;color:var(--ink-2);border-bottom:1px solid var(--line)}
.fa .gen .gi{position:relative;display:flex;align-items:baseline;justify-content:space-between;gap:8px;min-height:26px;padding:4px 8px 3px 14px;border-bottom:1px solid var(--line);font-size:12.5px}
.fa .gen .gi span{font-family:var(--mono);font-size:11.5px;color:var(--ink-2);white-space:nowrap}
.fa .gen .gi.on{background:var(--well)}
.fa .gen .gi.on::before{content:"";position:absolute;left:4px;top:50%;margin-top:-2.5px;width:5px;height:5px;background:var(--ink)}
.fa .newf{background:var(--well)}
.fa .wintitle{display:flex;gap:0;align-items:stretch;height:100%;margin-right:-12px}
.fa .wc{display:flex;align-items:center;justify-content:center;width:44px;height:34px;color:var(--ink)}
.fa .mac{display:flex;gap:8px;align-items:center}
.fa .mac i{width:12px;height:12px;border-radius:50%;border:1px solid var(--ink-2);display:block}
.fa .crumb{font-size:11.5px;color:var(--ink-2)}
.fa .setnav .row{min-height:32px}
.fa .frm{max-width:560px}
.fa .frm .field:last-child{margin-bottom:0}
.fa .sub{font-size:12.5px;font-weight:700;margin:4px 0 8px;padding-bottom:6px;border-bottom:1px solid var(--line)}
.fa .gnode{fill:var(--surface);stroke:var(--line-2)}
.fa .gnode.me{fill:var(--panel);stroke:var(--ink);stroke-width:1.5}
.fa .glink{fill:none;stroke:var(--ink-2);stroke-width:1}
.fa .glink.bad{stroke:var(--bad);stroke-width:2}
.fa .glink.sel{stroke:var(--ink);stroke-width:2}
.fa .garrow{fill:var(--ink-2)}.fa .garrow.bad{fill:var(--bad)}
.fa .gt{font:600 12px var(--sans);fill:var(--ink)}
.fa .gt2{font:400 11px var(--mono);fill:var(--ink-2)}
.fa .gt2.bad{fill:var(--bad);font-weight:600}
.fa .gst{font:600 10.5px var(--mono);fill:var(--ink-2);letter-spacing:.03em}
.fa .gcap{font:600 11.5px var(--sans);fill:var(--ink-2)}
.fa .gbadmark{fill:var(--bad)}
.fa .gx{stroke:var(--surface);stroke-width:1.3}
.fa .chart .ax{stroke:var(--line);stroke-width:1}
.fa .chart .ln{fill:none;stroke:var(--ink);stroke-width:1.5}
.fa .chart .ln2{fill:none;stroke:var(--line-2);stroke-width:1}
.fa .chart text{font:400 10.5px var(--mono);fill:var(--ink-2)}
.fa .spark{display:block}
.fa .spark path{fill:none;stroke:var(--ink-2);stroke-width:1.2}
.m .nowrap{white-space:nowrap}
.m .wrap{white-space:normal}
'''
def stylesheet():
    fa = FA_CSS.replace('__LIGHT__', css_vars(0)).replace('__DARK__', css_vars(1))
    return DOC_CSS + fa + ADD_CSS

# ---------------------------------------------------------------- icons (one sprite per page)
ICONS = {
 'search': '<path d="M2.75 2.75h7.5v7.5h-7.5zM10.25 10.25l3.75 3.75"/>',
 'settings': '<path d="M1.5 5h3.25M8.25 5h6.25M1.5 11h6.25M11.25 11h3.25"/><path d="M4.75 3.25h3.5v3.5h-3.5zM7.75 9.25h3.5v3.5h-3.5z"/>',
 'generate': '<path d="M2.75 6V2.75H6M10 2.75h3.25V6M13.25 10v3.25H10M6 13.25H2.75V10M8 5.25v5.5M5.25 8h5.5"/>',
 'run': '<path d="M4.75 3.1 12.9 8l-8.15 4.9z"/>',
 'boundary': '<path d="M1.75 4.75h4v6.5h-4zM10.25 4.75h4v6.5h-4z"/><path d="M8 1.5v13" stroke-dasharray="2 1.6"/>',
 'pulse': '<path d="M1.25 8.75h3l2-5.5 3 9.5 2-4h3.5"/>',
 'metrics': '<path d="M3.5 13.5v-5M8 13.5v-11M12.5 13.5v-7.5" stroke-width="2.4"/>',
 'migrate': '<path d="M1.75 3.75h6.5v8.5h-6.5zM1.75 8h6.5M10.25 8h4.5M12.5 5.5 15 8l-2.5 2.5"/>',
 'test': '<path d="M2.25 4.75 3.9 6.4l2.6-3.2M8.75 4.9h5.5M2.25 10.75 3.9 12.4l2.6-3.2M8.75 10.9h5.5"/>',
 'app': '<rect x="2.25" y="3.25" width="11.5" height="9.5"/><path d="M2.25 6.25h11.5"/>',
 'module': '<path d="M2.25 3.75h7v2.5h4.5v3.5h-4.5v2.5h-7z"/>',
 'x': '<path d="M3.5 3.5l9 9M12.5 3.5l-9 9"/>',
 'chev-r': '<path d="M6 3.5 10.5 8 6 12.5"/>',
 'chev-d': '<path d="M3.5 6 8 10.5 12.5 6"/>',
 'chev-u': '<path d="M3.5 10 8 5.5 12.5 10"/>',
 'folder': '<path d="M1.75 3.75h4.5l1.5 1.75h6.5v7.25H1.75z"/>',
 'copy': '<path d="M5.25 5.25h8v8h-8zM2.75 10.75v-8h8"/>',
 'stop': '<path d="M4 4h8v8H4z"/>',
 'list': '<path d="M2.5 4h11M2.5 8h11M2.5 12h7"/>',
 'open': '<path d="M6.5 3.25H2.75v10h10V9.5M9 2.75h4.25V7M13 3 7.5 8.5"/>',
 'plus': '<path d="M8 3v10M3 8h10"/>',
 'sync': '<path d="M2.5 5.5h10M10 3l2.5 2.5L10 8M13.5 10.5h-10M6 8l-2.5 2.5L6 13"/>',
 'lock': '<path d="M4.25 7.25h7.5v6h-7.5zM5.75 7.25v-3h4.5v3"/>',
 'clock': '<path d="M2.75 2.75h10.5v10.5H2.75zM8 5v3.25h2.75"/>',
 'route': '<path d="M2.5 12.5h4v-9h7M11 1.5l2.5 2-2.5 2"/>',
 'min': '<path d="M3.5 8h9"/>', 'max': '<path d="M3.75 3.75h8.5v8.5h-8.5z"/>',
}
MARKS = {  # 12 px marks. Structural findings are solid silhouettes, runtime status is drawn in strokes.
 'good': '<path d="M1.8 6.4 4.7 9.3 10.2 2.9" fill="none" stroke="currentColor" stroke-width="1.9"/>',
 'warn': '<path d="M6 1 11.4 10.8H.6z" fill="currentColor"/>',
 'bad': '<path fill-rule="evenodd" fill="currentColor" d="M1 1h10v10H1zM3.3 4.2 5.1 6 3.3 7.8l.9.9L6 6.9l1.8 1.8.9-.9L6.9 6l1.8-1.8-.9-.9L6 5.1 4.2 3.3z"/>',
 'off': '<rect x="1.75" y="1.75" width="8.5" height="8.5" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M4 6h4" stroke="currentColor" stroke-width="1.5"/>',
 'up': '<path d="M6 11V2.4M2.4 6 6 2.4 9.6 6" fill="none" stroke="currentColor" stroke-width="1.7"/>',
 'down': '<path d="M6 1v8.6M2.4 6 6 9.6 9.6 6" fill="none" stroke="currentColor" stroke-width="1.7"/>',
 'oos': '<path d="M3.6 1.5v9M8.4 1.5v9" fill="none" stroke="currentColor" stroke-width="2"/>',
 'unk': '<path d="M1 6h2.2M4.9 6h2.2M8.8 6H11" fill="none" stroke="currentColor" stroke-width="1.8"/>',
}
def sprite():
    s = ''.join('<symbol id="i-%s" viewBox="0 0 16 16"><g fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter" stroke-miterlimit="8">%s</g></symbol>' % (k, v) for k, v in ICONS.items())
    s += ''.join('<symbol id="m-%s" viewBox="0 0 12 12">%s</symbol>' % (k, v) for k, v in MARKS.items())
    s += '<symbol id="i-mark" viewBox="0 0 24 24"><path d="M3 4h9v4H8v8h4v4H3z" fill="currentColor"/><path class="t2" d="M21 4v16h-8v-5H9V9h4V4z"/></symbol>'
    s += '<symbol id="i-chk" viewBox="0 0 10 10"><path d="M1.6 5.2 4 7.6 8.4 2.6" fill="none" stroke="currentColor" stroke-width="1.8"/></symbol>'
    s += '<symbol id="i-dash" viewBox="0 0 10 10"><path d="M2 5h6" fill="none" stroke="currentColor" stroke-width="1.8"/></symbol>'
    return '<svg width="0" height="0" style="position:absolute" aria-hidden="true">%s</svg>' % s

def ic(name, size=13, cls=''):
    return '<svg class="ic %s" width="%d" height="%d" aria-hidden="true"><use href="#i-%s"/></svg>' % (cls, size, size, name)
def bm(size=16):
    return '<svg class="bm" width="%d" height="%d" viewBox="0 0 24 24" aria-hidden="true"><path d="M3 4h9v4H8v8h4v4H3z" fill="currentColor"/><path class="t2" d="M21 4v16h-8v-5H9V9h4V4z"/></svg>' % (size, size)
WORD = {'good': 'Good', 'warn': 'Warning', 'bad': 'Problem', 'off': 'Off', 'up': 'Up', 'down': 'Down', 'oos': 'Out of service', 'unk': 'Unknown'}
def S(kind, text=None, size=12):
    """State mark with an optional word. Always carries a title so the state is never colour only."""
    t = '<span>%s</span>' % text if text else ''
    return '<span class="st %s" title="%s"><svg width="%d" height="%d" aria-hidden="true"><use href="#m-%s"/></svg>%s</span>' % (kind, text or WORD[kind], size, size, kind, t)
def stamp(c): return '<span class="stamp">%s</span>' % c
def kbd(t): return '<span class="kbd">%s</span>' % t
def btn(label, cls='', icon=None, key=None):
    return '<span class="btn %s">%s%s%s</span>' % (cls, ic(icon) if icon else '', label, kbd(key) if key else '')
def kv(a, b): return '<div class="kv"><span>%s</span><span>%s</span></div>' % (a, b)
def box(title, right, body, cls='', style=''):
    return '<div class="box %s" style="%s"><h5><span>%s</span>%s</h5>%s</div>' % (cls, style, title, right if right.startswith('<span class="st') else '<span>%s</span>' % right, body)
def bar(label, pct, val, low=False):
    return '<div class="bar%s"><span>%s</span><span class="tr"><span class="fl" style="width:%s%%"></span></span><span class="val">%s</span></div>' % (' low' if low else '', label, pct, val)
def alert(inner, kind='bad', mark=None):
    m = {'bad': 'bad', 'warn': 'warn', 'note': None}[kind] if mark is None else mark
    return '<div class="alert %s">%s<div>%s</div></div>' % ('' if kind == 'bad' else kind, S(m) if m else '', inner)
def sq(on=False, cls='', mixed=False):
    inner = '<svg width="10" height="10"><use href="#i-%s"/></svg>' % ('dash' if mixed else 'chk') if (on or mixed) else ''
    return '<span class="sq %s %s">%s</span>' % ('on' if (on or mixed) else '', cls, inner)
def chk(label, flag='', on=False, cls='', mixed=False):
    f = ' <span class="mono dim">%s</span>' % flag if flag else ''
    return '<div class="chk %s">%s<span>%s%s</span></div>' % (cls, sq(on, cls, mixed), label, f)
def field(label, hint, inner, cls=''):
    h = ' <span>%s</span>' % hint if hint else ''
    return '<div class="field %s"><div class="lbl">%s%s</div>%s</div>' % (cls, label, h, inner)
def inp(text, cls='', right='', style=''):
    return '<div class="inp %s" style="%s"><span>%s</span>%s</div>' % (cls, style, text, right)
def seg(items, on=0):
    return '<span class="seg">%s</span>' % ''.join('<span class="%s">%s</span>' % ('on' if i == on else '', x) for i, x in enumerate(items))
def code(inner, style=''): return '<div class="code" style="%s">%s</div>' % (style, inner)
def cmd(*tokens): return code(' '.join('<span class="tk">%s</span>' % html.escape(t) for t in tokens))
CARET = '<span class="caret"></span>'
def table(cols, head, rows, sel=None, cls='', rowcls=None):
    """cols: css grid template. rows: list of cell lists. rowcls: {row index: extra class}."""
    g = 'grid-template-columns:%s' % cols
    out = '<div class="tbl %s"><div class="thead" style="%s">%s</div>' % (cls, g, ''.join('<span%s>%s</span>' % (' class="r"' if h.endswith('>') else '', h.rstrip('>')) for h in head))
    for i, r in enumerate(rows):
        c, x = r, (rowcls or {}).get(i, '')
        out += '<div class="trow %s %s" style="%s">%s</div>' % ('on' if i == sel else '', x, g, ''.join(
            '<span%s>%s</span>' % (' class="r"' if head[j].endswith('>') else '', v) for j, v in enumerate(c)))
    return out + '</div>'

# ---------------------------------------------------------------- mock data
MODS = [  # code, name, files, lines, tests, cov, uses, usedby, commits30, finding
 ('BIL', 'Billing', 96, '7,840', 58, 81, 1, 1, 9, 'good'),
 ('CAT', 'Catalog', 121, '9,105', 74, 77, 0, 2, 6, 'good'),
 ('CHK', 'Checkout', 84, '6,412', 31, 54, 4, 2, 14, 'bad'),
 ('INV', 'Inventory', 62, '4,380', 40, 72, 1, 0, 3, 'good'),
 ('NTF', 'Notifications', 38, '2,190', 12, 39, 2, 0, 2, 'warn'),
 ('SHP', 'Shipping', 71, '5,265', 29, 66, 1, 1, 11, 'good'),
 ('USR', 'Users', 79, '6,020', 63, 88, 0, 3, 5, 'good'),
]
BYCODE = {m[0]: m for m in MODS}
PROBLEMS_FIRST = ['CHK', 'NTF', 'BIL', 'CAT', 'INV', 'SHP', 'USR']
BYNAME = ['BIL', 'CAT', 'CHK', 'INV', 'NTF', 'SHP', 'USR']
APPS3 = [('SH', 'shop-api', 2), ('CR', 'crm', 0), ('LP', 'landing-pages', 1)]
APPS10 = APPS3 + [('BE', 'booking-engine', 0), ('LE', 'legacy-portal', 0), ('AS', 'admin-suite', 0), ('PL', 'platform', 5),
                  ('DS', 'docs-site', 0), ('BG', 'billing-gateway', 0), ('PA', 'partner-api', 0)]

def fmark(code, short=True):
    f = BYCODE[code][9]
    if f == 'bad': return S('bad', '2' if short else '2 problems')
    if f == 'warn': return S('warn', None if short else 'Low coverage')
    return S('good', None if short else 'Good')

# ---------------------------------------------------------------- window parts
def tbar(right='shop-api on Herd', find=True, platform='plain'):
    f = '<span class="tb-find">%s<span>Search or run a command</span>%s</span>' % (ic('search'), kbd('Ctrl K')) if find else '<span></span>'
    brand = '<span class="tb-brand">%s<span>Mortise</span></span>' % bm()
    r = '<span class="tb-right">%s</span>' % right
    if platform == 'mac':
        brand = '<span class="tb-brand"><span class="mac" title="Close, minimise, zoom: drawn by macOS"><i></i><i></i><i></i></span><span style="width:6px"></span>%s<span>Mortise</span></span>' % bm()
    if platform in ('win', 'linux'):
        r = '<span class="tb-right" style="display:flex;align-items:center;gap:12px"><span>%s</span><span class="wintitle"><span class="wc" title="Minimise">%s</span><span class="wc" title="Maximise">%s</span><span class="wc" title="Close">%s</span></span></span>' % (right, ic('min', 14), ic('max', 14), ic('x', 14))
    return '<div class="tbar">%s%s%s</div>' % (brand, f, r)
def tile(mono, name, problems=0, on=False, cls=''):
    t = name + (', %d problem%s' % (problems, '' if problems == 1 else 's') if problems else '')
    pip = '<span class="pip"><svg width="9" height="9"><use href="#m-bad"/></svg></span>' if problems else ''
    return '<div class="tile %s %s" title="%s">%s%s</div>' % ('on' if on else '', cls, t, mono, pip)
def rail(apps=APPS3, sel='SH', add=True, settings=False, add_on=False):
    t = ''.join(tile(m, n, p, m == sel) for m, n, p in apps)
    a = '<div class="tile add%s" title="Add app"%s>+</div>' % (' is-focus' if add_on else '', '') if add else ''
    s = '<div class="tile ico%s" title="Settings">%s</div>' % (' on' if settings else '', ic('settings', 16))
    return '<div class="rail">%s%s<div class="sp"></div>%s</div>' % (t, a, s)
def row(code, name, meta, mark='', on=False, cls='', rt=None):
    marks = '<span class="marks">%s%s</span>' % (S(rt) if rt else '', mark)
    return '<div class="row %s %s">%s<div class="t"><div class="n">%s</div><div class="meta">%s</div></div>%s</div>' % ('on' if on else '', cls, stamp(code), name, meta, marks)
def mod_row(code, sel=None, long=True, rt=None, cls=''):
    m = BYCODE[code]
    return row(code, m[1], '%d files, %d%%%s' % (m[2], m[5], ' covered' if long else ''), fmark(code), code == sel, cls, rt)
def all_row(sel=None, long=True, rt=None):
    return row('ALL', 'All modules', '551 files, 71%' + (' covered' if long else ''), S('bad', '2'), sel == 'ALL', '', rt)
def idx_head(name='shop-api', on='7 of 7 on', sub='Laravel 13.2, package 13.3.0', filt=None, rt='up', env='local'):
    f = filt or '<div class="filter">%s<span>Filter modules</span>%s</div>' % (ic('search'), kbd('/'))
    r = '<div class="idx-rt">%s<span>on %s</span></div>' % (S(rt, WORD[rt]), env) if rt else ''
    return '<div class="idx-head"><div class="app-name">%s<span>%s</span></div><div class="app-sub">%s</div>%s%s</div>' % (name, on, sub, r, f)
def idx_foot(sort='Sorted: problems first', new=True):
    return '<div class="idx-foot"><span>%s</span>%s</div>' % (sort, btn('New module', 'sm', 'generate') if new else '')
def index(sel='ALL', order=PROBLEMS_FIRST, long=True, head=None, foot=None, rt='up', mrt=False):
    rows = all_row(sel, long, None) + ''.join(mod_row(c, sel, long, ('up' if (rt and mrt) else None)) for c in order)
    return '<div class="idx">%s<div class="mods">%s</div>%s</div>' % (head or idx_head(rt=rt), rows, foot or idx_foot())
TABS = ['Summary', 'Boundary', 'Generate', 'Actions', 'Metrics']
def tabs(sel=0):
    return '<div class="tabs">%s</div>' % ''.join('<span class="tab%s">%s<i>%d</i></span>' % (' on' if i == sel else '', t, i + 1) for i, t in enumerate(TABS))
def work(title, tab, body, acts='', drawer='', stampc=None, path=None, notabs=False, crumb=None):
    tt = '<h4>%s</h4>' % title
    if stampc: tt += stamp(stampc)
    if path: tt += '<span class="path mono">%s</span>' % path
    if crumb: tt = '<span class="crumb">%s</span>' % crumb + tt
    return ('<div class="work"><div class="wtop"><div class="whead"><div class="wtitle">%s</div><div class="acts">%s</div></div>%s</div>'
            '<div class="wbody">%s</div>%s</div>') % (tt, acts, '' if notabs else tabs(tab), body, drawer)
def shell(r, i, w, style=''):
    return '<div class="shell" style="%s">%s%s%s</div>' % (style, r, i, w)
def win(inner, sw=True, cmp=False, theme='light', sm=False, style='', over=''):
    stage = '<div class="stage"><div aria-hidden="true">%s</div><div class="scrim"></div>%s</div>' % (inner, over) if over else inner
    return '<div class="mwrap"><div class="m fa%s%s" data-theme="%s"><div class="win%s" style="%s">%s</div></div></div>' % (
        ' cmp' if cmp else '', ' sw' if sw else '', theme, ' sm' if sm else '', style, stage)
def both(fn):
    """Light and dark side by side, fixed, so the comparison works without JavaScript."""
    return '<div class="pair">%s%s</div>' % (fn('light'), fn('dark'))
def swatch(inner, bg='surface', pad='14px 16px', style=''):
    """A small specimen in both themes."""
    one = lambda th: '<div class="m fa" data-theme="%s"><div style="background:var(--%s);padding:%s;%s">%s</div></div>' % (th, bg, pad, style, inner)
    return '<div class="pair">%s%s</div>' % (one('light'), one('dark'))
def panel(inner, bg='surface', pad='14px 16px', style='', sw=True):
    return '<div class="m fa%s" data-theme="light"><div style="background:var(--%s);padding:%s;border:1px solid var(--line-2);%s">%s</div></div>' % (' sw' if sw else '', bg, pad, style, inner)
MOD_ACTS = btn('Run tests', 'sm', 'test') + btn('Migrate (1)', 'sm pri', 'migrate') + btn('Open in editor', 'sm')

# ---------------------------------------------------------------- interface text registry
TEXT = {}   # key: (group, where, string, note)
USED = {}
def deftext(group, rows):
    for key, where, s, *note in rows:
        assert key not in TEXT, key
        TEXT[key] = (group, where, s, note[0] if note else '')
def t(key):
    USED[key] = USED.get(key, 0) + 1
    return TEXT[key][2]

# ---------------------------------------------------------------- assumptions registry
ASSUME = {}  # key: (package, short, long)
AUSED = {}
ASTATE = {}  # key: 'met' or 'in part'; a key not here is not met yet
def defassume(rows):
    for k, pkg, short, long in rows: ASSUME[k] = (pkg, short, long)

# ---------------------------------------------------------------- page assembly
PAGES = [('index.html', 'Overview'), ('architecture.html', 'Information architecture'), ('tokens.html', 'Tokens'),
         ('components.html', 'Components'), ('screens-start.html', 'Screens: start'), ('screens-read.html', 'Screens: read'),
         ('screens-write.html', 'Screens: write'), ('screens-global.html', 'Screens: global'), ('flows.html', 'Flows and keyboard'),
         ('reference.html', 'Text, platform, fonts'), ('open.html', 'Open questions')]
CUR = {'page': None}
SCREENS = {}  # id: (page, title)
def screen(sid, title, cap, mock, phase, data=None, assumes=(), more=()):
    """One screen or state: heading with a stable id, caption, drawing, notes."""
    SCREENS[sid] = (CUR['page'], title)
    n = '<div><b>Phase.</b> %s</div>' % phase
    if data: n += '<div><b>Data.</b> %s</div>' % data
    for m in more: n += '<div>%s</div>' % m
    if assumes:
        for a in assumes: AUSED.setdefault(a, []).append(sid)
        head = 'Assumes, not in the package today.'
        if any(a in ASTATE for a in assumes):
            head = 'Assumes, met in the packages.' if all(ASTATE.get(a) == 'met' for a in assumes) else 'Assumes, not fully in the packages today.'
        n += '<div><b>%s</b> %s</div>' % (head, ' '.join('<a class="sid" href="open.html#a-%s">%s</a> %s%s.' % (a, a, ASSUME[a][1], ' (%s)' % ASTATE[a] if a in ASTATE else '') for a in assumes))
    else:
        n += '<div><b>Assumes.</b> Nothing the packages lack today.</div>'
    return '<h3 id="%s"><span class="sid">%s</span>%s</h3><p class="cap">%s</p>%s<div class="notes">%s</div>' % (sid, sid, title, cap, mock, n)
def ref(sid):
    return '[[%s]]' % sid
DOCS = {}
def finish():
    def link(m):
        if m.group(1) not in SCREENS:
            print('UNRESOLVED', m.group(1)); return m.group(1)
        p, title = SCREENS[m.group(1)]
        return '<a class="sid" href="%s#%s" title="%s">%s</a>' % (p, m.group(1), html.escape(title), m.group(1))
    total = 0
    for f, d in DOCS.items():
        d = re.sub(r'\[\[([A-Z]\d+\.\d+)\]\]', link, d)
        assert '[[' not in d, f
        open(OUT + '/' + f, 'w', encoding='utf-8', newline='\n').write(d); total += len(d.encode())
        print('%-22s %7d bytes' % (f, len(d.encode())))
    print('total', total)
SWITCH_JS = '''(function(){var b=document.querySelectorAll('[data-set-theme]');function set(t){document.querySelectorAll('.sw').forEach(function(e){e.setAttribute('data-theme',t)});b.forEach(function(x){x.classList.toggle('on',x.getAttribute('data-set-theme')===t)})}
b.forEach(function(x){x.addEventListener('click',function(){set(x.getAttribute('data-set-theme'))})});if(location.hash==='#dark')set('dark')})();'''
def page(fname, title, lead, body, switch=True):
    nav = ''.join('<a href="%s"%s>%s</a>' % (f, ' class="cur"' if f == fname else '', n) for f, n in PAGES)
    bar = ('<div class="themebar"><span>Theme for the switchable drawings on this page. Kept in memory only. Without JavaScript they stay light; '
           'the paired drawings always show both.</span><span class="tt" role="group" aria-label="Theme for the drawings">'
           '<button type="button" class="on" data-set-theme="light">Light</button><button type="button" data-set-theme="dark">Dark</button></span></div>') if switch else ''
    doc = ('<!doctype html>\n<html lang="en">\n<head>\n<meta charset="utf-8">\n<meta name="viewport" content="width=device-width, initial-scale=1">\n'
           '<title>%s | Mortise design specification</title>\n'
           '<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Asap:wght@400;500;600;700&family=Inconsolata:wght@400;500;600&display=swap">\n'
           '<style>%s</style>\n</head>\n<body>\n%s\n<div class="doc">\n<nav class="top" aria-label="Specification pages">%s</nav>\n<h1>%s</h1>\n<p class="dim">%s</p>\n%s\n%s\n</div>\n%s\n</body>\n</html>\n') % (
           title, stylesheet(), sprite(), nav, title, lead, bar, body,
           '<script>%s</script>' % SWITCH_JS if switch else '')
    assert '—' not in doc, 'em dash in ' + fname
    DOCS[fname] = doc
    return len(doc)
def spec_table(head, rows, cls=''):
    return '<div class="tscroll"><table class="spec %s"><thead><tr>%s</tr></thead><tbody>%s</tbody></table></div>' % (
        cls, ''.join('<th>%s</th>' % h for h in head), ''.join('<tr>%s</tr>' % ''.join('<td>%s</td>' % c for c in r) for r in rows))
