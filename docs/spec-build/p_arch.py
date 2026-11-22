from lib import *
from data import *
r = ref

NAV = ('<svg class="diagram" viewBox="0 0 1000 430" width="1000" role="img" aria-label="Navigation model: title bar, app rail, module index, workspace with five tabs, runner drawer, and the layers above them">'
       '<g fill="none" stroke="#888" stroke-width="1"><rect x="10.5" y="30.5" width="640" height="380"/><path d="M10 64.5H650M70.5 64V410M250.5 64V410M250 124.5H650M250 320.5H650"/>'
       '<path d="M330.5 100V124M410.5 100V124M490.5 100V124M570.5 100V124" stroke="#555"/>'
       '<rect x="700.5" y="30.5" width="290" height="92"/><rect x="700.5" y="140.5" width="290" height="62"/><rect x="700.5" y="220.5" width="290" height="62"/><rect x="700.5" y="300.5" width="290" height="110"/></g>'
       '<g stroke="#fff" stroke-width="1.5" fill="none"><path d="M56 300H96M236 300H282"/><path d="M90 295l7 5-7 5M276 295l7 5-7 5" /></g>'
       '<g fill="#fff" font-family="system-ui,sans-serif" font-size="13">'
       '<text x="10" y="20" fill="#a8a8a8" font-size="12">The window. Three columns that are always there, one drawer that comes and goes.</text>'
       '<text x="24" y="52">Title bar</text><text x="250" y="52" fill="#a8a8a8" font-size="12">mark, entry to the command palette, current app and PHP</text>'
       '<text x="20" y="88" font-size="12">App</text><text x="20" y="104" font-size="12">rail</text><text x="20" y="150" fill="#a8a8a8" font-size="11">picks</text><text x="20" y="164" fill="#a8a8a8" font-size="11">an app</text>'
       '<text x="20" y="372" fill="#a8a8a8" font-size="11">+ add</text><text x="20" y="390" fill="#a8a8a8" font-size="11">settings</text>'
       '<text x="86" y="88">Module index</text><text x="86" y="112" fill="#a8a8a8" font-size="12">app name, versions,</text><text x="86" y="128" fill="#a8a8a8" font-size="12">runtime status</text>'
       '<text x="86" y="156" fill="#a8a8a8" font-size="12">filter</text><text x="86" y="186" font-size="12">All modules</text><text x="86" y="206" font-size="12">one row per module</text>'
       '<text x="86" y="230" fill="#a8a8a8" font-size="11">picks what the</text><text x="86" y="244" fill="#a8a8a8" font-size="11">workspace shows</text><text x="86" y="390" fill="#a8a8a8" font-size="12">order, New module</text>'
       '<text x="266" y="88">Workspace</text><text x="350" y="88" fill="#a8a8a8" font-size="12">title, code, actions of the selection</text>'
       '<text x="262" y="116" font-size="12">Summary 1</text><text x="338" y="116" font-size="12">Boundary 2</text><text x="418" y="116" font-size="12">Generate 3</text><text x="500" y="116" font-size="12">Actions 4</text><text x="580" y="116" font-size="12">Metrics 5</text>'
       '<text x="266" y="150" fill="#a8a8a8" font-size="12">The content of one tab for the selected row:</text><text x="266" y="170" fill="#a8a8a8" font-size="12">the whole app when "All modules" is selected,</text><text x="266" y="190" fill="#a8a8a8" font-size="12">one module otherwise.</text>'
       '<text x="266" y="222" fill="#a8a8a8" font-size="12">Second views inside a tab, left with Esc:</text><text x="266" y="242" fill="#a8a8a8" font-size="12">Findings list (in Summary), files of a link (in Boundary),</text><text x="266" y="262" fill="#a8a8a8" font-size="12">generator form and result (in Generate).</text>'
       '<text x="266" y="346">Runner drawer</text><text x="266" y="370" fill="#a8a8a8" font-size="12">command, output, status, history. Opens when a command starts.</text>'
       '<text x="712" y="52">Command palette</text><text x="712" y="74" fill="#a8a8a8" font-size="12">Over everything. Reaches every app, module,</text><text x="712" y="92" fill="#a8a8a8" font-size="12">generator, action, finding and setting.</text><text x="712" y="110" fill="#a8a8a8" font-size="12">Cmd or Ctrl K</text>'
       '<text x="712" y="162">Dialog</text><text x="712" y="184" fill="#a8a8a8" font-size="12">Only to confirm. Over a scrim.</text>'
       '<text x="712" y="242">Toast, tooltip</text><text x="712" y="264" fill="#a8a8a8" font-size="12">Say what happened or name a thing.</text>'
       '<text x="712" y="322">In place of index and workspace</text><text x="712" y="346" fill="#a8a8a8" font-size="12">Add app (from + on the rail)</text><text x="712" y="366" fill="#a8a8a8" font-size="12">Settings (from the rail): Apps, PHP,</text><text x="712" y="386" fill="#a8a8a8" font-size="12">General, Keyboard. Esc returns.</text></g></svg>')
SRC = ('<svg class="diagram" viewBox="0 0 1000 250" width="1000" role="img" aria-label="Four data sources feed the core, which feeds the screens">'
       '<g fill="none" stroke="#888"><rect x="10.5" y="10.5" width="300" height="48"/><rect x="10.5" y="70.5" width="300" height="48"/><rect x="10.5" y="130.5" width="300" height="48"/><rect x="10.5" y="190.5" width="300" height="48" stroke-dasharray="4 3"/>'
       '<rect x="400.5" y="80.5" width="180" height="88"/><rect x="670.5" y="10.5" width="320" height="228"/><path d="M310 34.5H355V110H400M310 94.5H400M310 154.5H400M310 214.5H355V140H400M580 124.5H670" stroke="#fff"/></g>'
       '<g fill="#fff" font-family="system-ui,sans-serif" font-size="13"><text x="22" y="30">Files and git</text><text x="22" y="48" fill="#a8a8a8" font-size="12">no PHP, no running app. Phase 1.</text>'
       '<text x="22" y="90">Actuator, over HTTP</text><text x="22" y="108" fill="#a8a8a8" font-size="12">a running app and a URL. Phase 1, optional.</text>'
       '<text x="22" y="150">Artisan, through the PHP runner</text><text x="22" y="168" fill="#a8a8a8" font-size="12">Herd, Sail, Docker or a command. Phase 2.</text>'
       '<text x="22" y="210">Metrics, over the actuator</text><text x="22" y="228" fill="#a8a8a8" font-size="12">same URL and credentials. Phase 3. Not built yet.</text>'
       '<text x="414" y="110">Core</text><text x="414" y="130" fill="#a8a8a8" font-size="12">one model of apps,</text><text x="414" y="148" fill="#a8a8a8" font-size="12">modules, links, findings</text>'
       '<text x="684" y="34">Screens</text><text x="684" y="58" fill="#a8a8a8" font-size="12">Index, Summary, Boundary, Findings: files and git</text><text x="684" y="80" fill="#a8a8a8" font-size="12">Runtime box, routes, scheduled tasks: actuator</text>'
       '<text x="684" y="102" fill="#a8a8a8" font-size="12">Generate, Actions, runner: artisan</text><text x="684" y="124" fill="#a8a8a8" font-size="12">Metrics tab: metrics over the actuator</text>'
       '<text x="684" y="160" fill="#a8a8a8" font-size="12">A screen never mixes two sources in one figure</text><text x="684" y="178" fill="#a8a8a8" font-size="12">without saying which: "read from local",</text><text x="684" y="196" fill="#a8a8a8" font-size="12">"from clover.xml", "needs PHP".</text></g></svg>')

def build():
    CUR['page'] = 'architecture.html'
    b = '<p>Section 2 of the specification: how the app is laid out and moved through, every surface it has, where its data comes from, and what is on screen in each phase.</p>'
    b += '<h2>2.1 Navigation model</h2>' + NAV
    b += ('<p>Three choices, left to right, each narrowing the next: an <strong>app</strong> on the rail, a <strong>row</strong> in the index (the whole app or one module), a <strong>tab</strong> in the workspace. '
          'The choice of tab is kept while moving through modules, so J and K with the Boundary tab open walk through the boundaries of every module. '
          'There is no back stack and no page navigation: the three choices are always visible, and Esc only ever closes the topmost thing.</p>')
    b += '<h2>2.2 Surface inventory</h2>'
    b += spec_table(['Surface', 'What it holds', 'How you reach it', 'Phase', 'Drawn in'], [
        ['Title bar, 34 px', 'Mark and name, entry to the command palette, the current app and how its PHP runs, window controls where Mortise draws them.', 'Always visible.', '1', 'every screen, %s' % 'section 8'],
        ['App rail, 52 px', 'One tile per app with a two letter monogram and a pip when the app has a problem. Add tile. Settings.', 'Always visible. Cmd or Ctrl with Option or Alt and 1 to 9.', '1', r('S2.1')],
        ['Module index, 264 px', 'App name, versions, runtime status of the chosen environment, filter, "All modules", one row per module with code stamp, counts, finding mark and (planned) runtime mark, order, New module.', 'Always visible. Cmd or Ctrl B collapses it to a strip.', '1', r('S3.1')],
        ['Workspace: Summary', 'All modules: totals, findings, coverage, runtime box, routes, scheduled tasks, table by module. One module: files by kind, dependencies, module state, runtime, routes, tasks.', 'Tab 1. The default.', '1', '%s %s %s' % (r('S4.1'), r('S4.2'), r('S4.3'))],
        ['Summary, second view: Findings', 'Every finding of the app, grouped by kind, and the checks that passed.', 'F, the Findings stat, the palette. Esc returns.', '1', r('S6.4')],
        ['Workspace: Boundary', 'One module: its graph and the files of the selected link. All modules: the app graph or, above 12 modules, the table of links; the rules.', 'Tab 2.', '1', '%s %s %s %s' % (r('S5.1'), r('S5.2'), r('S5.3'), r('S5.4'))],
        ['Boundary, second view: Public surface', 'For one module: the scan that proposes what the module offers to other modules, HTTP clients and the command line; the proposal; the accepted surface; differences found by a later scan.', 'The control at the top of Boundary, or P.', '1 to scan, accepting needs the package', '%s %s %s %s %s' % (r('S5.5'), r('S5.6'), r('S5.7'), r('S5.8'), r('S5.9'))],
        ['Workspace: Generate', 'Generator picker, the form of one generator, command and file preview, result.', 'Tab 3 or G. "New module" opens the form for <code>module:make</code>.', '2', '%s %s %s %s %s %s' % (r('S9.1'), r('S9.2'), r('S9.3'), r('S9.4'), r('S9.5'), r('S9.6'))],
        ['Workspace: Actions', 'Turn on or off, migrate, roll back, seed, run tests, check translations, each with options, command and run button. A closed group at the end holds the destructive actions: reset, refresh, drop all tables, delete module.', 'Tab 4.', '2', '%s %s %s %s %s' % (r('S10.1'), r('S10.2'), r('S10.3'), r('S10.4'), r('S10.5'))],
        ['Workspace: Metrics', 'Requests, response time, errors, jobs and database time per module.', 'Tab 5.', '3', '%s %s %s' % (r('S7.1'), r('S7.2'), r('S7.3'))],
        ['Runner drawer', 'The running or last command, its output, status, time, and the history of runs.', 'Opens when a command starts. Ctrl `.', '2', r('S11.1')],
        ['Command palette', 'Search over apps, modules, generators, actions, findings and settings.', 'Cmd or Ctrl K, or the entry in the title bar.', '1', '%s %s' % (r('S12.1'), r('S12.2'))],
        ['Add app', 'Folder, what was found, how PHP runs, runtime connection.', 'The add tile, Cmd or Ctrl O, the palette.', '1', '%s %s %s %s' % (r('S1.1'), r('S1.2'), r('S1.3'), r('S1.4'))],
        ['Settings', 'Apps and their environments with credentials, PHP, General (theme, text size, motion, editor), Keyboard.', 'The rail, Cmd or Ctrl and comma, the palette.', '1, PHP from 2', '%s %s %s %s %s' % (r('S8.1'), r('S8.5'), r('S8.2'), r('S8.3'), r('S8.4'))],
        ['Dialog', 'A confirmation before something is lost or before others are affected.', 'Raised by an action.', '2', r('S10.3')],
        ['Toast, tooltip', 'A short message after something happened out of sight; the name of a control.', 'By themselves.', '1', 'Components'],
    ])
    b += '<p>No surface was added for the actuator. Its data sits in the Summary tab and the index header, and Metrics keeps the tab it had.</p>'
    b += '<h2>2.3 Where the data comes from</h2>' + SRC
    b += spec_table(['Source', 'Needs', 'Gives', 'Phase'], [
        ['Files and git', 'The app folder. No PHP, no dependencies installed beyond <code>composer.lock</code>.', 'Module list and on or off state, file and line counts, kinds of class, links between modules from <code>use</code> statements, coverage from <code>clover.xml</code>, commits. Findings are computed from these.', '1'],
        ['Actuator', 'A running app, a base URL, the actuator path, credentials if the app asks. No PHP runner, no package change. The app needs <code>nwidart/laravel-actuator</code>, found in <code>composer.lock</code>.', 'Runtime status and components (<code>health</code>), environment, git, PHP and memory (<code>info</code>), routes (<code>mappings</code>), scheduled tasks (<code>scheduledtasks</code>). Per module status and module data once the packages offer them.', '1, optional'],
        ['Artisan', 'A way to run PHP for the app: Herd, Sail, Docker or a command.', 'Generators, actions, migration state, translation check, tests.', '2'],
        ['Metrics over the actuator', 'The actuator connection, and a metrics endpoint that does not exist yet.', 'Figures per module for the Metrics tab.', '3'],
    ])
    b += '<h2>2.4 Findings and runtime: two things that were both called health</h2>'
    b += spec_table(['', 'Findings', 'Runtime'], [
        ['What it is', 'What Mortise works out from the files: circular use, access to another module\'s internals, an invalid <code>module.json</code>, a namespace that does not match, a stale status entry, coverage under the limit.', 'What the running app reports through the actuator: the status of the app, of its components, and (planned) of each module.'],
        ['Words', 'problem, warning, good. A module is also on or off.', 'Up, Down, Out of service, Unknown. And, for the connection itself, No answer and Not authorized.'],
        ['Marks', 'Solid silhouettes: block with a cross, wedge, check, empty frame.', 'Strokes: arrow down, two bars, arrow up, three dashes.'],
        ['Where', 'Right end of an index row, the Findings stat, the alert at the top of a Summary, the Findings list, Boundary, the pip on the rail.', 'The line under the versions in the index header, the Runtime stat, the Runtime box on a Summary, and (planned) a second mark left of the finding mark on an index row.'],
        ['Needs', 'The folder.', 'A running app and a connected environment.'],
        ['Named in the interface', '"Findings"', '"Runtime"'],
    ])
    b += '<p>The word "health" is kept out of the interface for both. It appears only where the user has to type it: the endpoint path and the config keys of the actuator.</p>'
    b += '<h2>2.5 What is on screen in each phase</h2>'
    y, n, e = 'Yes', 'No', 'Shown, explains itself'
    b += spec_table(['', 'Phase 1, read only', 'Phase 2, write', 'Phase 3, live'], [
        ['Needs', 'The app folder. Optional: a running app with the actuator.', 'PHP for the app. Package additions P1 to P8.', 'Metrics endpoint in the actuator (A2). Package work P9, P10, A1 for the per module parts.'],
        ['Add app', 'Folder and runtime connection', 'Plus "Run PHP with"', 'Same'],
        ['App rail, module index', y, 'Plus "New module" in the footer', 'Plus the runtime mark per module, when the packages offer it'],
        ['Summary', 'Files, git, coverage, findings. With an environment: runtime box, routes, scheduled tasks.', 'Plus migration state, and the Run tests and Migrate buttons in the header', 'Plus runtime per module'],
        ['Findings list', 'Rules and file checks', 'Plus translations, and the waiting migrations under "To do"', 'Same'],
        ['Boundary', 'Links, rules, and the public surface scan with its proposal', 'Same', 'Same. Accepting a public surface arrives when the package can store one (P6), in whichever phase that is.'], ['Generate tab', e, y, y], ['Actions tab', e, 'Yes, with the closed group of destructive actions', y], ['Metrics tab', e, e, y],
        ['Runner drawer', 'Hidden', y, y], ['Command palette', 'Navigation, findings, settings', 'Plus generators and actions', 'Same'],
        ['Settings', 'Apps, General, Keyboard', 'Plus PHP', 'Same'],
    ])
    b += ('<p><strong>Decision: tabs are shown, actions are hidden.</strong> The five tabs are in every build, in the same order with the same keys, because the keys 1 to 5 and the place of each tab should not move between releases. '
          'A tab whose phase is not built opens an explanation in the app\'s voice, with no fake controls (%s). '
          'Buttons, footer actions and palette entries of a later phase are left out, because a disabled button on every module is noise and a palette that lists what cannot run wastes the first result.</p>'
          '<p>The same explanation block has a permanent use. In a build that has the phase, a tab still explains itself when this app cannot use it: PHP was not found, or the app\'s version of the package is too old. '
          'In that case the controls that exist are drawn disabled, each with the reason in its tooltip.</p>') % r('S7.4')
    b += '<h2>2.6 How it scales</h2>'
    b += spec_table(['Size', 'What changes'], [
        ['1 app, 5 modules', 'The rail holds one tile and the add tile. The index shows every module without scrolling, two lines a row. The app graph is drawn whole.'],
        ['10 apps, 60 modules each', 'The rail lists ten tiles; the first nine have a shortcut and the palette reaches all. The index uses one line a row, groups, and "problems first" pins the modules that need attention. The app graph gives way to the table of links; each module keeps its own graph.'],
        ['More', 'The rail scrolls, the index scrolls with a floating bar, tables scroll. Only visible rows are laid out.'],
    ])
    return page('architecture.html', 'Information architecture', 'Mortise design specification, section 2. Navigation model, surfaces, data sources, phases.', b, switch=False)
