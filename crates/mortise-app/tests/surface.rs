//! The public surface of a module: the second view of the Boundary tab.

mod common;

use common::{Desk, connected, pad_module, shop_api, shop_api_actuator};
use eframe::egui::accesskit::Toggled;
use eframe::egui::{Key, Modifiers, Role};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use mortise_app::app::MortiseApp;
use mortise_app::index::view::Selection;
use mortise_app::workspace::Tab;

const INVOICE: &str = r"Modules\Billing\Models\Invoice";
const CONTROLLER: &str = r"Modules\Billing\Http\Controllers\InvoiceController";

/// Opens the Boundary tab of Billing on its second view.
fn open_surface(harness: &mut Harness<'static, MortiseApp>) {
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Billing")
        .click();
    harness.run_steps(2);
    harness.key_press(Key::Num2);
    harness.run_steps(2);
    harness.key_press(Key::P);
    harness.run_steps(2);
}

/// Starts the scan with its key and steps until the proposal is on screen.
fn scan(harness: &mut Harness<'static, MortiseApp>) {
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    for _ in 0..40 {
        harness.step();
        if harness
            .query_by_label_contains("Proposed public surface")
            .is_some()
        {
            // One more frame, for what is drawn above the proposal.
            harness.run_steps(2);
            return;
        }
    }
    panic!("the scan did not end");
}

fn is_checked(harness: &Harness<'static, MortiseApp>, entry: &str) -> bool {
    let node = harness.get_by_role_and_label(Role::CheckBox, entry);
    node.accesskit_node().toggled() == Some(Toggled::True)
}

#[test]
fn p_switches_between_the_links_and_the_public_surface() {
    let (desk, mut harness) = shop_api();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Billing")
        .click();
    harness.run_steps(2);
    harness.key_press(Key::Num2);
    harness.run_steps(2);

    // The links first. The control names the key that switches.
    harness.get_by_role_and_label(Role::RadioButton, "Links");
    harness.get_by_label("Switch with");
    assert!(
        harness
            .query_by_label("Billing has no public surface yet")
            .is_none()
    );

    harness.key_press(Key::P);
    harness.run_steps(2);
    harness.get_by_role_and_label(Role::Heading, "Billing has no public surface yet");
    harness.get_by_label(
        "A scan reads the files of Billing and proposes what other modules, HTTP clients and \
         the command line may use. Nothing is written until you accept the proposal. Until \
         then the rule uses the folders Contracts and Events.",
    );
    // A scan never starts by itself.
    harness.get_by_role_and_label(Role::Button, "Scan Billing");
    assert!(
        harness
            .query_by_label_contains("Proposed public surface")
            .is_none()
    );

    // The choice of view is kept while moving through modules.
    harness.key_press(Key::J);
    harness.run_steps(2);
    harness.get_by_role_and_label(Role::Heading, "Catalog has no public surface yet");

    harness
        .get_by_role_and_label(Role::RadioButton, "Links")
        .click();
    harness.run_steps(2);
    assert!(
        harness
            .query_by_label("Catalog has no public surface yet")
            .is_none()
    );

    // The whole app has one view.
    harness
        .get_by_role_and_label(Role::ListBoxOption, "All modules")
        .click();
    harness.run_steps(2);
    assert!(
        harness
            .query_by_role_and_label(Role::RadioButton, "Public surface")
            .is_none()
    );
    assert!(!desk.path().join("shop-api/mortise.json").exists());
}

#[test]
fn a_scan_proposes_contracts_events_and_commands() {
    let (desk, mut harness) = shop_api();
    open_surface(&mut harness);
    scan(&mut harness);

    harness.get_by_role_and_label(Role::Heading, "Proposed public surface of Billing");
    harness.get_by_label("8 found, 7 proposed");
    harness.get_by_label("scanned just now");
    for group in [
        "Contracts",
        "Events",
        "Other classes that other modules use",
        "Console commands",
    ] {
        harness.get_by_role_and_label(Role::Heading, group);
    }

    // Each entry with why it is there and who uses it today.
    assert!(is_checked(
        &harness,
        r"Modules\Billing\Contracts\PaymentGateway"
    ));
    assert!(is_checked(
        &harness,
        r"Modules\Billing\Contracts\InvoiceNumbering"
    ));
    assert!(is_checked(&harness, r"Modules\Billing\Events\InvoicePaid"));
    assert!(is_checked(&harness, "billing:send-invoices"));
    assert!(is_checked(&harness, "billing:retry-failed-payments"));
    assert_eq!(
        harness
            .get_all_by_label("an interface in Contracts")
            .count(),
        2
    );
    assert_eq!(harness.get_all_by_label("an event").count(), 3);
    assert_eq!(harness.get_all_by_label("a console command").count(), 2);
    harness.get_by_label("Checkout, 1 file");
    harness.get_by_label("Checkout, Notifications, 3 files");

    // A class that is used from outside and is no contract and no event: shown, not proposed.
    assert!(!is_checked(&harness, INVOICE));
    harness.get_by_label("in Models, used by Checkout, 3 files");
    harness.get_by_label(
        "Used from outside, not proposed. Include it to allow these reads, or leave it out and \
         keep the finding on Checkout.",
    );

    // No environment is connected.
    harness.get_by_role_and_label(Role::Heading, "Routes");
    harness.get_by_label(
        "Routes were not read. Connect an environment or set how PHP runs, then scan again.",
    );

    // Accepting needs the package: the button is there, disabled, and says why.
    let accept = harness.get_by_role_and_label(Role::Button, "Accept 7 entries");
    assert!(accept.accesskit_node().is_disabled());
    accept.hover();
    harness.run_steps(12);
    harness.get_by_label("The package cannot store a public surface yet");

    // Nothing was written.
    assert!(!desk.path().join("shop-api/mortise.json").exists());
    assert_eq!(harness.state().app().unwrap().findings.problems(), 2);
}

#[test]
fn a_connected_app_gives_its_routes_and_what_the_scheduler_runs() {
    let stub = shop_api_actuator();
    let (_desk, mut harness) = connected(&stub);
    let before = stub.asked().len();
    open_surface(&mut harness);
    scan(&mut harness);

    harness.get_by_label("10 found, 9 proposed");
    harness.get_by_role_and_label(Role::Heading, "Routes, read from local");
    assert!(is_checked(&harness, "api/billing/*"));
    assert!(is_checked(&harness, "billing/*"));
    assert_eq!(
        harness.get_all_by_label("routes with this prefix").count(),
        2
    );
    harness.get_by_label("10 routes, HTTP clients");
    harness.get_by_label("4 routes, HTTP clients");
    // The schedule is as the app wrote it.
    harness.get_by_label("the scheduler, 0 6 * * *");
    assert_eq!(harness.get_all_by_label("not used outside yet").count(), 4);
    // The scan uses what the app already answered: it asks for nothing but the regular check.
    assert!(
        stub.asked()[before..]
            .iter()
            .all(|path| path == "/actuator/health")
    );
}

#[test]
fn an_entry_is_taken_in_or_left_out_by_click_and_by_space() {
    let (_desk, mut harness) = shop_api();
    open_surface(&mut harness);
    scan(&mut harness);

    // When the scan ends, focus is on the first entry: Space leaves it out.
    harness.key_press(Key::Space);
    harness.run_steps(2);
    assert!(!is_checked(
        &harness,
        r"Modules\Billing\Contracts\PaymentGateway"
    ));
    harness.get_by_label("8 found, 6 proposed");
    harness.key_press(Key::Space);
    harness.run_steps(2);
    assert!(is_checked(
        &harness,
        r"Modules\Billing\Contracts\PaymentGateway"
    ));

    // Taking the model in: its reads would be allowed, and the note goes.
    harness
        .get_by_role_and_label(Role::CheckBox, INVOICE)
        .click();
    harness.run_steps(2);
    assert!(is_checked(&harness, INVOICE));
    harness.get_by_label("8 found, 8 proposed");
    assert!(
        harness
            .query_by_label_contains("Used from outside, not proposed.")
            .is_none()
    );
    harness.get_by_role_and_label(Role::Button, "Accept 8 entries");
    // The proposal is not accepted: the finding on Checkout stays.
    assert_eq!(harness.state().app().unwrap().findings.problems(), 2);
}

#[test]
fn a_adds_an_entry_by_hand_when_the_module_has_it() {
    let (_desk, mut harness) = shop_api();
    open_surface(&mut harness);
    scan(&mut harness);

    harness.key_press(Key::A);
    harness.run_steps(2);
    let field = harness.get_by_role_and_label(Role::TextInput, "Add a class, a route or a command");
    assert!(field.accesskit_node().is_focused());

    // What is typed is checked against the files of the module.
    field.type_text(r"Modules\Billing\Nothing");
    harness.run_steps(2);
    harness.get_by_label("Nothing in Billing has this name.");
    harness.key_press(Key::Enter);
    harness.run_steps(2);
    harness.get_by_label("8 found, 7 proposed");

    harness.key_press(Key::A);
    harness.run_steps(2);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::TextInput, "Add a class, a route or a command")
        .type_text(CONTROLLER);
    harness.run_steps(2);
    harness.get_by_label("A class in Controllers. Enter adds it.");
    harness.key_press(Key::Enter);
    harness.run_steps(3);
    assert!(is_checked(&harness, CONTROLLER));
    harness.get_by_label("in Controllers, added by hand");
    harness.get_by_label("9 found, 8 proposed");

    // The same name again is in the list already.
    harness.key_press(Key::A);
    harness.run_steps(2);
    harness
        .get_by_role_and_label(Role::TextInput, "Add a class, a route or a command")
        .type_text(CONTROLLER);
    harness.run_steps(2);
    harness.get_by_label("It is in the list.");
}

#[test]
fn the_proposal_is_copied_as_text_and_a_second_scan_keeps_the_choices() {
    let (_desk, mut harness) = shop_api();
    open_surface(&mut harness);
    scan(&mut harness);
    harness
        .get_by_role_and_label(Role::CheckBox, INVOICE)
        .click();
    harness.run_steps(2);
    harness
        .get_by_role_and_label(Role::CheckBox, "billing:retry-failed-payments")
        .click();
    harness.run_steps(2);
    assert!(is_checked(&harness, INVOICE));
    assert!(!is_checked(&harness, "billing:retry-failed-payments"));

    harness
        .get_by_role_and_label(Role::Button, "Copy as text")
        .click();
    harness.run_steps(3);
    harness.get_by_label("Copied the proposal.");

    harness
        .get_by_role_and_label(Role::Button, "Scan again")
        .click();
    for _ in 0..40 {
        harness.step();
    }
    assert!(is_checked(&harness, INVOICE));
    assert!(!is_checked(&harness, "billing:retry-failed-payments"));
    harness.get_by_label("8 found, 7 proposed");
}

#[test]
fn a_scan_shows_how_far_it_is_and_esc_stops_it() {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    // 96 files: one frame reads 8 of them.
    pad_module(&root, "Billing", 81);
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();
    open_surface(&mut harness);

    harness
        .get_by_role_and_label(Role::Button, "Scan Billing")
        .click();
    harness.step();
    harness.step();
    harness.get_by_role_and_label(Role::Heading, "Scanning Billing");
    harness.get_by_label("nothing is written");
    harness.get_by_role_and_label(Role::ProgressIndicator, "Reading 8 of 96 files");
    harness.get_by_label("3 classes in 7 files");
    harness.get_by_label("not read");
    for line in [
        "Classes that other modules use",
        "Contracts",
        "Events",
        "Routes",
        "Console commands",
    ] {
        harness.get_by_label(line);
    }

    // Esc stops it. Nothing was written, so the view is as it was.
    harness.key_press(Key::Escape);
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "Billing has no public surface yet");
    assert!(
        harness
            .query_by_label_contains("Proposed public surface")
            .is_none()
    );

    // The button stops it too.
    harness
        .get_by_role_and_label(Role::Button, "Scan Billing")
        .click();
    harness.step();
    harness.step();
    harness
        .get_by_role_and_label(Role::Button, "Stop the scan")
        .click();
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "Billing has no public surface yet");

    // Left alone, it ends with the proposal.
    scan(&mut harness);
    harness.get_by_label("8 found, 7 proposed");
}

#[test]
fn the_palette_opens_the_view_and_does_not_scan() {
    let (_desk, mut harness) = shop_api();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run_steps(3);
    harness
        .get_by_role_and_label(Role::TextInput, "Search or run a command")
        .type_text("surface bil");
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::ListBoxOption, "Scan the public surface of Billing");
    harness.key_press(Key::Enter);
    harness.run_steps(3);

    assert_eq!(
        harness.state().selection(),
        Selection::Module("Billing".into())
    );
    assert_eq!(harness.state().tab(), Tab::Boundary);
    harness.get_by_role_and_label(Role::Heading, "Billing has no public surface yet");
    assert!(
        harness
            .query_by_label_contains("Proposed public surface")
            .is_none()
    );
}

#[test]
fn a_module_that_offers_nothing_says_so() {
    let (_desk, mut harness) = shop_api();
    // Inventory has no interface, no event and no command, and nothing uses its classes.
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Inventory")
        .click();
    harness.run_steps(2);
    harness.key_press(Key::Num2);
    harness.key_press(Key::P);
    harness.run_steps(2);
    scan(&mut harness);
    harness.get_by_label("0 found, 0 proposed");
    harness.get_by_label(
        "The scan found no interface, no event and no console command in Inventory, and no \
         class that another module uses.",
    );
    assert_eq!(harness.query_all_by_role(Role::CheckBox).count(), 0);
    harness.get_by_role_and_label(Role::Button, "Accept 0 entries");
}
