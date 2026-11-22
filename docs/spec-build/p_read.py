from lib import *
from data import *
from p_start import idx60

ENVSEG = seg(['local', 'staging', 'production'], 0)
def rt_box(body, title=None, right=ENVSEG, style=''):
    return '<div class="box" style="%s"><h5><span>%s</span><span>%s</span></h5>%s</div>' % (style, title or t('runtime.title'), right, body)
RT_OK = (kv('Status', '<span style="display:flex;gap:8px;align-items:center">%s<span class="dim">checked 8 s ago</span></span>' % S('up', t('word.up')))
         + kv('<span class="mono">ping</span>', S('up', 'Up')) + kv('<span class="mono">readinessState</span>', S('up', 'Up'))
         + kv('<span class="mono">diskSpace</span>', '<span style="display:flex;gap:8px;align-items:center"><span class="dim">198.6 GB free of 460.4 GB</span>%s</span>' % S('up', 'Up'))
         + kv('<span class="mono">db</span>', '<span style="display:flex;gap:8px;align-items:center"><span class="dim">mysql</span>%s</span>' % S('up', 'Up'))
         + kv('<span class="mono">cache</span>', '<span style="display:flex;gap:8px;align-items:center"><span class="dim">redis</span>%s</span>' % S('up', 'Up')))
RT_INFO = (kv('Environment', 'local, Laravel 13.2.0') + kv('Git', '<span><span class="mono">feature/tracking-code</span> at <span class="mono">49689af</span></span>')
           + kv('PHP', '8.4.27, fpm-fcgi') + kv('Memory', '38 MB now, 41 MB peak, limit 512 MB'))

def stats_all(n=5):
    cells = [(t('stat.on'), '<span class="big">7 of 7</span>'), (t('stat.lines'), '<span class="big">41,212</span>'), (t('stat.cov'), '<span class="big">71%</span>'),
             (t('stat.findings'), S('bad', '2 problems') + S('warn', '1 warning')), (t('stat.runtime'), S('up', 'Up') + '<span class="dim" style="font-size:11.5px">on local</span>')]
    if n == 4: cells = [cells[0], cells[2], cells[3], cells[4]]
    return '<div class="stats" style="%s">%s</div>' % ('grid-template-columns:repeat(5,minmax(0,1fr))' if n == 5 else '', ''.join('<div class="stat"><div class="k">%s</div><div class="v">%s</div></div>' % c for c in cells))
ALERT_ALL = alert('<b>Checkout</b> has 2 problems. It and Shipping use each other (6 files), and 3 of its files read <span class="mono">Billing\\Models\\Invoice</span> directly.')
COV = box('Coverage by module', '307 tests', ''.join(bar(BYCODE[c][1], BYCODE[c][5], '%d%%' % BYCODE[c][5], BYCODE[c][5] < 50) for c in ['USR', 'BIL', 'CAT', 'INV', 'SHP', 'CHK', 'NTF']))

def summary_all_cmp(th):
    body = stats_all(4) + ALERT_ALL + COV
    return win(tbar() + shell(rail(), index('ALL', long=False, foot=idx_foot('Problems first')), work(t('all'), 0, body)), sw=False, cmp=True, theme=th)

TASKS = [('<span class="mono">php artisan checkout:expire-carts</span>', '<span class="mono">*/5 * * * *</span>', '14:10 UTC', stamp('CHK')),
         ('<span class="mono">php artisan billing:send-invoices</span>', '<span class="mono">0 6 * * *</span>', 'tomorrow 06:00 UTC', stamp('BIL')),
         ('<span class="mono">Modules\\Inventory\\Jobs\\SyncStock</span>', '<span class="mono">0 * * * *</span>', '15:00 UTC', stamp('INV')),
         ('Poll shipment tracking', 'every 10 s', '<span class="dim">no fixed time</span>', stamp('SHP')),
         ('<span class="mono">php artisan queue:prune-batches</span>', '<span class="mono">0 3 * * *</span>', 'tomorrow 03:00 UTC', '<span class="dim">none</span>')]
def tasks_box(rows=TASKS, title='Scheduled tasks', right='5 tasks, read from local', cols='minmax(0,1.6fr) 110px 150px 60px'):
    return '<div class="box flush"><h5><span>%s</span><span>%s</span></h5>%s</div>' % (title, right, table(cols, ['Task', 'Schedule', 'Next run', 'Module'], rows))

def summary_all():
    routes = box('Routes by module', '106 routes, read from local', ''.join(bar(n, v * 100 // 22, v) for n, v in
              [('Catalog', 22), ('Checkout', 18), ('Users', 16), ('Billing', 14), ('Shipping', 11), ('Inventory', 9), ('Notifications', 4)])
              + '<div class="kv" style="border-top:1px solid var(--line);margin-top:4px"><span>Not in a module</span><span class="mono">12</span></div>')
    bymod = '<div class="box flush"><h5><span>By module</span><span>sorted by name</span></h5>%s</div>' % table(
        '34px minmax(0,1.4fr) 50px 60px 50px 70px 44px 60px 110px minmax(0,1fr)',
        ['', 'Module', 'Files>', 'Lines>', 'Tests>', 'Coverage>', 'Uses>', 'Used by>', 'Commits, 30 days>', 'Findings'],
        [[stamp(m[0]), '<b style="font-weight:600">%s</b>' % m[1], m[2], m[3], m[4], '%d%%' % m[5], m[6], m[7], m[8], fmark(m[0], False)] for m in MODS])
    body = (stats_all() + '<div class="alert">%s<div style="flex:1">%s</div>%s</div>' % (S('bad'), ALERT_ALL.split('<div>', 1)[1].rsplit('</div></div>', 1)[0], btn(t('b.showall'), 'sm', None, 'F'))
            + '<div class="cols"><div class="stack">%s%s</div>%s</div>' % (COV, routes, rt_box(RT_OK + RT_INFO)) + tasks_box() + bymod)
    return win(tbar() + shell(rail(), index('ALL'), work(t('all'), 0, body)))

FILES = box('Files by kind', '84 files, 6,412 lines', ''.join(bar(n, p, v) for n, p, v in [('Tests', 100, 31), ('Views', 39, 12), ('Migrations', 29, 9), ('Requests', 23, 7),
            ('Controllers', 19, 6), ('Events', 16, 5), ('Listeners', 16, 5), ('Models', 13, 4), ('Jobs', 10, 3), ('Policies', 6, 2)]))
DEPS = box('Dependencies', '4 out, 2 in', kv('Uses Catalog ' + stamp('CAT'), '<span class="mono dim">14 refs</span>') + kv('Uses Billing ' + stamp('BIL'), S('bad', '6 refs, 3 to internals'))
           + kv('Uses Users ' + stamp('USR'), '<span class="mono dim">3 refs</span>') + kv('Uses Shipping ' + stamp('SHP'), S('bad', '2 refs, circular'))
           + kv('Used by Shipping, Notifications', '<span class="mono dim">4, 2 refs</span>'))
def modstate(rt=True):
    return box('Module state', S('good', t('word.on')), kv('Routes', '18 (12 API, 6 web)')
               + kv('Migrations', '<span style="display:flex;gap:8px;align-items:center">9 ran %s</span>' % S('warn', t('word.waiting')))
               + kv('Git churn', '14 commits in 30 days, last 2 days ago'))
ALERT_CHK = alert('<b>2 problems.</b> Checkout and <b>Shipping</b> use each other (2 files here, 4 files there), and 3 files read <span class="mono">Modules\\Billing\\Models\\Invoice</span> directly.')
def drawer_done():
    return ('<div class="drawer"><div class="dbar"><span><b>Runner</b><span class="mono">php artisan module:migrate Checkout</span></span><span>%s<span>%s</span>%s</span></div>'
            '<div>Running Migration Checkout Module<br>&nbsp;&nbsp;2026_11_20_091200_add_tracking_code_to_checkouts_table .............. <span class="c-ok">184.20ms DONE</span></div></div>') % (S('good', t('m.done')), t('b.hide'), kbd('Ctrl `'))

ROUTES = [('GET', 'api/checkout/carts/{cart}', 'checkout.carts.show', 'CartController@show', 'api, auth:sanctum'),
          ('POST', 'api/checkout/carts', 'checkout.carts.store', 'CartController@store', 'api, auth:sanctum'),
          ('POST', 'api/checkout/carts/{cart}/items', 'checkout.carts.items.store', 'CartItemController@store', 'api, auth:sanctum'),
          ('DELETE', 'api/checkout/carts/{cart}/items/{item}', 'checkout.carts.items.destroy', 'CartItemController@destroy', 'api, auth:sanctum'),
          ('POST', 'api/checkout/orders', 'checkout.orders.store', 'OrderController@store', 'api, auth:sanctum'),
          ('GET', 'api/checkout/orders/{order}', 'checkout.orders.show', 'OrderController@show', 'api, auth:sanctum'),
          ('GET', 'checkout', 'checkout.show', 'CheckoutPageController@show', 'web, auth'),
          ('POST', 'checkout/confirm', 'checkout.confirm', 'CheckoutPageController@confirm', 'web, auth')]
def summary_one():
    rt = rt_box(kv('Status of Checkout', '<span style="display:flex;gap:8px;align-items:center">%s<span class="dim">checked 8 s ago</span></span>' % S('up', 'Up'))
                + kv('<span class="mono">paymentGateway</span>', '<span style="display:flex;gap:8px;align-items:center"><span class="dim">answered in 212 ms</span>%s</span>' % S('up', 'Up'))
                + kv('<span class="mono">orderQueue</span>', '<span style="display:flex;gap:8px;align-items:center"><span class="dim">3 jobs waiting</span>%s</span>' % S('up', 'Up')))
    tasks = '<div class="box flush"><h5><span>Scheduled tasks</span><span>1 task</span></h5>%s</div>' % table('minmax(0,1fr) 84px 76px', ['Task', 'Schedule', 'Next run'], [TASKS[0][:3]])
    routes = '<div class="box flush"><h5><span>Routes</span><span>18 routes, 8 shown, read from local</span></h5>%s</div>' % table(
        '60px minmax(0,1.5fr) minmax(0,1.1fr) minmax(0,1.1fr) 130px', ['Method', 'URI', 'Name', 'Action', 'Middleware'],
        [['<span class="mono">%s</span>' % a, '<span class="mono">%s</span>' % b, '<span class="mono dim">%s</span>' % c, '<span class="mono">%s</span>' % d, '<span class="dim">%s</span>' % e] for a, b, c, d, e in ROUTES])
    body = ALERT_CHK + '<div class="cols">%s<div class="stack">%s%s</div></div><div class="cols">%s%s</div>%s' % (FILES, DEPS, modstate(), rt, tasks, routes)
    return win(tbar() + shell(rail(), index('CHK', mrt=True), work('Checkout', 0, body, MOD_ACTS, drawer_done(), 'CHK', 'Modules/Checkout')))

# ------------------------------------------------------------ boundary graphs
def gnode(x, y, w, h, codev, name, sub='', cls='', subcls='', peg=False):
    s = '<rect class="gnode %s" x="%s" y="%s" width="%s" height="%s"/>' % (cls, x + .5, y + .5, w, h)
    if peg: s += '<rect x="%s" y="%s" width="5" height="5" style="fill:var(--ink)"/>' % (x + 5, y + h / 2 - 2)
    ox = x + (14 if peg else 8)
    s += '<rect x="%s" y="%s" width="32" height="16" rx="2" style="fill:none;stroke:var(--line-2)"/><text class="gst" x="%s" y="%s" text-anchor="middle">%s</text>' % (ox + .5, y + (h - 16) / 2 + .5, ox + 16.5, y + h / 2 + 4, codev)
    if sub:
        s += '<text class="gt" x="%s" y="%s">%s</text><text class="gt2 %s" x="%s" y="%s">%s</text>' % (ox + 40, y + h / 2 - 2, name, subcls, ox + 40, y + h / 2 + 12, sub)
    else:
        s += '<text class="gt" x="%s" y="%s">%s</text>' % (ox + 40, y + h / 2 + 4, name)
    return s
def glink(d, bad=False, sel=False):
    return '<path class="glink %s %s" d="%s"/>' % ('bad' if bad else '', 'sel' if sel else '', d)
def garrow(x, y, dirn='r', bad=False):
    p = {'r': 'M%s %sl-7-4v8z', 'l': 'M%s %sl7-4v8z', 'd': 'M%s %sl-4-7h8z', 'u': 'M%s %sl-4 7h8z'}[dirn] % (x, y)
    return '<path class="garrow %s" d="%s"/>' % ('bad' if bad else '', p)
def gbad(x, y):  # the problem mark sits on the link, so a problem link is not shown by colour alone
    return '<g transform="translate(%s %s)"><rect class="gbadmark" width="11" height="11"/><path class="gx" d="M2.8 2.8l5.4 5.4M8.2 2.8 2.8 8.2"/></g>' % (x - 5.5, y - 5.5)

def ego_graph(sel='BIL'):
    s = '<svg viewBox="0 0 860 268" width="100%" role="img" aria-label="Checkout in the middle, the modules that use it on the left, the modules it uses on the right" style="display:block;max-width:860px">'
    s += '<text class="gcap" x="20" y="16">Used by, 2</text><text class="gcap" x="350" y="16">Selected</text><text class="gcap" x="640" y="16">Uses, 4</text>'
    s += gnode(20, 76, 200, 40, 'SHP', 'Shipping', '4 refs, circular', '', 'bad') + gnode(20, 152, 200, 40, 'NTF', 'Notifications', '2 refs')
    s += gnode(350, 108, 180, 52, 'CHK', 'Checkout', '84 files', 'me')
    R = [('CAT', 'Catalog', '14 refs', 30, ''), ('BIL', 'Billing', '6 refs, 3 to internals', 88, 'bad'), ('USR', 'Users', '3 refs', 146, ''), ('SHP', 'Shipping', '2 refs, circular', 204, 'bad')]
    for c, n, sub, y, k in R:
        s += gnode(640, y, 200, 40, c, n, sub, 'me' if c == sel else '', k, c == sel)
    s += glink('M220 96H286V124H350', True) + garrow(350, 124, 'r', True) + gbad(286, 110)
    s += glink('M220 172H286V144H350') + garrow(350, 144, 'r')
    s += glink('M530 116H574V50H640') + garrow(640, 50)
    s += glink('M530 128H590V108H640', True) + garrow(640, 108, 'r', True) + gbad(590, 118)
    s += glink('M530 140H590V166H640') + garrow(640, 166)
    s += glink('M530 152H574V224H640', True) + garrow(640, 224, 'r', True) + gbad(574, 196)
    return s + '</svg>'

def app_graph():
    s = '<svg viewBox="0 0 560 316" width="100%" role="img" aria-label="The 7 modules of shop-api and the 9 links between them" style="display:block;max-width:620px">'
    N = {'NTF': (10, 92, 'Notifications'), 'INV': (10, 248, 'Inventory'), 'SHP': (184, 30, 'Shipping'), 'CHK': (184, 138, 'Checkout'),
         'BIL': (358, 84, 'Billing'), 'CAT': (358, 248, 'Catalog'), 'USR': (358, 166, 'Users')}
    w = {'NTF': 150, 'INV': 150, 'SHP': 140, 'CHK': 140, 'BIL': 130, 'CAT': 130, 'USR': 130}
    for c, (x, y, n) in N.items(): s += gnode(x, y, w[c], 34, c, n)
    # Notifications uses Checkout and Users
    s += glink('M160 114H172V150H184') + garrow(184, 150)
    s += glink('M85 92V14H520V183H488') + garrow(488, 183, 'l')
    # Checkout and Shipping use each other
    s += glink('M224 138V64', True) + garrow(224, 64, 'u', True) + glink('M284 64V138', True) + garrow(284, 138, 'd', True) + gbad(254, 101)
    # Checkout uses Billing (3 files read internals), Users, Catalog
    s += glink('M324 146H340V101H358', True) + garrow(358, 101, 'r', True) + gbad(340, 124)
    s += glink('M324 158H340V178H358') + garrow(358, 178)
    s += glink('M324 166H332V258H358') + garrow(358, 258)
    # Billing uses Users
    s += glink('M423 118V166') + garrow(423, 166, 'd')
    # Inventory uses Catalog
    s += glink('M160 270H358') + garrow(358, 270)
    s += '<text class="gt2" x="10" y="308">Arrow points at the module that is used. A square with a cross marks a link that breaks a rule.</text>'
    return s + '</svg>'

LINK_FILES = [(S('bad'), 'Modules/Checkout/app/Services/InvoiceSync.php', '12', 'Modules\\Billing\\Models\\Invoice'),
              (S('bad'), 'Modules/Checkout/app/Listeners/CreateInvoiceOnPaid.php', '8', 'Modules\\Billing\\Models\\Invoice'),
              (S('bad'), 'Modules/Checkout/app/Http/Controllers/OrderController.php', '9', 'Modules\\Billing\\Models\\Invoice'),
              (S('good'), 'Modules/Checkout/app/Listeners/MarkOrderPaid.php', '7', 'Modules\\Billing\\Events\\InvoicePaid'),
              (S('good'), 'Modules/Checkout/app/Listeners/SendReceipt.php', '6', 'Modules\\Billing\\Events\\InvoicePaid'),
              (S('good'), 'Modules/Checkout/app/Services/PaymentService.php', '11', 'Modules\\Billing\\Contracts\\PaymentGateway')]
def files_table(rows, sel=0):
    return table('16px minmax(0,1.5fr) 40px minmax(0,1fr)', ['', 'File', 'Line>', 'Reads'], [[a, '<span class="mono">%s</span>' % b, '<span class="mono">%s</span>' % c, '<span class="mono">%s</span>' % d] for a, b, c, d in rows], sel)

def boundary_one():
    graph = '<div style="display:flex;gap:10px;align-items:center">%s<span class="dim" style="font-size:11.5px">Switch with %s</span></div><div class="box" style="padding:10px 12px 8px">%s</div>' % (seg([t('ps.seg.links'), t('ps.seg.surface')], 0), kbd('P'), ego_graph())
    files = ('<div class="box flush"><h5><span>Checkout uses Billing</span>%s</h5>%s'
             '<div style="display:flex;justify-content:space-between;gap:10px;align-items:center;padding:8px 12px;border-top:1px solid var(--line-2);background:var(--panel)">'
             '<span class="dim" style="font-size:11.5px">Rule: other modules use only a module\'s public surface. Billing has none yet, so the folders <span class="mono">Contracts</span> and <span class="mono">Events</span> count.</span>'
             '<span class="btns">%s%s</span></div></div>') % (S('bad', '3 of 6 files read internals'), files_table(LINK_FILES), btn(t('b.editor'), 'sm pri', None, 'Enter'), btn('Copy path', 'sm'))
    return win(tbar() + shell(rail(), index('CHK'), work('Checkout', 1, graph + files, MOD_ACTS, '', 'CHK', 'Modules/Checkout')))

def circular_panel():
    rows = [(S('bad'), 'Modules/Checkout/app/Services/ShippingQuote.php', '7', 'Modules\\Shipping\\Contracts\\RateCalculator'),
            (S('bad'), 'Modules/Checkout/app/Listeners/RequestShipment.php', '6', 'Modules\\Shipping\\Contracts\\ShipmentRequester')]
    rows2 = [(S('bad'), 'Modules/Shipping/app/Listeners/CreateShipmentForOrder.php', '8', 'Modules\\Checkout\\Events\\OrderPaid'),
             (S('bad'), 'Modules/Shipping/app/Services/LabelPrinter.php', '10', 'Modules\\Checkout\\Contracts\\OrderLookup'),
             (S('bad'), 'Modules/Shipping/app/Http/Controllers/TrackingController.php', '9', 'Modules\\Checkout\\Contracts\\OrderLookup'),
             (S('bad'), 'Modules/Shipping/app/Jobs/SyncTracking.php', '11', 'Modules\\Checkout\\Events\\OrderShipped')]
    inner = ('<div class="box flush"><h5><span>Checkout and Shipping use each other</span>%s</h5>'
             '<div class="grp" style="padding-left:14px">Checkout uses Shipping<span>2 files</span></div>%s<div class="grp" style="padding-left:14px;border-top:1px solid var(--line-2)">Shipping uses Checkout<span>4 files</span></div>%s'
             '<div style="padding:8px 12px;border-top:1px solid var(--line-2);background:var(--panel);font-size:11.5px" class="dim">%s Every file in a circle is marked: removing either direction ends it.</div></div>') % (
        S('bad', 'Circular, 6 files'), files_table(rows, None), files_table(rows2, None), t('m.reported'))
    return panel(inner, 'surface', '14px 16px')

def rules_box():
    return box('Rules for shop-api', 'kept in <span class="mono">mortise.json</span> in the app folder',
               chk('No circular use', '', True)
               + '<div class="chk" style="align-items:flex-start;padding-top:4px">%s<span style="flex:1">%s<span class="hint2" style="display:block;margin:2px 0 4px">Accepted for 0 of 7 modules. For a module without one, these folders count:</span>%s</span></div>' % (sq(True), t('ps.rule'), inp('Contracts, Events', 'mono'))
               + chk('Fail CI when a rule breaks', '', False, 'is-dis') + '<div class="hint2" style="margin:0 0 2px 22px">Needs the command line tool, which does not exist yet.</div>')
def boundary_all():
    probs = ('<div class="box flush"><h5><span>Links that break a rule</span>%s</h5>'
             '<div class="lrow on"><span>Checkout and Shipping use each other</span><span class="mono dim">6 files</span></div>'
             '<div class="lrow"><span>Checkout reads Billing internals</span><span class="mono dim">3 files</span></div></div>') % S('bad', '2 problems')
    left = '<div class="box" style="padding:10px 12px 8px"><h5 style="border:0;margin:0;padding:0 0 6px"><span>9 links between 7 modules</span><span>%s</span></h5>%s</div>' % (seg(['All links', 'Problems only'], 0), app_graph())
    body = '<div class="cols" style="grid-template-columns:minmax(0,3fr) minmax(0,2fr)">%s<div class="stack">%s%s</div></div>' % (left, probs, rules_box())
    return win(tbar() + shell(rail(), index('ALL'), work(t('all'), 1, body)))


BIL_ACTS = btn('Run tests', 'sm', 'test') + btn('Open in editor', 'sm')
SURF_TOP = lambda right='': '<div style="display:flex;gap:10px;align-items:center">%s<span class="dim" style="font-size:11.5px">Switch with %s</span><span style="margin-left:auto;display:flex;gap:8px;align-items:center;font-size:11.5px">%s</span></div>' % (seg([t('ps.seg.links'), t('ps.seg.surface')], 1), kbd('P'), right)
SG = 'grid-template-columns:20px minmax(0,1.5fr) minmax(0,1fr) minmax(0,1fr)'
def srow(mark, entry, why, used, on=False, cls=''):
    return '<div class="trow %s %s" style="%s"><span>%s</span><span class="mono">%s</span><span class="dim">%s</span><span>%s</span></div>' % ('on' if on else '', cls, SG, mark, entry, why, used)
def sgrp(n, c): return '<div class="grp" style="padding-left:14px;border-bottom:1px solid var(--line)">%s<span>%s</span></div>' % (n, c)
NOTYET = '<span class="dim">not used outside yet</span>'
def surface_rows(mode):
    m = (lambda on=True: sq(on)) if mode == 'proposal' else (lambda on=True: S('good') if on else '')
    out = '<div class="tbl"><div class="thead" style="%s"><span></span><span>Entry</span><span>Why it is here</span><span>Used today by</span></div>' % SG
    out += sgrp('Contracts', '2') + srow(m(), 'Modules\\Billing\\Contracts\\PaymentGateway', t('ps.why.contract'), 'Checkout, 1 file', mode == 'proposal') + srow(m(), 'Modules\\Billing\\Contracts\\InvoiceNumbering', t('ps.why.contract'), NOTYET)
    out += sgrp('Events', '3') + srow(m(), 'Modules\\Billing\\Events\\InvoicePaid', t('ps.why.event'), 'Checkout, 2 files') + srow(m(), 'Modules\\Billing\\Events\\InvoiceIssued', t('ps.why.event'), NOTYET) + srow(m(), 'Modules\\Billing\\Events\\PaymentFailed', t('ps.why.event'), NOTYET)
    if mode == 'proposal':
        out += sgrp('Other classes that other modules use', '1') + srow(sq(False), 'Modules\\Billing\\Models\\Invoice', 'in Models, ' + t('ps.why.used'), S('warn', 'Checkout, 3 files'))
        out += '<div style="padding:5px 12px 7px 46px;font-size:11.5px;line-height:1.4;color:var(--ink-2);border-bottom:1px solid var(--line)">%s</div>' % t('ps.left')
    out += sgrp('Routes, read from local', '14') + srow(m(), 'api/billing/*', t('ps.why.route'), '9 routes, HTTP clients') + srow(m(), 'billing/*', t('ps.why.route'), '5 routes, HTTP clients')
    out += sgrp('Console commands', '2') + srow(m(), 'billing:send-invoices', t('ps.why.cmd'), 'the scheduler, daily at 06:00') + srow(m(), 'billing:retry-failed-payments', t('ps.why.cmd'), NOTYET)
    return out + '</div>'
def surf_work(body): return work('Billing', 1, body, BIL_ACTS, '', 'BIL', 'Modules/Billing')
def surf_none():
    inner = SURF_TOP() + '<div class="empty" style="flex:1;min-height:250px;padding:20px 4px">%s<h4>%s</h4><p style="max-width:62ch">%s</p><div class="btns">%s</div></div>' % (ic('boundary', 28), t('ps.none.h'), t('ps.none.p'), btn(t('ps.scan'), 'pri', None, 'Ctrl Enter'))
    return win(tbar() + shell(rail(), index('BIL'), surf_work(inner), '--shell-h:430px'))
def surf_scanning():
    steps = [('Classes that other modules use', S('good', '3 classes in 6 files')), ('Contracts', S('good', '2 interfaces')), ('Events', S('good', '3 events')),
             ('Routes', '<span class="busy">%sreading from local, 1.2 s</span>' % ic('run', 12)), ('Console commands', '<span class="dim">waiting</span>')]
    inner = SURF_TOP() + ('<div style="max-width:560px"><div style="display:flex;justify-content:space-between;font-size:11.5px;margin-bottom:4px"><span>%s</span><span class="mono">64%%</span></div><span class="prog"><i style="width:64%%"></i></span></div>' % t('ps.scanning'))
    inner += box('Scanning Billing', 'nothing is written', ''.join(kv(a, b) for a, b in steps), '', 'max-width:560px') + '<div class="btns">%s</div>' % btn(t('ps.stop'), '', 'stop', 'Esc')
    return panel('<div class="stack">%s</div>' % inner, 'surface', '14px 16px')
def surf_proposal():
    foot = ('<div style="display:flex;justify-content:space-between;gap:10px;align-items:center;padding:8px 12px;border-top:1px solid var(--line-2);background:var(--panel)">'
            '<div class="inp" style="flex:1;max-width:380px"><span class="ph">%s</span>%s</div><span class="btns">%s%s</span></div>') % (t('ps.add'), kbd('A'), btn(t('ps.again'), 'sm'), btn(t('ps.accept'), 'sm pri', None, 'Ctrl Enter'))
    inner = SURF_TOP('<span class="dim">scanned just now</span>') + '<div class="box flush"><h5><span>Proposed public surface of Billing</span><span>%s</span></h5>%s%s</div>' % (t('ps.ready'), surface_rows('proposal'), foot)
    return win(tbar() + shell(rail(), index('BIL'), surf_work(inner)))
def surf_accepted():
    foot = ('<div style="display:flex;justify-content:space-between;gap:10px;align-items:center;padding:8px 12px;border-top:1px solid var(--line-2);background:var(--panel)">'
            '<span class="dim" style="font-size:11.5px">Left out: <span class="mono">Modules\\Billing\\Models\\Invoice</span>. The 3 files of Checkout that read it stay a finding.</span><span class="btns">%s%s</span></div>') % (btn(t('ps.edit'), 'sm'), btn(t('ps.again'), 'sm'))
    inner = SURF_TOP() + '<div class="box flush"><h5><span>Public surface of Billing</span>%s</h5>%s%s</div>' % (S('good', t('ps.state')), surface_rows('accepted'), foot)
    return panel('<div class="stack">%s</div>' % inner, 'surface', '14px 16px')
def surf_diff():
    rows = ('<div class="tbl"><div class="thead" style="%s"><span></span><span>Entry</span><span>Difference</span><span>Used today by</span></div>' % SG
            + srow(sq(True), 'Modules\\Billing\\Events\\RefundIssued', S('warn', t('ps.diff.new')) + ' <span class="dim">add it</span>', NOTYET, True)
            + srow(sq(True), 'billing:retry-failed-payments', S('warn', t('ps.diff.gone')) + ' <span class="dim">remove it</span>', '<span class="dim">nothing</span>')
            + '<div style="padding:7px 12px 8px 14px;font-size:11.5px;color:var(--ink-2)">20 entries are the same as accepted and are not listed. <span class="mono">Modules\\Billing\\Models\\Invoice</span> is still left out.</div></div>')
    foot = ('<div style="display:flex;justify-content:flex-end;gap:8px;padding:8px 12px;border-top:1px solid var(--line-2);background:var(--panel)">%s%s</div>') % (btn(t('ps.diff.keep'), 'sm'), btn(t('ps.diff.accept'), 'sm pri', None, 'Ctrl Enter'))
    inner = SURF_TOP('<span class="dim">scanned just now</span>') + alert(t('ps.diff') + ' Nothing changes until you accept.', 'warn') + '<div class="box flush"><h5><span>Differences from the accepted surface of Billing</span><span>accepted on 22 November 2026</span></h5>%s%s</div>' % (rows, foot)
    return panel('<div class="stack">%s</div>' % inner, 'surface', '14px 16px')

LINKS60 = [('CHK', 'Checkout', 'SHP', 'Shipping', 2, 2, S('bad', 'Circular')), ('CHK', 'Checkout', 'BIL', 'Billing', 6, 3, S('bad', '3 files read internals')),
           ('PYM', 'Payments', 'INS', 'Invoices', 9, 4, S('bad', '4 files read internals')), ('PYM', 'Payments', 'ORD', 'Orders', 5, 3, S('bad', 'Circular')),
           ('SRC', 'Search', 'PRD', 'Products', 21, 6, S('bad', '2 files read internals')), ('ORD', 'Orders', 'PRD', 'Products', 48, 12, S('good', 'Good')),
           ('CRT', 'Cart', 'PRD', 'Products', 31, 9, S('good', 'Good')), ('CHK', 'Checkout', 'CRT', 'Cart', 27, 8, S('good', 'Good')),
           ('ORD', 'Orders', 'CST', 'Customers', 22, 7, S('good', 'Good')), ('INS', 'Invoices', 'ORD', 'Orders', 19, 5, S('good', 'Good')),
           ('SHP', 'Shipping', 'ORD', 'Orders', 17, 6, S('good', 'Good')), ('RFN', 'Refunds', 'PYM', 'Payments', 15, 4, S('good', 'Good')),
           ('PRC', 'Pricing', 'CRR', 'Currencies', 14, 3, S('good', 'Good')), ('RVW', 'Reviews', 'PRD', 'Products', 12, 4, S('good', 'Good'))]
def boundary_many():
    tb = table('34px minmax(0,1fr) 34px minmax(0,1fr) 60px 60px minmax(0,1.3fr)', ['', 'From', '', 'Uses', 'Refs>', 'Files>', 'State'],
               [[stamp(a), '<b style="font-weight:600">%s</b>' % b, stamp(c), '<b style="font-weight:600">%s</b>' % d, e, f, g] for a, b, c, d, e, f, g in LINKS60], 0)
    top = ('<div style="display:flex;gap:10px;align-items:center"><div class="filter" style="margin:0;width:240px">%s<span>Filter links</span>%s</div>%s'
           '<span class="dim" style="margin-left:auto;font-size:11.5px">214 links, 5 problems, 14 shown</span></div>') % (ic('search'), kbd('/'), seg(['All links', 'Problems only'], 0))
    note = alert('platform has 60 modules. Above 12 modules the graph of the whole app gives way to this table. Select one module for its own graph.', 'note')
    idx = '<div class="idx" style="position:relative">%s</div>' % idx60('ALL')
    body = top + '<div class="box flush">%s</div>' % tb + note
    apps = APPS10
    return win(tbar('platform on Sail') + shell(rail(apps, 'PL'), idx, work(t('all'), 1, body)))

def findings():
    def grp(n, c): return '<div class="grp" style="padding-left:14px;border-bottom:1px solid var(--line)">%s<span>%s</span></div>' % (n, c)
    g = 'grid-template-columns:20px 44px minmax(0,1fr) 190px 110px'
    def r(mark, codev, text, check, go, on=False):
        return '<div class="trow %s" style="%s;min-height:34px"><span>%s</span><span>%s</span><span class="wrap" style="line-height:1.35;padding:4px 0">%s</span><span class="dim">%s</span><span class="dim r">%s</span></div>' % ('on' if on else '', g, mark, stamp(codev), text, check, go)
    tb = ('<div class="tbl"><div class="thead" style="%s"><span></span><span>Module</span><span>Finding</span><span>Check</span><span class="r">Enter opens</span></div>' % g
          + grp('Problems', '2') + r(S('bad'), 'CHK', t('f.circular'), 'Rule: no circular use', 'Boundary', True) + r(S('bad'), 'CHK', t('f.internals'), 'Rule: Contracts and Events only', 'Boundary')
          + grp('Warnings', '1') + r(S('warn'), 'NTF', t('f.coverage'), 'Coverage limit', 'Summary')
          + grp('To do, not counted as findings', '1') + r('', 'CHK', t('f.waiting'), 'Migrations', 'Actions') + '</div>')
    passed = box('Checks without a finding', '4 of 7 checks', ''.join(kv(a, S('good', b)) for a, b in [
        ('Every <span class="mono">module.json</span> is valid JSON', '7 modules'), ('Declared namespace matches the folder', '7 modules'),
        ('No entry in <span class="mono">modules_statuses.json</span> without a module', '7 entries'), ('Translations are complete <span class="mono dim">module:lang</span>', '5 modules with lang files')]))
    body = '<div class="box flush">%s</div>%s' % (tb, passed)
    return win(tbar() + shell(rail(), index('ALL'), work(t('findings.title'), 0, body, '<span class="dim" style="font-size:11.5px">Back to Summary</span>' + kbd('Esc'), '', None, None, False, 'All modules /')))

def rt_state(kind):
    if kind == 'nopkg': b = '<div style="padding:6px 0 4px;line-height:1.5">%s</div>%s<div class="btns" style="margin-top:8px">%s</div>' % (t('x.rt.nopkg'), cmd('composer', 'require', 'nwidart/laravel-actuator'), btn(t('b.copycmd'), 'sm'))
    elif kind == 'noenv': b = '<div style="padding:6px 0 8px;line-height:1.5">%s</div><div class="btns">%s</div>' % (t('x.rt.noenv'), btn(t('b.addenv'), 'sm pri'))
    elif kind == 'reach': b = kv('Status', S('unk', 'No answer')) + '<div style="padding:6px 0 8px;line-height:1.5">%s</div><div class="btns">%s%s</div>' % (t('x.rt.reach'), btn(t('b.retry'), 'sm pri'), btn(t('b.settings'), 'sm'))
    elif kind == 'expose': b = kv('Status', S('up', 'Up')) + '<div style="padding:6px 0 6px;line-height:1.5">%s</div>%s<div class="btns" style="margin-top:8px">%s%s</div>' % (
        t('x.rt.expose'), code("'exposure' =&gt; [\n    'include' =&gt; ['health', 'info', 'mappings', 'scheduledtasks'],\n],"), btn(t('b.copyline'), 'sm'), btn(t('b.retry'), 'sm'))
    elif kind == 'auth': b = kv('Status', S('unk', 'Not authorized')) + '<div style="padding:6px 0 8px;line-height:1.5">%s</div><div class="btns">%s</div>' % (t('x.rt.auth'), btn(t('b.settings'), 'sm pri'))
    elif kind == 'down': b = (kv('Status', S('down', t('word.down'))) + kv('<span class="mono">db</span>', S('down', 'Down')) + '<div class="code" style="margin:6px 0">%s</div>' % t('x.rt.down').replace('db is down: ', '')
                              + kv('<span class="mono">ping, readinessState, diskSpace, cache</span>', S('up', '4 up')))
    elif kind == 'oos': b = kv('Status', S('oos', t('word.oos'))) + kv('<span class="mono">readinessState</span>', S('oos', 'Out of service')) + '<div style="padding:6px 0 4px;line-height:1.5" class="dim">%s</div>' % t('x.rt.oos') + kv('4 other components', S('up', '4 up'))
    elif kind == 'hidden': b = kv('Status', S('up', 'Up')) + '<div style="padding:6px 0 4px;line-height:1.5" class="dim">%s</div>' % t('x.rt.hidden')
    else: b = kv('Status', S('unk', 'No answer')) + '<div style="padding:6px 0 4px;line-height:1.5">%s</div>' % t('x.rt.500')
    return '<div class="m fa sw" data-theme="light"><div style="background:var(--surface);padding:12px;border:1px solid var(--line-2)">%s</div></div>' % rt_box(b, None, 'local')

def metrics_empty(kind):
    if kind == 'none':
        inner = '<div class="empty" style="flex:1;min-height:300px">%s<h4>%s</h4><p>%s</p><div class="btns">%s</div></div>' % (ic('metrics', 28), t('e.metrics.h'), t('e.metrics.p'), btn(t('b.connect'), 'pri'))
    else:
        inner = '<div class="empty" style="flex:1;min-height:300px">%s<h4>Metrics are not exposed on local</h4><p>%s</p><div class="btns">%s%s</div></div>' % (ic('metrics', 28), t('x.mx.expose'), btn(t('b.retry'), 'pri'), btn(t('b.settings')))
    idx = index('ALL', head=idx_head(rt=None)) if kind == 'none' else index('ALL')
    return win(tbar() + shell(rail(), idx, work(t('all'), 4, inner).replace('<div class="wbody">', '<div class="wbody" style="padding:0">'), '--shell-h:420px'))

MX = [('CAT', 'Catalog', 366, '84 ms', '0.1%', 4, '31%', 'M0 9 8 7 16 8 24 5 32 6 40 4 48 5 56 3'), ('CHK', 'Checkout', 187, '612 ms', '2.4%', 22, '58%', 'M0 6 8 7 16 5 24 9 32 4 40 6 48 3 56 5'),
      ('USR', 'Users', 133, '61 ms', '0.0%', 1, '22%', 'M0 8 8 8 16 7 24 7 32 6 40 7 48 6 56 6'), ('BIL', 'Billing', 80, '240 ms', '0.3%', 15, '44%', 'M0 10 8 9 16 9 24 8 32 9 40 7 48 8 56 8'),
      ('SHP', 'Shipping', 41, '130 ms', '0.2%', 9, '37%', 'M0 11 8 10 16 11 24 10 32 10 40 9 48 10 56 10'), ('INV', 'Inventory', 26, '95 ms', '0.0%', 3, '28%', 'M0 11 8 11 16 10 24 11 32 11 40 10 48 11 56 11'),
      ('NTF', 'Notifications', 9, '48 ms', '0.0%', 31, '12%', 'M0 12 8 12 16 12 24 11 32 12 40 12 48 12 56 12')]
def metrics_data():
    pts = [(0, 96), (40, 88), (80, 92), (120, 74), (160, 80), (200, 62), (240, 70), (280, 58), (320, 64), (360, 44), (400, 52), (440, 40), (480, 48), (520, 36), (560, 42), (600, 30), (640, 38), (680, 34), (720, 26), (760, 32)]
    chart = ('<svg class="chart" viewBox="0 0 820 132" width="100%%" role="img" aria-label="Requests per minute, all modules, last hour" style="display:block">'
             '<path class="ax" d="M40 10.5H810M40 46.5H810M40 82.5H810M40 118.5H810"/>'
             '<text x="34" y="14" text-anchor="end">1,200</text><text x="34" y="50" text-anchor="end">800</text><text x="34" y="86" text-anchor="end">400</text><text x="34" y="122" text-anchor="end">0</text>'
             '<path class="ln" d="M%s"/><text x="40" y="131">13:10</text><text x="810" y="131" text-anchor="end">14:10</text></svg>') % ' '.join('%s %s' % (x + 44, y) for x, y in pts)
    top = ('<div style="display:flex;gap:10px;align-items:center">%s%s<span class="dim" style="margin-left:auto;font-size:11.5px">read every 15 s, last read 8 s ago</span></div>') % (ENVSEG, seg(['15 min', '1 hour', '24 hours'], 1))
    tb = table('34px minmax(0,1.2fr) 90px 80px 70px 70px 100px 70px', ['', 'Module', 'Requests/min>', '95% under>', 'Errors>', 'Jobs/min>', 'Database time>', 'Last hour'],
               [[stamp(c), '<b style="font-weight:600">%s</b>' % n, r, p, e, j, d, '<svg class="spark" width="58" height="14" viewBox="0 0 58 14"><path d="%s"/></svg>' % sp] for c, n, r, p, e, j, d, sp in MX])
    body = top + box('Requests per minute, all modules', '842 now', chart) + '<div class="box flush">%s</div>' % tb
    return win(tbar() + shell(rail(), index('ALL'), work(t('all'), 4, body)))

def later_tab():
    inner = '<div class="empty" style="flex:1;min-height:300px">%s<h4>%s</h4><p>%s</p></div>' % (ic('generate', 28), t('e.later.h'), t('e.later.p'))
    return win(tbar() + shell(rail(), index('BIL', foot=idx_foot(t('sort.problems'), False)), work('Billing', 2, inner, btn(t('b.editor'), 'sm'), '', 'BIL', 'Modules/Billing').replace('<div class="wbody">', '<div class="wbody" style="padding:0">'), '--shell-h:420px'))

def build():
    CUR['page'] = 'screens-read.html'
    b = '<p>Section 5, part 2 of 4: everything Mortise reads and shows. Summary, Boundary, Findings, runtime status and Metrics. None of these screens writes to the app, except the rules file in %s.</p>' % ref('S5.3')
    b += ('<p><strong>Two names that are kept apart.</strong> <em>Findings</em> are what Mortise works out from the files: problems, warnings, good, drawn as solid marks. '
          '<em>Runtime</em> is what the running app reports through the actuator: Up, Down, Out of service, Unknown, drawn as stroke marks. The word "health" is not used in the interface for either.</p>')
    b += '<h2>5.4 Summary</h2>'
    b += screen('S4.1', 'Summary for "All modules", the home screen', 'Light and dark side by side, compact, so the pair fits. The full drawing follows.', both(summary_all_cmp), '1',
                'Files, git, <span class="mono">clover.xml</span>. The Runtime figure needs a connected environment.')
    b += screen('S4.2', 'Summary for "All modules", full', 'What a developer sees after choosing shop-api. The workspace scrolls; the stats, the findings line, coverage and runtime fit the first screen.', summary_all(), '1. The runtime box, routes and scheduled tasks need a connected actuator environment and are otherwise replaced by one of the states in %s.' % ref('S6.5'),
                'Stats, coverage and "By module" from files, git and <span class="mono">clover.xml</span>. Runtime status and components from the actuator <span class="mono">health</span> endpoint; the four lines under them from <span class="mono">info</span>. Routes from <span class="mono">mappings</span>, counted for a module when the route action starts with that module\'s namespace; closures and routes of other packages count as "Not in a module". Scheduled tasks from <span class="mono">scheduledtasks</span>.',
                ['P10'], ['<b>Placement decision.</b> Runtime status, its components and the info lines sit in one box on the app Summary. Routes and scheduled tasks sit below it. No tab and no surface was added.',
                          '<b>Polling.</b> <span class="mono">health</span> every 15 s while the window is visible. <span class="mono">info</span>, <span class="mono">mappings</span> and <span class="mono">scheduledtasks</span> when the app is opened, every 5 minutes, and on Ctrl R. Mortise never requests <span class="mono">env</span>.',
                          '<b>P10 is needed only for the Module column of scheduled tasks.</b> Today a task can be attributed only when its target is a class name (the third row). Commands and described tasks would show "none".',
                          '<b>Keys.</b> F opens the findings list. J and K move through "By module" when it has focus; Enter selects that module in the index.'])
    b += screen('S4.3', 'Summary for one module: Checkout', 'Checkout selected, runner drawer open after a migration.', summary_one(), '1 for the first four boxes. The Migrate and Run tests buttons and the drawer are phase 2.',
                'Files by kind from the folder a file is in. Dependencies from <span class="mono">use</span> statements and fully qualified names, read with a PHP parser. Routes from <span class="mono">mappings</span>. Migrations "ran" and "waiting" need PHP.',
                ['P4', 'P9', 'P10'], ['<b>Runtime per module.</b> The index rows and the runtime box show the status the module\'s own indicators report. The indicator names here are examples. The stroke mark never replaces the finding mark; a module can be Up with 2 problems.',
                                       '<b>Without P9.</b> The runtime box of a module is not drawn and the rows carry one mark. <b>Without P4.</b> The Migrations line reads "9 files" and the Migrate button has no count.'])
    b += '<h2>5.5 Boundary</h2>'
    b += screen('S5.1', 'Boundary for one module: graph and the files of a link', 'Checkout in the middle, what uses it on the left, what it uses on the right. The link to Billing is selected, so its files are listed below.', boundary_one(), '1',
                'Links from <span class="mono">use</span> statements and fully qualified class names in each module\'s PHP files.', ['P6'],
                ['<b>Two views.</b> The control at the top switches between the links of the module and its public surface (%s).' % ref('S5.5'), '<b>Graph.</b> Straight connectors only. A link that breaks a rule is 2 px, in the problem colour, and carries the problem mark on the line. Left and Right move between columns, Up and Down within one; the selected node has the peg.',
                 '<b>More than 8 neighbours on one side.</b> The 7 with the most references are drawn and an eighth node reads "12 more"; Enter on it turns the side into the table of %s, filtered to this module.' % ref('S5.4'),
                 '<b>File list.</b> Files that break the rule first. Enter opens the file at the line in the editor. The list is a table: J and K move, the header sorts.'])
    b += '<h3 id="S5.2"><span class="sid">S5.2</span>The files of a circular link</h3><p class="cap">The same panel when the link between Checkout and Shipping is selected.</p>'
    SCREENS['S5.2'] = (CUR['page'], 'The files of a circular link')
    b += circular_panel() + '<div class="notes"><div><b>Phase.</b> 1.</div><div><b>Assumes.</b> Nothing the packages lack today.</div></div>'
    b += screen('S5.3', 'Boundary for "All modules": the app graph and the rules', 'Shop-api has 7 modules, so the whole graph is drawn.', boundary_all(), '1. "Fail CI" is phase 3.',
                'The same links as %s, for every module.' % ref('S5.1'), ['P6', 'M1'],
                ['<b>Layout.</b> Modules are placed in columns from left to right: a module sits to the left of what it uses. Modules in a circle share a column.',
                 '<b>Rules.</b> Two rules in this version, kept in <span class="mono">mortise.json</span> in the app folder (owner decision) so a team shares them. It is the only file Mortise writes without a command, and the first change asks before creating it. No secret is ever written to it.',
                 '<b>The second rule.</b> A module with an accepted public surface (%s) is checked against it. A module without one is checked against the folder list, which is free text.' % ref('S5.7'),
                 '<b>Keys.</b> J and K move through "Links that break a rule"; Enter selects the first module of the link and opens its Boundary tab on that link.'])
    b += screen('S5.4', 'Boundary for an app with many modules', 'The app "platform" has 60 modules. Decision: above 12 modules the graph of the whole app is replaced by a table of links.', boundary_many(), '1', None, ['P6'],
                ['<b>Why 12.</b> At the smallest workspace (692 px wide) a column layout holds 4 columns of 3 to 4 nodes with room for connectors. Beyond that connectors cross and labels shrink under 11 px.',
                 '<b>Why a table.</b> It sorts, filters and scrolls, each row has a full name, and it is cheap to draw. A matrix was considered and set aside: 60 by 60 cells need hover to read.',
                 '<b>One module is not affected.</b> Its own graph depends on its neighbours, not on the size of the app.'])

    b += '<h2>5.5 Boundary, second view: the public surface of a module</h2>'
    b += ('<p>The owner decided that the package will define a module\'s public surface, and that Mortise offers a scan, on demand, that proposes one. The surface is everything of a module that something outside it may use: PHP classes for other modules, routes for HTTP clients, console commands for the command line and the scheduler. '
          'It lives in the Boundary tab of one module as a second view beside the links; no surface was added. The drawings use Billing, because its scan settles the open finding: Checkout reads <span class="mono">Billing\\Models\\Invoice</span>.</p>')
    b += screen('S5.5', 'Public surface: not scanned yet', 'Billing selected, Boundary tab, the second view. A scan never starts by itself.', surf_none(), '1 for the scan and the proposal. Accepting needs the package (P6).',
                None, ['P6'], ['<b>How the scan is started.</b> The button here, Ctrl Enter in this view, or the palette: "Scan the public surface of Billing". One module at a time; there is no scan of the whole app.',
                               '<b>Keys.</b> P switches between Links and Public surface. Left and Right do the same when the control has focus.'])
    b += '<h3 id="S5.6"><span class="sid">S5.6</span>Public surface: scanning</h3><p class="cap">The workspace body. A bar for the files, a line per kind of thing looked for.</p>'
    SCREENS['S5.6'] = (CUR['page'], 'Public surface: scanning')
    b += surf_scanning() + ('<div class="notes"><div><b>Phase.</b> 1.</div><div><b>What the scan reads, and from where.</b> Classes that other modules use: the links Mortise already has. Contracts: interfaces declared in the module. Events: classes in the module\'s events folder. '
                            'Console commands: command classes in the module, with the name in their signature. Routes: from the actuator\'s <span class="mono">mappings</span> when an environment is connected, otherwise from Laravel\'s route list once PHP is set (phase 2). If neither is there: %s</div>'
                            '<div><b>Stopping.</b> Esc or the button. A scan writes nothing, so stopping needs no confirmation and leaves the view as it was.</div><div><b>Assumes.</b> Nothing the packages lack today.</div></div>') % t('ps.noroutes')
    b += screen('S5.7', 'Public surface: proposal ready', 'What the scan found, grouped by kind, each with the reason it is there and who uses it today. A checked entry is in the proposal.', surf_proposal(), '1 for the proposal. The Accept button needs P6.',
                'The same sources as the scan.', ['P6'],
                ['<b>What is proposed and what is not.</b> Contracts, events, named routes and console commands are checked. A class that another module uses but that is not a contract or an event is listed unchecked, with the wedge: the scan shows it, the developer decides.',
                 '<b>Accept, remove, add.</b> Space on a row includes it or leaves it out. A adds an entry by hand: a class name, a route pattern or a command name, checked against the module\'s files as it is typed. Ctrl Enter accepts what is checked. "%s" runs the scan again and keeps the developer\'s changes where the entry still exists.' % t('ps.again'),
                 '<b>What happens to the result.</b> The accepted list becomes the declared public surface of Billing, stored where the package decides a module declares it. That place and its form are not decided, so this spec fixes neither. From then on the rule "%s" is checked for Billing against this list instead of the folder list. Findings are worked out again at once.' % t('ps.rule'),
                 '<b>In this example.</b> Leaving <span class="mono">Models\\Invoice</span> out keeps the 3 file finding on Checkout. Checking it would make those reads allowed and Checkout would have 1 problem.',
                 '<b>Until P6 exists.</b> The scan and the proposal work; Accept is disabled with the reason "The package cannot store a public surface yet", and the proposal can be copied as text.',
                 '<b>After accepting.</b> A toast: "%s"' % t('ps.accepted')])
    b += '<h3 id="S5.8"><span class="sid">S5.8</span>Public surface: accepted</h3><p class="cap">The view after accepting, and every time it is opened later.</p>'
    SCREENS['S5.8'] = (CUR['page'], 'Public surface: accepted'); AUSED.setdefault('P6', []).append('S5.8')
    b += surf_accepted() + ('<div class="notes"><div><b>Phase.</b> Needs P6.</div><div><b>Read only until "Change".</b> Change brings back the checkboxes of %s with the accepted state. The Links view of every module now marks a read of Billing as allowed or not by this list.</div>'
                            '<div><b>Assumes, not in the package today.</b> <a class="sid" href="open.html#a-P6">P6</a> A module\'s public surface, and where it is declared.</div></div>') % ref('S5.7')
    b += '<h3 id="S5.9"><span class="sid">S5.9</span>Public surface: a later scan finds a difference</h3><p class="cap">The code changed since the surface was accepted. Only the differences are listed.</p>'
    SCREENS['S5.9'] = (CUR['page'], 'Public surface: a later scan finds a difference'); AUSED.setdefault('P6', []).append('S5.9')
    b += surf_diff() + ('<div class="notes"><div><b>Phase.</b> Needs P6.</div><div><b>Two kinds of difference.</b> New in the code: something the scan would propose that is not in the accepted list. No longer in the code: an accepted entry whose class, route or command is gone. Each is a checked proposal; unchecking it keeps the accepted state for that entry.</div>'
                        '<div><b>Not a finding.</b> A difference is shown only after a scan the developer asked for. It does not raise a mark in the index.</div>'
                        '<div><b>No difference.</b> The view of %s stays, with a toast: "The scan matches the accepted surface."</div>'
                        '<div><b>Assumes, not in the package today.</b> <a class="sid" href="open.html#a-P6">P6</a> A module\'s public surface, and where it is declared.</div></div>') % ref('S5.8')
    b += '<h2>5.6 Findings and runtime status</h2>'
    b += screen('S6.4', 'Findings: the list for an app', 'Called "Health" in the brief. In the interface it is "Findings", so it cannot be mistaken for the runtime status. It opens from the Findings stat, from F, and from the palette, and Esc returns to the Summary.', findings(), '1 for the rules and file checks. The translation check and the waiting migration are phase 2.',
                'Rules from Boundary; checks on <span class="mono">module.json</span>, namespaces and <span class="mono">modules_statuses.json</span>; coverage limit per app, 50% unless changed.', ['P4', 'P8'],
                ['<b>Not a new surface.</b> The list is a second view of the Summary tab of "All modules". For one module the same findings appear in the alert at the top of its Summary.',
                 '<b>To do is not a finding.</b> A waiting migration is work in progress, so it does not change a state mark or a count.',
                 '<b>Other finding sentences.</b> %s &nbsp; %s &nbsp; %s &nbsp; %s' % (t('f.json'), t('f.namespace'), t('f.stale'), t('f.lang')),
                 '<b>Empty.</b> %s' % t('e.nofindings')])
    b += '<h3 id="S6.5"><span class="sid">S6.5</span>Runtime box: the states of the actuator connection</h3><p class="cap">The box from the app Summary in each state. Top row: nothing to connect to. Middle row: connected, something is missing. Bottom row: the app answers and is not up.</p>'
    SCREENS['S6.5'] = (CUR['page'], 'Runtime box: the states of the actuator connection')
    caps = {'nopkg': 'a. Package not installed.', 'noenv': 'b. Installed, no environment yet.', 'reach': 'c. App not reachable. "No answer" is Mortise\'s word, not a status from the app.',
            'expose': 'd. Endpoints not exposed. health answers, the others are missing from the discovery links.', 'auth': 'e. Not authorized: 401 or 403.', 'hidden': 'f. Details hidden by show_details.',
            'down': 'g. Down, with the failing component and its detail as the app wrote it.', 'oos': 'h. Out of service.', 'err': 'i. The actuator answers 500.'}
    for trio in (('nopkg', 'noenv', 'reach'), ('expose', 'auth', 'hidden'), ('down', 'oos', 'err')):
        b += '<div class="trio">%s</div>' % ''.join('<div>%s<p class="cellcap">%s</p></div>' % (rt_state(k), caps[k]) for k in trio)
    b += ('<div class="notes"><div><b>Phase.</b> 1, optional.</div><div><b>Data.</b> The package is found in <span class="mono">composer.lock</span>. What is exposed is read from the links at the actuator base path. '
          'A 503 from <span class="mono">health</span> is an answer, not a failed request: it carries Down or Out of service.</div>'
          '<div><b>Rule.</b> Mortise quotes the detail the app gives and adds nothing to it. Where the app hides details, Mortise says which setting hides them.</div>'
          '<div><b>Same states elsewhere.</b> The index header line reads "No answer on local" in c and e, and is absent in a and b. Routes and scheduled tasks show d in place of their content.</div>'
          '<div><b>Assumes.</b> Nothing the packages lack today.</div></div>')
    b += '<h2>5.7 Metrics</h2>'
    b += screen('S7.1', 'Metrics, not connected', 'No environment is connected for shop-api. The button opens Settings on the app\'s environments.', metrics_empty('none'), '3', None, ['A2'],
                ['<b>Same connection.</b> Metrics use the actuator environment of the app: same URL, same credentials, same environment switch as the runtime box. There is no separate Prometheus address.'])
    b += screen('S7.2', 'Metrics, endpoint not exposed', 'An environment is connected and answers, but the metrics endpoint is not among its links.', metrics_empty('expose'), '3', None, ['A2'],
                ['The exact name to add to <span class="mono">exposure.include</span> is not known until the endpoint exists; the sentence is written so that only the name has to be filled in.'])
    b += screen('S7.3', 'Metrics, connected with data', 'All modules. One chart for the selected row, a table for the rest. No colour: there are no limits in this version, so nothing here is a state.', metrics_data(), '3',
                'Requests, response time, errors, jobs and database time per module, read from the actuator every 15 s and kept in memory for the chosen range.', ['A1', 'A2'],
                ['<b>One module.</b> With a module selected, the chart shows that module and the table lists its routes by the same five figures.',
                 '<b>Chart.</b> Straight segments, 1.5 px ink, hairline grid. J and K move through the table and the chart follows the selected row.'])
    b += '<h2>5.8 A tab from a later phase</h2>'
    b += screen('S7.4', 'A tab whose phase is not built yet', 'Decision: the five tabs are always shown, in the same order with the same keys. A tab that is not built explains itself. Buttons and palette entries of a later phase are hidden instead.', later_tab(), 'A phase 1 build, Generate tab. Actions and Metrics use the same block.',
                None, (), ['<b>Why shown.</b> Keys 1 to 5 keep their meaning across releases, and the same block is needed for good in two other cases: %s &nbsp; %s' % (t('e.nophp.p'), t('e.oldpkg.p')),
                           '<b>Why actions are hidden.</b> A header with three disabled buttons on every module is noise. In a phase 1 build the module header has only "Open in editor", the index footer has no "New module", and the palette lists no generators or actions.',
                           '<b>Disabled, not hidden.</b> In a build that has the phase, a control the app cannot run (PHP not found, package too old) is drawn disabled with the reason in a tooltip: "%s"' % t('tip.nophp')])
    return page('screens-read.html', 'Screens: read', 'Mortise design specification, section 5, part 2. Summary, Boundary, Findings, runtime status, Metrics.', b)
