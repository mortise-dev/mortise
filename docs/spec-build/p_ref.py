from lib import *
from data import *

def tb_only(platform, right='shop-api on Herd', pre=''):
    return '<div class="mwrap"><div class="m fa sw" data-theme="light"><div class="win sm" style="min-width:900px;border-radius:%s">%s%s<div style="height:26px;background:var(--surface);border-top:0"></div></div></div></div>' % (
        '8px 8px 0 0', pre, tbar(right, True, platform))

def build():
    CUR['page'] = 'reference.html'
    b = '<p>Sections 8 to 11 of the specification: what the title bar holds on each platform, accessibility, every string of the interface in one table, and the licences of the two bundled fonts.</p>'
    b += '<h2>8. Platform notes</h2><p>The slim title bar is 34 px high and belongs to the window frame colour. It always holds the same three things: the mark with the name on the left, the entry to the command palette in the middle, the current app and how its PHP runs on the right. Only the window controls differ.</p>'
    b += '<h3 id="T1"><span class="sid">T1</span>macOS</h3><p class="cap">The three window buttons are drawn by macOS at the left, at their normal place. Mortise leaves 78 px free for them and starts the mark after that.</p>' + tb_only('mac')
    b += '<h3 id="T2"><span class="sid">T2</span>Windows</h3><p class="cap">Minimise, maximise and close at the right, 44 px wide each, full height of the bar, drawn by Mortise. All three take the well fill on hover; close is not red, because colour is kept for state.</p>' + tb_only('win')
    b += '<h3 id="T3"><span class="sid">T3</span>Linux</h3><p class="cap">The same three controls drawn by Mortise, on the right by default. A setting moves them to the left for desktops that put them there.</p>' + tb_only('linux')
    native = '<div style="height:28px;background:#3a3a3a;color:#ddd;display:flex;align-items:center;justify-content:space-between;padding:0 12px;font:13px system-ui,sans-serif"><span>Mortise</span><span style="letter-spacing:6px">_ &#9633; x</span></div>'
    b += '<h3 id="T4"><span class="sid">T4</span>Fallback: the platform\'s own title bar</h3><p class="cap">Where a custom title bar is not possible, the window keeps the platform\'s bar (grey here, not designed by Mortise) and the Mortise bar sits directly under it, unchanged except that it holds no window controls.</p>' + tb_only('plain', 'shop-api on Herd', native)
    b += spec_table(['', 'macOS', 'Windows', 'Linux'], [
        ['Window controls', 'Left. Drawn by the system (close, minimise, zoom).', 'Right. Drawn by Mortise: minimise, maximise or restore, close.', 'Right by default, left by setting. Drawn by Mortise.'],
        ['What Mortise draws', 'Everything except the three buttons. The bar is content that extends under a transparent system title bar.', 'The whole bar. The window has no system decorations.', 'The whole bar. The window has no system decorations.'],
        ['Dragging and double click', 'Any empty part of the bar drags the window. Double click zooms.', 'Any empty part drags. Double click maximises. The window edges resize.', 'Same as Windows.'],
        ['Window corner radius', '8 px, by the system', 'By the system (rounded on Windows 11, square on Windows 10)', '8 px where the compositor allows a transparent corner, otherwise square'],
        ['Menu bar', 'The system menu bar holds Mortise, File, View, Window and Help with the same commands as the palette.', 'None. The palette is the menu.', 'None. The palette is the menu.'],
        ['Modifier', 'Cmd. Option for the app shortcuts.', 'Ctrl. Alt for the app shortcuts.', 'Ctrl. Alt for the app shortcuts.'],
        ['Settings shortcut', 'Cmd ,', 'Ctrl ,', 'Ctrl ,'],
        ['Secrets', 'Keychain', 'Credential Manager', 'Secret Service; if none is running, Mortise says so and does not store the secret'],
        ['Folder dialog', 'The system\'s', 'The system\'s', 'The desktop\'s portal dialog'],
        ['When the fallback applies', 'Not expected.', 'If window snapping from the maximise button cannot be kept with a custom frame, the owner chooses: custom bar without that menu, or the fallback.', 'On a tiling window manager, or when the compositor insists on its own decorations. Also offered as a setting.'],
    ])
    b += ('<div class="notes"><div><b>Not checked against egui.</b> This section says what the design needs. Whether eframe can keep the macOS buttons over custom content, and how a frameless window behaves with Windows 11 snapping and with Wayland compositors, was not tested for this spec and is listed under "not checked".</div>'
          '<div><b>The app icon</b> is the oak block with the walnut and ash joint from the identity, unchanged, on every platform. A 16 px drawing exists for small sizes.</div></div>')

    b += '<h2>9. Accessibility</h2>'
    b += spec_table(['Subject', 'Rule', 'How it shows in the spec'], [
        ['Contrast', 'Text 4.5 to 1, marks and outlines 3 to 1, in both themes. Disabled text is held to 4.5 to 1 as well, although the standard does not ask for it.', 'Every pairing used is computed in Tokens, section 3.2, and the drawings were checked in a browser.'],
        ['State without colour', 'Every state has a shape and a word. Findings: check, wedge, block with a cross, empty frame. Runtime: arrow up, arrow down, two bars, three dashes. A problem link in the graph is thicker and carries the problem mark on the line.', 'Components C9. Where a mark stands alone, the tooltip carries the word.'],
        ['Selection without colour', 'The selected row is joined to the workspace and has a peg; the selected tab is joined to its panel; a selected list row has a line above and below and the peg. None of them is a hue.', 'Components C6, C7, C8.'],
        ['Focus', 'A 2 px ink ring, 1 px outside the control, on every focusable thing, in both themes. It is drawn over hover and selection. Focus is always visible while the keyboard is in use, and never trapped except in the palette and a dialog.', 'Components, every row. Focus order K1.'],
        ['Keyboard', 'Every action has a key path. No action needs a hover or a drag.', 'Section 7.'],
        ['Reduced motion', 'Follows the system setting, with an override in Settings. The drawer appears at its height, the toast appears and disappears, the new file highlight is shown for 600 ms and removed. Nothing else moves in the first place.', 'Tokens, section 3.6.'],
        ['Text size', '100, 110, 125 and 150% in Settings, on top of the display scale of the system. Type and sizes scale together, strokes stay 1 px. No text is cut: labels wrap, rows grow, and the index collapses earlier to keep the workspace.', 'Settings S8.3, window rules S13.1.'],
        ['Pointer target', 'At least 24 by 24 px for every control. Where a control is drawn smaller (22 px buttons, the 14 px checkbox, a 16 px keycap that acts as a button) the target extends beyond the drawing; a checkbox row is one target.', 'Tokens, size.hit-min.'],
        ['Screen readers', 'Every control exposes a role, a name and its state to the accessibility interface of the platform (egui does this through AccessKit, as far as is known without a check). Marks expose their word. The runner output is a live region that announces the end of a command, not every line.', 'For the builders. Not tested for this spec.'],
        ['Language', 'English only in this version. Sentences are complete and short so that they translate; no meaning is carried by an abbreviation.', 'Section 10.'],
        ['Time', 'A done toast leaves after 4 s and repeats nothing that is not also on screen. A failed toast stays. Nothing else times out.', 'Components C13.'],
    ])

    b += '<h2>10. Interface text</h2>'
    b += ('<p><strong>Voice.</strong> Plain and concrete, like a note left on a workpiece: what happened, to what, and what was left untouched. Sentence case. No exclamation marks, no apology, no "please". '
          'An action keeps one name from the button to the confirmation to the message: "Turn off", "Turn off Checkout?", "Turned off Checkout." An error says what happened, then what to do next. '
          'Text that comes from PHP, the database or the package is quoted as it is and never rewritten.</p>'
          '<p><strong>Two words that must not meet.</strong> <em>Findings</em> (problem, warning, good) are what Mortise works out from the files. <em>Runtime</em> (Up, Down, Out of service, Unknown) is what the running app reports through the actuator. '
          'The word "health" appears in the interface only inside a path or a config key the user must type. "On" and "Off" are a module\'s state in <code>modules_statuses.json</code>, and "Turn on" and "Turn off" are the actions; the artisan names <code>module:enable</code> and <code>module:disable</code> appear only in the command preview.</p>'
          '<p>The drawings read their strings from this table, so what is listed here is what is drawn. Names in the examples (shop-api, Checkout, counts) are placeholders for real values.</p>')
    groups = {}
    for k, (g, where, s, note) in TEXT.items(): groups.setdefault(g, []).append((where, s, note))
    for g, rows in groups.items():
        b += '<h3>%s</h3>' % g + spec_table(['Where', 'Text', 'Note'], [[w, html.escape(s), n] for w, s, n in rows])
    b += '<h3>The four runtime states and what else can be shown in their place</h3>'
    b += spec_table(['What the actuator answers', 'Shown as', 'Mark', 'Colour'], [
        ['<code>UP</code>', 'Up', 'arrow up', 'good'], ['<code>DOWN</code> (HTTP 503)', 'Down, with the failing component and its detail', 'arrow down', 'bad'],
        ['<code>OUT_OF_SERVICE</code> (HTTP 503)', 'Out of service, with the component', 'two bars', 'warn'], ['<code>UNKNOWN</code>', 'Unknown', 'three dashes', 'off'],
        ['No answer, a refused connection, a timeout', 'No answer. This is Mortise\'s word, not a status of the app.', 'three dashes', 'off'],
        ['HTTP 401 or 403', 'Not authorized', 'three dashes', 'off'], ['HTTP 500', 'No answer, with the sentence about the actuator configuration', 'three dashes', 'off']])

    b += '<h2>11. Fonts</h2>'
    b += ('<p>Both families will be bundled inside the app, so their licences were read in each font\'s own repository on 22 November 2026, not recalled. Both are free to bundle.</p>')
    b += spec_table(['Family', 'Repository', 'Licence file read', 'Licence', 'Copyright line in that file', 'Reserved font name'], [
        ['Asap', '<a href="https://github.com/Omnibus-Type/Asap" target="_blank" rel="noopener noreferrer">github.com/Omnibus-Type/Asap</a>',
         '<a href="https://github.com/Omnibus-Type/Asap/blob/master/OFL.txt" target="_blank" rel="noopener noreferrer">OFL.txt on branch master</a>', 'SIL Open Font License, version 1.1',
         'Copyright 2018 The Asap Project Authors (https://github.com/Omnibus-Type/Asap)', 'None declared in the copyright line'],
        ['Inconsolata', '<a href="https://github.com/googlefonts/Inconsolata" target="_blank" rel="noopener noreferrer">github.com/googlefonts/Inconsolata</a>',
         '<a href="https://github.com/googlefonts/Inconsolata/blob/main/OFL.txt" target="_blank" rel="noopener noreferrer">OFL.txt on branch main</a>', 'SIL Open Font License, version 1.1',
         'Copyright 2006 The Inconsolata Project Authors (https://github.com/cyrealtype/Inconsolata)', 'None declared in the copyright line'],
    ])
    b += ('<p><strong>What the licence allows and asks</strong>, from the text of the two files, which is the same from "PERMISSION &amp; CONDITIONS" on:</p><ul>'
          '<li>Condition 2: the fonts "may be bundled, redistributed and/or sold with any software, provided that each copy contains the above copyright notice and this license". They "can be included either as stand-alone text files, human-readable headers or in the appropriate machine-readable metadata fields within text or binary files as long as those fields can be easily viewed by the user".</li>'
          '<li>Condition 1: the fonts may not be sold by themselves. Bundling them in an app, free or paid, is not that.</li>'
          '<li>Conditions 3 and 4: a modified version may not use a reserved font name, and the authors\' names may not be used to promote it. Neither file declares a reserved name. Subsetting a font is a modification, so if the builders subset, they should keep the original family names only because no name is reserved; if a later release of either font declares one, a subset must be renamed.</li>'
          '<li>Condition 5: the fonts stay under this licence. The app itself can have any licence.</li></ul>'
          '<p><strong>What the app must do.</strong> Ship both <code>OFL.txt</code> files with their copyright lines, for example in a <code>licenses</code> folder next to the binary and on an "About" line in Settings. Leave the licence metadata inside the font files as it is.</p>'
          '<p><strong>Notes.</strong> The address in Inconsolata\'s copyright line, <code>github.com/cyrealtype/Inconsolata</code>, now redirects to the Google Fonts repository linked above. '
          'Which files to bundle: the static weights 400, 500, 600 and 700 of Asap and 400, 500 and 600 of Inconsolata, at normal width. Whether egui renders the variable versions was not checked, so static files are the safe choice. '
          'No replacement is needed, so none is proposed.</p>')
    return page('reference.html', 'Text, platform, fonts', 'Mortise design specification, sections 8 to 11. Platform notes, accessibility, interface text, font licences.', b)
