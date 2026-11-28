//! Updates of Mortise itself: ask for the newest release, download it, check its
//! signature, replace the app and start it again. Nothing is drawn here.
//!
//! The check is one request for the update manifest of the newest GitHub release. It
//! carries nothing of an app. Nothing is downloaded before the user asks for it, and
//! nothing replaces the app unless it carries the signature of the update key.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use base64::Engine as _;
use cargo_packager_updater::http::HeaderMap;
use cargo_packager_updater::semver::Version;
use cargo_packager_updater::{Config, Error, Update, UpdateFormat};
use eframe::egui;

pub mod strings;

/// Where a copy that cannot replace itself gets the newer version.
pub const DOWNLOAD_PAGE: &str = "https://github.com/mortise-dev/mortise/releases/latest";

/// The update manifest of the newest release. tag-release.yml publishes it.
const MANIFEST: &str =
    "https://github.com/mortise-dev/mortise/releases/latest/download/latest.json";

/// The public half of the update key, as `cargo packager signer generate` writes it.
/// CONTRIBUTING.md says where the private half lives.
const PUBLIC_KEY: &str = "";

/// The end of the name of the file this platform updates from, after `Mortise-vX.Y.Z-`.
/// tools/sign-updates.sh signs the files under these names.
const ARTIFACT: &str = if cfg!(target_os = "macos") {
    "macos-arm64.app.tar.gz"
} else if cfg!(target_os = "windows") {
    "windows-x64-setup.exe"
} else {
    "linux-x86_64.AppImage"
};

/// The automatic check asks at most this often, in seconds.
pub const CHECK_EVERY: u64 = 24 * 60 * 60;

const CHECK_TIMEOUT: Duration = Duration::from_secs(15);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// Replaces the app with a download that passed the signature check.
pub type Install = Box<dyn FnOnce() -> Result<(), String> + Send>;

/// Where updates come from. The window has the releases on GitHub; tests have their own.
pub trait Channel: Send + Sync {
    /// The newer version, if there is one.
    fn check(&self) -> Result<Option<String>, String>;
    /// Whether `version`, found by an earlier check, is still newer than this copy.
    fn is_newer(&self, version: &str) -> bool;
    /// Whether this copy can replace itself. A `.deb` install and an app in a
    /// place the user cannot write cannot.
    fn can_install(&self) -> bool;
    /// Downloads the newer version and checks its signature. `progress` gets the
    /// share that is downloaded, 0 to 1.
    fn download(&self, progress: &dyn Fn(f32)) -> Result<Install, String>;
    /// Starts the installed version. The caller then closes this one.
    fn restart(&self) -> Result<(), String>;
}

/// The releases of Mortise on GitHub.
pub struct Release {
    manifest: String,
    public_key: String,
    current: Version,
    /// The key of this platform in the manifest, such as `macos-aarch64`.
    target: String,
    artifact: &'static str,
}

impl Release {
    /// The releases this build updates from.
    pub fn of_this_build() -> Self {
        Self {
            manifest: MANIFEST.to_owned(),
            public_key: PUBLIC_KEY.to_owned(),
            current: env!("CARGO_PKG_VERSION")
                .parse()
                .expect("the version of the crate is valid"),
            target: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            artifact: ARTIFACT,
        }
    }

    /// Asks for the manifest and gives what it offers this platform, if that is newer.
    ///
    /// The request goes out with the certificates this computer trusts and follows the
    /// redirects of GitHub. It changes nothing in the process, which the check of
    /// `cargo-packager-updater` does on Linux.
    fn newer(&self) -> Result<Option<Update>, String> {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(CHECK_TIMEOUT))
            .tls_config(
                ureq::tls::TlsConfig::builder()
                    .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                    .build(),
            )
            .build();
        let text = ureq::Agent::new_with_config(config)
            .get(&self.manifest)
            .call()
            .and_then(|mut answer| answer.body_mut().read_to_string())
            .map_err(|error| error.to_string())?;
        let manifest: serde_json::Value =
            serde_json::from_str(&text).map_err(|error| error.to_string())?;
        let said = |value: &serde_json::Value, key: &str| {
            value[key]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| strings::manifest_lacks(key))
        };
        let version: Version = said(&manifest, "version")?
            .trim_start_matches('v')
            .parse()
            .map_err(|_| strings::manifest_lacks("version"))?;
        if version <= self.current {
            return Ok(None);
        }
        let platform = &manifest["platforms"][&self.target];
        let format = match said(platform, "format")?.as_str() {
            "app" => UpdateFormat::App,
            "appimage" => UpdateFormat::AppImage,
            "nsis" => UpdateFormat::Nsis,
            _ => return Err(strings::manifest_lacks("format")),
        };
        Ok(Some(Update {
            config: Config {
                pubkey: self.public_key.clone(),
                ..Default::default()
            },
            body: None,
            current_version: self.current.to_string(),
            version: version.to_string(),
            date: None,
            target: self.target.clone(),
            // Set when it installs: only a copy that can replace itself has one.
            extract_path: PathBuf::new(),
            download_url: said(platform, "url")?
                .parse()
                .map_err(|_| strings::manifest_lacks("url"))?,
            signature: said(platform, "signature")?,
            timeout: Some(DOWNLOAD_TIMEOUT),
            headers: HeaderMap::new(),
            format,
        }))
    }

    /// Downloads what the manifest offers and gives it back only when it is the file
    /// the update key signed for that version and this platform.
    ///
    /// The manifest itself is not signed. The signature covers the bytes and a comment
    /// that names the file, `Mortise-vX.Y.Z-<platform>`. So a manifest that calls an
    /// older signed file a newer version is refused: the name does not fit.
    fn verified(&self, progress: &dyn Fn(f32)) -> Result<(Update, Vec<u8>), String> {
        // Asked again, so the address and the signature are those of this moment.
        let update = match self.newer() {
            Ok(Some(update)) => update,
            Ok(None) => return Err(strings::NOT_AVAILABLE.to_owned()),
            Err(error) => return Err(strings::could_not_check(&error)),
        };
        let wanted = format!("Mortise-v{}-{}", update.version, self.artifact);
        if signed_name(&update.signature).as_deref() != Some(&wanted) {
            return Err(strings::BAD_SIGNATURE.to_owned());
        }
        let got = AtomicU64::new(0);
        let on_chunk = |chunk: usize, total: Option<u64>| {
            let got = got.fetch_add(chunk as u64, Ordering::Relaxed) + chunk as u64;
            if let Some(total) = total.filter(|total| *total > 0) {
                progress(got as f32 / total as f32);
            }
        };
        // `download_extended` returns the bytes only when the signature, which covers
        // the comment too, is that of the key.
        match update.download_extended(on_chunk, || {}) {
            Ok(bytes) => Ok((update, bytes)),
            Err(Error::Minisign(_) | Error::Base64(_) | Error::SignatureUtf8(_)) => {
                Err(strings::BAD_SIGNATURE.to_owned())
            }
            Err(error) => Err(strings::could_not_download(&error.to_string())),
        }
    }
}

/// The file a signature says it signed, from its trusted comment
/// `timestamp:<seconds>\tfile:<name>`. Trusted only once the signature verifies.
fn signed_name(signature: &str) -> Option<String> {
    let text = base64::engine::general_purpose::STANDARD
        .decode(signature)
        .ok()?;
    let signature = minisign_verify::Signature::decode(&String::from_utf8(text).ok()?).ok()?;
    signature
        .trusted_comment()
        .split('\t')
        .find_map(|part| part.strip_prefix("file:"))
        .map(str::to_owned)
}

impl Channel for Release {
    fn check(&self) -> Result<Option<String>, String> {
        match self.newer() {
            Ok(update) => Ok(update.map(|update| update.version)),
            Err(error) => Err(strings::could_not_check(&error)),
        }
    }

    fn is_newer(&self, version: &str) -> bool {
        version
            .parse::<Version>()
            .is_ok_and(|version| version > self.current)
    }

    fn can_install(&self) -> bool {
        place().is_some()
    }

    fn download(&self, progress: &dyn Fn(f32)) -> Result<Install, String> {
        let Some(place) = place() else {
            return Err(strings::NOT_AVAILABLE.to_owned());
        };
        let (mut update, bytes) = self.verified(progress)?;
        update.extract_path = place;
        Ok(Box::new(move || {
            install(&update, bytes).map_err(|error| strings::could_not_install(&error))
        }))
    }

    fn restart(&self) -> Result<(), String> {
        let Some(place) = place() else {
            return Err(strings::INSTALLED_RESTART.to_owned());
        };
        // On Windows the setup starts Mortise again itself.
        let started = if cfg!(target_os = "macos") {
            std::process::Command::new("open")
                .arg("-n")
                .arg(place)
                .spawn()
        } else if cfg!(target_os = "linux") {
            std::process::Command::new(place).spawn()
        } else {
            return Ok(());
        };
        match started {
            Ok(_) => Ok(()),
            Err(_) => Err(strings::INSTALLED_RESTART.to_owned()),
        }
    }
}

/// Puts a verified download in place: the `.app` on macOS, the AppImage on Linux.
#[cfg(not(windows))]
fn install(update: &Update, bytes: Vec<u8>) -> Result<(), String> {
    update.install(bytes).map_err(|error| error.to_string())
}

/// Starts the verified setup of the newer version: `/P` shows its progress only, `/R`
/// starts Mortise again when it is done. The window then closes the usual way.
///
/// `cargo-packager-updater` starts the setup through a hidden PowerShell and ends the
/// process at once, so a setup that cannot start is never seen. Mortise starts the
/// setup itself and hears when that fails. The setup installs for the current user
/// and needs no administrator.
#[cfg(windows)]
fn install(_: &Update, bytes: Vec<u8>) -> Result<(), String> {
    use std::io::Write as _;
    let said = |error: std::io::Error| error.to_string();
    let mut setup = tempfile::Builder::new()
        .prefix("mortise-update-")
        .suffix(".exe")
        .tempfile()
        .map_err(said)?;
    setup.write_all(&bytes).map_err(said)?;
    // The file stays: the setup runs after Mortise is gone.
    let (file, path) = setup.keep().map_err(|error| error.to_string())?;
    drop(file);
    std::process::Command::new(path)
        .args(["/P", "/R"])
        .spawn()
        .map(|_| ())
        .map_err(said)
}

/// What an update replaces and starts again, when this copy can replace itself.
///
/// - macOS: the `.app` folder, in a folder the user can write. Not an app that macOS
///   runs from a random read-only place because it was never moved out of its download
///   folder. Not an app in a folder that needs an administrator: the install of
///   `cargo-packager-updater` would then put paths into a shell command that runs as
///   administrator, so such a copy gets the download page. For the same reason, not a
///   path with a quote or a backslash in it.
/// - Linux: the AppImage, in a folder the user can write. A `.deb` install belongs to
///   the package manager.
/// - Windows: the program the setup installed, which has its uninstaller beside it.
///
/// A build run with `cargo run` is none of these.
fn place() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    if cfg!(target_os = "macos") {
        let app = executable.ancestors().nth(3)?;
        let is_app = app.extension().is_some_and(|extension| extension == "app");
        let moved_by_macos = app
            .components()
            .any(|part| part.as_os_str() == "AppTranslocation");
        let plain = !app.to_string_lossy().contains(['\'', '"', '\\']);
        let writable = tempfile::tempfile_in(app.parent()?).is_ok();
        (is_app && !moved_by_macos && plain && writable).then(|| app.to_owned())
    } else if cfg!(target_os = "linux") {
        let image = PathBuf::from(std::env::var_os("APPIMAGE")?);
        let writable = tempfile::tempfile_in(image.parent()?).is_ok();
        (image.is_file() && writable).then_some(image)
    } else if cfg!(target_os = "windows") {
        let installed = executable.parent()?.join("uninstall.exe").is_file();
        installed.then_some(executable)
    } else {
        None
    }
}

/// Where an update is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    /// Nothing is known.
    Idle,
    Checking,
    /// A check the user asked for found nothing newer.
    Latest,
    /// This version is newer.
    Available(String),
    Downloading(String),
    /// Downloaded and checked. It installs once no command runs in an app.
    Waiting(String),
    Installing(String),
    /// What went wrong, as the user reads it.
    Failed(String),
}

/// What the window does after a frame of updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// A check ended: keep when, and the newer version if there is one.
    Checked { newer: Option<String>, asked: bool },
    /// Something failed and the user must read it.
    Failed(String),
    /// The newer version is installed and starting: close the window.
    Restart,
}

enum Event {
    Checked(Result<Option<String>, String>),
    Downloaded(Result<Install, String>),
    Installed(Result<(), String>),
}

/// The update of this window: one check or one install at a time, off the frame.
pub struct Updates {
    channel: Arc<dyn Channel>,
    phase: Phase,
    /// The user asked for the check that runs.
    asked: bool,
    /// The share of the download, as the bits of an `f32`.
    share: Arc<AtomicU32>,
    /// A checked download that waits for the commands to end.
    ready: Option<Install>,
    /// The thread that replaces the app. Mortise does not end before it does.
    installing: Option<JoinHandle<()>>,
    /// Where the update was before the check that runs.
    before: Phase,
    events: Receiver<Event>,
    sender: Sender<Event>,
}

impl Updates {
    /// `seen` is the newer version an earlier check found, if any.
    pub fn new(channel: Arc<dyn Channel>, seen: Option<&str>) -> Self {
        let (sender, events) = std::sync::mpsc::channel();
        let phase = match seen.filter(|version| channel.is_newer(version)) {
            Some(version) => Phase::Available(version.to_owned()),
            None => Phase::Idle,
        };
        Self {
            channel,
            phase,
            asked: false,
            share: Arc::default(),
            ready: None,
            installing: None,
            before: Phase::Idle,
            events,
            sender,
        }
    }

    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    pub fn can_install(&self) -> bool {
        self.channel.can_install()
    }

    /// The share of the download that is here, 0 to 1.
    pub fn share(&self) -> f32 {
        f32::from_bits(self.share.load(Ordering::Relaxed))
    }

    /// Whether a check can start now.
    pub fn can_check(&self) -> bool {
        matches!(
            self.phase,
            Phase::Idle | Phase::Latest | Phase::Available(_) | Phase::Failed(_)
        )
    }

    /// Asks for the newest version. `asked` says the user pressed the button, so the
    /// answer is shown also when it is "nothing newer" or a failure.
    pub fn check(&mut self, ctx: &egui::Context, asked: bool) {
        if !self.can_check() {
            return;
        }
        self.before = std::mem::replace(&mut self.phase, Phase::Checking);
        self.asked = asked;
        let channel = Arc::clone(&self.channel);
        let _ = self.work(ctx, move || Event::Checked(channel.check()));
    }

    /// Downloads the newer version. It installs in `poll`.
    pub fn install(&mut self, ctx: &egui::Context) {
        let Phase::Available(version) = &self.phase else {
            return;
        };
        if !self.channel.can_install() {
            return;
        }
        self.phase = Phase::Downloading(version.clone());
        self.share.store(0, Ordering::Relaxed);
        let channel = Arc::clone(&self.channel);
        let share = Arc::clone(&self.share);
        let repaint = ctx.clone();
        let _ = self.work(ctx, move || {
            Event::Downloaded(channel.download(&|part| {
                share.store(part.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
                repaint.request_repaint();
            }))
        });
    }

    fn work(
        &self,
        ctx: &egui::Context,
        work: impl FnOnce() -> Event + Send + 'static,
    ) -> JoinHandle<()> {
        let sender = self.sender.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = sender.send(work());
            ctx.request_repaint();
        })
    }

    /// True while the app is being replaced. No command may start then: Mortise
    /// closes when it is done.
    pub fn is_installing(&self) -> bool {
        matches!(self.phase, Phase::Installing(_))
    }

    /// Waits until an install that runs has put the app in place. Mortise calls it
    /// on its way out, so that closing the window cannot leave half an app.
    pub fn finish(&mut self) {
        if let Some(installing) = self.installing.take() {
            let _ = installing.join();
        }
    }

    /// Takes what ended since the last frame. `busy` says a command runs in an app:
    /// a download waits for its end, because the install closes Mortise.
    pub fn poll(&mut self, ctx: &egui::Context, busy: bool) -> Vec<Outcome> {
        let mut outcomes = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            let asked = std::mem::take(&mut self.asked);
            match (event, self.phase.clone()) {
                (Event::Checked(Ok(newer)), _) => {
                    self.phase = match &newer {
                        Some(version) => Phase::Available(version.clone()),
                        None if asked => Phase::Latest,
                        None => Phase::Idle,
                    };
                    outcomes.push(Outcome::Checked { newer, asked });
                }
                // A check nobody asked for fails in silence, and what was known stays.
                (Event::Checked(Err(_)), _) if !asked => {
                    self.phase = std::mem::replace(&mut self.before, Phase::Idle);
                }
                (Event::Checked(Err(error)), _) => self.fail(error, &mut outcomes),
                (Event::Downloaded(Ok(install)), Phase::Downloading(version)) => {
                    self.ready = Some(install);
                    self.phase = Phase::Waiting(version);
                }
                (Event::Installed(Ok(())), _) => match self.channel.restart() {
                    Ok(()) => outcomes.push(Outcome::Restart),
                    Err(error) => self.fail(error, &mut outcomes),
                },
                (Event::Downloaded(Err(error)) | Event::Installed(Err(error)), _) => {
                    self.fail(error, &mut outcomes);
                }
                (Event::Downloaded(Ok(_)), _) => {}
            }
        }
        if let (false, Phase::Waiting(version)) = (busy, self.phase.clone())
            && let Some(install) = self.ready.take()
        {
            self.phase = Phase::Installing(version);
            // Off the frame: on macOS the install can ask for an administrator.
            self.installing = Some(self.work(ctx, move || Event::Installed(install())));
        }
        outcomes
    }

    fn fail(&mut self, error: String, outcomes: &mut Vec<Outcome>) {
        self.phase = Phase::Failed(error.clone());
        outcomes.push(Outcome::Failed(error));
    }
}

#[cfg(test)]
mod tests {
    use mortise_core::testing::{Reply, StubActuator};

    use super::*;

    // A key made for this test with `cargo packager signer generate`, and the signature
    // of `PAYLOAD` under the name `Mortise-v9.0.0-test` by it. The private half was thrown away.
    const TEST_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDYwRjdGRDAxQ0NCNjk1NzUKUldSMWxiYk1BZjMzWUNzdWJCcVR0OE1vZzVuYThneS9sc1lGSktxV04vQlR6THd5LzhvTnd3RmoK";
    const SIGNATURE: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIGNhcmdvLXBhY2thZ2VyIHNlY3JldCBrZXkKUlVSMWxiYk1BZjMzWUlJUG9tREhJdWtEK0JGbmhxWm1sSWZaZk5xLzY4Vy9vaDQyWkl4NHhxdVJJekpYQnJ0cEN3WlRFOTY1M1hhN1lCOVgwTXFobmJudkgxNWZ5VVZWWndJPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzk1ODYyNjk0CWZpbGU6TW9ydGlzZS12OS4wLjAtdGVzdApQelZweHQrM0pvanhlYlRPd3lkT1dKS1F6OUQ3YkhWb1NFRTFlWWpJM21GV21jVG01SW4wSyt2b3FTaTNwZEg3THpubGZSeVd6emtJdTczRktNamJBUT09Cg==";
    const PAYLOAD: &str = "a signed update of Mortise";

    /// A release server that offers `version` with the signature of 9.0.0.
    fn release(server: &StubActuator, current: &str, version: &str) -> Release {
        let manifest = format!(
            r#"{{"version":"{version}","platforms":{{"test":{{"url":"{}/update","signature":"{SIGNATURE}","format":"appimage"}}}}}}"#,
            server.base_url
        );
        server.set("/latest.json", Reply::ok(&manifest));
        Release {
            manifest: format!("{}/latest.json", server.base_url),
            public_key: TEST_KEY.to_owned(),
            current: current.parse().unwrap(),
            target: "test".to_owned(),
            artifact: "test",
        }
    }

    #[test]
    fn a_download_is_kept_only_with_the_signature_of_the_key_for_that_version() {
        let server = StubActuator::start();
        let offered = release(&server, "1.0.0", "9.0.0");
        assert_eq!(offered.check(), Ok(Some("9.0.0".to_owned())));
        assert!(offered.is_newer("9.0.0") && !offered.is_newer("1.0.0"));

        server.set("/update", Reply::ok(PAYLOAD));
        assert!(offered.verified(&|_| {}).is_ok());

        server.set("/update", Reply::ok("something else under the same name"));
        let refused = offered.verified(&|_| {}).err();
        assert_eq!(refused.as_deref(), Some(strings::BAD_SIGNATURE));

        // A manifest that calls the signed 9.0.0 a newer version: the name does not fit.
        server.set("/update", Reply::ok(PAYLOAD));
        let renamed = release(&server, "9.2.0", "9.5.0").verified(&|_| {}).err();
        assert_eq!(renamed.as_deref(), Some(strings::BAD_SIGNATURE));

        // The copy that already is the newest gets nothing.
        assert_eq!(release(&server, "9.0.0", "9.0.0").check(), Ok(None));
    }

    /// Releases whose download is ready at once and counts its installs.
    struct Ready(Arc<AtomicU32>);

    impl Channel for Ready {
        fn check(&self) -> Result<Option<String>, String> {
            Ok(Some("9.0.0".to_owned()))
        }
        fn is_newer(&self, _: &str) -> bool {
            true
        }
        fn can_install(&self) -> bool {
            true
        }
        fn download(&self, _: &dyn Fn(f32)) -> Result<Install, String> {
            let installs = Arc::clone(&self.0);
            Ok(Box::new(move || {
                installs.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }))
        }
        fn restart(&self) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn a_download_installs_only_once_no_command_runs() {
        let ctx = egui::Context::default();
        let installs = Arc::new(AtomicU32::new(0));
        let mut updates = Updates::new(Arc::new(Ready(Arc::clone(&installs))), Some("9.0.0"));
        updates.install(&ctx);

        // A command runs: the download waits, however long.
        let mut outcomes = Vec::new();
        while *updates.phase() != Phase::Waiting("9.0.0".to_owned()) {
            outcomes.extend(updates.poll(&ctx, true));
            std::thread::yield_now();
        }
        assert!(updates.poll(&ctx, true).is_empty());
        assert_eq!(installs.load(Ordering::SeqCst), 0);

        // The command ended.
        while !outcomes.contains(&Outcome::Restart) {
            outcomes.extend(updates.poll(&ctx, false));
            std::thread::yield_now();
        }
        assert_eq!(installs.load(Ordering::SeqCst), 1);
    }
}
