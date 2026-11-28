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
use std::time::Duration;

use cargo_packager_updater::semver::Version;
use cargo_packager_updater::url::Url;
use cargo_packager_updater::{Config, Error, Update, UpdaterBuilder};
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
    /// place that cannot be written cannot.
    fn can_install(&self) -> bool;
    /// Downloads the newer version and checks its signature. `progress` gets the
    /// share that is downloaded, 0 to 1.
    fn download(&self, progress: &dyn Fn(f32)) -> Result<Install, String>;
    /// Starts the installed version. The caller then closes this one.
    fn restart(&self) -> Result<(), String>;
}

/// The releases of Mortise on GitHub.
pub struct Release {
    manifest: Url,
    public_key: String,
    current: Version,
}

impl Release {
    /// The releases this build updates from.
    pub fn of_this_build() -> Self {
        Self {
            manifest: MANIFEST.parse().expect("the manifest address is valid"),
            public_key: PUBLIC_KEY.to_owned(),
            current: env!("CARGO_PKG_VERSION")
                .parse()
                .expect("the version of the crate is valid"),
        }
    }

    fn newer(&self) -> Result<Option<Update>, Error> {
        let config = Config {
            endpoints: vec![self.manifest.clone()],
            pubkey: self.public_key.clone(),
            ..Default::default()
        };
        UpdaterBuilder::new(self.current.clone(), config)
            .timeout(CHECK_TIMEOUT)
            .build()?
            .check()
    }
}

impl Channel for Release {
    fn check(&self) -> Result<Option<String>, String> {
        match self.newer() {
            Ok(update) => Ok(update.map(|update| update.version)),
            Err(error) => Err(strings::could_not_check(&error.to_string())),
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
        // Asked again, so the address and the signature are those of this moment.
        let mut update = match self.newer() {
            Ok(Some(update)) => update,
            Ok(None) => return Err(strings::NOT_AVAILABLE.to_owned()),
            Err(error) => return Err(strings::could_not_check(&error.to_string())),
        };
        update.timeout = Some(DOWNLOAD_TIMEOUT);
        let got = AtomicU64::new(0);
        let on_chunk = |chunk: usize, total: Option<u64>| {
            let got = got.fetch_add(chunk as u64, Ordering::Relaxed) + chunk as u64;
            if let Some(total) = total.filter(|total| *total > 0) {
                progress(got as f32 / total as f32);
            }
        };
        // `download_extended` returns the bytes only when their signature is that of the key.
        let bytes = match update.download_extended(on_chunk, || {}) {
            Ok(bytes) => bytes,
            Err(Error::Minisign(_) | Error::Base64(_) | Error::SignatureUtf8(_)) => {
                return Err(strings::BAD_SIGNATURE.to_owned());
            }
            Err(error) => return Err(strings::could_not_download(&error.to_string())),
        };
        Ok(Box::new(move || {
            update
                .install(bytes)
                .map_err(|error| strings::could_not_install(&error.to_string()))
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

/// What an update replaces and starts again, when this copy can replace itself.
///
/// - macOS: the `.app` folder. Not an app that macOS runs from a random read-only
///   place because it was never moved out of its download folder, and not an app on
///   a volume that cannot be written. A folder that needs an administrator is fine:
///   the install asks for one.
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
        let folder = app.parent()?;
        let writable = match tempfile::tempfile_in(folder) {
            Ok(_) => true,
            Err(error) => error.kind() == std::io::ErrorKind::PermissionDenied,
        };
        (is_app && !moved_by_macos && writable).then(|| app.to_owned())
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
        self.phase = Phase::Checking;
        self.asked = asked;
        let channel = Arc::clone(&self.channel);
        self.work(ctx, move || Event::Checked(channel.check()));
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
        self.work(ctx, move || {
            Event::Downloaded(channel.download(&|part| {
                share.store(part.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
                repaint.request_repaint();
            }))
        });
    }

    fn work(&self, ctx: &egui::Context, work: impl FnOnce() -> Event + Send + 'static) {
        let sender = self.sender.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = sender.send(work());
            ctx.request_repaint();
        });
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
                // A check nobody asked for fails in silence.
                (Event::Checked(Err(_)), _) if !asked => self.phase = Phase::Idle,
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
            self.work(ctx, move || Event::Installed(install()));
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
    // of `PAYLOAD` by it. The private half was thrown away.
    const TEST_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDYwRjdGRDAxQ0NCNjk1NzUKUldSMWxiYk1BZjMzWUNzdWJCcVR0OE1vZzVuYThneS9sc1lGSktxV04vQlR6THd5LzhvTnd3RmoK";
    const SIGNATURE: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIGNhcmdvLXBhY2thZ2VyIHNlY3JldCBrZXkKUlVSMWxiYk1BZjMzWVBOalZKSU5Yd1ZrS09BRFdxU1VaWmpFWDR3WDhYV044MElIVXh5WmRvK3BvaVJtYVN5bXRyV21LRTQwaUNJZFV1MmxiWTBKZWR3aENSU3BMeTVZZ1FzPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzk1ODYwODg0CWZpbGU6cGF5bG9hZC5BcHBJbWFnZQpkOXQ0bktmdmVXTDB3eVdxMHlHcHBOaG0rL25nbnBzdGJCdjJzbVkzMVN0Nk1JQUZQbEdpMUtZYUp3bkxGcWhYTTR5eVVQZEQxb21VakNlQUJrck1BQT09Cg==";
    const PAYLOAD: &str = "a signed update of Mortise";

    /// A release server that offers 9.0.0 with the signature of `PAYLOAD`.
    fn release(server: &StubActuator, current: &str) -> Release {
        let manifest = format!(
            r#"{{"version":"9.0.0","url":"{}/update","signature":"{SIGNATURE}","format":"appimage"}}"#,
            server.base_url
        );
        server.set("/latest.json", Reply::ok(&manifest));
        Release {
            manifest: format!("{}/latest.json", server.base_url).parse().unwrap(),
            public_key: TEST_KEY.to_owned(),
            current: current.parse().unwrap(),
        }
    }

    #[test]
    fn a_download_is_kept_only_with_the_signature_of_the_key() {
        let server = StubActuator::start();
        let release = release(&server, "1.0.0");
        assert_eq!(release.check(), Ok(Some("9.0.0".to_owned())));
        assert!(release.is_newer("9.0.0") && !release.is_newer("1.0.0"));

        server.set("/update", Reply::ok(PAYLOAD));
        assert!(release.download(&|_| {}).is_ok());

        server.set("/update", Reply::ok("something else under the same name"));
        let refused = release.download(&|_| {}).err();
        assert_eq!(refused.as_deref(), Some(strings::BAD_SIGNATURE));

        // The copy that already is the newest gets nothing.
        assert_eq!(self::release(&server, "9.0.0").check(), Ok(None));
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
