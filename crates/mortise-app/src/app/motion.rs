//! What the operating system says about motion. Each platform is asked in safe Rust.

/// True when the operating system is set to reduce motion.
///
/// macOS answers through AppKit.
#[cfg(target_os = "macos")]
pub(super) fn system_reduces_motion() -> bool {
    objc2_app_kit::NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion()
}

/// True when the operating system is set to reduce motion.
///
/// Windows answers through `UISettings.AnimationsEnabled`, the switch
/// "Animation effects" in Settings > Accessibility > Visual effects.
/// A failed call counts as full motion.
#[cfg(target_os = "windows")]
pub(super) fn system_reduces_motion() -> bool {
    windows::UI::ViewManagement::UISettings::new()
        .and_then(|settings| settings.AnimationsEnabled())
        .is_ok_and(|enabled| !enabled)
}

/// True when the operating system is set to reduce motion.
///
/// Linux answers through the Settings interface of the XDG desktop portal.
/// A thread reads it once and then follows its changes, so the window never
/// waits on D-Bus. Without a session bus or a portal, motion stays full.
#[cfg(target_os = "linux")]
pub(super) fn system_reduces_motion() -> bool {
    portal::reduces_motion()
}

/// True when the operating system is set to reduce motion.
///
/// No other platform is asked: motion stays full.
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub(super) fn system_reduces_motion() -> bool {
    false
}

#[cfg(target_os = "linux")]
mod portal {
    use std::sync::Once;
    use std::sync::atomic::{AtomicBool, Ordering};

    use zbus::blocking::{Connection, Proxy};
    use zbus::zvariant::{OwnedValue, Value};

    /// The key of the portal itself (xdg-desktop-portal 1.21 and later).
    const APPEARANCE: (&str, &str) = ("org.freedesktop.appearance", "reduced-motion");
    /// The key of GNOME, which older portals pass on.
    const GNOME: (&str, &str) = ("org.gnome.desktop.interface", "enable-animations");

    static REDUCED: AtomicBool = AtomicBool::new(false);
    static WATCH: Once = Once::new();

    pub(super) fn reduces_motion() -> bool {
        WATCH.call_once(|| {
            // A thread that cannot start leaves motion full.
            let _ = std::thread::Builder::new()
                .name("mortise-motion".to_owned())
                .spawn(|| {
                    let _ = watch();
                });
        });
        REDUCED.load(Ordering::Relaxed)
    }

    /// Reads both keys, then follows their changes until the bus closes.
    fn watch() -> zbus::Result<()> {
        let connection = Connection::session()?;
        let settings = Proxy::new(
            &connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Settings",
        )?;
        // Listen first, so that a change between the read and the listen is not lost.
        let changes = settings.receive_signal("SettingChanged")?;
        let read = |(namespace, key): (&str, &str)| {
            settings
                .call::<_, _, OwnedValue>("Read", &(namespace, key))
                .ok()
                .and_then(|value| reduces(namespace, key, &value))
        };
        let mut state = State {
            appearance: read(APPEARANCE),
            gnome: read(GNOME),
        };
        REDUCED.store(state.reduced(), Ordering::Relaxed);

        for message in changes {
            let Ok((namespace, key, value)) =
                message.body().deserialize::<(String, String, OwnedValue)>()
            else {
                continue;
            };
            let found = reduces(&namespace, &key, &value);
            match (namespace.as_str(), key.as_str()) {
                APPEARANCE => state.appearance = found,
                GNOME => state.gnome = found,
                _ => continue,
            }
            REDUCED.store(state.reduced(), Ordering::Relaxed);
        }
        Ok(())
    }

    /// What each key said last. The key of the portal wins over the key of GNOME.
    struct State {
        appearance: Option<bool>,
        gnome: Option<bool>,
    }

    impl State {
        fn reduced(&self) -> bool {
            self.appearance.or(self.gnome).unwrap_or(false)
        }
    }

    /// Whether a value of one of the two keys asks for less motion.
    /// `reduced-motion` is 1 for "reduce" and 0 for "no preference";
    /// `enable-animations` is false when animations are off.
    fn reduces(namespace: &str, key: &str, value: &Value<'_>) -> Option<bool> {
        // `Read` wraps the value in one more variant than `SettingChanged` does.
        if let Value::Value(inner) = value {
            return reduces(namespace, key, inner);
        }
        match ((namespace, key), value) {
            (APPEARANCE, Value::U32(preference)) => Some(*preference == 1),
            (GNOME, Value::Bool(enabled)) => Some(!enabled),
            _ => None,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn reads_both_keys_and_the_portal_wins() {
            let (ns, key) = APPEARANCE;
            assert_eq!(reduces(ns, key, &Value::U32(1)), Some(true));
            assert_eq!(reduces(ns, key, &Value::U32(0)), Some(false));
            let wrapped = Value::Value(Box::new(Value::U32(1)));
            assert_eq!(reduces(ns, key, &wrapped), Some(true));
            let (ns, key) = GNOME;
            assert_eq!(reduces(ns, key, &Value::Bool(false)), Some(true));
            assert_eq!(reduces(ns, key, &Value::U32(1)), None);

            let state = State {
                appearance: Some(false),
                gnome: Some(true),
            };
            assert!(!state.reduced());
            let state = State {
                appearance: None,
                gnome: Some(true),
            };
            assert!(state.reduced());
        }
    }
}
