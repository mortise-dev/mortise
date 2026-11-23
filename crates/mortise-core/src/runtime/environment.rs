//! A running copy of an app, and what Mortise needs to ask it.
//!
//! Nothing here is a secret: the kind of credentials, a user name and a header name
//! are kept in Mortise's settings, the secret itself in the credential store.

use serde::{Deserialize, Serialize};

/// What the app's middleware asks for.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Credentials {
    /// Mortise sends no credentials. Right for a local app whose actuator is open.
    #[default]
    None,
    /// A token, sent as `Authorization: Bearer`.
    Bearer,
    /// HTTP basic. The user name is not a secret.
    Basic { user: String },
    /// A header with a name and a value. The name is not a secret.
    Header { name: String },
}

impl Credentials {
    /// True when the kind needs a secret from the credential store.
    pub fn needs_secret(&self) -> bool {
        *self != Credentials::None
    }
}

/// A running copy of an app that Mortise can ask for its runtime status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    /// `local`, `staging`, `production`.
    pub name: String,
    /// `https://shop-api.test`
    pub base_url: String,
    /// The path of the actuator under the base URL: `actuator` unless the app changed it.
    pub path: String,
    #[serde(default)]
    pub credentials: Credentials,
}

impl Environment {
    /// The address of the actuator, or of one of its endpoints.
    pub fn url(&self, endpoint: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path = self.path.trim_matches('/');
        let mut url = if path.is_empty() {
            base.to_owned()
        } else {
            format!("{base}/{path}")
        };
        if !endpoint.is_empty() {
            url.push('/');
            url.push_str(endpoint);
        }
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_address_of_an_endpoint_is_joined_without_double_slashes() {
        let environment = Environment {
            name: "local".into(),
            base_url: "https://shop-api.test/".into(),
            path: "/actuator/".into(),
            credentials: Credentials::None,
        };
        assert_eq!(environment.url(""), "https://shop-api.test/actuator");
        assert_eq!(
            environment.url("health"),
            "https://shop-api.test/actuator/health"
        );
        let root = Environment {
            path: String::new(),
            ..environment
        };
        assert_eq!(root.url("info"), "https://shop-api.test/info");
    }
}
