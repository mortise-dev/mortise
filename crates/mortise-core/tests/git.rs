//! Git data, read from repositories that are made for the test in a temporary folder.

use std::path::{Path, PathBuf};
use std::process::Command;

use mortise_core::{Git, read_app};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
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

/// Runs git in `dir` with a fixed author and without the settings of this computer.
fn git(dir: &Path, days_ago: i64, args: &[&str]) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let date = format!("{} +0000", now - days_ago * 86_400);
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=Mortise Test",
            "-c",
            "user.email=test@mortise.invalid",
        ])
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_AUTHOR_DATE", &date)
        .env("GIT_COMMITTER_DATE", &date)
        .output()
        .expect("git is installed");
    assert!(
        status.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&status.stderr)
    );
}

fn commit_all(dir: &Path, days_ago: i64, message: &str) {
    git(dir, days_ago, &["add", "-A"]);
    git(dir, days_ago, &["commit", "-q", "-m", message]);
}

fn touch(root: &Path, file: &str) {
    let path = root.join(file);
    let mut content = std::fs::read_to_string(&path).unwrap_or_default();
    content.push_str("\n// changed\n");
    std::fs::write(path, content).unwrap();
}

#[test]
fn counts_the_commits_of_30_days_for_each_module() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("shop-api");
    copy_dir(&fixture("shop-api"), &root);
    git(&root, 90, &["init", "-q", "-b", "main"]);
    commit_all(&root, 90, "first");

    touch(&root, "Modules/Checkout/app/Models/Order.php");
    commit_all(&root, 40, "too old to count");
    touch(&root, "Modules/Checkout/app/Models/Order.php");
    touch(&root, "Modules/Shipping/app/Jobs/SyncTracking.php");
    commit_all(&root, 10, "two modules in one commit");
    touch(&root, "Modules/Checkout/routes/web.php");
    touch(&root, "Modules/Checkout/routes/api.php");
    commit_all(&root, 2, "two files of one module");
    touch(&root, "composer.json");
    commit_all(&root, 1, "outside the modules");

    let app = read_app(&root).unwrap();
    let Git::Repository { branch, commit } = &app.git else {
        panic!("expected a repository, got {:?}", app.git);
    };
    assert_eq!(branch.as_deref(), Some("main"));
    assert!(commit.as_ref().is_some_and(|id| id.len() >= 7));

    let churn = |name: &str| {
        app.modules
            .iter()
            .find(|module| module.name == name)
            .unwrap()
            .churn
            .unwrap()
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let days_since = |time: Option<i64>| (now - time.unwrap()) / 86_400;

    assert_eq!(churn("Checkout").commits, 2);
    assert_eq!(days_since(churn("Checkout").last_commit), 2);
    assert_eq!(churn("Shipping").commits, 1);
    assert_eq!(days_since(churn("Shipping").last_commit), 10);
    // Quiet for 30 days: the last commit is still known.
    assert_eq!(churn("Billing").commits, 0);
    assert_eq!(days_since(churn("Billing").last_commit), 90);
}

#[test]
fn an_app_in_a_subfolder_of_a_repository_is_counted_by_its_own_paths() {
    let dir = tempfile::tempdir().unwrap();
    let repository = dir.path().join("monorepo");
    let root = repository.join("apps").join("shop-api");
    copy_dir(&fixture("shop-api"), &root);
    git(&repository, 5, &["init", "-q", "-b", "trunk"]);
    commit_all(&repository, 5, "first");
    touch(&root, "Modules/Users/app/Models/User.php");
    commit_all(&repository, 1, "users");

    let app = read_app(&root).unwrap();
    assert!(matches!(&app.git, Git::Repository { branch: Some(branch), .. } if branch == "trunk"));
    let users = app
        .modules
        .iter()
        .find(|module| module.name == "Users")
        .unwrap();
    assert_eq!(users.churn.unwrap().commits, 2);
    let billing = app
        .modules
        .iter()
        .find(|module| module.name == "Billing")
        .unwrap();
    assert_eq!(billing.churn.unwrap().commits, 1);
    // Nothing is uncommitted yet.
    assert_eq!(users.uncommitted, Some(0));

    // The app is below the top of the repository, so the prefix matters.
    touch(&root, "Modules/Users/app/Models/User.php");
    std::fs::write(root.join("Modules/Users/app/Staged.php"), "<?php\n").unwrap();
    git(
        &repository,
        0,
        &["add", "apps/shop-api/Modules/Users/app/Staged.php"],
    );
    std::fs::write(root.join("Modules/Users/app/Loose.php"), "<?php\n").unwrap();
    std::fs::write(root.join("Modules/Users/app/Cache.log"), "x").unwrap();
    std::fs::write(repository.join(".git/info/exclude"), "*.log\n").unwrap();
    let app = read_app(&root).unwrap();
    let uncommitted = |name: &str| {
        app.modules
            .iter()
            .find(|module| module.name == name)
            .unwrap()
            .uncommitted
    };
    assert_eq!(uncommitted("Users"), Some(3));
    assert_eq!(uncommitted("Billing"), Some(0));
}

#[test]
fn a_repository_without_a_commit_and_a_detached_head() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("shop-api");
    copy_dir(&fixture("shop-api"), &root);
    git(&root, 3, &["init", "-q", "-b", "main"]);

    let app = read_app(&root).unwrap();
    assert_eq!(
        app.git,
        Git::Repository {
            branch: Some("main".into()),
            commit: None
        }
    );
    let billing = &app.modules[0];
    assert_eq!(billing.churn.unwrap().commits, 0);
    assert_eq!(billing.churn.unwrap().last_commit, None);

    commit_all(&root, 3, "first");
    git(&root, 3, &["checkout", "-q", "--detach"]);
    let app = read_app(&root).unwrap();
    assert!(matches!(
        &app.git,
        Git::Repository {
            branch: None,
            commit: Some(_)
        }
    ));
}

#[test]
fn reading_git_data_changes_nothing_in_the_repository() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("shop-api");
    copy_dir(&fixture("shop-api"), &root);
    git(&root, 3, &["init", "-q", "-b", "main"]);
    commit_all(&root, 3, "first");
    touch(&root, "Modules/Users/app/Models/User.php");

    let index = root.join(".git/index");
    let before = (
        std::fs::read(&index).unwrap(),
        std::fs::metadata(&index).unwrap().modified().unwrap(),
    );
    read_app(&root).unwrap();
    let after = (
        std::fs::read(&index).unwrap(),
        std::fs::metadata(&index).unwrap().modified().unwrap(),
    );
    assert!(before == after, "the git index was written");
}
