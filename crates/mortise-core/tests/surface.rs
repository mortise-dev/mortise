//! The public surface scan, on the Billing module of the shop-api fixture.

use std::path::{Path, PathBuf};

use mortise_core::{
    AcceptedSurface, Proposal, ScanCounts, SurfaceKind, SurfaceScan, Typed, read_app,
};

fn shop_api() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/shop-api")
}

fn routes() -> Vec<String> {
    let mut routes: Vec<String> = (0..9)
        .map(|i| format!("api/billing/invoices/{i}"))
        .collect();
    routes.extend((0..5).map(|i| format!("billing/invoices/{i}")));
    routes
}

fn scheduled() -> Vec<(String, String)> {
    vec![
        (
            "php artisan billing:send-invoices".to_owned(),
            "0 6 * * *".to_owned(),
        ),
        (
            "php artisan checkout:expire-carts".to_owned(),
            "*/5 * * * *".to_owned(),
        ),
    ]
}

fn billing() -> Proposal {
    let app = read_app(&shop_api()).unwrap();
    let mut scan = SurfaceScan::start(&app, "Billing").unwrap();
    while !scan.step(4) {}
    scan.finish(Some(&routes()), &scheduled())
}

fn names(proposal: &Proposal, kind: SurfaceKind) -> Vec<&str> {
    proposal
        .of(kind)
        .map(|(_, entry)| entry.name.as_str())
        .collect()
}

#[test]
fn the_scan_reads_the_files_in_steps_and_counts_as_it_goes() {
    let app = read_app(&shop_api()).unwrap();
    let mut scan = SurfaceScan::start(&app, "Billing").unwrap();
    assert_eq!(scan.module(), "Billing");
    // The PHP files of the module, without its tests.
    assert_eq!(scan.total(), 15);
    assert_eq!(scan.read(), 0);
    // What other modules use is known before a file is read: the links are there.
    assert_eq!(
        scan.counts(),
        ScanCounts {
            used_classes: 3,
            used_files: 7,
            contracts: 0,
            events: 0,
            commands: 0
        }
    );

    assert!(!scan.step(5));
    assert_eq!(scan.read(), 5);
    assert!(!scan.step(5));
    assert!(scan.step(5));
    assert_eq!(scan.read(), 15);
    assert_eq!(
        scan.counts(),
        ScanCounts {
            used_classes: 3,
            used_files: 7,
            contracts: 2,
            events: 3,
            commands: 2
        }
    );

    assert!(SurfaceScan::start(&app, "Nothing").is_none());
}

#[test]
fn contracts_events_routes_and_commands_are_proposed() {
    let proposal = billing();
    assert_eq!(proposal.module, "Billing");
    assert!(proposal.routes_read);

    // What is used from outside comes first in its kind.
    assert_eq!(
        names(&proposal, SurfaceKind::Contract),
        [
            r"Modules\Billing\Contracts\PaymentGateway",
            r"Modules\Billing\Contracts\InvoiceNumbering"
        ]
    );
    let gateway = &proposal.entries[0];
    assert_eq!(gateway.folder, "Contracts");
    assert_eq!(gateway.used_by, [("Checkout".to_owned(), 1)]);
    assert!(gateway.proposed);
    assert!(proposal.entries[1].used_by.is_empty());

    assert_eq!(
        names(&proposal, SurfaceKind::Event),
        [
            r"Modules\Billing\Events\InvoicePaid",
            r"Modules\Billing\Events\InvoiceIssued",
            r"Modules\Billing\Events\PaymentFailed"
        ]
    );
    assert_eq!(
        proposal.entries[2].used_by,
        [("Checkout".to_owned(), 2), ("Notifications".to_owned(), 1)]
    );

    assert_eq!(
        names(&proposal, SurfaceKind::Route),
        ["api/billing/*", "billing/*"]
    );
    let routes: Vec<usize> = proposal
        .of(SurfaceKind::Route)
        .map(|(_, entry)| entry.routes)
        .collect();
    assert_eq!(routes, [9, 5]);

    // The command the scheduler runs comes first, with its schedule as the app wrote it.
    assert_eq!(
        names(&proposal, SurfaceKind::Command),
        ["billing:send-invoices", "billing:retry-failed-payments"]
    );
    let commands: Vec<Option<&str>> = proposal
        .of(SurfaceKind::Command)
        .map(|(_, entry)| entry.schedule.as_deref())
        .collect();
    assert_eq!(commands, [Some("0 6 * * *"), None]);
}

#[test]
fn a_class_that_is_used_from_outside_is_shown_and_not_proposed() {
    let proposal = billing();
    let others: Vec<_> = proposal
        .of(SurfaceKind::OtherClass)
        .map(|(_, entry)| entry)
        .collect();
    assert_eq!(others.len(), 1);
    assert_eq!(others[0].name, r"Modules\Billing\Models\Invoice");
    assert_eq!(others[0].folder, "Models");
    assert_eq!(others[0].used_by, [("Checkout".to_owned(), 3)]);
    assert!(!others[0].proposed);

    // 2 contracts, 3 events, 1 other class, 2 route patterns, 2 commands.
    assert_eq!(proposal.found(), 10);
    assert_eq!(proposal.proposed(), 9);
}

#[test]
fn without_routes_the_proposal_says_that_they_were_not_read() {
    let app = read_app(&shop_api()).unwrap();
    let mut scan = SurfaceScan::start(&app, "Billing").unwrap();
    while !scan.step(100) {}
    let proposal = scan.finish(None, &[]);
    assert!(!proposal.routes_read);
    assert_eq!(proposal.of(SurfaceKind::Route).count(), 0);
    assert_eq!(proposal.found(), 8);

    // A change keeps the accepted routes it could not check.
    let mut changed = proposal.clone();
    let accepted = AcceptedSurface {
        routes: vec!["api/billing/*".to_owned()],
        ..Default::default()
    };
    changed.take_choices_of(&accepted);
    assert_eq!(changed.surface(), accepted);
}

#[test]
fn a_name_typed_by_hand_is_checked_against_the_module() {
    let mut proposal = billing();
    assert_eq!(
        proposal.check(r"Modules\Billing\Http\Controllers\InvoiceController"),
        Typed::Class {
            folder: "Controllers".into()
        }
    );
    // A leading backslash is the same class.
    assert_eq!(
        proposal.check(r"\Modules\Billing\Http\Controllers\InvoiceController"),
        Typed::Class {
            folder: "Controllers".into()
        }
    );
    assert_eq!(
        proposal.check(r"Modules\Billing\Models\Invoice"),
        Typed::Listed
    );
    assert_eq!(
        proposal.check(r"Modules\Checkout\Models\Order"),
        Typed::Unknown
    );
    assert_eq!(proposal.check("billing:send-invoices"), Typed::Listed);
    assert_eq!(proposal.check("billing:nothing"), Typed::Unknown);
    assert_eq!(
        proposal.check("api/billing/invoices/3"),
        Typed::Route { routes: 1 }
    );
    assert_eq!(proposal.check("api/shipping/*"), Typed::Unknown);
    assert_eq!(proposal.check("  "), Typed::Unknown);

    // An added class stands with the other classes, and is in the proposal.
    assert!(proposal.add(r"Modules\Billing\Http\Controllers\InvoiceController"));
    assert!(!proposal.add(r"Modules\Billing\Http\Controllers\InvoiceController"));
    assert!(!proposal.add("billing:nothing"));
    let others = names(&proposal, SurfaceKind::OtherClass);
    assert_eq!(
        others,
        [
            r"Modules\Billing\Models\Invoice",
            r"Modules\Billing\Http\Controllers\InvoiceController"
        ]
    );
    let added = proposal.entries.iter().find(|entry| entry.by_hand).unwrap();
    assert!(added.proposed);
    assert_eq!(proposal.proposed(), 10);
    // The kinds stay together.
    let kinds: Vec<SurfaceKind> = proposal.entries.iter().map(|entry| entry.kind).collect();
    assert!(kinds.is_sorted());
}

#[test]
fn a_second_scan_keeps_what_the_developer_chose() {
    let mut first = billing();
    // Left out: an event. Taken in: the model. Added by hand: a controller.
    first.entries[3].proposed = false;
    let model = first
        .entries
        .iter_mut()
        .find(|entry| entry.kind == SurfaceKind::OtherClass)
        .unwrap();
    model.proposed = true;
    assert!(first.add(r"Modules\Billing\Http\Controllers\InvoiceController"));

    let mut second = billing();
    second.keep_choices_of(&first);
    assert_eq!(second, first);
}

#[test]
fn a_proposal_becomes_a_surface_and_a_later_scan_is_compared_with_it() {
    let mut proposal = billing();
    // What is checked is stored, each list sorted by byte order.
    let surface = proposal.surface();
    assert_eq!(
        surface.classes,
        [
            r"Modules\Billing\Contracts\InvoiceNumbering",
            r"Modules\Billing\Contracts\PaymentGateway",
            r"Modules\Billing\Events\InvoiceIssued",
            r"Modules\Billing\Events\InvoicePaid",
            r"Modules\Billing\Events\PaymentFailed",
        ]
    );
    assert_eq!(surface.routes, ["api/billing/*", "billing/*"]);
    assert_eq!(
        surface.commands,
        ["billing:retry-failed-payments", "billing:send-invoices"]
    );
    assert_eq!(
        surface.to_stdin(),
        r#"{"classes":["Modules\\Billing\\Contracts\\InvoiceNumbering","Modules\\Billing\\Contracts\\PaymentGateway","Modules\\Billing\\Events\\InvoiceIssued","Modules\\Billing\\Events\\InvoicePaid","Modules\\Billing\\Events\\PaymentFailed"],"routes":["api/billing/*","billing/*"],"commands":["billing:retry-failed-payments","billing:send-invoices"]}"#
    );
    assert!(proposal.differences(&surface).is_empty());
    // Narrower routes that cover every route of the code are not a difference.
    let narrower = AcceptedSurface {
        routes: vec!["api/billing/invoices/*".to_owned(), "billing/*".to_owned()],
        ..surface.clone()
    };
    assert!(proposal.differences(&narrower).is_empty());

    // Later: the code has a new event, and an accepted command and class are gone.
    let mut accepted = surface.clone();
    accepted
        .classes
        .retain(|class| !class.ends_with("PaymentFailed"));
    accepted
        .classes
        .push(r"Modules\Billing\Models\Refund".to_owned());
    accepted.commands.push("billing:old".to_owned());
    // A leading backslash is the same class, so it is neither new nor gone.
    for class in &mut accepted.classes {
        if class.ends_with("InvoicePaid") {
            class.insert(0, '\\');
        }
    }
    let differences = proposal.differences(&accepted);
    let found: Vec<(&str, bool)> = differences
        .iter()
        .map(|difference| (difference.entry.name.as_str(), difference.new))
        .collect();
    assert_eq!(
        found,
        [
            (r"Modules\Billing\Events\PaymentFailed", true),
            (r"Modules\Billing\Models\Refund", false),
            ("billing:old", false),
        ]
    );
    // Taking every difference gives the scan; leaving one out keeps its accepted state.
    assert_eq!(accepted.with_differences(&differences), surface);
    let mut kept = differences.clone();
    kept[2].taken = false;
    assert_eq!(
        accepted.with_differences(&kept).commands,
        [
            "billing:old",
            "billing:retry-failed-payments",
            "billing:send-invoices"
        ]
    );

    // A change of the accepted surface starts from what it holds.
    let mut only = AcceptedSurface::default();
    only.classes
        .push(r"\modules\billing\models\invoice".to_owned());
    proposal.take_choices_of(&only);
    assert_eq!(
        proposal.surface().classes,
        [r"Modules\Billing\Models\Invoice"]
    );
}

#[test]
fn the_proposal_as_text_lists_what_is_in_it() {
    let proposal = billing();
    assert_eq!(
        proposal.as_text(),
        r"Public surface of Billing, proposed by Mortise

Contracts
  Modules\Billing\Contracts\PaymentGateway
  Modules\Billing\Contracts\InvoiceNumbering

Events
  Modules\Billing\Events\InvoicePaid
  Modules\Billing\Events\InvoiceIssued
  Modules\Billing\Events\PaymentFailed

Routes
  api/billing/*
  billing/*

Console commands
  billing:send-invoices
  billing:retry-failed-payments
"
    );
}

#[test]
fn the_scan_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("shop-api");
    copy_dir(&shop_api(), &root);
    let before = listing(&root);
    let app = read_app(&root).unwrap();
    let mut scan = SurfaceScan::start(&app, "Billing").unwrap();
    while !scan.step(3) {}
    scan.finish(Some(&routes()), &scheduled());
    assert_eq!(listing(&root), before);
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// Every file under a folder with its size and the time it was changed.
fn listing(dir: &Path) -> Vec<(PathBuf, u64, std::time::SystemTime)> {
    let mut files = Vec::new();
    let mut folders = vec![dir.to_owned()];
    while let Some(folder) = folders.pop() {
        for entry in std::fs::read_dir(folder).unwrap() {
            let entry = entry.unwrap();
            let metadata = entry.metadata().unwrap();
            if metadata.is_dir() {
                folders.push(entry.path());
            } else {
                files.push((entry.path(), metadata.len(), metadata.modified().unwrap()));
            }
        }
    }
    files.sort();
    files
}

/// Scans every module of the owner's two real apps. Only reads. Run with:
/// `MORTISE_REAL_APPS=/path/to/apps cargo test -p mortise-core --test surface -- --ignored real_apps --nocapture`
#[test]
#[ignore = "needs MORTISE_REAL_APPS, the folder that holds laravel-12 and laravel-13"]
fn real_apps_scan_without_failing() {
    let apps = PathBuf::from(std::env::var("MORTISE_REAL_APPS").expect("MORTISE_REAL_APPS"));
    for folder in ["laravel-13", "laravel-12"] {
        let root = apps.join(folder);
        let before = listing(&root.join("Modules"));
        let app = read_app(&root).unwrap();
        for module in &app.modules {
            let mut scan = SurfaceScan::start(&app, &module.name).unwrap();
            while !scan.step(8) {}
            let total = scan.total();
            let proposal = scan.finish(None, &[]);
            println!(
                "{folder} {}: {total} files read\n{}",
                module.name,
                proposal.as_text()
            );
        }
        assert_eq!(
            listing(&root.join("Modules")),
            before,
            "{folder}: the scan wrote something"
        );
    }
}

#[test]
#[ignore = "prints the proposals of the fixture for a person to read"]
fn print_fixture_proposals() {
    let app = read_app(&shop_api()).unwrap();
    for module in &app.modules {
        let mut scan = SurfaceScan::start(&app, &module.name).unwrap();
        while !scan.step(8) {}
        println!(
            "{}: {} entries",
            module.name,
            scan.finish(None, &[]).found()
        );
    }
}
