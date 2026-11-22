import { shot, table, k } from './lib.mjs';

export function body(d) {
  return `
<h1>Laravel Modules companion: three design directions</h1>
<p class="dim">Phase 0, design and information architecture. 22 November 2026. Directions are labelled F1, F2, F3. No app code exists yet; every screen here is a mock in HTML and CSS.</p>
<ul class="toc"><li><a href="#directions">The directions</a></li><li><a href="#shared">What they share</a></li><li><a href="#flows">Flows side by side</a></li><li><a href="#compare">Comparison</a></li><li><a href="#pick">Recommendation</a></li><li><a href="#notes">Notes on the data</a></li></ul>

<p>Three complete ideas for the same product. They differ in structure first and in look second: F1 is a map you zoom, F2 is a grid with a cursor, F3 is a log you type into. Each has its own page with the navigation model, screen inventory, five flows, keyboard model, token sheet, six mock screens in light and dark, egui notes and weaknesses.</p>
<p>All three use the same invented app: <code>shop-api</code> (Laravel 13.2, package 13.3.0) with 7 modules. Checkout has the problems: it and Shipping use each other, and 3 of its files read a Billing model directly. The switch at the top right changes the theme of the mocks on this page.</p>

<h2 id="directions">The directions</h2>

<h3>F1 Site plan</h3>
<p>${d.f1.concept}</p>
${shot(d.f1.home('t-auto'), 'F1 home. <a href="f1.html">Open the F1 page</a> for the module plot, the build drawer, the Why sheet and the token sheet.')}

<h3>F2 Crosstab</h3>
<p>${d.f2.concept}</p>
${shot(d.f2.home('t-auto'), 'F2 home. <a href="f2.html">Open the F2 page</a> for the row sheet, the create pane, the dependency matrix and the token sheet.')}

<h3>F3 Logbook</h3>
<p>${d.f3.concept}</p>
${shot(d.f3.home('t-auto'), 'F3 home. <a href="f3.html">Open the F3 page</a> for the module thread, the draft, the command line and the token sheet.')}

<h2 id="shared">What they share</h2>
<ul>
<li><b>No colour per module.</b> The baseline gives each module a colour. That works for 7 and fails for 60, so all three identify a module by name and by a stable position (a plot, a row, a thread). Colour is kept for state.</li>
<li><b>State is a shape and a word, then a colour.</b> Problem is a filled diamond, warning an outlined triangle, good a check or nothing, off a dashed outline. Every text and mark pair in the token sheets is checked against WCAG AA in both themes; the ratios are printed on each page.</li>
<li><b>Nothing is written before you see it.</b> Every generator shows the exact artisan command and the files, and planned files are drawn in the main view (dashed outlines in F1, "+1" in F2, a draft entry in F3).</li>
<li><b>Forms are the real commands.</b> The model form lists the options of <code>module:make-model</code> as they are in the package today: migration, policy, factory, seeder, controller, request, resource, all, fillable, Eloquent attributes, force.</li>
<li><b>Built for a custom-drawn UI.</b> Flat fills, 1 and 2 px strokes, small radii, at most one shadow (on menus), no blur, no native controls, no window chrome. Fonts are all under the SIL Open Font License.</li>
<li><b>A failed command explains itself.</b> The output is shown as it came, with one sentence above the actions that says what stopped and what state things are in.</li>
</ul>

<h2 id="flows">Flows side by side</h2>
<p>Number of actions. One action is a click, a key or key chord, or filling one field.</p>
${table(['Flow', 'F1 Site plan', 'F2 Crosstab', 'F3 Logbook'], [
    ['(a) Add an app', '3', '3', '3'],
    ['(b) Find the module with a problem, see what is wrong', '0 to find, 1 to read', '0 to find, 1 to read', '0 to find, 1 to read'],
    ['(c) Model with migration and policy in an existing module', '6 from the room, 8 from the plan', '6', '2 typed, 7 with the mouse'],
    ['(d) Run migrations for one module, read the output', '3', '3', '2'],
    ['(e) Why two modules depend on each other', '1 or 2', '2', '2, or 1 from a problem line'],
  ])}
<p>The counts are close. The directions differ less in speed than in what you understand while doing the task: F1 shows where the module sits, F2 shows how it compares, F3 shows what was done.</p>

<h2 id="compare">Comparison</h2>
${table(['', 'Strongest at', 'Weakest at', 'Build cost in egui', 'Best for'], [
    ['<b>F1 Site plan</b>', 'Understanding structure. A broken boundary looks broken on the first screen. Most recognisable.', 'Sorting and comparing numbers. Needs zoom and scroll at 60 modules.', 'Highest. Own canvas, level layout, link routing, canvas focus. The routing risk is contained by drawing only broken rules and the selection.', 'A solo developer or a small team with 5 to 25 modules per app; anyone new to a codebase; a lead who cares about boundaries.'],
    ['<b>F2 Crosstab</b>', 'Scale and keyboard. 60 modules are 60 rows; the dependency matrix stays readable; creating is one cell and one key.', 'First impression and explanation. It is still a table, and the matrix must be learned.', 'Lowest. A table with fixed header exists; pinned columns and the cell cursor are the custom parts.', 'Teams with many apps and 30 to 60 modules each; people who like spreadsheets.'],
    ['<b>F3 Logbook</b>', 'Doing things fast from the keyboard, and keeping a record of what was done.', 'Overview. It tells, it does not show. Slowest with a mouse.', 'Low to medium. Mostly lists and text; the parser and the unequal-height log are the custom parts.', 'A developer who lives in the terminal and mostly wants generators and actions.'],
  ])}

<h2 id="pick">Recommendation</h2>
<p><b>F1 Site plan.</b></p>
<ul>
<li><b>It is about the thing the package is for.</b> People split an app into modules to keep parts separate. F1 is the only direction where separation is what you look at, and where breaking it is visible without reading. Faster generators are useful; a visible boundary is the reason to open a second app next to the editor.</li>
<li><b>The home screen answers all three reasons to open the app</b> with no click: what is here (plots and levels), what is wrong (marks, red links, "Needs attention"), where to build (select and ${k('B')}).</li>
<li><b>The expensive part is already on the list.</b> The module graph is on the phase 1 list of the brainstorm, and it needs a layout in any direction that draws it. F1 makes that work the home screen instead of one more page. (F2 avoids it by using a matrix.)</li>
<li><b>It cannot be mistaken for an admin panel</b>, which matters for an open source app that has to be recognised from one screenshot.</li>
</ul>
<p>Two loans cover its weak spots, and both are small because they come from the other directions:</p>
<ul>
<li><b>From F2:</b> the module index in F1 is a sortable grid of the same rows. Build it first; it gives a working app before the canvas exists, and above about 25 modules it becomes the second way in. The dependency matrix can sit there as a view for large apps.</li>
<li><b>From F3:</b> let the Go to field take a command line. <code>make model Coupon in checkout -m -p</code> then opens the build drawer filled in, which brings flow (c) down to 2 actions for people who type.</li>
</ul>
<p>What would change this: if the first users mostly have 40 or more modules per app, start from F2 and add the plan as a view. If there is no time for a custom canvas before a first release, ship the index grid alone and keep the plan for the second.</p>
<p class="dim">Name: "Companion" says what the app is next to, not what it does. If F1 is chosen, "Plot" or "Siteplan" would carry the idea; for F2 "Lattice"; for F3 "Logbook".</p>

<h2 id="notes">Notes on the data</h2>
<ul>
<li>Numbers per module (files, lines, tests, coverage, uses, used by) are taken from the modules table of the existing brainstorm page. The per-kind counts for modules other than Checkout are invented to add up to those file totals.</li>
<li>The existing page says "10 links" on its graph screen; its table adds up to 9 links, and its drawing shows 9. These pages use 9.</li>
<li>The existing page counts "1 problem" for Checkout and then lists two. These pages count 2 problems (circular use, access to a Billing model) and treat the waiting migration and Notifications' coverage as warnings.</li>
<li>Generator count: the package has 41 generator commands besides <code>module:make</code> (40 named <code>module:make-*</code> and <code>module:route-provider</code>).</li>
<li>The file preview in the mocks follows the package's default paths (<code>app/Models</code>, <code>app/Policies</code>, <code>database/migrations</code>). In the real app the list has to come from the package, as the brainstorm page already says.</li>
<li>The reference lists for "why" (file names, line numbers, class names) and the migration error are invented.</li>
</ul>`;
}
