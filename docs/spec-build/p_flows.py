from lib import *
from data import *

def flow(fid, title, goal, mouse, keys, note=''):
    ol = lambda steps: '<ol>%s</ol>' % ''.join('<li>%s</li>' % s for s in steps)
    return ('<h3 id="%s"><span class="sid">%s</span>%s</h3><p class="cap">%s</p><div class="flowgrid"><div><p><strong>With the mouse</strong></p>%s</div><div><p><strong>With the keyboard</strong></p>%s</div></div>%s') % (
        fid, fid, title, goal, ol(mouse), ol(keys), '<div class="notes">%s</div>' % note if note else '')
r = ref

def build():
    CUR['page'] = 'flows.html'
    b = ('<p>Sections 6 and 7 of the specification. The first five flows are the ones the workbench structure was chosen for; the sixth was added with the public surface scan. Each is given step by step, once for the mouse and once for the keyboard, with the screen each step lands on. '
         'Shortcuts in the steps are written for Windows and Linux; on macOS Ctrl is Cmd, except where the table in section 7 says otherwise.</p>')
    b += '<h2>6. Flows</h2>'
    b += flow('F1', 'Add an app', 'From an empty Mortise to the modules of booking-engine on screen. 3 actions.',
              ['Click the dashed <strong>+</strong> tile on the rail, or "Add app folder" on the first run screen %s. The add panel opens %s.' % (r('S1.1'), r('S1.2')),
               'Click "Choose folder" and pick <code>~/code/booking-engine</code> in the folder dialog, or drop the folder on the window. The panel shows what was found %s. If something is wrong it says so at the field %s.' % (r('S1.3'), r('S1.4')),
               'Click "Add booking-engine". Its tile appears on the rail and is selected; the index lists its modules and the workspace shows its Summary %s.' % r('S4.2')],
              ['Press Ctrl O from anywhere. The folder dialog of the operating system opens at once.',
               'Choose the folder there and confirm. Focus lands on the "Add booking-engine" button %s; Tab reaches the PHP and runtime choices if they need changing.' % r('S1.3'),
               'Press Enter or Ctrl Enter. Focus moves to the "All modules" row of the new app %s.' % r('S4.2')],
              '<div><b>Also.</b> Ctrl K, "add", Enter opens the same panel. A toast confirms: "%s"</div>' % t('m.added'))
    b += flow('F2', 'Find the module with a problem and see what is wrong', 'From the home screen to the two findings of Checkout. 1 action.',
              ['On the home screen %s the index already shows the problem mark and "2" on Checkout, and the alert names it. Click the Checkout row. Its Summary opens with both problems in the alert at the top %s.' % (r('S4.1'), r('S4.3')),
               'Optional: click "Show all 3 findings" on the home screen for the list of the whole app %s, or the Boundary tab for the files %s.' % (r('S6.4'), r('S5.1'))],
              ['Press J. With the index sorted problems first, Checkout is the first module under "All modules", and its Summary is on screen in the same frame %s.' % r('S4.3'),
               'Optional: press F for the findings list %s, where Enter on a finding opens it where it lives; or press 2 for Boundary %s.' % (r('S6.4'), r('S5.1'))],
              '<div><b>If the index is sorted by name.</b> Press S once, or read the pip on the rail and the Findings stat. With 60 modules the pinned group "Needs attention" does the same job %s.</div>' % r('S3.1'))
    b += flow('F3', 'Create a model with a migration and a policy', 'A new model PaymentIntent in Billing, with its migration and policy. 4 actions. Phase 2.',
              ['Click Billing in the index, then the Generate tab. The picker opens with Model first %s.' % r('S9.1'),
               'Click Model. Type <code>PaymentIntent</code> in "Model name" and the fillable attributes if wanted %s.' % r('S9.2'),
               'Check Migration and Policy. Read the command and the three files on the right. If a file exists already the preview says so %s.' % r('S9.3'),
               'Click "Create 3 files". The result replaces the preview and the runner drawer shows the output %s; on a failure it says which file and what to do %s.' % (r('S9.5'), r('S9.6'))],
              ['Press Ctrl K, type <code>model bil</code>, press Enter. The form opens with Billing as the module and focus in "Model name" %s.' % r('S9.2'),
               'Type <code>PaymentIntent</code>, Tab, type the fillable attributes.',
               'Tab to Migration, Space. Tab to Policy, Space. The command and the file list update as you go.',
               'Press Ctrl Enter %s. Enter then opens the model in the editor.' % r('S9.5')],
              '<div><b>Without the palette.</b> J or K to Billing, G, type <code>mod</code>, Enter.</div>')
    b += flow('F4', 'Run migrations for one module and read the output', 'The waiting migration of Checkout runs and its output is on screen. 2 actions. Phase 2.',
              ['Click Checkout in the index %s.' % r('S4.3'),
               'Click "Migrate (1)" in the module header. The runner drawer opens under the workspace with the command and its output as it arrives; when it ends the status reads "%s" or "Exit code 1" with the next steps %s.' % (t('m.done'), r('S11.1'))],
              ['Press J to select Checkout.',
               'Press 4 for Actions; Migrate is selected because a migration waits %s. Press Ctrl Enter. Focus stays where it was; the drawer shows the output %s. Ctrl ` hides it.' % (r('S10.1'), r('S11.1'))],
              '<div><b>With options.</b> A dry run or seeding afterwards are the two checkboxes on the Actions tab. In production Mortise asks first %s.</div>'
              '<div><b>From the palette.</b> Ctrl K, <code>migrate chk</code>, Enter.</div>' % r('S10.3'))
    b += flow('F5', 'See why two modules depend on each other', 'From Checkout to the six files that make Checkout and Shipping use each other. 2 actions.',
              ['Click Checkout in the index.',
               'Click the Boundary tab. Checkout is in the middle with Shipping on both sides, both links drawn as problems %s. Click either Shipping node: the files of both directions are listed below, with the line and the class each one reads %s.' % (r('S5.1'), r('S5.2'))],
              ['Press J to select Checkout.',
               'Press 2. Focus is on the first problem link. Arrow keys move between the modules of the graph %s; on Shipping the list below shows the files %s. Tab moves into the list, J and K move through it, Enter opens a file at its line in the editor.' % (r('S5.1'), r('S5.2'))],
              '<div><b>From the app.</b> On "All modules", Boundary lists the links that break a rule; Enter on "Checkout and Shipping use each other" lands on the same view %s.</div>' % r('S5.3'))

    b += flow('F6', 'Propose and accept the public surface of a module', 'Added after the owner\'s answer on the public surface. From Billing to an accepted list of what Billing offers to other modules, to HTTP clients and to the command line. 4 actions. Accepting needs the package (P6).',
              ['Click Billing in the index, then the Boundary tab, then "Public surface" in the control at the top %s.' % r('S5.5'),
               'Click "Scan Billing". The view shows the scan as it reads %s, then the proposal %s.' % (r('S5.6'), r('S5.7')),
               'Read the list. Click a checkbox to leave an entry out or take one in; type in the field at the bottom to add one by hand.',
               'Click "Accept 21 entries". The view shows the accepted surface %s and the findings are worked out again.' % r('S5.8')],
              ['J or K to Billing, 2 for Boundary, P for Public surface %s.' % r('S5.5'),
               'Ctrl Enter starts the scan %s. Esc stops it. When it ends, focus is on the first entry of the proposal %s.' % (r('S5.6'), r('S5.7')),
               'J and K move through the entries, Space includes one or leaves it out, A goes to the add field, Enter there adds the entry.',
               'Ctrl Enter accepts %s.' % r('S5.8')],
              '<div><b>Later.</b> The same Ctrl Enter in the accepted view scans again. If the code has changed, only the differences are listed, each to accept or to leave %s.</div>'
              '<div><b>From the palette.</b> Ctrl K, <code>surface bil</code>, Enter opens the view; it does not start the scan.</div>' % r('S5.9'))

    b += '<h2>7. Keyboard</h2><p>The complete table. Single keys work when focus is not in a text field. On macOS the modifier is Cmd where Windows and Linux use Ctrl, with one exception in the second row. Shortcuts are fixed in the first release (owner decision); keys that are awkward on some layouts have a second binding instead.</p>'
    rows, last = [], None
    for scope, what, mac, other, note in SHORTCUTS:
        rows.append(['<strong>%s</strong>' % scope if scope != last else '', what, '<code>%s</code>' % mac, '<code>%s</code>' % other, note]); last = scope
    b += spec_table(['Where', 'Does', 'macOS', 'Windows and Linux', 'Note'], rows)
    b += ('<p><strong>Rules behind the table.</strong></p><ul>'
          '<li>Everything can be done without a pointer. Nothing is reachable only by hover or by dragging: the index width and the drawer height have keys, and the palette reaches every command.</li>'
          '<li>Single letters never act while a text field has focus. Esc leaves the field first.</li>'
          '<li>A shortcut is shown where its action is: in the button, in the tab, in the palette row, in the tooltip.</li>'
          '<li>Keys that are awkward on some keyboard layouts have a second binding with a modifier: <code>/</code> and <code>`</code>.</li></ul>')
    b += '<h3 id="K1"><span class="sid">K1</span>Focus order of the main window</h3><p>Tab moves through controls in this order; Shift Tab goes back. F6 jumps between the four regions. A list, a table, the tab row and a segmented control are each one stop: arrows move inside them.</p>'
    b += ('<svg class="diagram" viewBox="0 0 900 300" width="900" role="img" aria-label="The main window with its focus regions numbered 1 to 5">'
          '<g fill="none" stroke="#777" stroke-width="1"><rect x="10.5" y="10.5" width="879" height="279"/><path d="M10 44.5H890M70.5 44V290M300.5 44V290M300 110.5H890M300 210.5H890"/></g>'
          '<g fill="#fff" font-family="system-ui,sans-serif" font-size="13"><text x="24" y="32">1  Title bar: search entry</text>'
          '<text x="22" y="70">2</text><text x="22" y="92" font-size="11" fill="#a8a8a8">tiles</text><text x="22" y="108" font-size="11" fill="#a8a8a8">add</text><text x="22" y="124" font-size="11" fill="#a8a8a8">set-</text><text x="22" y="138" font-size="11" fill="#a8a8a8">tings</text>'
          '<text x="86" y="70">3  Module index</text><text x="86" y="94" font-size="12" fill="#a8a8a8">a. filter field</text><text x="86" y="114" font-size="12" fill="#a8a8a8">b. the list of modules (one stop)</text>'
          '<text x="86" y="134" font-size="12" fill="#a8a8a8">c. order, d. New module</text>'
          '<text x="316" y="70">4  Workspace</text><text x="316" y="94" font-size="12" fill="#a8a8a8">a. header buttons, b. the tab row (one stop)</text>'
          '<text x="316" y="136" font-size="12" fill="#a8a8a8">c. the controls of the tab, in reading order: left column top to bottom, then the right column,</text>'
          '<text x="316" y="154" font-size="12" fill="#a8a8a8">ending on the primary button</text>'
          '<text x="316" y="236">5  Runner drawer</text><text x="316" y="258" font-size="12" fill="#a8a8a8">a. handle, b. its buttons, c. the output (scrolls with the arrow keys)</text></g></svg>')
    b += ('<ul><li><strong>At start</strong>, focus is on the selected row of the index, so J, K and the digit keys work at once.</li>'
          '<li><strong>Selecting a module</strong> with J or K leaves focus in the index. Enter or Right moves it to the first control of the tab; Esc brings it back.</li>'
          '<li><strong>Switching tab</strong> with a digit leaves focus where it was, except on Generate, where it goes to the first field of the form.</li>'
          '<li><strong>The palette and a dialog</strong> hold focus until they close, then return it to where it was. Tab does not leave them.</li>'
          '<li><strong>Window controls</strong> drawn by Mortise (Windows, Linux) are not in the Tab order; the platform\'s own keys work for them.</li>'
          '<li><strong>A command that ends</strong> does not move focus. The drawer opening never takes focus.</li></ul>')
    return page('flows.html', 'Flows and keyboard', 'Mortise design specification, sections 6 and 7. The five flows for mouse and keyboard, the shortcut table, the focus order.', b, switch=False)
