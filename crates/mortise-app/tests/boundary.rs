//! The Boundary tab: links, the files of a link, circles, the rules and the table of links.

mod common;

use common::{Desk, shop_api, wait_until};
use eframe::egui::accesskit::Toggled;
use eframe::egui::{Key, Modifiers, Rect, Role, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use mortise_app::app::MortiseApp;
use mortise_app::app::workspace::Tab;
use mortise_app::boundary::strings;
use mortise_app::index::view::Selection;
use mortise_app::strings as crate_strings;
use mortise_app::ui::theme::size;
use mortise_core::{Rules, RulesSource};

fn rows(harness: &Harness<'static, MortiseApp>) -> Vec<String> {
    harness
        .query_all_by_role(Role::Row)
        .filter_map(|row| row.accesskit_node().label())
        .collect()
}

#[test]
fn the_boundary_of_a_module_shows_its_neighbours_and_the_first_broken_link() {
    let (_desk, mut harness) = shop_api();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    harness.key_press(Key::Num2);
    harness.run();
    assert_eq!(harness.state().tab(), Tab::Boundary);

    harness.get_by_label("Used by, 2");
    harness.get_by_label("Selected");
    harness.get_by_label("Uses, 4");

    // The link to Billing breaks a rule, so its files are listed: internals first.
    harness.get_by_role_and_label(Role::Heading, "Checkout uses Billing");
    harness.get_by_label("3 of 6 files read internals");
    assert_eq!(
        rows(&harness),
        [
            "Modules/Checkout/app/Http/Controllers/OrderController.php",
            "Modules/Checkout/app/Listeners/CreateInvoiceOnPaid.php",
            "Modules/Checkout/app/Services/InvoiceSync.php",
            "Modules/Checkout/app/Listeners/MarkOrderPaid.php",
            "Modules/Checkout/app/Listeners/SendReceipt.php",
            "Modules/Checkout/app/Services/PaymentService.php",
        ]
    );
    harness
        .get_by_label_contains("Billing has none yet, so the folders Contracts and Events count.");
}

#[test]
fn a_circle_lists_the_files_of_both_directions() {
    let (_desk, mut harness) = shop_api();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    harness.key_press(Key::Num2);
    harness.run();
    // Shipping is on both sides of Checkout. Either node shows the circle.
    harness
        .get_all_by_role_and_label(Role::ListBoxOption, "Shipping")
        .next()
        .unwrap()
        .click();
    harness.run();

    harness.get_by_role_and_label(Role::Heading, "Checkout and Shipping use each other");
    harness.get_by_label("Circular, 6 files");
    harness.get_by_label("Checkout uses Shipping");
    harness.get_by_label("2 files");
    harness.get_by_label("Shipping uses Checkout");
    harness.get_by_label("4 files");
    assert_eq!(rows(&harness).len(), 6);
    harness.get_by_label_contains("Reported on Checkout, the side with fewer files to change.");
}

#[test]
fn a_link_one_way_in_a_circle_of_three_is_shown_as_part_of_the_circle() {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    // Users uses Shipping, so Checkout, Shipping and Users make one circle. It reads a
    // class outside the public surface of Shipping.
    let services = root.join("Modules/Users/app/Services");
    std::fs::create_dir_all(&services).unwrap();
    std::fs::write(
        services.join("ShippingRates.php"),
        "<?php\n\nnamespace Modules\\Users\\Services;\n\nuse Modules\\Shipping\\Services\\LabelPrinter;\n\n\
         class ShippingRates\n{\n    public function __construct(private LabelPrinter $printer) {}\n}\n",
    )
    .unwrap();
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();
    harness.key_press(Key::Num2);
    harness.run();

    // The circle lands on Users, which carries it, and on the module Users uses.
    harness
        .get_by_role_and_label(
            Role::Row,
            "Checkout, Shipping and Users use each other in a circle",
        )
        .click();
    harness.run();
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Users".into())
    );
    harness.get_by_role_and_label(Role::Heading, "Users uses Shipping");
    harness.get_by_label("Circular, 1 file");
    harness.get_by_label_contains("Reported on Users, the module of the circle");
    assert!(
        harness
            .query_by_label_contains("Rule: other modules use only")
            .is_none()
    );

    // With "No circular use" off, no circle is reported: the public surface rule is.
    harness
        .get_by_role_and_label(Role::ListBoxOption, "All modules")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(Role::CheckBox, "No circular use")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Create mortise.json")
        .click();
    wait_until(&mut harness, "the circle rule is off", |app| {
        app.app().is_some_and(|app| !app.rules.no_circular_use)
    });
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Users")
        .click();
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "Users uses Shipping");
    harness.get_by_label("1 of 1 file read internals");
    harness
        .get_by_label_contains("Rule: other modules use only a module's public surface. Shipping");
    assert!(harness.query_by_label_contains("Circular").is_none());
    assert!(harness.query_by_label_contains("Reported on").is_none());

    // Two modules that use each other: both directions, under the same rule. The index
    // comes before the graph, which also has a node for Checkout.
    harness
        .get_all_by_role_and_label(Role::ListBoxOption, "Checkout")
        .next()
        .unwrap()
        .click();
    harness.run();
    harness
        .get_all_by_role_and_label(Role::ListBoxOption, "Shipping")
        .last()
        .unwrap()
        .click();
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "Checkout and Shipping use each other");
    harness.get_by_label("6 files");
    harness.get_by_label_contains("Checkout and Shipping have none yet");
    assert!(harness.query_by_label_contains("Reported on").is_none());
}

#[test]
fn copy_path_copies_the_selected_file_and_says_so() {
    let (_desk, mut harness) = shop_api();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    harness.key_press(Key::Num2);
    harness.run();
    harness
        .get_by_role_and_label(
            Role::Row,
            "Modules/Checkout/app/Services/PaymentService.php",
        )
        .click();
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Copy path")
        .click();
    harness.run_steps(3);
    harness.get_by_label("Copied the path.");
}

#[test]
fn the_boundary_of_the_app_lists_the_links_that_break_a_rule() {
    let (_desk, mut harness) = shop_api();
    harness.key_press(Key::Num2);
    harness.run();

    harness.get_by_role_and_label(Role::Heading, "8 links between 7 modules");
    harness.get_by_role_and_label(Role::Heading, "Links that break a rule");
    harness.get_by_label("2 problems");
    assert_eq!(
        rows(&harness),
        [
            "Checkout and Shipping use each other",
            "Checkout reads Billing internals"
        ]
    );
    harness.get_by_role_and_label(Role::Heading, "Rules for shop-api");
    harness.get_by_label(
        "Accepted for 1 of 7 modules. For a module without one, these folders count:",
    );
    // Without the key, no module has a list and the third rule is off.
    harness.get_by_label(strings::NO_LIST_YET);
    let switch = harness.get_by_role_and_label(Role::CheckBox, crate_strings::RULE_ALLOWED_USE);
    assert_eq!(switch.accesskit_node().toggled(), Some(Toggled::False));

    // A row lands on the module's Boundary, on that link.
    harness
        .get_by_role_and_label(Role::Row, "Checkout and Shipping use each other")
        .click();
    harness.run();
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Checkout".into())
    );
    assert_eq!(harness.state().tab(), Tab::Boundary);
    harness.get_by_role_and_label(Role::Heading, "Checkout and Shipping use each other");
}

#[test]
fn a_node_of_the_app_graph_selects_its_module() {
    let (_desk, mut harness) = shop_api();
    harness.key_press(Key::Num2);
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Inventory")
        .click();
    harness.run();
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Inventory".into())
    );
    harness.get_by_role_and_label(Role::Heading, "Inventory uses Catalog");
}

#[test]
fn the_first_change_of_a_rule_asks_before_mortise_json_is_created() {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    let crm = desk.app("crm");
    desk.knows(&[(&root, "SA"), (&crm, "CR")]);
    let mut harness = desk.start();
    harness.key_press(Key::Num2);
    harness.run();
    let rules_file = root.join("mortise.json");

    // Cancel, by Esc: nothing is written.
    harness
        .get_by_role_and_label(Role::CheckBox, "No circular use")
        .click();
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "Create mortise.json in shop-api?");
    // The dialog holds the keys: Ctrl O does not ask for another app under it.
    desk.picks.borrow_mut().push_back(crm);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    harness.run();
    assert_eq!(desk.picks.borrow().len(), 1);
    harness.key_press(Key::Escape);
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Create mortise.json in shop-api?")
            .is_none()
    );
    assert!(!rules_file.exists());

    // Confirm: the file is written and the links are checked again.
    harness
        .get_by_role_and_label(Role::CheckBox, "No circular use")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Create mortise.json")
        .click();
    wait_until(&mut harness, "the rules are read from the file", |app| {
        app.app()
            .is_some_and(|app| app.rules_source == RulesSource::File)
    });
    let (rules, _) = Rules::load(&root);
    assert!(!rules.no_circular_use);
    assert_eq!(rows(&harness), ["Checkout reads Billing internals"]);
    harness.get_by_label("1 problem");

    // The file exists now: the next change is written without asking.
    harness
        .get_by_role_and_label(
            Role::CheckBox,
            "Other modules use only a module's public surface",
        )
        .click();
    wait_until(&mut harness, "no rule is on", |app| {
        app.app().is_some_and(|app| !app.rules.public_surface_only)
    });
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Create mortise.json in shop-api?")
            .is_none()
    );
    harness.get_by_label("No link breaks a rule.");

    // "Fail CI" is kept beside the rules, for `mortise check`.
    harness
        .get_by_role_and_label(Role::CheckBox, "Fail CI when a rule breaks")
        .click();
    wait_until(&mut harness, "CI does not fail", |app| {
        app.app().is_some_and(|app| !app.rules.fail_ci)
    });
    assert!(!Rules::load(&root).0.fail_ci);
}

#[test]
fn the_folders_of_the_second_rule_can_be_changed() {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    // A coverage block that cannot be read is ignored on its own: the rules stay.
    std::fs::write(
        root.join("mortise.json"),
        r#"{"rules": {"public-surface-only": {"folders": ["Contracts"]}}, "coverage": {"limit": 101}}"#,
    )
    .unwrap();
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();
    harness.key_press(Key::Num2);
    harness.run();

    harness.get_by_label(
        "The coverage block of mortise.json is ignored: coverage limit 101 is over 100. \
         Coverage comes from Settings until it is corrected.",
    );
    let field = harness.get_by_role_and_label(Role::TextInput, "Folders that count as public");
    field.focus();
    field.type_text(", Events, Models");
    harness.run();
    // Leaving the field applies the change.
    harness
        .get_by_role_and_label(Role::CheckBox, "No circular use")
        .focus();
    wait_until(&mut harness, "Models is public", |app| {
        app.app()
            .is_some_and(|app| app.rules.public_folders == ["Contracts", "Events", "Models"])
    });
    assert_eq!(rows(&harness), ["Checkout and Shipping use each other"]);
    // The save keeps the coverage block as it was.
    let saved = std::fs::read_to_string(root.join("mortise.json")).unwrap();
    assert!(saved.contains(r#""limit": 101"#), "{saved}");
}

#[test]
fn the_third_rule_marks_a_link_outside_its_list_and_its_switch_keeps_the_lists() {
    let desk = Desk::new();
    let root = desk.path().join("layered");
    common::copy_dir(&common::fixture("layered"), &root);
    desk.knows(&[(&root, "LA")]);
    let mut harness = desk.start();
    harness.key_press(Key::Num2);
    harness.run();

    assert_eq!(rows(&harness), ["Audit uses Billing outside its list"]);
    harness.get_by_label(
        "3 of 5 modules have a list in mortise.json. A module without one is not checked.",
    );
    let switch = harness.get_by_role_and_label(Role::CheckBox, crate_strings::RULE_ALLOWED_USE);
    assert_eq!(switch.accesskit_node().toggled(), Some(Toggled::True));

    // The row lands on the link: every file is marked, and the footer names the rule.
    harness
        .get_by_role_and_label(Role::Row, "Audit uses Billing outside its list")
        .click();
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "Audit uses Billing");
    harness.get_by_label("Outside its list, 2 files");
    harness.get_by_label(
        "Rule: a module uses only the modules of its list. mortise.json lets Audit use no module.",
    );

    // The switch writes `enabled` and keeps the lists.
    harness
        .get_by_role_and_label(Role::ListBoxOption, "All modules")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(Role::CheckBox, crate_strings::RULE_ALLOWED_USE)
        .click();
    wait_until(&mut harness, "the third rule is off", |app| {
        app.app().is_some_and(|app| {
            app.rules
                .allowed_use
                .as_ref()
                .is_some_and(|allowed| !allowed.enabled)
        })
    });
    let allowed = Rules::load(&root).0.allowed_use.unwrap();
    assert!(!allowed.enabled);
    assert_eq!(allowed.modules.len(), 3);
    harness.get_by_label("No link breaks a rule.");
}

#[test]
fn the_graph_of_12_modules_gives_each_node_its_full_name() {
    let mut desk = Desk::new();
    desk.size = vec2(1100.0, 700.0);
    let root = common::app_with_modules(&desk, "platform", 12);
    // The modules use each other in a chain: the graph has as many columns as it can.
    // Checkout and Shipping use each other, so they share one.
    for (from, to) in [
        ("Accounts", "Addresses"),
        ("Addresses", "Audit"),
        ("Audit", "Carts"),
        ("Carts", "Coupons"),
        ("Coupons", "Inventory"),
        ("Inventory", "Notifications"),
        ("Billing", "Catalog"),
        ("Catalog", "Users"),
    ] {
        let dir = root.join("Modules").join(from).join("app/Services");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("Next.php"),
            format!(
                "<?php\n\nnamespace Modules\\{from}\\Services;\n\n\
                 use Modules\\{to}\\Contracts\\Next as Uses;\n\nclass Next implements Uses {{}}\n"
            ),
        )
        .unwrap();
    }
    desk.knows(&[(&root, "PL")]);
    let mut harness = desk.start();
    harness.key_press(Key::Num2);
    harness.run();
    harness.get_by_label_contains("between 12 modules");

    // A node may draw only its stamp, so its name is in full for a screen reader.
    let names: Vec<String> = harness
        .state()
        .app()
        .unwrap()
        .modules
        .iter()
        .map(|module| module.name.clone())
        .collect();
    assert_eq!(names.len(), 12);
    // Each node holds its code stamp, and no node runs into another.
    let mut nodes: Vec<Rect> = names
        .iter()
        .map(|name| harness.get_by_role_and_label(Role::Button, name).rect())
        .collect();
    nodes.sort_by(|a, b| a.left().total_cmp(&b.left()));
    for node in &nodes {
        assert!(node.width() >= size::STAMP_W + 16.0, "{node:?}");
    }
    let mut lefts: Vec<f32> = nodes.iter().map(|node| node.left()).collect();
    lefts.dedup();
    assert_eq!(lefts.len(), 11, "{nodes:?}");
    for (i, a) in nodes.iter().enumerate() {
        for b in &nodes[i + 1..] {
            assert!(!a.intersects(*b), "{a:?} {b:?}");
        }
    }

    // At 1100 by 700 the head of the rules box has no room for its note beside the title.
    let title = harness
        .get_by_role_and_label(Role::Heading, "Rules for platform")
        .rect();
    let note = harness.get_by_label(strings::RULES_KEPT).rect();
    assert!(!title.intersects(note), "{title:?} {note:?}");
    // The label of a rule wraps inside the box, whose right edge the folders field meets,
    // and the hint under it moves down.
    let rule = harness
        .get_by_role_and_label(
            Role::CheckBox,
            "Other modules use only a module's public surface",
        )
        .rect();
    let field = harness
        .get_by_role_and_label(Role::TextInput, "Folders that count as public")
        .rect();
    let hint = harness.get_by_label_contains("Accepted for").rect();
    assert!(rule.right() <= field.right(), "{rule:?} {field:?}");
    assert!(rule.bottom() <= hint.top(), "{rule:?} {hint:?}");

    // And in full in the tooltip.
    let before = harness.get_all_by_label("Addresses").count();
    harness
        .get_by_role_and_label(Role::Button, "Addresses")
        .hover();
    harness.run_steps(12);
    assert_eq!(harness.get_all_by_label("Addresses").count(), before + 1);

    // Tab reaches every node, and each one is scrolled into view.
    tab_through_the_graph(&mut harness, &names);

    // At 960 by 400 and 150% text size the graph runs past the window both ways: the
    // page scrolls down to a node as the graph scrolls across.
    let mut settings = desk.saved();
    settings.general.text_size = 150;
    settings.save(&desk.settings_path()).unwrap();
    desk.size = vec2(960.0, 400.0);
    let mut harness = desk.start();
    harness.key_press(Key::Num2);
    harness.run();
    tab_through_the_graph(&mut harness, &names);
}

/// Tabs from the links control across the graph. Each node that gets focus is inside
/// the graph, which runs from 13 px left of its title to 13 px right of its control,
/// and inside the window.
fn tab_through_the_graph(harness: &mut Harness<'static, MortiseApp>, names: &[String]) {
    let heading = harness
        .get_by_role_and_label(Role::Heading, "16 links between 12 modules")
        .rect();
    let control = harness
        .get_by_role_and_label(Role::RadioButton, "Problems only")
        .rect();
    let (left, right) = (heading.left() - 13.0, control.right() + 16.0);
    let window = harness.ctx.content_rect();
    harness
        .get_by_role_and_label(Role::RadioButton, "Problems only")
        .focus();
    harness.run();
    let mut reached = std::collections::BTreeSet::new();
    for _ in 0..40 {
        harness.key_press(Key::Tab);
        harness.run_steps(3);
        let Some(node) = harness
            .query_all_by_role(Role::Button)
            .find(|node| node.is_focused())
        else {
            continue;
        };
        let name = node.accesskit_node().label().unwrap_or_default();
        if names.contains(&name) {
            let rect = node.rect();
            assert!(
                left <= rect.left() && rect.right() <= right,
                "{name} {rect:?}"
            );
            assert!(
                window.top() <= rect.top() && rect.bottom() <= window.bottom(),
                "{name} {rect:?} {window:?}"
            );
            reached.insert(name);
        }
    }
    assert_eq!(reached.len(), 12);
}

#[test]
fn above_12_modules_the_app_graph_gives_way_to_a_table_of_links() {
    let desk = Desk::new();
    let root = common::app_with_modules(&desk, "platform", 14);
    desk.knows(&[(&root, "PL")]);
    let mut harness = desk.start();
    harness.key_press(Key::Num2);
    harness.run();

    harness.get_by_label_contains(
        "platform has 14 modules. Above 12 modules the graph of the whole app",
    );
    harness.get_by_label("8 links, 3 problems, 8 shown");
    // Problems first.
    let all = rows(&harness);
    assert_eq!(all.len(), 8);
    assert_eq!(
        &all[..3],
        [
            "Checkout uses Billing",
            "Shipping uses Checkout",
            "Checkout uses Shipping"
        ]
    );

    harness
        .get_by_role_and_label(Role::RadioButton, "Problems only")
        .click();
    harness.run();
    assert_eq!(rows(&harness).len(), 3);

    harness
        .get_by_role_and_label(Role::Row, "Shipping uses Checkout")
        .click();
    harness.run();
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Shipping".into())
    );
    harness.get_by_role_and_label(Role::Heading, "Checkout and Shipping use each other");
}

#[test]
fn a_module_with_many_users_names_them_and_keeps_problems_in_the_graph() {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    // Seven copies of Notifications also use Checkout: its users take more than a line
    // in the summary and more than 8 nodes in the graph. Webhooks reads an internal class.
    for name in [
        "Alerts",
        "Audit",
        "Exports",
        "Mailers",
        "Reminders",
        "Reports",
        "Webhooks",
    ] {
        let dir = root.join("Modules").join(name);
        common::copy_dir(&root.join("Modules/Notifications"), &dir);
        common::rename_in_files(&dir, "Notifications", name);
    }
    common::rename_in_files(
        &root.join("Modules/Webhooks"),
        "Checkout\\Events\\OrderPaid",
        "Checkout\\Models\\Order",
    );
    // A link to or from a module that is off breaks no rule: Webhooks must be on.
    let statuses = root.join("modules_statuses.json");
    let json = std::fs::read_to_string(&statuses).unwrap();
    std::fs::write(&statuses, json.replace('{', "{\n    \"Webhooks\": true,")).unwrap();
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "Dependencies");
    harness.get_by_label("4 out, 9 in");
    harness.get_by_label("Uses Billing");
    harness.get_by_label("6 refs, 3 to internals");
    harness.get_by_label("2 refs, circular");
    // The names wrap in the room the counts leave them.
    let names = harness
        .get_by_label(
            "Used by Alerts, Audit, Exports, Mailers, Notifications, Reminders, Reports, \
             Shipping, Webhooks",
        )
        .rect();
    let counts = harness
        .get_by_label("1, 1, 1, 1, 1, 1, 1, 4, 1 refs")
        .rect();
    assert!(
        names.right() + 10.0 <= counts.left(),
        "{names:?} {counts:?}"
    );

    // The graph draws the links that break a rule before the "2 more" node: Webhooks
    // has a node beside its row in the index.
    harness.key_press(Key::Num2);
    harness.run();
    harness.get_by_label("2 more");
    assert_eq!(
        harness
            .get_all_by_role_and_label(Role::ListBoxOption, "Webhooks")
            .count(),
        2
    );
}
