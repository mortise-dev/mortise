//! The text of updates. None of it is in the spec.

pub const UPDATES: &str = "Updates";
pub const UPDATES_HINT: &str =
    "asks GitHub for the newest version, at most once a day; nothing of your apps is sent";
pub const CHECK_CHOICES: &[&str] = &["At start", "Off"];
pub const CHECK_NOW: &str = "Check now";
pub const INSTALL_AND_RESTART: &str = "Install and restart";
pub const OPEN_DOWNLOAD_PAGE: &str = "Open the download page";
pub const CHECKING: &str = "Checking for a newer version.";
pub const LATEST: &str = "This is the newest version.";
pub const WAITING: &str = "Downloaded. It installs when the running command ends.";
pub const INSTALLING: &str = "Installing. Mortise restarts when it is done.";
pub const NOT_AVAILABLE: &str =
    "The newer version is no longer offered. Check again in a few minutes.";
pub const BAD_SIGNATURE: &str = "The download does not carry the signature of Mortise, so it was not installed. \
     Download Mortise from the download page instead.";
pub const INSTALLED_RESTART: &str =
    "The newer version is installed. Close Mortise and open it again.";

/// `Mortise 1.2.0 is available.`
pub fn available(version: &str) -> String {
    format!("Mortise {version} is available.")
}

/// `Mortise 1.2.0 is available. This copy cannot replace itself: get it from the download page.`
pub fn available_by_hand(version: &str) -> String {
    format!(
        "Mortise {version} is available. This copy cannot replace itself: get it from the download page."
    )
}

/// `Downloading Mortise 1.2.0`
pub fn downloading(version: &str) -> String {
    format!("Downloading Mortise {version}")
}

/// `Downloading Mortise 1.2.0. Mortise restarts when it is installed.`
pub fn downloading_notice(version: &str) -> String {
    format!("Downloading Mortise {version}. Mortise restarts when it is installed.")
}

/// `Could not check for a newer version: ... Check your connection, then check again.`
pub fn could_not_check(error: &str) -> String {
    format!(
        "Could not check for a newer version: {error}. Check your connection, then check again."
    )
}

/// `Could not download the newer version: ... Check your connection, then try again.`
pub fn could_not_download(error: &str) -> String {
    format!("Could not download the newer version: {error}. Check your connection, then try again.")
}

/// `Could not install the newer version: ... Download Mortise from the download page instead.`
pub fn could_not_install(error: &str) -> String {
    format!(
        "Could not install the newer version: {error}. Download Mortise from the download page instead."
    )
}
