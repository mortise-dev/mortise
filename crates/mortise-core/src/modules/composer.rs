//! Versions from `composer.lock`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A package version as `composer.lock` records it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    /// The `version` field, unchanged: `v13.38.0`, `1.0.0`, `dev-master`, `12.x-dev`.
    pub raw: String,
    /// The branch alias of a dev version, when the package declares one: `13.x-dev`.
    pub alias: Option<String>,
}

impl Version {
    /// The version without the `v` that tags often carry: `13.38.0`. A dev version is unchanged.
    pub fn full(&self) -> &str {
        match self.raw.strip_prefix(['v', 'V']) {
            Some(rest) if rest.starts_with(|c: char| c.is_ascii_digit()) => rest,
            _ => &self.raw,
        }
    }

    /// Major and minor, `13.38`. A dev version is unchanged.
    pub fn short(&self) -> &str {
        let full = self.full();
        if !full.starts_with(|c: char| c.is_ascii_digit()) {
            return full;
        }
        match full.match_indices('.').nth(1) {
            Some((second_dot, _))
                if full[..second_dot]
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '.') =>
            {
                &full[..second_dot]
            }
            _ => full,
        }
    }
}

#[derive(Deserialize)]
struct LockFile {
    #[serde(default)]
    packages: Vec<LockPackage>,
    #[serde(default, rename = "packages-dev")]
    packages_dev: Vec<LockPackage>,
}

#[derive(Deserialize)]
struct LockPackage {
    name: String,
    version: String,
    #[serde(default)]
    extra: Option<LockExtra>,
}

#[derive(Deserialize)]
struct LockExtra {
    #[serde(default, rename = "branch-alias")]
    branch_alias: Option<BTreeMap<String, String>>,
}

/// The packages of a `composer.lock`, from `packages` and `packages-dev`.
pub(crate) struct Lock {
    versions: BTreeMap<String, Version>,
}

impl Lock {
    pub(crate) fn parse(json: &[u8]) -> Result<Self, serde_json::Error> {
        let file: LockFile = serde_json::from_slice(json)?;
        let versions = file
            .packages
            .into_iter()
            .chain(file.packages_dev)
            .map(|package| {
                let alias = package
                    .extra
                    .and_then(|extra| extra.branch_alias)
                    .and_then(|mut aliases| aliases.remove(&package.version));
                // Composer package names are case insensitive.
                (
                    package.name.to_ascii_lowercase(),
                    Version {
                        raw: package.version,
                        alias,
                    },
                )
            })
            .collect();
        Ok(Self { versions })
    }

    pub(crate) fn version(&self, package: &str) -> Option<Version> {
        self.versions.get(package).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(raw: &str) -> Version {
        Version {
            raw: raw.into(),
            alias: None,
        }
    }

    #[test]
    fn drops_the_v_of_a_tag_and_keeps_dev_versions() {
        assert_eq!(version("v13.38.0").full(), "13.38.0");
        assert_eq!(version("1.0.0").full(), "1.0.0");
        assert_eq!(version("dev-master").full(), "dev-master");
        assert_eq!(version("12.x-dev").full(), "12.x-dev");
        // Not a tag: a branch whose name starts with v.
        assert_eq!(version("vnext").full(), "vnext");
    }

    #[test]
    fn short_is_major_and_minor() {
        assert_eq!(version("v13.38.0").short(), "13.38");
        assert_eq!(version("13.2.0").short(), "13.2");
        assert_eq!(version("13.2").short(), "13.2");
        assert_eq!(version("13.2.0-beta.1").short(), "13.2");
        assert_eq!(version("dev-master").short(), "dev-master");
        assert_eq!(version("12.x-dev").short(), "12.x-dev");
    }

    #[test]
    fn reads_packages_and_dev_packages_with_the_branch_alias() {
        let lock = Lock::parse(
            br#"{
                "packages": [
                    {"name": "laravel/framework", "version": "v13.38.0",
                     "extra": {"branch-alias": {"dev-master": "13.0.x-dev"}}},
                    {"name": "nwidart/laravel-modules", "version": "dev-master",
                     "extra": {"branch-alias": {"dev-master": "13.x-dev"}}}
                ],
                "packages-dev": [
                    {"name": "Nwidart/Laravel-Actuator", "version": "1.0.0"}
                ]
            }"#,
        )
        .unwrap();

        let laravel = lock.version("laravel/framework").unwrap();
        assert_eq!((laravel.raw.as_str(), laravel.alias), ("v13.38.0", None));

        let package = lock.version("nwidart/laravel-modules").unwrap();
        assert_eq!(package.raw, "dev-master");
        assert_eq!(package.alias.as_deref(), Some("13.x-dev"));

        assert_eq!(
            lock.version("nwidart/laravel-actuator").unwrap().raw,
            "1.0.0"
        );
        assert_eq!(lock.version("laravel/sail"), None);
    }

    #[test]
    fn a_lock_without_packages_is_empty_not_an_error() {
        assert!(
            Lock::parse(b"{}")
                .unwrap()
                .version("laravel/framework")
                .is_none()
        );
        assert!(Lock::parse(b"not json").is_err());
    }
}
