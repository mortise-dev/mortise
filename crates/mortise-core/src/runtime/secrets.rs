//! The secrets of the environments: a token, a password or a header value.
//!
//! A secret goes to the credential store of the operating system: Keychain,
//! Credential Manager, Secret Service. It is never written to a file. When no
//! store is running the secret is kept in memory, until Mortise closes.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Mutex;

/// The name Mortise's entries have in the credential store.
#[cfg(feature = "os-secrets")]
const SERVICE: &str = "Mortise";

/// Where a secret went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kept {
    /// In the credential store of this computer.
    Stored,
    /// No credential store is running: the secret is kept until Mortise closes.
    UntilClose,
}

/// The secrets Mortise keeps, by app folder and environment name.
pub struct Secrets {
    /// False for a store that is only in memory, which tests use.
    os: bool,
    /// What was read or written in this run. `None`: the store has no such secret.
    known: Mutex<BTreeMap<String, Option<String>>>,
}

impl Secrets {
    /// The credential store of the operating system.
    #[cfg(feature = "os-secrets")]
    pub fn os() -> Self {
        Self {
            os: true,
            known: Mutex::default(),
        }
    }

    /// A store that forgets everything when it is dropped.
    pub fn memory() -> Self {
        Self {
            os: false,
            known: Mutex::default(),
        }
    }

    /// Keeps a secret, in place of the one that was there.
    pub fn set(&self, root: &Path, environment: &str, secret: &str) -> Kept {
        let key = key(root, environment);
        let kept = if !self.os || store_set(&key, secret) {
            Kept::Stored
        } else {
            Kept::UntilClose
        };
        self.known().insert(key, Some(secret.to_owned()));
        kept
    }

    /// The secret of an environment, when there is one.
    pub fn get(&self, root: &Path, environment: &str) -> Option<String> {
        let key = key(root, environment);
        if let Some(known) = self.known().get(&key) {
            return known.clone();
        }
        let secret = if self.os { store_get(&key) } else { None };
        self.known().insert(key, secret.clone());
        secret
    }

    /// True when the environment has a secret. The secret itself is not given out.
    pub fn has(&self, root: &Path, environment: &str) -> bool {
        self.get(root, environment).is_some()
    }

    /// Forgets the secret of an environment.
    pub fn remove(&self, root: &Path, environment: &str) {
        let key = key(root, environment);
        if self.os {
            store_remove(&key);
        }
        self.known().insert(key, None);
    }

    fn known(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, Option<String>>> {
        self.known.lock().expect("the secrets are never poisoned")
    }
}

/// The account name of an entry: the app folder and the environment.
fn key(root: &Path, environment: &str) -> String {
    format!("{}#{environment}", root.display())
}

#[cfg(feature = "os-secrets")]
fn store_set(key: &str, secret: &str) -> bool {
    keyring::Entry::new(SERVICE, key)
        .and_then(|entry| entry.set_password(secret))
        .is_ok()
}

#[cfg(feature = "os-secrets")]
fn store_get(key: &str) -> Option<String> {
    keyring::Entry::new(SERVICE, key)
        .and_then(|entry| entry.get_password())
        .ok()
}

#[cfg(feature = "os-secrets")]
fn store_remove(key: &str) {
    if let Ok(entry) = keyring::Entry::new(SERVICE, key) {
        let _ = entry.delete_credential();
    }
}

#[cfg(not(feature = "os-secrets"))]
fn store_set(_: &str, _: &str) -> bool {
    false
}

#[cfg(not(feature = "os-secrets"))]
fn store_get(_: &str) -> Option<String> {
    None
}

#[cfg(not(feature = "os-secrets"))]
fn store_remove(_: &str) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_secret_is_kept_by_app_and_environment() {
        let secrets = Secrets::memory();
        let shop = Path::new("/code/shop-api");
        assert!(!secrets.has(shop, "staging"));
        assert_eq!(secrets.set(shop, "staging", "s3cret"), Kept::Stored);
        assert_eq!(secrets.get(shop, "staging").as_deref(), Some("s3cret"));
        assert_eq!(secrets.get(shop, "local"), None);
        assert_eq!(secrets.get(Path::new("/code/crm"), "staging"), None);
        secrets.remove(shop, "staging");
        assert!(!secrets.has(shop, "staging"));
    }
}
