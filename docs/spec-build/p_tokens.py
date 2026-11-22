from lib import *
from data import *
import json

PAIRS = [  # fg, bg, minimum, where it is used
 ('ink', 'surface', 4.5, 'Primary text in the workspace'), ('ink', 'panel', 4.5, 'Primary text in the index and header strip'),
 ('ink', 'frame', 4.5, 'Brand in the title bar'), ('ink', 'well', 4.5, 'Text on a hovered or selected list row'), ('ink', 'bad-soft', 4.5, 'Text in a problem alert'),
 ('ink-2', 'surface', 4.5, 'Secondary text'), ('ink-2', 'panel', 4.5, 'Secondary text in the index, monogram on a rail tile'),
 ('ink-2', 'frame', 4.5, 'App and runner in the title bar'), ('ink-2', 'well', 4.5, 'Keycap label, disabled field text, unselected segment'),
 ('ink-2', 'line', 4.5, 'Label of a disabled primary button'),
 ('ink-3', 'surface', 4.5, 'Placeholder, disabled label'), ('ink-3', 'panel', 4.5, 'Tab key hint, disabled label in the header strip'),
 ('key-fg', 'key', 4.5, 'Primary button, selected rail tile, tooltip'), ('key-fg', 'key-hover', 4.5, 'Primary button, hover'), ('key-fg', 'key-press', 4.5, 'Primary button, pressed'),
 ('bad-fg', 'bad', 4.5, 'Filled destructive button'),
 ('good', 'surface', 4.5, 'Good and Up, word and mark'), ('good', 'panel', 4.5, 'Good and Up in the index'), ('good', 'well', 4.5, 'Good on a selected row'),
 ('warn', 'surface', 4.5, 'Warning and Out of service'), ('warn', 'panel', 4.5, 'Warning in the index'), ('warn', 'well', 4.5, 'Warning on a selected row'),
 ('bad', 'surface', 4.5, 'Problem and Down, destructive outline button'), ('bad', 'panel', 4.5, 'Problem in the index'), ('bad', 'well', 4.5, 'Problem on a selected row'),
 ('bad', 'bad-soft', 4.5, 'Mark and outline of a problem alert'),
 ('off', 'surface', 4.5, 'Off and Unknown'), ('off', 'panel', 4.5, 'Off in the index'),
 ('code-fg', 'code-bg', 4.5, 'Runner text'), ('code-dim', 'code-bg', 4.5, 'Runner secondary'), ('code-ok', 'code-bg', 4.5, 'Runner success'), ('code-err', 'code-bg', 4.5, 'Runner error'),
 ('line-2', 'surface', 3, 'Outline of buttons, fields, checkboxes, boxes'), ('line-2', 'panel', 3, 'Outline of stamps and the filter field in the index'),
 ('line-2', 'well', 3, 'Outline of a keycap and of the segmented control'),
 ('ink-2', 'frame', 3, 'Outline of rail tiles and of the search entry'), ('ink', 'surface', 3, 'Focus ring, peg, selected stamp, bars'),
 ('ink', 'panel', 3, 'Focus ring and peg in the index'), ('ink', 'frame', 3, 'Focus ring on the rail, window control glyphs'), ('ink', 'well', 3, 'Peg on a selected list row'),
 ('bad', 'frame', 3, 'Problem pip on a rail tile'), ('line-2', 'bad-soft', 3, 'Not used. Listed because a stamp inside an alert would land here'),
]
FORBID = [('ink-3', 'frame', 'Text. The rule found in review.'), ('off', 'frame', 'Text. Same value as ink-3.'), ('warn', 'frame', 'Text. A warning word never sits on the title bar or rail.'),
          ('line-2', 'frame', 'Outline that carries meaning. On frame it is used only for the two joints between frame and panel.'),
          ('ink-3', 'well', 'Passes by 0.01. Not used for running text; a selected or hovered row with disabled text switches that text to ink-2.')]

def build():
    CUR['page'] = 'tokens.html'
    b = ('<p>Section 3 of the specification. The values are final for phase 0. Neutrals are the Slate set chosen by the owner; state colours and the oak and walnut app icon are unchanged from the Mortise identity. '
         'Everything below is also in one JSON block at the end of the page, which is what the builders should copy from.</p>')
    b += '<h2>3.1 Colour</h2>'
    b += spec_table(['Token', 'Light', 'Dark', 'Role'], [['<code>%s</code>' % k, '<span class="dot" style="background:%s"></span>%s' % (v[0], v[0]), '<span class="dot" style="background:%s"></span>%s' % (v[1], v[1]), v[2]] for k, v in COLOR.items()]
                    + [['<code>%s</code>' % k, v[0], v[1], v[2]] for k, v in ALPHA.items()])
    chips = lambda i: '<div class="chips">%s</div>' % ''.join('<div class="chip" style="background:%s;color:%s"><b>%s</b>%s</div>' % (v[i], '#000' if lum(v[i]) > .3 else '#fff', k, v[i]) for k, v in COLOR.items())
    b += '<div class="pair"><div>%s</div><div>%s</div></div>' % (chips(0), chips(1))
    b += ('<p><strong>Four rules that come with the values.</strong></p><ol>'
          '<li><code>ink-3</code> never on <code>frame</code>. It is 4.13 to 1 there in the light theme, under the 4.5 limit for text. <code>off</code> has the same value and the same rule.</li>'
          '<li><code>warn</code> never as text on <code>frame</code>: 4.40 to 1 in the light theme.</li>'
          '<li><code>line-2</code> never as the outline of a control on <code>frame</code>: 2.84 to 1 in the light theme, under the 3 to 1 limit for marks. Rail tiles and the search entry in the title bar use <code>ink-2</code> for their outline. This is a change to the identity stylesheet, made necessary by the Slate values. The owner accepted it on 22 November 2026; it is a rule, not a question.</li>'
          '<li>Colour is only state. Good, warn, bad and off are the only chromatic values in the interface. <code>key-hover</code> and <code>key-press</code> are ink mixed 16% and 30% toward surface.</li></ol>')
    b += '<h2>3.2 Contrast</h2><p>WCAG 2.x contrast ratio, computed from the hex values when this page was built. Text is held to 4.5 to 1 and marks, outlines and other shapes to 3 to 1, in both themes. Every pairing drawn anywhere in this specification is in the table; the page build stops if one falls under its minimum.</p>'
    rows = []
    for fg, bg, mn, use in PAIRS:
        l, d = cr(COLOR[fg][0], COLOR[bg][0]), cr(COLOR[fg][1], COLOR[bg][1])
        if 'Not used' not in use: assert l >= mn and d >= mn, (fg, bg, l, d)
        rows.append(['Text' if mn == 4.5 else 'Mark', '<code>%s</code> on <code>%s</code>' % (fg, bg), '%.2f' % l, '%.2f' % d, '%s to 1' % mn, use])
    b += spec_table(['Kind', 'Pair', 'Light', 'Dark', 'Minimum', 'Used for'], rows)
    b += '<p><strong>Pairs that are not allowed.</strong> They fail, or pass too narrowly to rely on.</p>'
    b += spec_table(['Pair', 'Light', 'Dark', 'Not allowed for'], [['<code>%s</code> on <code>%s</code>' % (fg, bg), '%.2f' % cr(COLOR[fg][0], COLOR[bg][0]), '%.2f' % cr(COLOR[fg][1], COLOR[bg][1]), why] for fg, bg, why in FORBID])
    b += '<p>The drawings on every page of this specification were also checked in a browser: for each piece of text, the colour it is drawn in against the fill behind it, in both themes. No text under 4.5 to 1 was found, and no <code>ink-3</code> on <code>frame</code>.</p>'
    b += '<h2>3.3 Type</h2><p>Two families, both bundled: Asap for the interface, Inconsolata for commands, paths, runner output, module codes and key hints. Sizes are in logical pixels at 100% text size. Numbers are tabular everywhere.</p>'
    b += spec_table(['Token', 'Size / line', 'Weight', 'Family', 'Used for'], [['<code>%s</code>' % n, '%s / %s' % (s, l), w, 'Asap' if f == 'sans' else 'Inconsolata', u] for n, s, l, w, f, u in TYPE])
    spec = ('<div style="display:flex;flex-direction:column;gap:6px"><div style="font-size:18px;font-weight:700;line-height:22px">Checkout <span class="stamp">CHK</span></div>'
            '<div style="font-size:19px;font-weight:600;line-height:26px">41,212 lines of PHP, 71% covered</div><div style="font-size:13px;font-weight:600">Notifications <span style="font-weight:400;font-size:11.5px;color:var(--ink-2)">38 files, 39% covered</span></div>'
            '<div>3 files in Checkout read Modules\\Billing\\Models\\Invoice directly.</div><div class="mono" style="font-size:12px">php artisan module:make-model PaymentIntent Billing --migration --policy</div>'
            '<div style="color:var(--ink-2);font-size:11.5px">ABCDEFGHIJKLMNOPQRSTUVWXYZ abcdefghijklmnopqrstuvwxyz 0123456789</div></div>')
    b += swatch(spec)
    b += '<p>Text size in Settings scales every type token and every size token by the same factor: 100, 110, 125 or 150%. Strokes stay 1 px.</p>'
    b += '<h2>3.4 Spacing and size</h2><p>The unit is 4 px. Spacing steps: %s px. Panel padding is 12 px in the index and 16 px in the workspace; gaps between boxes are 12 px; gaps between controls are 8 px.</p>' % ', '.join(str(s) for s in SPACE)
    names = {'titlebar-h': 'Title bar height', 'rail-w': 'App rail width', 'rail-tile': 'Rail tile, square', 'index-w': 'Module index width, default', 'index-w-min': 'Module index, narrowest by dragging',
             'index-w-max': 'Module index, widest by dragging', 'index-strip-w': 'Module index collapsed to its strip', 'index-row-h': 'Index row, two lines', 'index-row-compact-h': 'Index row, one line',
             'control-h': 'Button', 'control-sm-h': 'Small button', 'field-h': 'Text field', 'tab-h': 'Tab', 'table-head-h': 'Table header', 'table-row-h': 'Table row', 'list-line-h': 'Line in a box',
             'palette-w': 'Palette width', 'palette-input-h': 'Palette query row', 'palette-row-h': 'Palette result row', 'dialog-w': 'Dialog width', 'toast-w': 'Toast width', 'tooltip-max-w': 'Tooltip, widest',
             'drawer-h-default': 'Runner drawer, default height', 'drawer-h-min': 'Runner drawer, lowest', 'drawer-collapsed-h': 'Runner drawer collapsed', 'scrollbar-w': 'Scrollbar track',
             'scrollbar-thumb-w': 'Scrollbar thumb', 'scrollbar-thumb-min': 'Scrollbar thumb, shortest', 'state-mark': 'State mark, square', 'stamp-w': 'Code stamp width', 'stamp-h': 'Code stamp height',
             'checkbox': 'Checkbox, square', 'icon': 'Icon, square', 'icon-in-control': 'Icon inside a control', 'peg': 'Peg, square', 'bar-h': 'Bar and progress height',
             'window-min-w': 'Window, narrowest', 'window-min-h': 'Window, lowest', 'hit-min': 'Pointer target, smallest in either direction'}
    items = list(SIZE.items()); half = (len(items) + 1) // 2
    tb = lambda part: spec_table(['Token', 'px', 'What'], [['<code>%s</code>' % k, v, names[k]] for k, v in part])
    b += '<div class="pair"><div>%s</div><div>%s</div></div>' % (tb(items[:half]), tb(items[half:]))
    b += '<h2>3.5 Radius, stroke, elevation</h2>'
    b += spec_table(['Token', 'Value', 'Used for'], [
        ['<code>radius.joint</code>', '0 px', 'Anything that forms a joint: selected index row, selected tab, selected list row'], ['<code>radius.stamp</code>', '2 px', 'Code stamp, keycap, checkbox, segment'],
        ['<code>radius.control</code>', '3 px', 'Button, field, box, alert, rail tile, tooltip'], ['<code>radius.overlay</code>', '4 px', 'Palette, dialog, toast'], ['<code>radius.window</code>', '8 px', 'The window, where the platform lets the app draw it'],
        ['<code>stroke.hairline</code>', '1 px, <code>line</code>', 'Lines inside a panel or box'], ['<code>stroke.joint</code>', '1 px, <code>line-2</code>', 'Where two panels meet, and around controls'],
        ['<code>stroke.icon</code>', '1.5 px', 'Icons, on a 16 px grid, straight and 45 degree strokes, square ends, mitred corners'], ['<code>stroke.checkbox</code>', '1.5 px', 'Checkbox outline'],
        ['<code>stroke.focus-ring</code>', '2 px, <code>ink</code>, 1 px outside the control', 'Keyboard focus, on every focusable thing'], ['<code>stroke.problem-link</code>', '2 px, <code>bad</code>', 'A link in the graph that breaks a rule'],
        ['<code>elevation.none</code>', 'no shadow', ELEV['none']], ['<code>elevation.overlay</code>', 'x 0, y 14, blur 34', ELEV['overlay']], ['<code>elevation.toast</code>', 'x 0, y 6, blur 16', ELEV['toast']], ['<code>elevation.tooltip</code>', 'no shadow', ELEV['tooltip']]])
    b += '<p>For egui: flat fills, 1 px strokes and rounded rectangles only. The two shadows are the only blurred shapes; if a blurred shadow is costly, a 2 px ink outline replaces it. Nothing uses backdrop blur or gradients.</p>'
    b += '<h2>3.6 Motion</h2>' + spec_table(['Token', 'Value'], [['<code>motion.%s</code>' % k, v] for k, v in MOTION.items()])
    b += '<p>Nothing moves unless the user or a command caused it. No hover transitions, no loops, no spinner: a running command shows elapsed time counting, and work of known size shows a bar.</p>'
    b += '<h2>3.7 All tokens as JSON</h2><p>One block, every value above. Colours with alpha are kept apart from the solid ones. Sizes are logical pixels; durations are milliseconds.</p>'
    b += '<pre class="json" id="tokens-json">%s</pre>' % html.escape(json.dumps(tokens_json(), indent=1))
    return page('tokens.html', 'Tokens', 'Mortise design specification, section 3. Final values for Slate, light and dark.', b, switch=False)
