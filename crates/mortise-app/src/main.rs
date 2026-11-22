// On Windows, a release build opens no console window.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::path::PathBuf;

use mortise_app::chrome::Chrome;

fn main() -> eframe::Result {
    // Until apps can be added in the window, the app folder is the first argument.
    let folder = std::env::args_os().nth(1).map(PathBuf::from);

    // For looking at the frame of another platform while developing.
    let chrome = match std::env::var("MORTISE_CHROME").as_deref() {
        Ok("mac") => Chrome::Mac,
        Ok("custom") => Chrome::Custom {
            controls_left: false,
        },
        Ok("custom-left") => Chrome::Custom {
            controls_left: true,
        },
        Ok("system") => Chrome::System,
        _ => Chrome::native(),
    };

    mortise_app::app::run(chrome, folder)
}
