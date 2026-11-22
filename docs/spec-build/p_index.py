from lib import *
from data import *
from p_read import summary_all_cmp
r = ref

def build():
    CUR['page'] = 'index.html'
    b = ('<p><strong>Mortise</strong> is a desktop companion for developers who build Laravel apps with <code>nwidart/laravel-modules</code>. It opens the folder of an app, lists its modules, shows what is in each one, '
         'which module uses which and where a boundary is broken, what the running app reports, and later creates modules and files and runs the package\'s commands with their output on screen. '
         'It is a native app written in Rust with egui, open source, for macOS, Linux and Windows, in a light and a dark theme.</p>'
         '<p>This document closes phase 0, design and information architecture. It is what the people who build the app work from. No app code exists yet. All data in the drawings is invented: an app named shop-api with seven modules.</p>')
    b += both(summary_all_cmp) + '<p class="cap">The home screen, light and dark: shop-api, "All modules" selected, Summary tab. Described in %s.</p>' % r('S4.1')
    b += '<h2>1.1 Decisions already made by the owner</h2><p>These are fixed. The spec does not reopen them.</p>'
    b += spec_table(['Subject', 'Decision'], [
        ['Name', 'Mortise.'],
        ['Structure', 'A three column, keyboard first workbench: a narrow app rail, a module index that is always visible, a workspace with five tabs (Summary, Boundary, Generate, Actions, Metrics), a command palette on Cmd or Ctrl K, a runner drawer at the bottom for command output.'],
        ['Identity', 'Timber joinery. No accent hue: colour only for state. The selected row is keyed through the joint line into the workspace. Three letter module code stamps. Asap and Inconsolata.'],
        ['Background hue', 'Slate, a blue grey, replaces the brown neutrals. State colours and the oak and walnut app icon are unchanged.'],
        ['Title bar', 'The slim title bar with the mark and the entry to the command palette stays.'],
        ['Themes and platforms', 'Light and dark, both first class. macOS, Linux and Windows.'],
        ['Technology', 'Rust with egui: immediate mode, drawn on the GPU, everything custom. Flat fills, strokes, rounded rectangles, simple shadows, vector icons and bundled fonts. No backdrop blur, no native controls.'],
        ['Distribution', 'Open source desktop app. A paid cloud service comes much later and is not part of this spec.'],
        ['Phases', '1 is read only: apps, modules, statistics, boundary graph, findings. It needs no PHP and no package change. 2 writes: new module, generators, actions. It needs PHP and package additions. 3 is live metrics.'],
        ['Outlines on the frame colour <span class="dim">(answer to Q1)</span>', 'Accepted. Controls on <code>frame</code> are outlined in <code>ink-2</code>, because <code>line-2</code> is under 3 to 1 there in the light theme. It is a token rule.'],
        ['Boundary rules file <span class="dim">(Q2)</span>', 'Rules are kept in <code>mortise.json</code> in the app folder, with that name.'],
        ['A module\'s public surface <span class="dim">(Q3)</span>', 'The package will define it. Mortise offers a scan, on demand, that proposes the public surface of one module, including what it exposes over HTTP and on the command line. Where the accepted surface is stored is the package\'s decision and is not made yet.'],
        ['Destructive commands <span class="dim">(Q6)</span>', 'Mortise offers all four: <code>module:migrate-reset</code>, <code>module:migrate-refresh</code>, <code>module:migrate-fresh</code> and <code>module:delete</code>, with the strongest confirmation in the app.'],
        ['Tests <span class="dim">(Q7)</span>', 'Per module. The package will provide a command that runs the tests of one module.'],
        ['Actuator credentials <span class="dim">(Q12)</span>', 'Left to this spec: none, bearer token, HTTP basic, custom header. Secrets go to the credential store of the operating system, never to <code>mortise.json</code> or a config file, and are not shown again.'],
        ['A circular link <span class="dim">(Q4)</span>', 'Is counted once, on the module with fewer files in the circle; the other module shows the link in its Boundary tab. %s' % r('S5.2')],
        ['A waiting migration <span class="dim">(Q5)</span>', 'Is work to do, not a finding. It is listed under "To do" and changes no state mark. %s' % r('S6.4')],
        ['Shortcuts <span class="dim">(Q8)</span>', 'Fixed in the first release, not changeable in Settings. <code>/</code> and <code>`</code> have second bindings, Ctrl F and Ctrl J, for layouts where they are hard to type.'],
        ['Runner toggle on macOS <span class="dim">(Q9)</span>', 'Ctrl ` on every platform, because Cmd ` belongs to macOS for cycling windows.'],
        ['Tabs of a later phase <span class="dim">(Q10)</span>', 'The five tabs are always shown and a tab that is not built explains itself. Buttons, footer actions and palette entries of a later phase are hidden. %s' % r('S7.4')],
        ['Runtime status on the rail <span class="dim">(Q11)</span>', 'A runtime Down raises no pip on a rail tile, for any environment. The pip is for findings of the kind problem only. %s' % r('S2.1')],
        ['Module groups <span class="dim">(Q13)</span>', 'Kept by Mortise per app, not in <code>module.json</code> or any file of the app. %s' % r('S3.1')],
        ['Features outside this spec <span class="dim">(Q14)</span>', 'Recipes, the stub editor, the explorer, git owners and the upgrade helper are not part of phases 1 to 3 as specified here. "Save as recipe" stays out of the Generate form.'],
        ['Reading <code>.env</code> <span class="dim">(Q15)</span>', 'Mortise reads two lines, <code>APP_URL</code> and <code>APP_ENV</code>, and no others. %s' % r('S1.3')],
        ['Metrics without limits <span class="dim">(Q16)</span>', 'The Metrics tab shows numbers without limits or colour. The desktop app has no user-set limits; alerts belong to the later cloud plan. %s' % r('S7.3')],
        ['Actuator', 'Apps also have <code>nwidart/laravel-actuator</code> 1.0.0. It is a data source from phase 1 on, optional. <code>nwidart/laravel-modules</code> will ship an actuator integration with module aware health indicators, and the per module metrics endpoint will live in the actuator. Both need package work first.'],
    ])
    b += '<h2>1.2 Decisions made in this spec</h2><p>Where the brief left a choice. Each is argued where it is drawn.</p>'
    b += spec_table(['Subject', 'Decision', 'Where'], [
        ['Boundary for an app with many modules', 'Above 12 modules the graph of the whole app is replaced by a sortable table of links, problems first. One module keeps its own graph, with at most 8 neighbours drawn a side.', r('S5.4')],
        ['Destructive actions', 'Roll back, replacing an existing file or module with <code>--force</code>, and stopping a running migration, rollback or seed always ask first, with focus on Cancel. Turn off asks only when other modules use the module. Migrate and seed ask only in production.', r('S10.3')],
        ['The four commands that lose the most', 'Reset, refresh, drop all tables, delete module. Kept in a closed group "Destructive" at the end of the action list, with no header button, no shortcut and no direct palette run. Each dialog names what is lost. Dropping all tables asks for the app name to be typed and deleting a module for the module name. Dropping all tables is offered only for the whole app. In production Mortise does not run any of the four; the command can only be copied.', '%s %s' % (r('S10.4'), r('S10.5'))],
        ['Public surface scan', 'A second view of the Boundary tab of one module, beside its links. Started by hand, one module at a time. It proposes contracts, events, named routes and console commands, and lists without proposing any other class that another module already uses. The developer checks, unchecks and adds entries, then accepts. A later scan lists only the differences. Accepting waits for the package.', '%s %s %s %s %s' % (r('S5.5'), r('S5.6'), r('S5.7'), r('S5.8'), r('S5.9'))],
        ['Credentials in the environment form', 'One control with four kinds; the fields under it change with the kind. User name and header name are readable, secrets are dots while typed and never drawn afterwards. No reveal button.', '%s %s' % (r('S8.5'), r('S1.3'))],
        ['Minimum window', '960 by 600 px. The rail never changes; the index gives up width before the workspace does and collapses to a 76 px strip of stamps and marks.', r('S13.1')],
        ['Two things called health', 'What Mortise computes from files is called Findings, with the words problem, warning and good and solid marks. What the actuator reports is called Runtime, with Up, Down, Out of service and Unknown and stroke marks. "Health" is not used in the interface.', 'Architecture 2.4, %s' % r('S6.5')],
        ['Where actuator data goes', 'Runtime status, its components and the info lines in one box on the app Summary, and as a line in the index header. Routes by module and scheduled tasks below it; a module\'s routes and tasks on its own Summary. Per module runtime status in the index row and on the module Summary. No new surface. Mortise never requests <code>env</code>.', '%s %s %s' % (r('S4.2'), r('S4.3'), r('S3.1'))],
        ['Metrics connection', 'Through the app\'s actuator environment: same URL, credentials and environment switch. States: not connected, endpoint not exposed, connected.', '%s %s %s' % (r('S7.1'), r('S7.2'), r('S7.3'))],
        ['The list called Health in the brief', 'It is the Findings list, a second view of the Summary tab of "All modules", not a sixth tab.', r('S6.4')],
        ['Default order of the index', 'By name, so a module keeps its place. "Problems first" is one key away and is kept per app.', r('S3.1')],
        ['Settings', 'Take the place of index and workspace, with four sections: Apps, PHP, General, Keyboard. The editor command and text size live under General.', r('S8.1')],
        ['Errors from PHP and the database', 'Are quoted, never rewritten. Mortise adds one sentence of its own: what stopped, and what to do next.', r('S11.1')],
    ])
    b += '<h2>1.3 How to read this</h2>'
    b += ('<ul><li><strong>Pages follow the twelve sections of the brief.</strong> The list is below and at the top of every page.</li>'
          '<li><strong>Every screen and state has an id</strong> such as %s, shown before its title. Flows, tables and notes refer to screens by id, and each id is a link.</li>'
          '<li><strong>Drawings are real HTML and CSS</strong> using the Mortise stylesheet with the Slate values. They are drawings of an egui app, not a web app: nothing in them is interactive.</li>'
          '<li><strong>Themes.</strong> Each page with drawings has a switch at the top for its switchable drawings; the state is kept in memory only. Where the two themes matter for a decision, both are drawn side by side and need no JavaScript. Adding <code>#dark</code> to a page address opens it in dark.</li>'
          '<li><strong>Under each drawing</strong>: the phase it belongs to, where its data comes from, and what it assumes that the packages do not offer today, with a code such as P2 that links to the list in section 12.</li>'
          '<li><strong>Shortcuts in drawings</strong> are written for Windows and Linux. On macOS Ctrl is Cmd, with the one exception named in section 7.</li>'
          '<li><strong>Sizes</strong> are logical pixels at 100%% text size.</li></ul>') % r('S4.2')
    b += '<h2>1.4 Pages</h2>'
    b += spec_table(['Page', 'Sections of the brief', 'Holds'], [
        ['<a href="index.html">Overview</a>', '1', 'This page.'],
        ['<a href="architecture.html">Information architecture</a>', '2', 'Navigation model, surface inventory, data sources, findings and runtime, what is on screen in each phase.'],
        ['<a href="tokens.html">Tokens</a>', '3', 'Colour, contrast of every pairing, type, spacing, size, radius, stroke, elevation, motion, and one JSON block.'],
        ['<a href="components.html">Components</a>', '4', '19 components, each in every state, with sizes.'],
        ['<a href="screens-start.html">Screens: start</a>', '5', 'First run, add app and its failures, app rail, module index.'],
        ['<a href="screens-read.html">Screens: read</a>', '5', 'Summary, Boundary with the public surface scan, Findings, runtime states, Metrics, a tab from a later phase.'],
        ['<a href="screens-write.html">Screens: write</a>', '5', 'Generate, Actions, the destructive actions and their confirmations, runner drawer.'],
        ['<a href="screens-global.html">Screens: global</a>', '5', 'Command palette, Settings, window behaviour.'],
        ['<a href="flows.html">Flows and keyboard</a>', '6, 7', 'Six flows for mouse and keyboard, the shortcut table, the focus order.'],
        ['<a href="reference.html">Text, platform, fonts</a>', '8, 9, 10, 11', 'Title bar on each platform, accessibility, every string, font licences.'],
        ['<a href="open.html">Open questions</a>', '12', 'No questions are left open. What the screens assume from the packages, what was not checked.'],
    ])
    b += '<h2>1.5 All screens and states</h2><ul class="toc">%s</ul>' % ''.join('<li>%s %s</li>' % (r(k), v[1]) for k, v in sorted(SCREENS.items(), key=lambda kv: [int(n) for n in kv[0][1:].split('.')]))
    b += ('<h2>1.6 Not in this spec</h2><ul><li>The cloud service, teams, alerts and hosted history.</li><li>Recipes, the stub editor, the explorer of routes and events, git owners and the upgrade helper from the first feature list. "Save as recipe" was removed from the Generate form for that reason.</li>'
          '<li>The command line twin for CI. It appears once, as a rule that is not available yet.</li><li>The website, the installer and the update mechanism.</li><li>Any code.</li></ul>')
    return page('index.html', 'Mortise: design specification', 'Phase 0, design and information architecture. 22 November 2026. For the people who build the app.', b, switch=False)
