from lib import *
from data import *

def shell2(r, w, style=''):
    return '<div style="display:grid;grid-template-columns:var(--rail-w) minmax(0,1fr);min-height:var(--shell-h);%s">%s%s</div>' % (style, r, w)

def first_run(th='light', sw=False):
    body = ('<div class="work"><div class="empty" style="flex:1">%s<h4>%s</h4><p>%s</p><div class="btns">%s%s</div></div></div>') % (
        bm(36), t('e.noapps.h'), t('e.noapps.p').replace('nwidart/laravel-modules', '<span class="mono nowrap">nwidart/laravel-modules</span>'),
        btn(t('b.addfolder'), 'pri', None, 'Ctrl O'), btn(t('b.palette'), '', None, 'Ctrl K'))
    return win(tbar('No app', find=True) + shell2(rail([], None), body, 'min-height:340px'), sw=sw, cmp=not sw, theme=th)

def add_work(body, title='Add app'):
    return ('<div class="work"><div class="wtop"><div class="whead" style="padding-bottom:12px"><div class="wtitle"><h4>%s</h4></div>'
            '<div class="acts"><span class="dim" style="font-size:11.5px">Back</span>%s</div></div></div><div class="wbody">%s</div></div>') % (title, kbd('Esc'), body)

FOLDER_EMPTY = field('App folder', 'the folder that holds artisan', '<div class="inp is-focus"><span class="ph">Path to the app, or drop the folder on this window' + CARET + '</span></div>')
def found(rows): return box('Found in this folder', 'read from disk, nothing run', ''.join(kv(a, b) for a, b in rows))

def add_empty():
    left = '<div class="frm">%s<div class="btns">%s<span class="dim" style="font-size:11.5px">Opens the folder dialog of the operating system</span></div></div>' % (FOLDER_EMPTY, btn(t('b.choose'), 'pri', 'folder'))
    right = alert('Mortise reads <span class="mono">composer.lock</span>, the <span class="mono">module.json</span> files and <span class="mono">modules_statuses.json</span>. It writes nothing into the app.', 'note')
    return win(tbar('No app') + shell2(rail([], None, add_on=True), add_work('<div class="cols">%s<div>%s</div></div>' % (left, right)), 'min-height:300px'))

def php_field(on=0, cmdline='herd php artisan', result=None, hint='Found: this folder is a Herd site.'):
    res = result or '<div class="hint2" style="display:flex;gap:8px;align-items:center">%s<span class="mono">%s</span></div>' % (S('good', 'Answered'), t('m.php.ok'))
    return field('Run PHP with', hint, '<div style="display:flex;flex-direction:column;gap:6px;align-items:flex-start">%s<div style="display:flex;gap:8px;align-self:stretch">%s%s</div>%s</div>' % (
        seg(['Herd', 'Sail', 'Docker', 'Custom'], on), inp(cmdline, 'mono', '', 'flex:1'), btn(t('b.test'), ''), res))

BUL = '&bull;' * 16
def CREDS(): return [t('cr.none'), t('cr.bearer'), t('cr.basic'), t('cr.header')]
def env_fields(name='local', url='https://booking-engine.test', cred=0, result=True, secret=''):
    r = '<div class="hint2" style="display:flex;gap:8px;align-items:center">%s<span>%s</span></div>' % (S('up', 'Up'), t('m.conn.ok').replace('Up. ', '')) if result else ''
    return ('<div style="display:grid;grid-template-columns:110px minmax(0,1fr) 110px;gap:8px">%s%s%s</div>%s'
            '<div style="display:flex;gap:8px;align-items:flex-end;margin-top:0">%s<div style="margin-bottom:12px">%s</div>%s</div>%s') % (
        field('Name', '', inp(name)), field('Base URL', 'from APP_URL in .env', inp(url, 'mono')), field('Actuator path', '', inp('actuator', 'mono')),
        field('Credentials', 'what the app\'s middleware asks for', seg(CREDS(), cred)), secret, btn(t('b.test')), '', r)

def add_ready():
    left = ('<div class="frm">%s%s<div class="sub">Runtime, optional</div>%s<div class="btns" style="margin-top:14px">%s%s</div></div>') % (
        field('App folder', '', inp('~/code/booking-engine', 'mono', btn(t('b.choose'), 'sm'))), php_field(), env_fields(cred=1, secret='<div class="field" style="flex:1;margin-bottom:12px"><div class="lbl">%s</div>%s</div>' % (t('cr.token'), inp('<span class="mono">%s</span>' % BUL))) + '<div class="hint2" style="display:flex;gap:6px;align-items:center">%s%s</div>' % (ic('lock', 12), t('cr.where')),
        btn(t('b.addapp'), 'pri', None, 'Ctrl Enter'), btn(t('b.cancel')))
    right = found([('Laravel', '13.2 <span class="dim">composer.lock</span>'), ('<span class="mono">nwidart/laravel-modules</span>', '13.3.0'),
                   ('Modules', '5 in <span class="mono">Modules/</span>, 5 on'), ('<span class="mono">nwidart/laravel-actuator</span>', '1.0.0'),
                   ('Coverage report', '<span class="dim">none found, coverage shows as unknown</span>'), ('Git', 'branch <span class="mono">main</span>')])
    right += '<div style="margin-top:12px">%s</div>' % box('Shown in the rail as', 'you can change both in Settings', kv('Name', 'booking-engine') + kv('Monogram', '<span class="tile" style="width:28px;height:28px">BE</span>'))
    return win(tbar('No app') + shell2(rail(APPS3, None, add_on=True), add_work('<div class="cols">%s<div>%s</div></div>' % (left, right)), ''))

def add_fail(kind):
    if kind == 'laravel':
        b = field('App folder', '', inp('~/code/notes', 'mono is-err', btn(t('b.choose'), 'sm')) + '<div class="errmsg">%s<span>%s</span></div>' % (S('bad'), t('x.notlaravel')))
        b += '<div class="btns">%s%s</div>' % (btn(t('b.addapp').replace('booking-engine', 'app'), 'pri is-dis'), btn(t('b.cancel')))
    elif kind == 'pkg':
        b = field('App folder', '', inp('~/code/booking-engine', 'mono is-err', btn(t('b.choose'), 'sm')) + '<div class="errmsg">%s<span>%s</span></div>' % (S('bad'), t('x.nopkg')))
        b += cmd('composer', 'require', 'nwidart/laravel-modules') + '<div class="btns" style="margin-top:10px">%s%s</div>' % (btn(t('b.copycmd')), btn('Check again', 'pri'))
    else:
        b = field('App folder', '', inp('~/code/booking-engine', 'mono', btn(t('b.choose'), 'sm')))
        b += php_field(0, 'herd php artisan', '<div class="errmsg">%s<span>%s</span></div>' % (S('bad'), t('x.nophp')), 'Not found: Herd, Sail, a compose file.')
        b += '<div class="btns">%s%s</div>' % (btn(t('b.addro'), 'pri'), btn(t('b.cancel')))
    return '<div class="mwrap"><div class="m fa sw" data-theme="light"><div class="win sm">%s</div></div></div>' % add_work('<div>%s</div>' % b)

def rail_fig(apps, sel, cap, th, tip=None, w=54, scroll=False):
    tipx = '<div style="position:absolute;left:50px;top:%dpx;white-space:nowrap" class="tip">landing-pages, 1 problem</div>' % tip if tip else ''
    r = rail(apps, sel)
    return '<figure><div class="m fa" data-theme="%s"><div class="win sm" style="width:%dpx;position:relative;overflow:visible;background:var(--panel)"><div style="display:grid;border-radius:7px 0 0 7px;overflow:hidden;width:53px;border-right:1px solid var(--line-2)">%s</div>%s</div></div><figcaption>%s</figcaption></figure>' % (
        th, w, r.replace('class="rail"', 'class="rail" style="height:%dpx;border-right:0"' % (560 if len(apps) > 3 else 300)), tipx, cap)

def idx_fig(inner, cap, h=470, w=266):
    return ('<figure><div class="m fa sw" data-theme="light"><div class="win sm" style="width:%dpx;border-radius:0"><div class="idx" style="min-height:%dpx;border-right:0;position:relative">%s</div></div></div>'
            '<figcaption style="max-width:%dpx">%s</figcaption></figure>') % (w, h, inner, w, cap)
def idx_inner(head, rows, foot):
    return '%s<div class="mods">%s</div>%s' % (head, rows, foot)

def idx_default():
    rows = all_row('ALL') + ''.join(mod_row(c) for c in BYNAME)
    return idx_fig(idx_inner(idx_head(), rows, idx_foot(t('sort.name'))), 'Default. Order by name, "All modules" always first. Two lines per row up to 20 modules.')
def idx_problems():
    rows = all_row() + ''.join(mod_row(c, 'CHK') for c in PROBLEMS_FIRST)
    return idx_fig(idx_inner(idx_head(), rows, idx_foot(t('sort.problems'))), 'Problems first (S). Problems, then warnings, then the rest by name. Checkout is selected.')
def idx_filtered():
    f = '<div class="filter is-focus has">%s<span>ch%s</span>%s</div>' % (ic('search'), CARET, kbd('Esc'))
    rows = all_row() + mod_row('CHK', 'CHK')
    return idx_fig(idx_inner(idx_head(filt=f), rows, idx_foot('1 of 7 modules', False)), 'Filtered. "ch" matches the name and the code. "All modules" stays. The first match is selected, so Enter opens it.')
def idx_nomatch():
    f = '<div class="filter is-focus has">%s<span>paym%s</span>%s</div>' % (ic('search'), CARET, kbd('Esc'))
    rows = all_row() + '<div style="padding:12px 12px 12px 14px;color:var(--ink-2)">%s</div>' % t('e.nomatch')
    return idx_fig(idx_inner(idx_head(filt=f), rows, idx_foot('0 of 7 modules', False)), 'Filter with no match.')
def idx_off():
    rows = all_row() + ''.join(
        (row('NTF', 'Notifications', '38 files, turned off', S('off', 'Off'), False, 'off') if c == 'NTF' else mod_row(c)) for c in BYNAME)
    return idx_fig(idx_inner(idx_head(on='6 of 7 on'), rows, idx_foot(t('sort.name'))), 'A module turned off. Dashed stamp, lighter name, the empty frame and the word Off. It keeps its place in the order and has no finding state.')
def idx_runtime():
    rows = all_row('ALL') + ''.join(mod_row(c, None, True, 'down' if c == 'BIL' else 'up') for c in PROBLEMS_FIRST)
    return idx_fig(idx_inner(idx_head(rt='down'), rows, idx_foot(t('sort.problems'))), 'Runtime status per module, planned. The stroke mark on the left is what the running app reports; the solid mark on the right is the finding state. Billing is down and has no finding.')

P60 = [('grp', 'Needs attention', '5'), ('CHK', 'Checkout', S('bad', '2')), ('PYM', 'Payments', S('bad', '2')), ('SRC', 'Search', S('bad', '1')),
       ('IMP', 'Imports', S('warn')), ('NTF', 'Notifications', S('warn')),
       ('grp', 'Commerce', '16'), ('CRT', 'Cart', S('good')), ('CAT', 'Catalog', S('good')), ('CPN', 'Coupons', S('good')), ('DSC', 'Discounts', S('good')),
       ('FLF', 'Fulfilment', S('good')), ('GFT', 'GiftCards', 'off'), ('INV', 'Inventory', S('good')), ('INS', 'Invoices', S('good')), ('ORD', 'Orders', S('good')),
       ('PRC', 'Pricing', S('good')), ('PRD', 'Products', S('good')), ('PRO', 'Promotions', S('good'))]
def rows60(sel='ALL'):
    out = row('ALL', 'All modules', '4,912 files, 68% covered', S('bad', '5'), sel == 'ALL')
    for c, n, m in P60:
        if c == 'grp': out += '<div class="grp">%s<span>%s</span></div>' % (n, m)
        elif m == 'off': out += row(c, n, '', S('off', 'Off'), False, 'c1 off')
        else: out += row(c, n, '', m, c == sel, 'c1')
    return out
def idx60(sel='ALL', h=640, border=False):
    head = idx_head('platform', '57 of 60 on', 'Laravel 13.2, package 13.3.0')
    sc = '<div class="scrollv" style="position:absolute;right:0;top:%dpx;bottom:38px"><i style="top:2px;height:150px"></i></div>' % 104
    return '%s<div class="mods" style="overflow:hidden">%s</div>%s%s' % (head, rows60(sel), idx_foot(t('sort.problems')), sc)
def idx_60():
    return idx_fig(idx60(), '60 modules. One line per row above 20 modules, groups from the app\'s own list, and under "problems first" a pinned group on top. The scrollbar floats over the row padding so the selected row still reaches the joint.', 640)

def build():
    CUR['page'] = 'screens-start.html'
    b = '<p>Section 5 of the specification, part 1 of 4: getting an app into Mortise, the app rail and the module index. Screen ids such as <span class="sid">S1.1</span> are stable and are what the flows refer to. Shortcuts are written for Windows and Linux; on macOS Ctrl is Cmd.</p>'
    b += '<h2>5.1 First run and adding an app</h2>'
    b += screen('S1.1', 'First run, no apps', 'Light and dark side by side. The module index is absent because no app is selected; the rail holds only the add tile and Settings.',
                both(lambda th: first_run(th)), '1',
                'None. The app list lives in Mortise\'s own config file.',
                more=['<b>Keys.</b> Ctrl O opens the folder dialog. Ctrl K opens the palette, which offers "Add app" and "Settings" only. Enter activates the focused primary button.'])
    b += screen('S1.2', 'Add app: choose the folder', 'The add tile on the rail has keyboard focus. The panel takes the place of the index and the workspace until the app is added or Esc is pressed.',
                add_empty(), '1',
                'The folder comes from the operating system\'s folder dialog (the one native dialog in the app), from a path typed or pasted into the field, or from a folder dropped on the window.',
                more=['<b>States.</b> While Mortise reads the folder the button shows <span class="mono">%s</span> with a bar. Reading 551 files takes well under a second, so the bar is drawn only after 300 ms.' % t('m.scan')])
    b += screen('S1.3', 'Add app: what was found, how PHP runs, runtime connection', 'After a folder is chosen. Everything on the right is read from disk. The two sections on the left are the only choices.',
                add_ready(), '1 for the folder and the runtime section. The "Run PHP with" section appears from phase 2.',
                'Versions from <span class="mono">composer.lock</span>. Modules from the <span class="mono">module.json</span> files and <span class="mono">modules_statuses.json</span>. Herd, Sail and Docker are detected from the folder: a Herd site link, <span class="mono">vendor/bin/sail</span>, a compose file. From <span class="mono">.env</span> Mortise reads two lines and no others: <span class="mono">APP_URL</span>, to prefill the base URL, and <span class="mono">APP_ENV</span>, to ask before a command in production. Test runs <span class="mono">artisan --version</span> and one request to the actuator.',
                more=['<b>Credentials.</b> Four kinds: none, bearer token, HTTP basic, and a custom header with a name and a value. The actuator runs whatever middleware the app configures, and its <span class="mono">viewActuator</span> gate needs an authenticated user, which in Laravel is usually a token guard: hence the bearer token. Basic and a custom header cover a proxy in front of the app. All four are drawn in %s.' % ref('S8.5'),
                      '<b>Secrets.</b> A token, password or header value goes to the credential store of the operating system. It is never written to <span class="mono">mortise.json</span> or to Mortise\'s own config file, and after entry it is not shown again (%s).' % ref('S8.1'),
                      '<b>Runtime is optional.</b> The app is added without it. The section is hidden when <span class="mono">composer.lock</span> has no <span class="mono">nwidart/laravel-actuator</span>; the Summary then says how to install it.'])
    b += '<h3 id="S1.4"><span class="sid">S1.4</span>Add app: the three failures</h3><p class="cap">Each failure is shown in the panel, at the field it belongs to, with the next step.</p>'
    SCREENS['S1.4'] = (CUR['page'], 'Add app: the three failures')
    b += '<div class="trio"><div>%s<p class="cellcap">a. Not a Laravel app. The add button is disabled until the folder changes.</p></div><div>%s<p class="cellcap">b. The package is not installed. The command can be copied. "Check again" reads composer.lock again.</p></div><div>%s<p class="cellcap">c. PHP not found, phase 2. The app can still be added and read; Generate and Actions then say what is missing.</p></div></div>' % (
        add_fail('laravel'), add_fail('pkg'), add_fail('php'))
    b += '<div class="notes"><div><b>Phase.</b> a and b: 1. c: 2.</div><div><b>Other failures, same pattern.</b> %s &nbsp; %s</div><div><b>Assumes.</b> Nothing the packages lack today.</div></div>' % (t('x.nolock'), t('x.dup'))

    b += '<h2>5.2 App rail</h2>'
    b += '<h3 id="S2.1"><span class="sid">S2.1</span>App rail with 1 app and with 10 apps</h3><p class="cap">Light, then dark. The rail is 52 px wide and never changes width.</p>'
    SCREENS['S2.1'] = (CUR['page'], 'App rail with 1 app and with 10 apps')
    b += '<div class="strip">%s%s%s%s%s%s</div>' % (
        rail_fig(APPS3[:1], 'SH', '1 app', 'light'), rail_fig(APPS10, 'SH', '10 apps', 'light'),
        rail_fig(APPS10, 'PL', 'pointer on LP', 'light', 92, 230),
        rail_fig(APPS3[:1], 'SH', '1 app', 'dark'), rail_fig(APPS10, 'SH', '10 apps', 'dark'), rail_fig(APPS10, 'PL', 'pointer on LP', 'dark', 92, 230))
    b += ('<div class="notes"><div><b>Phase.</b> 1.</div><div><b>Tile.</b> 34 px square, 8 px apart. Two letter monogram: the initials of the first two words of the folder name, or its first two letters. '
          'If that is taken, the first two letters; if that is taken too, the first letter and the next free letter. The user can change it in Settings.</div>'
          '<div><b>Pip.</b> A 9 px problem mark on a tile means the app has at least one finding of the kind problem. Warnings do not raise a pip. Runtime status is not shown on the rail: apps that are not running locally are normal.</div>'
          '<div><b>Order.</b> The order apps were added in, changed by dragging or in Settings. Ctrl Alt 1 to 9 select the first nine.</div>'
          '<div><b>Overflow.</b> The add tile and Settings stay pinned at the bottom. When the tiles do not fit between them, the tile list scrolls with the wheel and with Up and Down; no scrollbar is drawn in a 52 px column, a 1 px line at the top or bottom edge shows there is more.</div>'
          '<div><b>Outlines on frame.</b> Tile outlines use ink-2, not line-2, which is under 3 to 1 on frame in the light theme (see Tokens).</div>'
          '<div><b>Assumes.</b> Nothing the packages lack today.</div></div>')

    b += '<h2>5.3 Module index</h2>'
    b += '<h3 id="S3.1"><span class="sid">S3.1</span>Module index: default, problems first, filtered, no match, a module turned off</h3><p class="cap">The index column alone, 264 px wide. Use the theme switch for dark.</p>'
    SCREENS['S3.1'] = (CUR['page'], 'Module index: default, problems first, filtered, a module turned off')
    b += '<div class="strip">%s%s%s%s</div><div class="strip">%s%s%s</div>' % (idx_default(), idx_problems(), idx_filtered(), idx_nomatch(), idx_off(), idx_runtime(), idx_60())
    for a in ('P9',): AUSED.setdefault(a, []).append('S3.1')
    b += ('<div class="notes"><div><b>Phase.</b> 1. "New module" in the footer appears from phase 2.</div>'
          '<div><b>Data.</b> Module list and on or off state from <span class="mono">module.json</span> and <span class="mono">modules_statuses.json</span>; files counted on disk; coverage from the last <span class="mono">clover.xml</span>; finding state from the checks in %s.</div>'
          '<div><b>Header.</b> App name, modules on, versions. The line with the stroke mark is the runtime status of the selected environment and is drawn only when one is connected.</div>'
          '<div><b>Order.</b> By name is the default because a module then keeps its place. S switches to problems first and back; the choice is kept per app. The shop-api drawings in the rest of the spec use problems first.</div>'
          '<div><b>Finding state of a row.</b> The worst finding of the module: problem with its count, warning, or good. A circular link is counted once, on the module with fewer files in the circle, because that is the cheaper side to cut; the other module shows the link in its Boundary tab.</div>'
          '<div><b>Groups.</b> Group names and membership are kept by Mortise per app, not by the package. Without groups the list is flat.</div>'
          '<div><b>Assumes, not in the package today.</b> Only the sixth drawing: <a class="sid" href="open.html#a-P9">P9</a> runtime status per module. Without it the row has one mark.</div></div>') % ref('S6.4')
    return page('screens-start.html', 'Screens: start', 'Mortise design specification, section 5, part 1. First run, add app, app rail, module index.', b)
