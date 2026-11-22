from lib import *
from data import *

GEN = [  # group, [(label, artisan name)]
 ('Data', [('Model', 'module:make-model'), ('Migration', 'module:make-migration'), ('Factory', 'module:make-factory'), ('Seeder', 'module:make-seed'),
           ('Observer', 'module:make-observer'), ('Scope', 'module:make-scope'), ('Cast', 'module:make-cast'), ('Policy', 'module:make-policy'), ('Repository', 'module:make-repository')]),
 ('HTTP', [('Controller', 'module:make-controller'), ('Request', 'module:make-request'), ('Resource', 'module:make-resource'), ('Middleware', 'module:make-middleware'),
           ('Rule', 'module:make-rule'), ('Route provider', 'module:route-provider')]),
 ('Events and queues', [('Event', 'module:make-event'), ('Listener', 'module:make-listener'), ('Job', 'module:make-job'), ('Job middleware', 'module:make-job-middleware'),
                        ('Notification', 'module:make-notification'), ('Mail', 'module:make-mail'), ('Channel', 'module:make-channel'), ('Event provider', 'module:make-event-provider')]),
 ('Views', [('View', 'module:make-view'), ('Component class', 'module:make-component'), ('Component view', 'module:make-component-view'),
            ('Inertia page', 'module:make-inertia-page'), ('Inertia component', 'module:make-inertia-component')]),
 ('Classes', [('Class', 'module:make-class'), ('Action', 'module:make-action'), ('Service', 'module:make-service'), ('Interface', 'module:make-interface'),
              ('Trait', 'module:make-trait'), ('Enum', 'module:make-enum'), ('Helper', 'module:make-helper'), ('Exception', 'module:make-exception')]),
 ('Console and setup', [('Command', 'module:make-command'), ('Config', 'module:make-config'), ('Provider', 'module:make-provider'), ('Stub replacement', 'module:make-replacement')]),
 ('Tests', [('Test', 'module:make-test')]),
]
assert sum(len(g[1]) for g in GEN) == 41
RUNHINT = '<span class="dim" style="font-size:11.5px">Run with</span>' + kbd('Ctrl Enter')

def picker():
    items = '<div class="gh">In the app</div><div class="gi"><b style="font-weight:600">New module</b><span>module:make</span></div><div></div><div></div>'
    for g, rows in GEN:
        items += '<div class="gh">%s</div>' % g
        items += ''.join('<div class="gi%s"><b style="font-weight:600">%s</b><span>%s</span></div>' % (' on' if n == 'module:make-model' else '', l, n) for l, n in rows)
    top = ('<div style="display:flex;gap:10px;align-items:center"><div class="filter is-focus" style="margin:0;width:300px">%s<span>Find a generator%s</span>%s</div>'
           '<span class="dim" style="margin-left:auto;font-size:11.5px">41 generators for Billing, and New module. Enter opens the form.</span></div>') % (ic('search'), CARET, kbd('/'))
    return win(tbar() + shell(rail(), index('BIL'), work('Generate in Billing', 2, top + '<div class="gen">%s</div>' % items, '', '', 'BIL')))

MODEL_CHKS = [('Controller', '-c', False), ('Factory', '-f', False), ('Migration', '-m', True), ('Request', '-r', False), ('Resource', '-R', False), ('Policy', '-p', True), ('Seeder', '-s', False), ('All of these', '-a', False)]
def model_form(clash=False):
    cks = ''
    for l, f, on in MODEL_CHKS:
        extra = ' ' + S('warn', 'exists') if (clash and l == 'Policy') else ''
        cks += chk(l + extra, f, on and not (clash and l == 'Policy'), '', l == 'All of these' and False)
    return ('<div>%s%s%s%s%s</div>') % (
        field('Generator', '41 available', inp('<b>Model</b> <span class="mono dim">module:make-model</span>', '', kbd('Change'))),
        field('Model name', '', inp('PaymentIntent', 'mono')),
        field('Fillable attributes', '--fillable, comma separated', inp('order_id,amount,currency,status', 'mono')),
        field('Also create', '', '<div class="chks">%s</div>' % cks),
        '<div class="field opt">%s</div>' % chk('Use Eloquent attributes such as <span class="mono">#[Fillable]</span>', '--attributes'))
CMD_MODEL = ['php', 'artisan', 'module:make-model', 'PaymentIntent', 'Billing', '--migration', '--policy', '--fillable=order_id,amount,currency,status']
TREE = ('<pre class="tree">Modules/Billing/\n  app/Models/<b>PaymentIntent.php</b>\n  app/Policies/<b>PaymentIntentPolicy.php</b>%s\n  database/migrations/\n    <b>2026_11_22_140500_create_payment_intents_table.php</b></pre>')
def gen_work(left, right, drawer='', extra=''):
    return work('Generate in Billing', 2, '<div class="cols">%s<div class="stack">%s</div></div>%s' % (left, right, extra), RUNHINT, drawer, 'BIL')
def gen_form():
    right = ('<div><div class="lbl">Command</div>%s</div>%s<div class="btns">%s%s</div>') % (
        cmd(*CMD_MODEL), box('Files this creates', S('good', t('m.preview.ok')), TREE % '', '', 'flex:1'), btn(t('b.create'), 'pri', None, 'Ctrl Enter'), btn(t('b.copycmd')))
    return win(tbar() + shell(rail(), index('BIL'), gen_work(model_form(), right)))
def gen_clash():
    right = ('<div><div class="lbl">Command</div>%s</div>%s%s<div class="btns">%s%s</div>') % (
        cmd(*[c for c in CMD_MODEL if c != '--policy']), box('Files this creates', S('warn', t('m.preview.clash')), TREE % '   <span style="color:var(--warn);font-weight:600">exists</span>', '', ''),
        alert('<span class="mono">app/Policies/PaymentIntentPolicy.php</span> exists. A generator does not write over a file, so the policy was taken out of the command. Create it afterwards with "Replace" if you want the generated one.', 'warn'),
        btn('Create the other 2 files', 'pri', None, 'Ctrl Enter'), btn(t('b.copycmd')))
    return panel('<div class="cols">%s<div class="stack">%s</div></div>' % (model_form(True), right), 'surface', '14px 16px')

def drawer(cmdline, status, out, extra='', tools=None):
    tl = tools if tools is not None else '<span>%s</span><span>%s</span><span>%s</span>%s' % (t('b.again'), t('b.copyout'), t('b.history'), '<span>%s</span>%s' % (t('b.hide'), kbd('Ctrl `')))
    return '<div class="drawer"><div class="dbar"><span><b>Runner</b><span class="mono">%s</span></span><span>%s%s</span></div><div>%s</div>%s</div>' % (cmdline, status, tl, out, extra)
P = '/code/shop-api/Modules/Billing/'
OUT_OK = ('<span class="c-dim">$ php artisan module:make-model PaymentIntent Billing --migration --policy --fillable=order_id,amount,currency,status</span><br>'
          '&nbsp;&nbsp;Generating file %sapp/Models/PaymentIntent.php ......... <span class="c-ok">DONE</span><br>'
          '&nbsp;&nbsp;Generating file %sdatabase/migrations/2026_11_22_140500_create_payment_intents_table.php ... <span class="c-ok">DONE</span><br>'
          '&nbsp;&nbsp;Generating file %sapp/Policies/PaymentIntentPolicy.php ... <span class="c-ok">DONE</span>') % (P, P, P)
OUT_FAIL = OUT_OK.rsplit('<br>', 1)[0] + '<br><br>&nbsp;&nbsp;<span class="c-err">ERROR&nbsp; File : %sapp/Policies/PaymentIntentPolicy.php already exists.</span>' % P
def frow(mark, path, right='', new=False):
    return '<div class="lrow%s"><span style="display:flex;gap:8px;align-items:center;min-width:0">%s<span class="mono" style="overflow-wrap:anywhere;line-height:1.35;padding:5px 0">%s</span></span>%s</div>' % (' newf' if new else '', mark, path, right)
def toast(kind, text, action=''):
    return '<div class="toast">%s<div class="tx">%s%s</div>%s</div>' % (S(kind), text, '<div class="btns">%s</div>' % action if action else '', ic('x', 12))
def gen_created():
    right = ('<div class="box flush"><h5><span>Created in Billing</span>%s</h5>%s%s%s</div><div class="btns">%s%s</div>') % (
        S('good', '3 files'), frow(S('good'), 'app/Models/PaymentIntent.php', kbd('Open'), True), frow(S('good'), 'app/Policies/PaymentIntentPolicy.php', kbd('Open'), True),
        frow(S('good'), 'database/migrations/2026_11_22_140500_create_payment_intents_table.php', kbd('Open'), True),
        btn('Open PaymentIntent.php', 'pri', None, 'Enter'), btn(t('b.another')))
    tst = '<div style="position:absolute;right:16px;bottom:128px">%s</div>' % toast('good', t('m.created'))
    w = gen_work(model_form(), right, drawer('php artisan module:make-model PaymentIntent Billing ...', S('good', 'Done in 412 ms'), OUT_OK), tst).replace('<div class="work">', '<div class="work" style="position:relative">')
    return win(tbar() + shell(rail(), index('BIL', head=idx_head(on='7 of 7 on')), w))
def gen_failed():
    right = ('<div class="box flush"><h5><span>Created 2 of 3 files</span>%s</h5>%s%s%s</div>%s<div class="btns">%s%s</div>') % (
        S('bad', '1 failed'), frow(S('good'), 'app/Models/PaymentIntent.php', kbd('Open')), frow(S('good'), 'database/migrations/2026_11_22_140500_create_payment_intents_table.php', kbd('Open')),
        frow(S('bad'), 'app/Policies/PaymentIntentPolicy.php', '<span class="st bad">exists, left as it is</span>'),
        alert(t('x.exists')), btn(t('b.overwrite'), 'danger'), btn('Open the existing file'))
    w = gen_work(model_form(), right, drawer('php artisan module:make-model PaymentIntent Billing ...', S('bad', 'Exit code 1') + '<span class="mono">398 ms</span>', OUT_FAIL))
    return win(tbar() + shell(rail(), index('BIL'), w))

def module_form():
    left = ('<div>%s%s%s%s<div class="field opt"><div class="lbl">Author <span>optional, written to composer.json</span></div><div style="display:grid;grid-template-columns:1fr 1fr 1fr;gap:8px">%s%s%s</div></div></div>') % (
        field('Module names', 'one or more, separated by spaces', inp('Loyalty', 'mono')),
        field('Kind', '--web is the default', seg(['Web', 'API', 'Plain'], 1)),
        field('Options', '', chk('Inertia controller and pages, no Blade views', '--inertia') + chk('Turn the module on after creation', 'unchecked adds --disabled', True)
              + chk('Replace the module if it exists', '--force')),
        '', inp('<span class="ph">Name</span>'), inp('<span class="ph">Email</span>'), inp('<span class="ph">Vendor</span>'))
    tree = ('<pre class="tree">Modules/<b>Loyalty</b>/\n  <b>module.json</b>\n  <b>composer.json</b>\n  app/Http/Controllers/<b>LoyaltyController.php</b>\n  app/Providers/<b>LoyaltyServiceProvider.php</b>\n'
            '  app/Providers/<b>EventServiceProvider.php</b>\n  app/Providers/<b>RouteServiceProvider.php</b>\n  config/<b>config.php</b>\n  database/seeders/<b>LoyaltyDatabaseSeeder.php</b>\n  routes/<b>api.php</b>\n  tests/Feature/  tests/Unit/</pre>')
    right = ('<div><div class="lbl">Command</div>%s</div>%s<div class="btns">%s%s</div>') % (
        cmd('php', 'artisan', 'module:make', 'Loyalty', '--api'),
        box('Files this creates', S('good', 'Modules/Loyalty does not exist yet'), tree + '<div class="hint2">Then the command runs <span class="mono">composer dump-autoload</span> in the app.</div>'),
        btn(t('b.createmod'), 'pri', None, 'Ctrl Enter'), btn(t('b.copycmd')))
    return win(tbar() + shell(rail(), index('ALL'), work('New module in shop-api', 2, '<div class="cols">%s<div class="stack">%s</div></div>' % (left, right), RUNHINT)))

# ------------------------------------------------------------ actions
ACTS = [('Turn off', S('good', 'On')), ('Migrate', S('warn', '1 waiting')), ('Roll back', '<span class="dim">last batch, 1</span>'), ('Seed', '<span class="dim">1 seeder</span>'),
        ('Run tests', '<span class="dim">31 tests</span>'), ('Check translations', '<span class="dim">en, fr</span>')]
DESTR = [(t('d.reset'), '<span class="dim">9 ran</span>'), (t('d.refresh'), '<span class="dim">9 ran</span>'), (t('d.delete'), '<span class="dim">84 files</span>')]
def act_list(sel=1, rows=ACTS, destr=None, dsel=None, closed=None):
    out = ''.join('<div class="lrow%s"><span>%s</span>%s</div>' % (' on' if i == sel else '', n, r) for i, (n, r) in enumerate(rows))
    if destr is None:
        out += '<div class="lrow dgrp"><span style="display:flex;gap:6px;align-items:center">%s%s</span><span class="dim">%s</span></div>' % (ic('chev-r', 11), t('d.group'), closed or t('d.closed'))
    else:
        out += '<div class="lrow dgrp"><span style="display:flex;gap:6px;align-items:center">%s%s</span><span class="dim">open</span></div>' % (ic('chev-d', 11), t('d.group'))
        out += ''.join('<div class="lrow dact%s"><span>%s</span>%s</div>' % (' on' if i == dsel else '', n, r) for i, (n, r) in enumerate(destr))
        out += '<div style="padding:6px 10px 7px 26px;font-size:11.5px;line-height:1.4;color:var(--ink-2)">Dropping all tables is under "All modules".</div>'
    return '<div class="box flush" style="align-self:start">%s</div>' % out
def act_detail(title, text, body, command, buttons):
    return ('<div class="stack"><div><div style="font-size:15px;font-weight:700;line-height:20px">%s</div><div class="dim" style="margin-top:2px">%s</div></div>%s'
            '<div><div class="lbl">Command</div>%s</div><div class="btns">%s</div></div>') % (title, text, body, cmd(*command), buttons)
def d_migrate():
    return act_detail('Migrate Checkout', 'Runs the migrations of Checkout that have not run yet.',
        box('Waiting', '1 migration', '<div class="kv"><span class="mono">2026_11_20_091200_add_tracking_code_to_checkouts_table</span><span></span></div>')
        + '<div>%s%s</div>' % (chk('Dry run: print the SQL, change nothing', '--pretend'), chk('Seed afterwards', '--seed'))
        + field('Database connection', '--database, empty for the default', inp('<span class="ph">default</span>', '', '', 'max-width:260px')),
        ['php', 'artisan', 'module:migrate', 'Checkout'], btn(t('b.migrate'), 'pri', 'migrate', 'Ctrl Enter') + btn(t('b.copycmd')))
def d_off():
    return act_detail('Turn off Checkout', 'Checkout is on. The app stops loading its providers, routes and migrations. Its files stay.',
        alert('Shipping and Notifications use Checkout. Mortise asks before it turns off a module that others use.', 'warn'),
        ['php', 'artisan', 'module:disable', 'Checkout'], btn(t('b.off'), 'pri', None, 'Ctrl Enter') + btn(t('b.copycmd')))
def d_on():
    return act_detail('Turn on Checkout', 'Checkout is off. Turning it on does not run its migrations.', '',
        ['php', 'artisan', 'module:enable', 'Checkout'], btn(t('b.on'), 'pri', None, 'Ctrl Enter') + btn(t('b.copycmd')))
def d_rollback():
    return act_detail('Roll back Checkout', 'Runs the down method of the last migration batch of Checkout.',
        box('Last batch', '1 migration', '<div class="kv"><span class="mono">2026_11_18_101500_add_coupon_code_to_orders_table</span><span></span></div>')
        + '<div>%s</div>' % chk('Dry run: print the SQL, change nothing', '--pretend')
        + '<div class="hint2">Destructive. Mortise asks before it runs. The command has no option to choose how many batches.</div>',
        ['php', 'artisan', 'module:migrate-rollback', 'Checkout'], btn(t('b.rollback'), 'danger', None, 'Ctrl Enter') + btn(t('b.dry')) + btn(t('b.copycmd')))
def d_seed():
    return act_detail('Seed Checkout', 'Runs the seeders of Checkout. Rows are added; running it twice can add them twice.',
        '<div style="display:grid;grid-template-columns:1fr 1fr;gap:10px">%s%s</div>' % (
            field('Seeder class', '--class, empty for the module seeder', inp('<span class="ph">CheckoutDatabaseSeeder</span>', 'mono')),
            field('Database connection', '--database', inp('<span class="ph">default</span>'))),
        ['php', 'artisan', 'module:seed', 'Checkout'], btn(t('b.seed'), 'pri', None, 'Ctrl Enter') + btn(t('b.copycmd')))
TESTCMD = code('<span class="tk">php</span> <span class="tk">artisan</span> <span class="tk c-dim">[the package\'s command for the tests of one module]</span> <span class="tk">Checkout</span>')
def d_tests():
    return ('<div class="stack"><div><div style="font-size:15px;font-weight:700;line-height:20px">Run tests of Checkout</div><div class="dim" style="margin-top:2px">31 tests in <span class="mono">Modules/Checkout/tests</span>. '
            'The package will provide a command that runs the tests of one module. Its name and options are not decided, so the line below has a placeholder and no options are drawn.</div></div>'
            '<div><div class="lbl">Command</div>%s</div><div class="btns">%s%s</div></div>') % (TESTCMD, btn(t('b.tests'), 'pri', 'test', 'Ctrl Enter'), btn(t('b.copycmd')))
def lost(rows):
    return '<div class="box flush"><h5><span>What is lost</span>%s</h5>%s</div>' % (S('bad', 'cannot be undone'), ''.join('<div class="lrow"><span>%s</span><span class="dim">%s</span></div>' % r for r in rows))
def d_reset(prod=False):
    body = (lost([('Every table and column created by the 9 migrations of Checkout that ran', 'with all rows'), ('<span class="mono">checkouts</span>, <span class="mono">carts</span>, <span class="mono">cart_items</span>, <span class="mono">orders</span>, <span class="mono">order_items</span> and 2 more', '7 tables')])
            + '<div>%s</div>' % chk('Dry run: print the SQL, change nothing', '--pretend')
            + field('Database connection', '--database, empty for the default', inp('<span class="ph">default</span>', '', '', 'max-width:260px')))
    if prod: body = alert(t('d.prod'), 'warn') + body
    return act_detail('Reset the migrations of Checkout', 'Rolls back every migration of Checkout that has run, newest first.', body,
        ['php', 'artisan', 'module:migrate-reset', 'Checkout'], btn(t('d.reset'), 'danger' + (' is-dis' if prod else '')) + btn(t('b.dry'), 'is-dis' if prod else '') + btn(t('b.copycmd')))
def d_refresh():
    return act_detail('Refresh the migrations of Checkout', 'Rolls back every migration of Checkout, then runs them all again. The command has no dry run.',
        lost([('Every row in the tables of Checkout', '7 tables come back empty')]) + '<div>%s</div>' % chk('Seed afterwards', '--seed')
        + field('Database connection', '--database, empty for the default', inp('<span class="ph">default</span>', '', '', 'max-width:260px')),
        ['php', 'artisan', 'module:migrate-refresh', 'Checkout'], btn(t('d.refresh'), 'danger') + btn(t('b.copycmd')))
def d_delete():
    return act_detail('Delete the module Checkout', 'Removes the module from the app. The command always asks, so Mortise asks and then passes --force.',
        lost([('The folder <span class="mono">Modules/Checkout</span>', '84 files, 6,412 lines'), ('Changes that are not committed', '12 files'), ('Its line in <span class="mono">modules_statuses.json</span>', '')])
        + '<div class="hint2">Stays: the tables of Checkout in the database and the record of its migrations. Shipping and Notifications use Checkout and will break.</div>',
        ['php', 'artisan', 'module:delete', 'Checkout', '--force'], btn(t('d.delete'), 'danger') + btn(t('b.copycmd')))
def d_fresh():
    return act_detail('Drop all tables of shop-api and migrate', 'Runs db:wipe on the database, then the migrations of the app, then those of every module that is on. The command refuses a dry run.',
        lost([('Every table of the database, also those that no module owns', 'with all rows'), ('Views, if chosen below', ''), ('Types, if chosen below', 'Postgres only')])
        + '<div class="chks">%s%s%s</div>' % (chk('Seed afterwards', '--seed'), chk('Also drop views', '--drop-views'), chk('Also drop types', '--drop-types'))
        + field('Database connection', '--database, empty for the default', inp('<span class="ph">default</span>', '', '', 'max-width:260px')),
        ['php', 'artisan', 'module:migrate-fresh', '--all'], btn(t('d.fresh'), 'danger') + btn(t('b.copycmd')))
def destructive_screen(over='', dsel=0):
    body = '<div class="cols" style="grid-template-columns:250px minmax(0,1fr)">%s%s</div>' % (act_list(None, ACTS, DESTR, dsel), d_reset() if dsel == 0 else d_delete())
    return win(tbar() + shell(rail(), index('CHK'), work('Checkout', 3, body, '', '', 'CHK', 'Modules/Checkout'), '--shell-h:520px'), over=over)
def typed(label, value, ok):
    v = '<span class="mono">%s%s</span>' % (value, CARET) if value else '<span class="ph">%s</span>' % ''
    return '<div style="margin-top:2px"><div class="lbl">%s</div><div class="inp%s">%s</div></div>' % (label, ' is-focus' if value else '', v)
def dlg_reset():
    return dialog(t('d.reset.h'), t('d.reset.p'), '<div class="code" style="font-size:11.5px">checkouts, carts, cart_items, orders, order_items, order_addresses, order_payments</div><div class="hint2">7 tables. Nothing outside Checkout is touched.</div>',
                  btn(t('b.cancel'), 'is-focus', None, 'Esc') + btn(t('d.reset.b'), 'danger fill'), btn(t('b.dry')))
def dlg_refresh():
    return dialog(t('d.refresh.h'), t('d.refresh.p'), '<div class="code" style="font-size:11.5px">checkouts, carts, cart_items, orders, order_items, order_addresses, order_payments</div><div class="hint2">7 tables. This command has no dry run.</div>',
                  btn(t('b.cancel'), 'is-focus', None, 'Esc') + btn(t('d.refresh.b'), 'danger fill'))
def dlg_fresh(done=False):
    return dialog(t('d.fresh.h'), t('d.fresh.p'), '<div class="code" style="font-size:11.5px">php artisan module:migrate-fresh --all</div>' + typed(t('d.type.app'), 'shop-api' if done else '', done),
                  btn(t('b.cancel'), '' if done else 'is-focus', None, 'Esc') + btn(t('d.fresh.b'), 'danger fill' + ('' if done else ' is-dis')))
def dlg_delete(done=False):
    return dialog(t('d.delete.h'), t('d.delete.p'), alert(t('d.delete.git'), 'warn') + typed(t('d.type'), 'Checkout' if done else '', done),
                  btn(t('b.cancel'), '' if done else 'is-focus', None, 'Esc') + btn(t('d.delete.b'), 'danger fill' + ('' if done else ' is-dis')))
def actions_screen(over='', sel=1):
    body = '<div class="cols" style="grid-template-columns:250px minmax(0,1fr)">%s%s</div>' % (act_list(sel), d_migrate() if sel == 1 else d_rollback())
    return win(tbar() + shell(rail(), index('CHK'), work('Checkout', 3, body, RUNHINT, '', 'CHK', 'Modules/Checkout'), '--shell-h:470px'), over=over)
ALLROWS = []
def actions_all():
    rows = [('Migrate all modules', S('warn', '1 waiting')), ('Seed all modules', '<span class="dim">5 seeders</span>'), ('Run all module tests', '<span class="dim">307 tests</span>'), ('Check translations', '<span class="dim">5 modules</span>')]
    ALLROWS[:] = rows
    d = act_detail('Migrate all modules', 'Runs the waiting migrations of every module that is on, in the package\'s order.',
                   box('Waiting', '1 migration in 1 module', kv('<span>%s <span class="mono">2026_11_20_091200_add_tracking_code_to_checkouts_table</span></span>' % stamp('CHK'), ''))
                   + '<div>%s%s</div>' % (chk('Dry run: print the SQL, change nothing', '--pretend'), chk('Seed afterwards', '--seed')),
                   ['php', 'artisan', 'module:migrate', '--all'], btn('Migrate (1)', 'pri', 'migrate', 'Ctrl Enter') + btn(t('b.copycmd')))
    return panel('<div class="cols" style="grid-template-columns:250px minmax(0,1fr)">%s%s</div>' % (act_list(0, rows, None, None, '1 action, closed'), d), 'surface', '14px 16px')

def dialog(title, text, extra, buttons, left=''):
    return '<div class="dlg"><div class="dlg-b"><h4>%s</h4><p>%s</p>%s</div><div class="dlg-f">%s%s</div></div>' % (title, text, extra, '<span class="lft">%s</span>' % left if left else '', buttons)
def dlg_rollback():
    return dialog(t('c.rb.h'), t('c.rb.p'), '<div class="code" style="font-size:11.5px">2026_11_18_101500_add_coupon_code_to_orders_table</div>',
                  btn(t('b.cancel'), 'is-focus', None, 'Esc') + btn(t('c.rb.b'), 'danger fill'), btn(t('b.dry')))
def dlg_stage(d, th):
    return '<div class="m fa" data-theme="%s"><div style="background:var(--surface);position:relative"><div style="background:var(--scrim);padding:22px;display:flex;justify-content:center">%s</div></div></div>' % (th, d)
def dlg_pair(d): return '<div class="pair">%s%s</div>' % (dlg_stage(d, 'light'), dlg_stage(d, 'dark'))

# ------------------------------------------------------------ runner
def runner_box(inner, cap, handle=True):
    return '<div><div class="m fa sw" data-theme="light"><div class="win sm" style="border-radius:0"><div class="work">%s%s</div></div></div><p class="cellcap">%s</p></div>' % (
        '<div class="handle"><i></i></div>' if handle else '', inner, cap)
def r_running():
    out = ('<span class="c-dim">$ php artisan module:migrate --all</span><br>&nbsp;&nbsp;Running Migration Billing Module<br>&nbsp;&nbsp;Nothing to migrate.<br>'
           '&nbsp;&nbsp;Running Migration Catalog Module<br>&nbsp;&nbsp;Nothing to migrate.<br>&nbsp;&nbsp;Running Migration Checkout Module<br>'
           '&nbsp;&nbsp;2026_11_20_091200_add_tracking_code_to_checkouts_table ........ <span class="c-dim">RUNNING</span>')
    tools = '<span>%s</span>%s<span>%s</span>%s' % (t('b.stop'), kbd('Ctrl .'), t('b.hide'), kbd('Ctrl `'))
    return drawer('php artisan module:migrate --all', '<span class="st" style="color:var(--code-fg)">%s<span>%s</span></span>' % (ic('run', 12), t('m.running')), out, '', tools)
def r_done():
    out = '<span class="c-dim">$ php artisan module:migrate Checkout</span><br>&nbsp;&nbsp;Running Migration Checkout Module<br>&nbsp;&nbsp;2026_11_20_091200_add_tracking_code_to_checkouts_table .............. <span class="c-ok">184.20ms DONE</span>'
    return drawer('php artisan module:migrate Checkout', S('good', t('m.done')), out)
def r_failed(narrow=False):
    out = ('<span class="c-dim">$ php artisan module:migrate Checkout</span><br>&nbsp;&nbsp;Running Migration Checkout Module<br>&nbsp;&nbsp;2026_11_20_091200_add_tracking_code_to_checkouts_table ... <span class="c-err">FAIL</span><br><br>'
           '&nbsp;&nbsp;<span class="c-err">SQLSTATE[42S21]: Column already exists: 1060 Duplicate column name \'tracking_code\'</span>')
    extra = ('<div style="font-family:var(--sans);font-size:12.5px;line-height:1.5;margin-top:10px;padding-top:8px;border-top:1px solid rgba(147,160,176,.35);max-width:86ch">%s</div>'
             '<div style="display:flex;gap:14px;align-items:center;margin-top:8px;font-family:var(--sans);font-size:12px;color:var(--code-fg)"><span style="font-weight:600">%s %s</span><span>%s <span class="mono">--pretend</span></span><span>%s</span></div>') % (
        t('x.run'), t('b.openmig'), kbd('Enter'), t('b.dry'), t('b.copyout'))
    if narrow: return drawer('module:migrate Checkout', S('bad', 'Exit code 1') + '<span class="mono">412 ms</span>', out, extra, '<span>%s</span>%s' % (t('b.hide'), kbd('Ctrl `')))
    return drawer('php artisan module:migrate Checkout', S('bad', 'Exit code 1') + '<span class="mono">412 ms</span>', out, extra)
def r_history():
    H = [('14:05', 'module:migrate Checkout', S('good', '186 ms'), True), ('14:03', 'module:make-model PaymentIntent Billing', S('bad', 'Exit code 1'), False),
         ('13:58', 'tests of Checkout', S('good', '4.2 s'), False), ('13:51', 'module:migrate Checkout --pretend', S('good', '141 ms'), False), ('13:40', 'module:seed Checkout', S('good', '388 ms'), False)]
    rows = ''.join('<div class="hr%s"><span><span class="mono">%s</span>&nbsp; <span class="mono">%s</span></span>%s</div>' % (' on' if on else '', a, b, c) for a, b, c, on in H)
    out = 'Running Migration Checkout Module<br>&nbsp;&nbsp;2026_11_20_091200_add_tracking_code_to_checkouts_table .............. <span class="c-ok">184.20ms DONE</span>'
    tools = '<span>%s</span>%s<span>%s</span><span style="color:var(--code-fg);font-weight:600">%s</span><span>%s</span>%s' % (t('b.again'), kbd('Enter'), t('b.copyout'), t('b.history'), t('b.hide'), kbd('Ctrl `'))
    return drawer('5 runs today in shop-api', '', '<div class="dhist"><div>%s</div><div>%s</div></div>' % (rows, out), '', tools)
def r_collapsed():
    return '<div class="dcol"><span><b>Runner</b><span class="mono">php artisan module:migrate Checkout</span>%s</span><span><span>Show</span>%s</span></div>' % (S('good', t('m.done')), kbd('Ctrl `'))
def r_empty():
    return drawer('', '', '<span class="c-dim" style="font-family:var(--sans);font-size:12px">%s</span>' % t('e.norun'), '', '<span>%s</span>%s' % (t('b.hide'), kbd('Ctrl `')))

def build():
    CUR['page'] = 'screens-write.html'
    b = ('<p>Section 5, part 3 of 4: everything that runs a command. Generate, Actions and the runner drawer. All of it is phase 2: it needs PHP, and several parts need additions to the package, named under each screen.</p>'
         '<p><strong>The rule for every command.</strong> The exact command is on screen before it runs, it can be copied instead of run, the files or rows it touches are named first where Mortise can know them, and the output is shown as the command wrote it.</p>')
    b += '<h2>5.9 Generate</h2>'
    b += screen('S9.1', 'Generator picker', 'All 42 make commands of the package by their artisan names: 41 that create a file in the selected module, and <code>module:make</code>, which creates a module. Model is the selected row.', picker(), '2',
                'The list and each form come from the package.', ['P3'],
                ['<b>Names.</b> Read from <span class="mono">src/Commands/Make</span> of the package on 22 November 2026. Note <span class="mono">module:route-provider</span>, which has no "make-" in its name, and <span class="mono">module:make-component</span>, which creates the component class.',
                 '<b>Groups.</b> The seven groups are Mortise\'s, not the package\'s. A generator the package adds later appears in a last group, "Other", until Mortise places it.',
                 '<b>Keys.</b> 3 or G opens the tab with the field focused. Typing filters by label and by artisan name. Up, Down, Left and Right move; Enter opens the form; Esc clears the filter.',
                 '<b>Without P3.</b> Mortise ships a description of the 42 commands for the package version it was built against, and marks the picker "for nwidart/laravel-modules 13.3" when the app has another version.'])
    b += screen('S9.2', 'The form for module:make-model, with command and file preview', 'Billing is selected, so the module is filled in. Every control is an argument or option of the command: <code>model</code>, <code>module</code>, <code>--fillable</code>, <code>--attributes</code>, the seven files <code>-c -f -m -r -R -p -s</code>, and <code>-a</code> for all seven.', gen_form(), '2',
                'The command is built from the form on every keystroke. The file list comes from a dry run of the command.', ['P2', 'P3'],
                ['<b>"All of these".</b> Checking it checks the seven boxes and the command shows <span class="mono">--all</span>. Clearing one box returns to the separate flags.',
                 '<b>Validation.</b> The name field shows its error under the field as soon as the name cannot be a class name: %s' % t('x.field.name'),
                 '<b>Buttons.</b> The primary button names what it does and counts the files. "Copy command" puts the line on the clipboard and shows "%s"' % t('m.copied'),
                 '<b>Not in this version.</b> "Save as recipe" from the earlier mocks is left out; recipes are not designed in this spec.'])
    b += '<h3 id="S9.3"><span class="sid">S9.3</span>Preview when a file exists already</h3><p class="cap">The workspace body only. The dry run reports that the policy file exists.</p>'
    SCREENS['S9.3'] = (CUR['page'], 'Preview when a file exists already'); AUSED.setdefault('P2', []).append('S9.3')
    b += gen_clash() + ('<div class="notes"><div><b>Phase.</b> 2.</div><div><b>Why not run it anyway.</b> <span class="mono">module:make-model</span> calls <span class="mono">module:make-policy</span> without <span class="mono">--force</span>, so the run would create two files and fail on the third. Mortise says so before the run and changes the command in plain sight.</div>'
                        '<div><b>Assumes, not in the package today.</b> <a class="sid" href="open.html#a-P2">P2</a> Dry run for generators.</div></div>')
    b += screen('S9.4', 'The new module form for module:make', 'Reached from "New module" in the index footer, from the picker, from Ctrl N and from the palette. "All modules" is selected because the target is the app.', module_form(), '2',
                'The options are the command\'s own: <code>--web</code>, <code>--api</code>, <code>--plain</code>, <code>--inertia</code>, <code>--disabled</code>, <code>--force</code>, <code>--author-name</code>, <code>--author-email</code>, <code>--author-vendor</code>.', ['P2'],
                ['<b>Kind.</b> One of three, because the command resolves <span class="mono">--web</span>, <span class="mono">--api</span> and <span class="mono">--plain</span> to a single type. <span class="mono">--inertia</span> is a separate switch in the command, so it is a separate checkbox.',
                 '<b>The file list is an example.</b> It must come from the dry run; the real list depends on the app\'s <span class="mono">config/modules.php</span> and its stubs.',
                 '<b>Replace.</b> Checking "Replace the module if it exists" makes the primary button destructive and asks first: "%s" %s' % (t('c.force.h'), t('c.force.p')),
                 '<b>Name error, in the package\'s words.</b> %s' % t('x.field.module'),
                 '<b>After creation.</b> The new module is selected in the index and its Summary opens. If autoload fails: %s' % t('x.dump')])
    b += screen('S9.5', 'Result: created', 'After Ctrl Enter. The form keeps its values, the preview is replaced by the result, the runner drawer opens with the output, and a toast confirms.', gen_created(), '2',
                'The list of created files is read from the command\'s result.', ['P5'],
                ['<b>New files.</b> The rows carry the well fill for 600 ms, then it fades. Enter opens the first file in the editor.',
                 '<b>Without P5.</b> Mortise reads the lines "Generating file ... DONE" from the output. That works today and breaks if the wording changes.'])
    b += screen('S9.6', 'Result: failed', 'The policy file existed. Two files were created, one was not, and the command ended with exit code 1.', gen_failed(), '2', None, ['P5'],
                ['<b>What happened, then what to do.</b> The alert says which file was left alone. "%s" runs <span class="mono">php artisan module:make-policy PaymentIntentPolicy Billing --force</span> after the confirmation: "%s" %s' % (t('b.overwrite'), t('c.ow.h'), t('c.ow.p')),
                 '<b>No toast for a failure that is on screen.</b> The result box and the drawer already show it. A failed toast is used when the failure happens out of view, for example a run started from the palette on another tab.'])
    b += '<h2>5.10 Actions</h2>'
    b += screen('S10.1', 'Actions for one module: Migrate', 'Six actions on the left, the selected one on the right with its options, its command and its run button. Migrate is selected because a migration is waiting.', actions_screen(), '2',
                'On or off from <span class="mono">modules_statuses.json</span>. Waiting migrations and the last batch need the migration state.', ['P4'],
                ['<b>Keys.</b> 4 opens the tab. J and K move through the actions, Tab goes into the options, Ctrl Enter runs. The Migrate button in the module header runs the same command with no options.',
                 '<b>Check translations</b> runs <span class="mono">module:lang Checkout</span> and shows its output in the runner.'])
    b += '<h3 id="S10.2"><span class="sid">S10.2</span>The other actions: turn off and on, roll back, seed, run tests, and "All modules"</h3><p class="cap">The right-hand panel for each action. The last drawing is the tab when "All modules" is selected.</p>'
    SCREENS['S10.2'] = (CUR['page'], 'The other actions: turn off and on, roll back, seed, run tests')
    for a in ('P4', 'P7'): AUSED.setdefault(a, []).append('S10.2')
    cells = [(d_off(), 'a. Turn off. Asks first only because other modules use Checkout.'), (d_on(), 'b. Turn on, when the module is off.'),
             (d_rollback(), 'c. Roll back. Always asks first.'), (d_seed(), 'd. Seed.'), (d_tests(), 'e. Run tests.')]
    b += '<div class="pair">%s</div>' % ''.join('<div>%s<p class="cellcap">%s</p></div>' % (panel(c, 'surface', '14px 16px'), cap) for c, cap in cells)
    b += actions_all() + '<p class="cellcap">f. Actions for "All modules". The same commands with <code>--all</code>. Turning every module on or off at once is not offered. The closed group at the end holds "Drop all tables and migrate".</p>'
    b += ('<div class="notes"><div><b>Phase.</b> 2.</div><div><b>Tests.</b> The owner decided that the package will provide a command for the tests of one module. No name or option is invented here. <b>Options shown.</b> Only options the commands have. <span class="mono">module:migrate</span>: <span class="mono">--pretend</span>, <span class="mono">--seed</span>, <span class="mono">--database</span> (also <span class="mono">--subpath</span> and <span class="mono">--direction</span>, not drawn). '
          '<span class="mono">module:migrate-rollback</span>: <span class="mono">--pretend</span>, <span class="mono">--database</span>. <span class="mono">module:seed</span>: <span class="mono">--class</span>, <span class="mono">--database</span>.</div>'
          '<div><b>Assumes, not in the package today.</b> <a class="sid" href="open.html#a-P4">P4</a> Migration state as data, for "Waiting" and "Last batch". <a class="sid" href="open.html#a-P7">P7</a> Tests of one module.</div></div>')
    b += '<h3 id="S10.3"><span class="sid">S10.3</span>Confirmation before a destructive action</h3><p class="cap">The dialog over the Actions tab, then the dialog alone in light and dark.</p>'
    SCREENS['S10.3'] = (CUR['page'], 'Confirmation before a destructive action')
    b += actions_screen('<div class="over">%s</div>' % dlg_rollback(), 2) + dlg_pair(dlg_rollback())
    b += spec_table(['Action', 'Command', 'Mortise asks first', 'Why'], [
        ['Turn on', '<code>module:enable</code>', 'No', 'Changes one line of <code>modules_statuses.json</code> and can be undone.'],
        ['Turn off', '<code>module:disable</code>', 'Only when a module that is on uses it', 'Nothing is lost, but the modules that use it can fail.'],
        ['Migrate', '<code>module:migrate</code>', 'Only when the app\'s <code>APP_ENV</code> is production', 'Adds to the schema. In production the package itself asks, so Mortise asks and then passes <code>--force</code>.'],
        ['<strong>Roll back</strong>', '<code>module:migrate-rollback</code>', '<strong>Always</strong>', '<strong>Destructive.</strong> Drops columns and tables with their data.'],
        ['Seed', '<code>module:seed</code>', 'Only in production', 'Adds rows.'],
        ['Run tests', 'the package\'s command for one module', 'No', 'Reads and reports.'],
        ['Check translations', '<code>module:lang</code>', 'No', 'Reads and reports.'],
        ['<strong>Replace an existing file</strong>', 'any generator with <code>--force</code>', '<strong>Always</strong>', '<strong>Destructive.</strong> The file\'s content is lost.'],
        ['<strong>Replace an existing module</strong>', '<code>module:make --force</code>', '<strong>Always</strong>', '<strong>Destructive.</strong> Writes over the module\'s files.'],
        ['<strong>Stop a running migration, rollback or seed</strong>', 'the running process', '<strong>Always</strong>', 'Can leave the database between two states.'],
        ['Remove an app from Mortise', 'none', 'Always', 'Forgets saved credentials. Touches no file of the app.'],
        ['<strong>Reset migrations</strong>', '<code>module:migrate-reset</code>', '<strong>Always, naming the tables</strong>', '<strong>Destructive.</strong> Drops everything the module\'s migrations created. %s' % ref('S10.4')],
        ['<strong>Refresh migrations</strong>', '<code>module:migrate-refresh</code>', '<strong>Always, naming the tables</strong>', '<strong>Destructive.</strong> The module\'s tables come back empty. No dry run.'],
        ['<strong>Drop all tables and migrate</strong>', '<code>module:migrate-fresh --all</code>', '<strong>Always, and the app name must be typed</strong>', '<strong>Destructive.</strong> Every table of the database. No dry run.'],
        ['<strong>Delete module</strong>', '<code>module:delete --force</code>', '<strong>Always, and the module name must be typed</strong>', '<strong>Destructive.</strong> The module\'s folder and everything in it that is not committed.'],
    ])
    b += ('<div class="notes"><div><b>Dialog rules.</b> 440 px wide. The title is the question and names the object. The body says what is lost. Focus starts on Cancel, so Enter never confirms a destructive action; the destructive button is the only filled red control in the app and is reached with Tab. Esc cancels. Where a dry run exists it is offered on the left.</div>'
          '<div><b>The four that lose the most</b> are kept apart from these and have a stronger dialog: %s and %s.</div>' % (ref('S10.4'), ref('S10.5'))
          + '<div><b>Other confirmations, same dialog.</b> "%s" %s &nbsp; "%s" %s &nbsp; "%s" %s &nbsp; "%s" %s (buttons: %s, %s)</div>'
          '<div><b>Assumes, not in the package today.</b> <a class="sid" href="open.html#a-P4">P4</a> to name the migrations in the dialog. Without it the dialog offers the dry run instead of the list.</div></div>') % (
        t('c.off.h'), t('c.off.p'), t('c.prod.h'), t('c.prod.p'), t('c.rm.h'), t('c.rm.p'), t('c.stop.h'), t('c.stop.p'), t('c.stop.k'), t('c.stop.b'))
    AUSED.setdefault('P4', []).append('S10.3')

    b += '<h3 id="S10.4"><span class="sid">S10.4</span>The four destructive actions, kept apart</h3><p class="cap">The owner decided that Mortise offers <code>module:migrate-reset</code>, <code>module:migrate-refresh</code>, <code>module:migrate-fresh</code> and <code>module:delete</code>. They sit in a group of their own at the end of the action list, closed until opened. Here it is open with "Reset migrations" selected.</p>'
    SCREENS['S10.4'] = (CUR['page'], 'The four destructive actions, kept apart')
    b += destructive_screen()
    cells = [(d_refresh(), 'a. Refresh migrations.'), (d_delete(), 'b. Delete module. The command line shows --force because Mortise has already asked.'),
             (d_fresh(), 'c. Drop all tables and migrate. Only under "All modules".'), (d_reset(True), 'd. Any of the four when the app reports production: the run button is disabled and the command can only be copied.')]
    b += '<div class="pair">%s</div>' % ''.join('<div>%s<p class="cellcap">%s</p></div>' % (panel(c, 'surface', '14px 16px'), cap) for c, cap in cells)
    for a in ('P4',): AUSED.setdefault(a, []).append('S10.4')
    b += ('<div class="notes"><div><b>Phase.</b> 2.</div>'
          '<div><b>Kept apart, five ways.</b> 1. A group named "%s" under the everyday actions, closed each time the tab or the module changes; J and K stop at its row and Enter opens it. 2. No button in the module header, no shortcut, and Ctrl Enter only opens the dialog. '
          '3. The palette lists them only when the query contains the command\'s own word (reset, refresh, fresh, delete), and choosing one opens this tab on the action instead of running it. 4. The run button is the red outline; the only filled red button is inside the dialog. 5. The panel names what is lost before the dialog does.</div>'
          '<div><b>Roll back stays with the everyday actions.</b> It undoes one batch and is part of normal work on a migration; it keeps its own confirmation (%s).</div>'
          '<div><b>What each command does, read in its source.</b> <span class="mono">module:migrate-reset</span> rolls back every migration of the module that ran; options <span class="mono">--pretend</span>, <span class="mono">--database</span>, <span class="mono">--subpath</span>, <span class="mono">--direction</span>. '
          '<span class="mono">module:migrate-refresh</span> calls reset, then <span class="mono">module:migrate</span>, then <span class="mono">module:seed</span> with <span class="mono">--seed</span>; options <span class="mono">--database</span>, <span class="mono">--seed</span>; no dry run. '
          '<span class="mono">module:migrate-fresh</span> runs <span class="mono">db:wipe</span> on the whole database, creates the migrations table, runs the app\'s own migrations and then <span class="mono">module:migrate</span> for the modules named; options <span class="mono">--database</span>, <span class="mono">--seed</span>, <span class="mono">--drop-views</span>, <span class="mono">--drop-types</span>; it refuses <span class="mono">--pretend</span>. '
          '<span class="mono">module:delete</span> removes the module\'s line from the statuses file and deletes its folder; it does not touch the database.</div>'
          '<div><b>Decision: dropping all tables is offered only for the whole app.</b> For one module the command still wipes the whole database and then brings back only the app\'s tables and that module\'s, leaving the other modules without tables. On a module\'s tab the group says so: %s</div>'
          '<div><b>Production.</b> When <span class="mono">APP_ENV</span> is production Mortise does not run any of the four. The button is disabled, the reason is on the panel, and the command can be copied: %s</div>'
          '<div><b>If the app prohibits them.</b> The package can refuse these commands (<span class="mono">Module::prohibitDestructiveCommands()</span>). Mortise cannot know before running; it shows the refusal: %s</div>'
          '<div><b>Assumes, not in the package today.</b> <a class="sid" href="open.html#a-P4">P4</a> Migration state as data, for "9 ran" and the list of tables. Without it the panel and the dialog say "every migration of Checkout that ran" and offer the dry run to read the SQL.</div></div>') % (
        t('d.group'), ref('S10.3'), t('d.onefresh'), t('d.prod'), t('d.prohibited'))
    b += '<h3 id="S10.5"><span class="sid">S10.5</span>The strongest confirmation</h3><p class="cap">Over the tab, then each dialog. Reset and refresh name the tables. Dropping all tables and deleting a module also ask for a name to be typed; each is shown before and after typing, in light and in dark.</p>'
    SCREENS['S10.5'] = (CUR['page'], 'The strongest confirmation'); AUSED.setdefault('P4', []).append('S10.5')
    b += destructive_screen('<div class="over" style="padding-top:70px">%s</div>' % dlg_delete(), 2)
    b += '<div class="pair"><div>%s<p class="cellcap">a. Reset migrations. A dry run is offered on the left.</p></div><div>%s<p class="cellcap">b. Refresh migrations.</p></div></div>' % (dlg_stage(dlg_reset(), 'light').replace('class="m fa"', 'class="m fa sw"'), dlg_stage(dlg_refresh(), 'light').replace('class="m fa"', 'class="m fa sw"'))
    b += '<div class="pair">%s%s</div><p class="cellcap">c. Drop all tables and migrate. Left: as it opens, focus on Cancel, the red button disabled. Right: after "shop-api" is typed.</p>' % (dlg_stage(dlg_fresh(), 'light'), dlg_stage(dlg_fresh(True), 'dark'))
    b += '<div class="pair">%s%s</div><p class="cellcap">d. Delete module. Left: as it opens. Right: after "Checkout" is typed.</p>' % (dlg_stage(dlg_delete(), 'light'), dlg_stage(dlg_delete(True), 'dark'))
    b += ('<div class="notes"><div><b>Phase.</b> 2.</div>'
          '<div><b>What every one of the four dialogs has.</b> The title names the action and the object. The body names exactly what is lost: tables by name, the number of files, what is not committed. Focus starts on Cancel. Enter never confirms. Esc cancels. The confirming button repeats the action and is the only filled red control.</div>'
          '<div><b>Decision: two of them need typing.</b> Dropping all tables (the app name) and deleting a module (the module name). These two lose things that nothing in the app can bring back and that reach beyond one module\'s tables. The name must match exactly; until it does the red button is disabled. Reset and refresh lose one module\'s rows and do not need typing.</div>'
          '<div><b>Delete counts what git cannot restore.</b> The line about files that are not committed comes from git and is left out when there are none. Without git the dialog says "Mortise cannot tell whether these files are in git."</div>'
          '<div><b>After the run.</b> %s &nbsp; %s &nbsp; %s &nbsp; %s After a delete the index selects "All modules".</div>'
          '<div><b>Assumes, not in the package today.</b> <a class="sid" href="open.html#a-P4">P4</a> to name the tables and count the migrations.</div></div>') % (t('m.reset'), t('m.refreshed'), t('m.fresh'), t('m.deleted'))
    b += '<h2>5.11 Runner drawer</h2>'
    b += '<h3 id="S11.1"><span class="sid">S11.1</span>Runner drawer: running, finished, failed, history, collapsed, empty</h3><p class="cap">The drawer alone, at the bottom of the workspace. It is dark in both themes, like a terminal; the pair at the end shows the failed state in both themes.</p>'
    SCREENS['S11.1'] = (CUR['page'], 'Runner drawer: running, finished, failed, history')
    b += '%s%s%s%s%s%s' % (
        runner_box(r_running(), 'a. Running. Elapsed time counts up; there is no spinner. Stop asks first for a migration, rollback or seed.'),
        runner_box(r_done(), 'b. Finished.'), runner_box(r_failed(), 'c. Failed. Exit code, the output as written, then one sentence from Mortise and the next steps.'),
        runner_box(r_history(), 'd. History. Up and Down move, Enter runs the selected command again.'),
        runner_box(r_collapsed(), 'e. Collapsed to one line, 28 px.', False), runner_box(r_empty(), 'f. Nothing has run yet.'))
    b += '<div class="pair">%s%s</div><p class="cellcap">g. The failed state in the light theme and in the dark theme, at the narrowest drawer width.</p>' % tuple('<div class="m fa" data-theme="%s"><div class="win sm" style="border-radius:0"><div class="work"><div class="handle"><i></i></div>%s</div></div></div>' % (th, r_failed(True)) for th in ('light', 'dark'))
    b += ('<div class="notes"><div><b>Phase.</b> 2.</div>'
          '<div><b>Behaviour.</b> Opens when a command starts and stays until hidden. Ctrl ` shows and hides it. The top edge is a handle: drag, or focus it and use Up and Down, between 120 px and half the workspace height. Default 200 px.</div>'
          '<div><b>Output.</b> As the process wrote it, with its ANSI colours mapped to runner text, dim, success and error. Mortise adds the first line (the command), the status on the right, and for a failure one sentence of its own under a rule. It does not rewrite a database or PHP error; it quotes it.</div>'
          '<div><b>One command at a time per app.</b> While one runs, run buttons for that app are disabled with the tooltip "module:migrate --all is running".</div>'
          '<div><b>History.</b> The last 50 runs per app, kept in Mortise\'s own data folder with their output up to 1 MB each.</div>'
          '<div><b>If the command cannot start.</b> %s</div><div><b>Assumes.</b> Nothing the packages lack today.</div></div>') % t('x.spawn')
    return page('screens-write.html', 'Screens: write', 'Mortise design specification, section 5, part 3. Generate, Actions, runner drawer.', b)
