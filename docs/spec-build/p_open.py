from lib import *
from data import *

def build():
    CUR['page'] = 'open.html'
    b = '<p>Section 12 of the specification: the open questions, of which none are left, everything a screen assumes that the packages do not offer today, and what was not checked while writing this.</p>'
    b += ('<h2>12.1 Open questions for the owner</h2><p>None are left. The owner answered all sixteen questions of the first version on 22 November 2026; the answers are decisions on the <a href="index.html">overview</a>, section 1.1, each with its question number.</p>')
    b += ('<h2>12.2 What the screens assume that does not exist yet</h2><p>Read on 22 November 2026 from the source of <code>nwidart/laravel-modules</code> (version 13.3.0 in the mock data) and from the README, configuration and endpoint reference of <code>nwidart/laravel-actuator</code> 1.0.0. '
          'Nothing in this list is invented as an API: no endpoint path, option name or JSON field is fixed by this spec. Each line says in words what a screen needs.</p>')
    b += spec_table(['', 'Package', 'What is needed', 'What exists today', 'Screens that depend on it'],
                    [['<span class="sid" id="a-%s">%s</span>' % (k, k), '<code>%s</code>' % v[0] if '/' in v[0] else v[0], '<strong>%s</strong>' % v[1], v[2], ' '.join(ref(s) for s in sorted(set(AUSED.get(k, [])), key=lambda x: [int(n) for n in x[1:].split('.')])) or 'none drawn'] for k, v in ASSUME.items()])
    b += ('<p><strong>What needs nothing new.</strong> Phase 1 without the actuator needs no PHP and no package change: apps, modules, statistics, the Boundary graph, the rules and the findings on files. '
          'Phase 1 with the actuator uses four endpoints that exist in 1.0.0: <code>health</code> (exposed by default), and <code>info</code>, <code>mappings</code> and <code>scheduledtasks</code>, which the app must add to <code>exposure.include</code> and should protect with middleware. Mortise never requests <code>env</code>.</p>'
          '<p><strong>Assumed outside the packages.</strong> A coverage report in clover format exists in the app (otherwise coverage reads "unknown"). Git is installed (otherwise churn is absent). An editor command is set (otherwise "Open in editor" says so). '
          'A credential store is running on the computer (otherwise a secret is kept only until Mortise closes). Herd, Sail or Docker are detected from the folder; the exact commands in the drawings (<code>herd php artisan</code>, <code>./vendor/bin/sail artisan</code>, <code>docker compose exec app php artisan</code>) are examples to be confirmed by the builders.</p>')
    b += '<h2>12.3 Not checked</h2><ul>' + ''.join('<li>%s</li>' % x for x in [
        'No part of this was built in egui. Sizes, the joint, the floating scrollbar and the shadows are specified from the identity page and from reasoning about flat shapes, not from a prototype.',
        'The custom title bar on each platform (macOS buttons over custom content, Windows 11 snapping, Wayland decorations) was not tested. Section 8 says what the design needs.',
        'How Asap and Inconsolata rasterise in egui at 11.5 to 13 px on a 1x display was not looked at. The identity page already names this as a risk for Asap.',
        'The drawings were rendered in Chrome at 1280 px wide, in both themes, and looked at. They were not opened in Safari or Firefox, and not at other widths apart from the two 960 px windows.',
        'Contrast was computed from the token values and measured on the rendered drawings for text. Outlines and marks were checked from the token table only.',
        'The four destructive commands were read in source on 22 November 2026, not run. Table names and counts in their dialogs are invented.',
        'The public surface scan is a design. No parser was run on a module to see what such a scan really finds.',
        'The package commands were read in source, not run. Output lines in the runner drawings follow the strings in the source; spacing and colours of real output will differ.',
        'The actuator was read in its documentation and configuration file, not installed or called.',
        'Screen reader behaviour was not tested; the claim about egui and AccessKit is from memory.',
        'The Herd, Sail and Docker command lines were not run.',
        'The font licences were read in the repositories linked in section 11. The font files themselves and their embedded licence fields were not opened.',
        'The package may change after 22 November 2026. Command names and options were read on that day from the local checkout.']) + '</ul>'
    return page('open.html', 'Open questions', 'Mortise design specification, section 12. No open questions are left. What the screens assume from the packages, what was not checked.', b, switch=False)
