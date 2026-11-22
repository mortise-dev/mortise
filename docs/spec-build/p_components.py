from lib import *
from data import *
from p_write import toast, dialog, dlg_rollback

def cell(cap, inner, w=''):
    return '<div style="display:flex;flex-direction:column;gap:5px;align-items:flex-start;%s">%s<span style="font-size:11px;color:var(--ink-2)">%s</span></div>' % (w, inner, cap)
def rowspec(cells, bg='surface', gap='18px 22px'):
    return swatch('<div style="display:flex;flex-wrap:wrap;gap:%s;align-items:flex-end">%s</div>' % (gap, ''.join(cells)), bg)
def comp(cid, title, size, body, notes=''):
    return '<h3 id="%s"><span class="sid">%s</span>%s</h3><p class="cap">%s</p>%s%s' % (cid, cid, title, size, body, '<div class="notes">%s</div>' % notes if notes else '')
STATES = [('default', ''), ('hover', 'is-hover'), ('keyboard focus', 'is-focus'), ('pressed', 'is-press'), ('disabled', 'is-dis')]

def build():
    CUR['page'] = 'components.html'
    b = ('<p>Section 4 of the specification. Every component is drawn in each state it has, in light and dark side by side. The components and their look come from the Mortise identity and are not redesigned here; '
         'this page adds the states the identity did not draw (hover, focus, pressed, disabled, error) and the components it did not have (segmented control, table, tooltip, toast, dialog, scrollbar, progress, runtime marks).</p>'
         '<p><strong>States, one rule each.</strong> Hover: the fill becomes <code>well</code>. Pressed: the fill becomes <code>line</code>. Keyboard focus: a 2 px <code>ink</code> ring, 1 px outside the control, on top of any other state. '
         'Selected: the joint, or on a list row the well fill with a line above and below, and the peg. Disabled: the outline drops to <code>line</code> and the label to <code>ink-3</code>; a disabled control keeps its place and says why in a tooltip. '
         'Error: the outline becomes <code>bad</code> and a sentence with the problem mark sits under the field. Hover and pressed exist for the pointer only; nothing is revealed on hover that the keyboard cannot reach.</p>')
    b += comp('C1', 'Buttons', 'Height 26 px, small 22 px. Padding 10 px, small 8 px. Label 12 px weight 600, small 11.5 px. Icon 13 px with a 6 px gap. Radius 3 px. The pointer target is at least 24 px high even where the button is drawn 22 px.',
              ''.join(rowspec([cell(n, btn(lbl, '%s %s' % (k, s), icn)) for n, s in STATES]) for lbl, k, icn in
                      [('Create 3 files', 'pri', None), ('Copy command', '', None), ('Run tests', 'sm', 'test'), ('Roll back', 'danger', None), ('Roll back 1 migration', 'danger fill', None)])
              + rowspec([cell('ghost, default', btn('Hide', 'ghost')), cell('ghost, hover', btn('Hide', 'ghost is-hover')), cell('icon button', btn('', 'icon', 'copy')), cell('icon, hover', btn('', 'icon is-hover', 'copy')),
                         cell('icon, focus', btn('', 'icon is-focus', 'copy')), cell('with key hint', btn('Add app folder', 'pri', None, 'Ctrl O')), cell('with count', btn('Migrate (1)', 'sm pri', 'migrate'))]),
              '<div><b>Kinds.</b> Primary: one per view, ink fill. Default: surface fill, line-2 outline. Small: in headers and rows. Destructive outline: opens a confirmation. Destructive filled: only inside a confirmation dialog. Ghost: in the runner bar and dialogs for the quiet choice. Icon: square, always with a tooltip.</div>'
              '<div><b>Disabled destructive.</b> Same as a disabled default button: no red on something that cannot be used.</div>')
    b += comp('C2', 'Text field', 'Height 28 px, padding 9 px, text 12.5 px, radius 3 px. Label 11.5 px weight 600 above, 4 px gap; a hint in weight 400 follows the label. Paths, names and commands are set in Inconsolata.',
              rowspec([cell('default, empty', inp('<span class="ph">Model name</span>', '', '', 'width:170px')), cell('hover', inp('<span class="ph">Model name</span>', 'is-hover', '', 'width:170px')),
                       cell('keyboard focus', inp('PaymentIntent' + CARET, 'mono is-focus', '', 'width:170px')), cell('filled', inp('PaymentIntent', 'mono', '', 'width:170px')),
                       cell('disabled', inp('Billing', 'is-dis', '', 'width:170px')),
                       cell('error', inp('payment intent', 'mono is-err', '', 'width:260px') + '<div class="errmsg" style="max-width:260px">%s<span>%s</span></div>' % (S('bad'), t('x.field.name'))),
                       cell('with a button inside', inp('~/code/shop-api', 'mono', btn('Choose folder', 'sm'), 'width:260px')), cell('secret, saved', inp('<span style="display:flex;gap:6px;align-items:center">%s%s</span>' % (ic('lock', 12), t('m.cred')), 'is-dis', btn('Replace', 'sm'), 'width:260px'))]),
              '<div><b>Error.</b> Shown when the field loses focus or on submit, and cleared as soon as the value is valid. The sentence says what is wrong and gives a correct example.</div>')
    b += comp('C3', 'Filter field', 'Height 26 px, padding 8 px, text 12 px. Search icon 13 px on the left, key hint on the right. Used in the index, the generator picker and tables.',
              rowspec([cell('default', '<div class="filter" style="margin:0;width:220px">%s<span>Filter modules</span>%s</div>' % (ic('search'), kbd('/'))),
                       cell('hover', '<div class="filter is-hover" style="margin:0;width:220px">%s<span>Filter modules</span>%s</div>' % (ic('search'), kbd('/'))),
                       cell('keyboard focus, empty', '<div class="filter is-focus" style="margin:0;width:220px">%s<span>%sFilter modules</span>%s</div>' % (ic('search'), '<span class="caret" style="margin:0 2px 0 0"></span>', kbd('Esc'))),
                       cell('with text', '<div class="filter is-focus has" style="margin:0;width:220px">%s<span>ch%s</span>%s</div>' % (ic('search'), CARET, kbd('Esc')))], 'panel'),
              '<div>The key hint shows how to get in ( / ) and, once inside, how to get out (Esc). Esc clears the text first and leaves the field second.</div>')
    b += comp('C4', 'Checkbox', 'Box 14 px, outline 1.5 px, radius 2 px, 8 px to the label. The row is 24 px high and the whole row is the target.',
              rowspec([cell('off', chk('Migration', '-m')), cell('on', chk('Migration', '-m', True)), cell('mixed', chk('All of these', '-a', False, '', True)), cell('hover', chk('Migration', '-m', False, 'is-hover')),
                       cell('keyboard focus', '<div class="is-focus" style="padding:0 4px">%s</div>' % chk('Migration', '-m', True)), cell('disabled, off', chk('Fail CI when a rule breaks', '', False, 'is-dis')), cell('disabled, on', chk('Turn the module on', '', True, 'is-dis'))]))
    b += comp('C5', 'Segmented control', 'Track height 26 px with 2 px padding, segments 20 px high, 10 px padding, label 12 px weight 600. One segment is always selected. The selected segment is a surface piece with an ink outline, not a filled one, so the primary button stays the only ink block.',
              rowspec([cell('default', seg(['Herd', 'Sail', 'Docker', 'Custom'], 0)), cell('hover on Sail', seg(['Herd', '<span class="is-hover" style="border:0;padding:0;height:auto">Sail</span>', 'Docker', 'Custom'], 0).replace('<span class=""><span class="is-hover" style="border:0;padding:0;height:auto">Sail</span></span>', '<span class="is-hover">Sail</span>')),
                       cell('keyboard focus', '<span class="is-focus" style="display:inline-flex;border-radius:3px">%s</span>' % seg(['Light', 'Dark', 'System'], 2)),
                       cell('one segment not available', seg(['local', 'staging', 'production'], 0).replace('<span class="">production</span>', '<span class="is-dis">production</span>'))]),
              '<div>Left and Right change the selection when the control has focus. A segment that is not available is struck through and says why in a tooltip.</div>')
    tabsx = lambda cls: '<div style="background:var(--panel);border-bottom:1px solid var(--line-2);padding:8px 10px 0"><div class="tabs" style="padding:0">%s</div></div>' % cls
    b += comp('C6', 'Tab', 'Height 31 px, padding 12 px, label 12.5 px weight 500, 600 when selected. The digit is the key that opens the tab. The selected tab takes the surface colour and joins the workspace below it.',
              rowspec([cell('default, selected, hover on Generate', tabsx('<span class="tab">Summary<i>1</i></span><span class="tab on">Boundary<i>2</i></span><span class="tab is-hover">Generate<i>3</i></span><span class="tab">Actions<i>4</i></span>')),
                       cell('keyboard focus on the selected tab', tabsx('<span class="tab on is-focus">Summary<i>1</i></span><span class="tab">Boundary<i>2</i></span>')),
                       cell('keyboard focus on another tab', tabsx('<span class="tab on">Summary<i>1</i></span><span class="tab is-focus">Boundary<i>2</i></span>'))], 'surface'),
              '<div>A tab is never disabled. A tab that cannot do its work opens and explains why (%s).</div>' % ref('S7.4'))
    idxw = lambda rows: '<div style="width:264px;background:var(--panel);border-right:1px solid var(--line-2)"><div class="mods">%s</div></div>' % rows
    b += comp('C7', 'Index row', 'Two lines: 40 px high, stamp column 36 px, name 13 px weight 600, second line 11.5 px. One line: 26 px. The selected row takes the surface colour, a 1 px line-2 outline on three sides, runs 1 px over the joint line, and carries a 5 px peg.',
              rowspec([cell('default', idxw(mod_row('BIL'))), cell('hover', idxw(mod_row('BIL', cls='is-hover'))), cell('keyboard focus', idxw(mod_row('BIL', cls='is-focus'))), cell('selected', idxw(mod_row('CHK', 'CHK'))),
                       cell('selected, with focus', idxw(mod_row('CHK', 'CHK', cls='is-focus'))), cell('warning', idxw(mod_row('NTF'))), cell('turned off', idxw(row('NTF', 'Notifications', '38 files, turned off', S('off', 'Off'), False, 'off'))),
                       cell('with runtime status, Down', idxw(mod_row('BIL', rt='down'))), cell('one line', idxw(row('PYM', 'Payments', '', S('bad', '2'), False, 'c1'))), cell('one line, selected', idxw(row('PYM', 'Payments', '', S('bad', '2'), True, 'c1')))], 'frame', '14px 18px'),
              '<div><b>Focus and selection are different.</b> J and K move both together in the index. With Tab, focus can sit on a row that is not selected; Enter or Space selects it.</div>'
              '<div><b>For egui.</b> The index panel must not draw its own right edge. Paint the joint line as two segments that skip the selected row, then paint the row over it.</div>')
    tb = table('34px minmax(0,1fr) 60px 70px 110px', ['', 'Module', 'Files>', 'Coverage>', 'Findings'], [
        [stamp('BIL'), '<b style="font-weight:600">Billing</b>', 96, '81%', S('good', 'Good')], [stamp('CAT'), '<b style="font-weight:600">Catalog</b>', 121, '77%', S('good', 'Good')],
        [stamp('CHK'), '<b style="font-weight:600">Checkout</b>', 84, '54%', S('bad', '2 problems')], [stamp('INV'), '<b style="font-weight:600">Inventory</b>', 62, '72%', S('good', 'Good')],
        [stamp('NTF'), '<b style="font-weight:600">Notifications</b>', 38, '39%', S('warn', 'Low coverage')], [stamp('SHP'), 'Shipping', 71, '66%', 'not read yet']], 2, '', {1: 'is-hover', 3: 'is-focus', 5: 'is-dis'})
    tb = tb.replace('<span>Module</span>', '<span class="srt">Module %s</span>' % ic('chev-u', 10))
    b += comp('C8', 'Table header and row', 'Header 26 px, 11.5 px weight 600 in ink-2, line-2 underneath. Row 28 px, hairline between rows. Numbers are right aligned and tabular. The sorted column is drawn in ink with a chevron.',
              swatch('<div class="box flush" style="max-width:560px">%s</div><div style="font-size:11px;color:var(--ink-2);margin-top:6px">Rows from the top: default, hover, selected, keyboard focus, default, disabled. Header: sorted by Module, ascending.</div>' % tb),
              '<div>A header cell is a button: Enter or a click sorts, again reverses. J and K, or Up and Down, move the selection; Enter does what the "Enter opens" hint says.</div>')
    marks = lambda: ''.join(cell(c, S(k, w)) for k, w, c in [('good', 'Good', 'check'), ('warn', 'Low coverage', 'wedge'), ('bad', '2 problems', 'block with a cross'), ('off', 'Off', 'empty frame with a dash')])
    rmarks = lambda: ''.join(cell(c, S(k, w)) for k, w, c in [('up', 'Up', 'arrow up'), ('down', 'Down', 'arrow down'), ('oos', 'Out of service', 'two bars'), ('unk', 'Unknown', 'three dashes')])
    b += comp('C9', 'State marks', '12 px square, beside a word or a number wherever there is room, and with a tooltip where there is not. Two families that never mix.',
              rowspec([cell('Findings: solid silhouettes', '<div style="display:flex;gap:22px">%s</div>' % marks()), cell('Runtime: strokes, 1.7 px', '<div style="display:flex;gap:22px">%s</div>' % rmarks()),
                       cell('in a row, both families', '<span style="display:flex;gap:8px">%s%s</span>' % (S('up'), S('bad', '2'))), cell('20 px, in an empty state', S('warn', None, 20))]),
              '<div><b>Findings</b> are what Mortise works out from the files. <b>Runtime</b> is what the running app reports. A module can be Up with 2 problems, or Down and good; the two marks sit side by side and never stand in for each other.</div>'
              '<div><b>Good is the lightest mark</b> so that many healthy rows stay quiet. Up is drawn as lightly for the same reason.</div>')
    b += comp('C10', 'Module code stamp', '34 by 18 px, Inconsolata 10.5 px weight 600, letter spacing 0.03 em, radius 2 px, 1 px outline. Three capital letters, unique within an app.',
              rowspec([cell('default', stamp('CHK')), cell('in a selected row', '<span class="stamp" style="color:var(--ink);border-color:var(--ink)">CHK</span>'), cell('module turned off', '<span class="stamp" style="border-style:dashed">NTF</span>'),
                       cell('all modules', stamp('ALL')), cell('beside a title', '<span style="font-size:18px;font-weight:700;display:flex;gap:10px;align-items:baseline">Checkout %s</span>' % stamp('CHK')),
                       cell('in a sentence', '<span>Uses Billing %s</span>' % stamp('BIL'))], 'panel'),
              '<div><b>How a code is made.</b> Mortise proposes one from the name: the first three letters of a short name, or the first letter and the consonants that follow (Checkout: CHK, Notifications: NTF, Shipping: SHP). It must be unique within the app; ALL is reserved. The user can change a code in Settings. A code never changes by itself once given, so it can be learned. The exact proposal rule is for the builders; uniqueness and stability are the requirements.</div>')
    b += comp('C11', 'Key hint', 'Height 16 px, padding 4 px, Inconsolata 10.5 px weight 500, radius 2 px, well fill, line-2 outline. Keys of one chord share a cap; a sequence gets one cap per key.',
              rowspec([cell('single key', kbd('/')), cell('chord', kbd('Ctrl K')), cell('chord on macOS', kbd('Cmd K')), cell('in a button', btn('Create 3 files', 'pri', None, 'Ctrl Enter')),
                       cell('in the runner bar', '<span style="background:var(--code-bg);padding:6px 10px;display:inline-flex" class="drawer"><span class="kbd">Ctrl `</span></span>'), cell('a word as a hint', kbd('Change'))]))
    b += comp('C12', 'Tooltip', 'Key fill, key-fg text 11.5 px, padding 4 by 8 px, radius 3 px, at most 240 px wide, no shadow. Appears 500 ms after the pointer rests, at once on keyboard focus, 6 px from the control.',
              rowspec([cell('name', '<span class="tip">%s</span>' % t('tip.tile')), cell('name and shortcut', '<span class="tip">%s%s</span>' % (t('tip.add'), kbd('Ctrl O'))), cell('why a control is disabled', '<span class="tip">%s</span>' % t('tip.nophp')),
                       cell('a mark without a word', '<span class="tip">%s</span>' % t('tip.rt')), cell('where a figure comes from', '<span class="tip">%s</span>' % t('tip.cov'))]),
              '<div>A tooltip names, it does not instruct. Anything a user needs in order to act is on screen without it. Other tooltips: "%s", "%s", "%s".</div>' % (t('tip.settings'), t('tip.stamp'), t('tip.sort')))
    b += comp('C13', 'Toast', '320 px wide, surface fill, 1 px ink outline, radius 4 px, one shadow. Bottom right of the workspace, 16 px from its edges, above the runner drawer. One at a time; a new one replaces the old.',
              rowspec([cell('done, leaves after 4 s', toast('good', t('m.created'))), cell('failed, stays until closed', toast('bad', 'module:migrate Checkout stopped with exit code 1.', btn('Show output', 'sm'))),
                       cell('note', toast('good', t('m.copied')))]),
              '<div>Toasts confirm something that happened out of sight. They carry at most one action. Esc closes the toast when nothing else is open. Other toasts: %s</div>' % ' &nbsp; '.join(t(k) for k in ('m.added', 'm.turnedoff', 'm.turnedon', 'm.migrated', 'm.rolled', 'm.seeded', 'm.tests', 'm.stopped')))
    b += comp('C14', 'Dialog', '440 px wide, surface body with 16 px padding, panel footer, 1 px ink outline, radius 4 px, the overlay shadow, over the scrim. Title 15 px weight 700.',
              '<div class="pair">%s%s</div>' % tuple('<div class="m fa" data-theme="%s"><div style="background:var(--surface)"><div style="background:var(--scrim);padding:20px;display:flex;justify-content:center">%s</div></div></div>' % (th, d)
                                                 for th, d in (('light', dlg_rollback()), ('dark', dialog(t('c.off.h'), t('c.off.p'), '', btn(t('b.cancel'), 'is-focus', None, 'Esc') + btn(t('c.off.b'), 'pri'))))),
              '<div>Left: a destructive confirmation, focus on Cancel. Right: a confirmation that loses nothing, with an ink primary button. Dialogs are used only to confirm; forms live in the workspace. Rules and the list of what asks first are in %s.</div>' % ref('S10.3'))
    hd = lambda cls: '<div style="width:220px;border:1px solid var(--line-2);border-top:0"><div style="height:22px;background:var(--surface)"></div><div class="handle %s"><i></i></div><div style="height:26px;background:var(--code-bg)"></div></div>' % cls
    b += comp('C15', 'Drawer handle', 'The top 7 px of the runner drawer. A 1 px line-2 edge with a 28 px grip. The pointer changes to the vertical resize shape.',
              rowspec([cell('default', hd('')), cell('hover', hd('is-hover')), cell('dragging', hd('is-press')), cell('keyboard focus', '<div class="is-focus">%s</div>' % hd(''))]),
              '<div>With focus, Up and Down change the height by 24 px, Home and End jump to the highest and lowest. Double click, or Ctrl `, collapses the drawer to its 28 px line.</div>')
    sc = lambda cls, bg='surface', th='': '<div style="display:flex;height:96px;width:120px;background:var(--%s);border:1px solid var(--line-2)"><div style="flex:1"></div><div class="scrollv"><i class="%s" style="top:8px;height:36px;%s"></i></div></div>' % (bg, cls, th)
    b += comp('C16', 'Scrollbar', 'Track 10 px, no fill. Thumb 6 px wide, radius 3 px, at least 24 px long. It floats over the right padding of the content and takes no width of its own.',
              rowspec([cell('default', sc('')), cell('hover', sc('is-hover')), cell('dragging', sc('is-press')), cell('in the runner', sc('', 'code-bg', 'background:var(--code-dim)'))]),
              '<div>Always visible while the content is longer than its area, so there is no need to move the pointer to learn that a list continues. Horizontal bars are the same, turned.</div>')
    b += comp('C17', 'Progress and busy', 'Bar 6 px high, well track, ink fill, no radius. Busy is the run icon and elapsed time; there is no spinner anywhere.',
              rowspec([cell('known amount', '<div style="width:240px"><div style="display:flex;justify-content:space-between;font-size:11.5px;margin-bottom:4px"><span>%s</span><span class="mono">57%%</span></div><span class="prog"><i style="width:57%%"></i></span></div>' % t('m.scan')),
                       cell('busy, unknown amount', '<span class="busy">%s%s</span>' % (ic('run', 12), t('m.running'))), cell('busy, in a button', btn('Test, 0.8 s', 'is-dis', 'run')),
                       cell('on a rail tile while reading an app', '<div style="background:var(--frame);padding:8px"><div class="tile" style="overflow:hidden">BE<span style="position:absolute;left:0;right:0;bottom:0;height:2px;background:var(--line)"><i style="display:block;width:57%;height:2px;background:var(--ink)"></i></span></div></div>')]),
              '<div>The bar is drawn only after 300 ms, so quick work does not flash. Elapsed time counts in tenths of a second up to 10 s, then in seconds.</div>')
    emp = '<div class="empty" style="min-height:0;padding:20px 22px;border:1px solid var(--line-2)">%s<h4>%s</h4><p>%s</p><div class="btns">%s</div></div>' % (ic('metrics', 28), t('e.metrics.h'), t('e.metrics.p'), btn(t('b.connect'), 'pri'))
    b += comp('C18', 'Empty-state block', 'Left aligned, 26 by 28 px padding, 12 px between parts. A 28 px icon or the mark, a title 18 px weight 700, one paragraph of at most 46 characters a line, then the action.', swatch(emp),
              '<div>An empty state says what is missing and gives the one action that fills it. Other titles and paragraphs: "%s" %s &nbsp; "%s" %s &nbsp; "%s" %s</div>' % (t('e.nophp.h'), t('e.nophp.p'), t('e.oldpkg.h'), t('e.oldpkg.p'), t('e.later.h'), t('e.later.p')))
    b += comp('C19', 'Alert', 'Padding 9 by 12 px, radius 3 px, 1 px outline, mark 12 px with a 9 px gap, text 12.5 px. Sits at the top of the view it concerns.',
              swatch('<div class="stack" style="max-width:640px">%s%s%s</div>' % (alert(t('f.internals')), alert(t('f.coverage'), 'warn'), alert('platform has 60 modules. Above 12 modules the graph of the whole app gives way to this table.', 'note'))),
              '<div>Problem: bad-soft fill, bad outline, problem mark. Warning: surface fill, warn outline, wedge. Note: panel fill, line-2 outline, no mark. An alert may carry one small button on the right.</div>')
    return page('components.html', 'Components', 'Mortise design specification, section 4. Every component in every state, with sizes in pixels.', b, switch=False)
