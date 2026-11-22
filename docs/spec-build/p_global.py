from lib import *
from data import *
from p_read import ALERT_CHK, FILES, DEPS, modstate
from p_start import CREDS, BUL

def palr(l, hint, on=False):
    return '<div class="pal-r%s"><span class="l">%s</span><span class="hint">%s</span></div>' % (' on' if on else '', l, hint)
def pal(query, body, ph=False, style=''):
    q = '<span class="q ph">%s</span>' % query if ph else '<span class="q">%s%s</span>' % (query, CARET)
    if ph: q = '<span class="q"><span class="caret" style="margin:0 2px 0 0"></span><span class="ph">%s</span></span>' % query
    return ('<div class="pal" style="%s"><div class="pal-in">%s%s%s</div>%s'
            '<div class="pal-f"><span>Up and Down to move, Enter to run, Tab to switch app</span><span>shop-api on Herd</span></div></div>') % (style, ic('search', 16), q, kbd('Esc'), body)
def pal_results():
    b = '<div class="pal-g">Modules in shop-api</div>' + palr('<b>Checkout</b> %s %s' % (stamp('CHK'), S('bad', '2 problems')), 'Open ' + kbd('Enter'), True)
    b += '<div class="pal-g">Actions for Checkout</div>'
    b += palr('Migrate <b>Checkout</b> <span class="mono dim">module:migrate Checkout</span>', '1 waiting')
    b += palr('Run tests of <b>Checkout</b> <span class="dim">31 tests</span>', 'Action')
    b += palr('Check translations in <b>Checkout</b> <span class="mono dim">module:lang</span>', 'Action')
    b += '<div class="pal-g">Generators</div>' + palr('Generate a model in <b>Checkout</b> <span class="mono dim">module:make-model</span>', 'Form')
    b += '<div class="pal-g">Findings</div>' + palr('<b>Checkout</b> and <b>Shipping</b> use each other', 'Boundary')
    b += palr('3 files in <b>Checkout</b> read Billing internals', 'Boundary')
    return pal('check', b)
def pal_empty():
    b = '<div class="pal-g">Recent</div>' + palr('<b>Checkout</b> %s' % stamp('CHK'), 'Open ' + kbd('Enter'), True)
    b += palr('Migrate <b>Checkout</b> <span class="mono dim">module:migrate Checkout</span>', 'ran 2 minutes ago')
    b += palr('Generate a model in <b>Billing</b> <span class="mono dim">module:make-model</span>', 'Form')
    b += '<div class="pal-g">Go to</div>' + palr('All modules', 'Summary') + palr('Findings in shop-api %s' % S('bad', '2 problems'), kbd('F')) + palr('Settings', kbd('Ctrl ,'))
    b += '<div class="pal-g">Apps</div>' + palr('crm', kbd('Ctrl Alt 2')) + palr('landing-pages %s' % S('bad', '1 problem'), kbd('Ctrl Alt 3')) + palr('Add app', kbd('Ctrl O'))
    return pal('Search modules, generators and actions', b, True, 'position:static;transform:none;margin:0 auto;width:auto;max-width:560px')
def pal_none():
    b = '<div style="padding:18px 14px 14px;color:var(--ink-2);line-height:1.5">%s</div>' % t('e.palnone')
    return pal('zzq', b, False, 'position:static;transform:none;margin:0 auto;width:auto;max-width:560px')
def pal_stage(p): return '<div class="m fa sw" data-theme="light"><div style="background:var(--well);padding:22px 0;border:1px solid var(--line-2)">%s</div></div>' % p
def palette_full():
    body = ALERT_CHK + '<div class="cols">%s<div class="stack">%s%s</div></div>' % (FILES, DEPS, modstate())
    return win(tbar() + shell(rail(), index('CHK'), work('Checkout', 0, body, MOD_ACTS, '', 'CHK', 'Modules/Checkout')), over=pal_results())

# ------------------------------------------------------------ settings
SECTIONS = [('Apps', 'app'), ('PHP', 'run'), ('General', 'settings'), ('Keyboard', 'list')]
def set_idx(sel):
    rows = ''.join('<div class="row c1%s" style="min-height:32px"><span style="display:flex;justify-content:center;color:var(--ink-2)">%s</span><div class="t"><div class="n">%s</div></div><span></span></div>' % (
        ' on' if i == sel else '', ic(icn, 14), n) for i, (n, icn) in enumerate(SECTIONS))
    return ('<div class="idx"><div class="idx-head"><div class="app-name">Settings<span>Mortise 0.1</span></div><div class="app-sub">Kept on this computer</div></div>'
            '<div class="mods">%s</div><div class="idx-foot"><span>Back to shop-api</span>%s</div></div>') % (rows, kbd('Esc'))
def set_win(sel, title, body, h='--shell-h:520px'):
    w = ('<div class="work"><div class="wtop"><div class="whead" style="padding-bottom:12px"><div class="wtitle"><h4>%s</h4></div></div></div><div class="wbody">%s</div></div>') % (title, body)
    return win(tbar() + shell(rail(APPS3, None, settings=True), set_idx(sel), w, h))
def set_apps():
    apps = table('40px minmax(0,1fr) minmax(0,1.3fr) 190px 70px 100px', ['', 'App', 'Folder', 'Versions', 'PHP', 'Environments'], [
        ['<span class="tile" style="width:26px;height:22px;font-size:10px">SH</span>', '<b style="font-weight:600">shop-api</b>', '<span class="mono">~/code/shop-api</span>', 'Laravel 13.2, package 13.3.0', 'Herd', '3'],
        ['<span class="tile" style="width:26px;height:22px;font-size:10px">CR</span>', '<b style="font-weight:600">crm</b>', '<span class="mono">~/code/crm</span>', 'Laravel 13.0, package 13.1.0', 'Sail', '1'],
        ['<span class="tile" style="width:26px;height:22px;font-size:10px">LP</span>', '<b style="font-weight:600">landing-pages</b>', '<span class="mono">~/code/landing-pages</span>', 'Laravel 13.2, package 13.3.0', 'Docker', 'none']], 0)
    envs = table('90px minmax(0,1.4fr) 80px minmax(0,1fr) 130px', ['Environment', 'Base URL', 'Path', 'Credentials', 'Last answer'], [
        ['<b style="font-weight:600">local</b>', '<span class="mono">https://shop-api.test</span>', '<span class="mono">actuator</span>', 'None', S('up', 'Up')],
        ['<b style="font-weight:600">staging</b>', '<span class="mono">https://staging.shop-api.example</span>', '<span class="mono">actuator</span>', '%s Bearer token, saved' % ic('lock', 12), S('up', 'Up')],
        ['<b style="font-weight:600">production</b>', '<span class="mono">https://shop-api.example</span>', '<span class="mono">actuator</span>', '%s HTTP basic, saved' % ic('lock', 12), S('unk', 'Not authorized')]], 1)
    edit = ('<div style="display:grid;grid-template-columns:110px minmax(0,1fr) 110px;gap:8px">%s%s%s</div>'
            '<div style="display:flex;gap:10px;align-items:flex-end;flex-wrap:wrap">%s<div class="field" style="flex:1;min-width:240px"><div class="lbl">Token</div>'
            '<div class="inp is-dis"><span style="display:flex;gap:6px;align-items:center">%s%s</span><span class="btns">%s%s</span></div></div></div>') % (
        field('Name', '', inp('staging')), field('Base URL', '', inp('https://staging.shop-api.example', 'mono')), field('Actuator path', '', inp('actuator', 'mono')),
        field('Credentials', '', seg(CREDS(), 1)), ic('lock', 12), t('m.cred'), btn(t('b.replace'), 'sm'), btn(t('b.remove'), 'sm'))
    detail = ('<div class="sub" style="margin-top:6px">shop-api</div><div style="display:grid;grid-template-columns:minmax(0,1fr) 84px minmax(0,1.5fr) minmax(0,1.4fr) 100px;gap:8px">%s%s%s%s%s</div>'
              '<div class="sub" style="display:flex;justify-content:space-between;align-items:baseline"><span>Environments of shop-api</span>%s</div><div class="box flush">%s</div>%s'
              '<div class="btns" style="margin-top:2px">%s</div>') % (
        field('Name', '', inp('shop-api')), field('Monogram', '', inp('SH', 'mono')), field('Folder', '', inp('~/code/shop-api', 'mono', btn(t('b.choose'), 'sm'))),
        field('Coverage report', 'clover.xml', inp('build/coverage/clover.xml', 'mono')), field('Coverage limit', '', inp('50%')),
        btn(t('b.addenv'), 'sm', 'plus'), envs, edit, btn('Remove shop-api from Mortise', 'danger'))
    return set_win(0, 'Apps', '<div class="box flush">%s</div>%s' % (apps, detail), '--shell-h:640px')
def cred_panel(i):
    where = '<div class="hint2" style="display:flex;gap:6px;align-items:center">%s%s</div>' % (ic('lock', 12), t('cr.where'))
    sec = lambda: inp('<span class="mono">%s</span>' % BUL)
    if i == 0: body = '<div class="hint2">Mortise sends no credentials. Right for a local app whose actuator is open.</div>'
    elif i == 1: body = field(t('cr.token'), 'sent as Authorization: Bearer', '<div class="inp is-focus"><span class="mono">%s%s</span></div>' % (BUL, CARET)) + where
    elif i == 2: body = '<div style="display:grid;grid-template-columns:1fr 1fr;gap:8px">%s%s</div>' % (field(t('cr.user'), '', inp('mortise', 'mono')), field(t('cr.pass'), '', sec())) + where
    else: body = '<div style="display:grid;grid-template-columns:1fr 1fr;gap:8px">%s%s</div>' % (field(t('cr.hname'), '', inp('X-Actuator-Key', 'mono')), field(t('cr.hvalue'), '', sec())) + where
    return panel(field('Credentials', '', seg(CREDS(), i)) + body, 'surface', '14px 16px')
def set_php():
    apps = table('minmax(0,1fr) 100px minmax(0,1.4fr) 270px', ['App', 'Runs with', 'Command', 'Last test'], [
        ['<b style="font-weight:600">shop-api</b>', 'Herd', '<span class="mono">herd php artisan</span>', S('good', 'Laravel Framework 13.2.0')],
        ['<b style="font-weight:600">crm</b>', 'Sail', '<span class="mono">./vendor/bin/sail artisan</span>', S('good', 'Laravel Framework 13.0.4')],
        ['<b style="font-weight:600">landing-pages</b>', 'Docker', '<span class="mono">docker compose exec app php artisan</span>', S('bad', 'No answer: service "app" is not running')]], 0)
    from p_start import php_field
    detail = ('<div class="frm"><div class="sub">shop-api</div>%s%s%s</div>') % (
        php_field(), field('Working folder', 'where the command starts', inp('~/code/shop-api', 'mono')),
        field('Stop a command after', 'so a command that waits for input cannot hang', inp('10 minutes', '', '', 'max-width:160px')))
    ex = box('What each choice runs', 'the part before the command name', kv('Herd', '<span class="mono">herd php artisan</span>') + kv('Sail', '<span class="mono">./vendor/bin/sail artisan</span>')
             + kv('Docker', '<span class="mono">docker compose exec &lt;service&gt; php artisan</span>') + kv('Custom', '<span class="mono dim">what you type, for example /usr/bin/php artisan</span>'))
    return set_win(1, 'PHP', '<div class="box flush">%s</div><div class="cols">%s<div>%s</div></div>' % (apps, detail, ex))
def set_general():
    body = ('<div class="frm">%s%s%s%s%s%s</div>') % (
        field('Theme', 'System follows the operating system and changes with it', seg(['System', 'Light', 'Dark'], 0)),
        field('Text size', 'every size in the interface scales with it', seg(['100%', '110%', '125%', '150%'], 0)),
        field('Motion', 'System follows the reduced motion setting', seg(['System', 'Reduced', 'Full'], 0)),
        field('Module index rows', 'Automatic is two lines up to 20 modules, one line above', seg(['Automatic', 'Two lines', 'One line'], 0)),
        field('Open files with', '', '<div style="display:flex;flex-direction:column;gap:6px;align-items:flex-start">%s%s</div>' % (seg(['VS Code', 'PhpStorm', 'Zed', 'Custom'], 0), inp('code --goto {file}:{line}', 'mono', '', 'align-self:stretch'))),
        field('Window controls on Linux', 'where the close, minimise and maximise buttons sit', seg(['Right', 'Left'], 0)))
    return set_win(2, 'General', body)
KEYS = [('Open the command palette', 'Ctrl K', 'Anywhere'), ('Move through the modules', 'J  K', 'Module index'), ('Filter modules', '/', 'Main window'),
        ('Summary, Boundary, Generate, Actions, Metrics', '1  2  3  4  5', 'Main window'), ('Run the generator or action', 'Ctrl Enter', 'Generate, Actions'),
        ('Show or hide the runner', 'Ctrl `', 'Anywhere'), ('Collapse or open the module index', 'Ctrl B', 'Anywhere'), ('Open the findings list', 'F', 'Main window'),
        ('Add app', 'Ctrl O', 'Anywhere'), ('Settings', 'Ctrl ,', 'Anywhere')]
def set_keys():
    top = '<div style="display:flex;gap:10px;align-items:center"><div class="filter" style="margin:0;width:260px">%s<span>Find a shortcut</span>%s</div><span class="dim" style="margin-left:auto;font-size:11.5px">On macOS Ctrl is Cmd. Shortcuts are fixed in this version.</span></div>' % (ic('search'), kbd('/'))
    tb = table('minmax(0,1.6fr) 160px 160px', ['Does', 'Keys', 'Where'], [[a, ' '.join(kbd(k) for k in b.split('  ')), '<span class="dim">%s</span>' % c] for a, b, c in KEYS])
    return set_win(3, 'Keyboard', top + '<div class="box flush">%s</div><div class="dim" style="font-size:11.5px">10 of 38 shown. The full table is in section 7.</div>' % tb)

# ------------------------------------------------------------ window behaviour
def strip_idx(sel='CHK'):
    def r(c, mark, on=False):
        return '<div class="row c1%s" style="grid-template-columns:34px auto;column-gap:6px;padding-right:8px;min-height:30px">%s%s</div>' % (' on' if on else '', stamp(c), mark)
    rows = r('ALL', S('bad')) + ''.join(r(c, S(BYCODE[c][9]), c == sel) for c in PROBLEMS_FIRST)
    return ('<div class="idx"><div class="idx-head" style="padding:12px 0 10px;display:flex;justify-content:center"><span class="btn icon sm" title="Filter modules">%s</span></div><div class="mods">%s</div>'
            '<div class="idx-foot" style="justify-content:center;padding:8px 0"><span class="kbd" title="Open the index">Ctrl B</span></div></div>') % (ic('search'), rows)
def win_min(strip=True):
    body = ALERT_CHK + ('<div class="cols">%s<div class="stack">%s%s</div></div>' % (FILES, DEPS, modstate()) if strip else FILES + DEPS)
    idx = strip_idx() if strip else index('CHK', long=False)
    w = work('Checkout', 0, body, MOD_ACTS, '', 'CHK', None if not strip else 'Modules/Checkout')
    inner = tbar() + shell(rail(), idx, w, 'height:565px;min-height:0;overflow:hidden')
    inner = inner.replace('class="tb-find"', 'class="tb-find" style="width:200px"')
    inner = inner.replace('<span>Search or run a command</span>', '<span>Search or run</span>')
    return '<div class="mwrap"><div class="m fa sw" data-theme="light" style="--idx-w:%s"><div class="win" style="width:960px;min-width:960px;height:600px">%s</div></div></div>' % ('76px' if strip else '220px', inner)

def build():
    CUR['page'] = 'screens-global.html'
    b = '<p>Section 5, part 4 of 4: what is reachable from everywhere. The command palette, Settings, and how the window behaves at its smallest.</p>'
    b += '<h2>5.12 Command palette</h2>'
    b += screen('S12.1', 'Command palette: grouped results', 'Ctrl K from anywhere. The palette is 620 px wide, 58 px from the top of the window, over a scrim. The query "check" finds a module, its actions, a generator and two findings.', palette_full(),
                '1 for modules, apps, findings and navigation. Actions and generators appear from phase 2.', 'Everything the app already holds in memory. Nothing is run to fill the palette.', (),
                ['<b>Groups, in this order.</b> Modules of the current app, actions for the best matching module, generators, findings, other apps, Mortise commands (Settings, Add app, theme). A group with no match is not shown. At most 4 rows per group; the list scrolls after 12 rows.',
                 '<b>Matching.</b> Letters in order, not necessarily adjacent, against names, codes and artisan names. "mm bil" finds "module:make-model" in Billing.',
                 '<b>Keys.</b> Up and Down move, Enter runs, Esc closes. Tab moves the scope to the next app on the rail and the footer shows which. An action with options opens its form; an action without options that is destructive still asks first.',
                 '<b>Current row.</b> Well fill, a line above and below, and the peg. No colour.'])
    b += '<h3 id="S12.2"><span class="sid">S12.2</span>Command palette: empty query and no results</h3><p class="cap">The palette alone.</p>'
    SCREENS['S12.2'] = (CUR['page'], 'Command palette: empty query and no results')
    b += '<div class="pair"><div>%s<p class="cellcap">a. Empty query: recent items, places, apps. Shortcuts are shown where one exists, so the palette also teaches them.</p></div><div>%s<p class="cellcap">b. No results. The sentence names three kinds of thing to try.</p></div></div>' % (pal_stage(pal_empty()), pal_stage(pal_none()))
    b += '<div class="notes"><div><b>Phase.</b> 1. The two "Recent" rows that run a command appear from phase 2.</div><div><b>Assumes.</b> Nothing the packages lack today.</div></div>'
    b += '<h2>5.13 Settings</h2>'
    b += ('<p>Settings open from the rail, from Ctrl , and from the palette. They take the place of the index and the workspace: the index column lists the four sections with the same joint as a module row, and Esc returns to the app. '
          'There is no Save button. A change applies when the field loses focus and can be undone with Ctrl Z while Settings are open.</p>')
    b += screen('S8.1', 'Settings: Apps', 'The apps, then the selected app with its environments. The staging environment is being edited; its token was entered earlier.', set_apps(), '1',
                'Mortise\'s own config file. Secrets are in the operating system\'s keychain, not in that file.', (),
                ['<b>Credentials.</b> After entry a secret is never drawn again, not even masked by length. The field shows "%s" with Replace and Remove. Replace empties the field for a new value.' % t('m.cred'),
                 '<b>Environments.</b> One or more per app, each with a name, a base URL, the actuator path and credentials. The runtime box, the index header and Metrics all use the environment chosen in the runtime box.',
                 '<b>Remove.</b> "%s" %s' % (t('c.rm.h'), t('c.rm.p'))])
    b += '<h3 id="S8.5"><span class="sid">S8.5</span>Environment form: the four kinds of credentials</h3><p class="cap">The credentials part of the environment form, as it is being filled in. The same form is used in the add-app flow (%s).</p>' % ref('S1.3')
    SCREENS['S8.5'] = (CUR['page'], 'Environment form: the four kinds of credentials')
    b += '<div class="pair">%s</div>' % ''.join('<div>%s<p class="cellcap">%s</p></div>' % (cred_panel(i), c) for i, c in enumerate([
        'a. None.', 'b. Bearer token. While it is typed the token is drawn as dots.', 'c. HTTP basic. The user name is not a secret and stays readable; the password is.', 'd. Custom header. The name stays readable; the value is a secret.']))
    b += ('<div class="notes"><div><b>Phase.</b> 1.</div>'
          '<div><b>Why these four.</b> The owner left the choice to this spec. The actuator\'s <span class="mono">viewActuator</span> gate needs an authenticated user, which in Laravel is usually a token guard, so a bearer token is the common case. HTTP basic and a custom header cover a proxy or gateway in front of the app. None is right for a local app.</div>'
          '<div><b>Where secrets go.</b> The credential store of the operating system: Keychain, Credential Manager, Secret Service. Never <span class="mono">mortise.json</span>, which is in the repository, and never Mortise\'s own config file. The config file holds only the kind, the user name and the header name.</div>'
          '<div><b>Not shown again.</b> While typing, dots; there is no reveal button. After the field loses focus it reads "%s" with Replace and Remove, as in %s. To check a secret, press Test.</div>'
          '<div><b>If no credential store is running</b> (possible on Linux), Mortise says so and keeps the secret only until it closes.</div><div><b>Assumes.</b> Nothing the packages lack today.</div></div>') % (t('m.cred'), ref('S8.1'))
    b += screen('S8.2', 'Settings: PHP', 'How PHP runs, per app. The third app shows a failed test.', set_php(), '2',
                'Test runs <span class="mono">artisan --version</span> through the chosen command and shows the first line of the answer.', (),
                ['<b>Failure.</b> %s' % t('x.nophp'), '<b>Docker.</b> The service name is part of the command and is edited there.'])
    b += screen('S8.3', 'Settings: General', 'Theme, text size, motion, index rows, editor, and the side of the window controls on Linux.', set_general(), '1', None, (),
                ['<b>Theme.</b> System is the default. A change redraws in the same frame. Both themes are complete; neither is derived from the other at run time.',
                 '<b>Editor.</b> The command receives the file and the line. If it cannot start: %s' % t('x.editor')])
    b += screen('S8.4', 'Settings: Keyboard', 'A searchable list of the shortcuts. Read only in this version.', set_keys(), '1', None, ())
    b += '<h2>5.14 Window behaviour</h2>'
    b += '<h3 id="S13.1"><span class="sid">S13.1</span>The minimum window, 960 by 600</h3><p class="cap">Drawn at its real size. First with the index collapsed to its strip, which is what Mortise does by itself at this width. Then with the index opened by hand, which stacks the workspace into one column.</p>'
    SCREENS['S13.1'] = (CUR['page'], 'The minimum window, 960 by 600')
    b += win_min(True) + '<p class="cellcap">a. 960 px wide, index strip 76 px, workspace 832 px. Stamps and state marks stay; J, K and the joint work as before.</p>' + win_min(False) + '<p class="cellcap">b. 960 px wide, index opened with Ctrl B at 220 px, workspace 688 px. Boxes stack and the workspace scrolls. The module path in the header is dropped.</p>'
    b += spec_table(['Window width', 'App rail', 'Module index', 'Workspace', 'Title bar'], [
        ['1076 px and wider', '52 px, fixed', 'The width the user set, 264 px by default, 220 to 320 px by dragging its right edge', 'The rest, 760 px or more. Two columns.', 'Search entry 320 px'],
        ['1032 to 1075 px', '52 px', 'Gives up width first: shrinks toward 220 px so the workspace keeps 760 px', '760 px. Two columns.', 'Under 1100 px the search entry is 200 px and reads "Search or run"'],
        ['960 to 1031 px', '52 px', 'Collapses to the 76 px strip of stamps and marks. Ctrl B opens it at 220 px over nothing: the workspace gives way.', '832 to 903 px with the strip, two columns. 688 to 759 px with the index open, one column.', 'Search entry 200 px'],
        ['Under 960 px', 'The window does not go narrower.', '', '', ''],
    ])
    b += ('<div class="notes"><div><b>Phase.</b> 1.</div><div><b>Minimum size.</b> 960 by 600 px, at every text size. With text at 150% the same rules apply to scaled widths, so the index collapses earlier.</div>'
          '<div><b>Height.</b> Title bar 34 px, workspace header 76 px. The runner drawer takes at most half of what is left and at least 120 px; under 700 px of window height it opens at 160 px instead of 200.</div>'
          '<div><b>One rule.</b> The workspace is what the user came for, so the index gives up width before the workspace does. The rail never changes.</div>'
          '<div><b>Ctrl B.</b> At any width it switches the index between its strip and its full width, and Mortise then stops deciding for the user until the window is resized.</div>'
          '<div><b>Assumes.</b> Nothing the packages lack today.</div></div>')
    return page('screens-global.html', 'Screens: global', 'Mortise design specification, section 5, part 4. Command palette, Settings, window behaviour.', b)
