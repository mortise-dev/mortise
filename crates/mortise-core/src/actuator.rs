//! The runtime status of a running app, read through `nwidart/laravel-actuator` 1.0.0.
//!
//! The base path is read for its list of links, then four endpoints: `health`, `info`,
//! `mappings` and `scheduledtasks`. `env` is never requested, and no redirect is followed. What the app answers is called Runtime; it is not
//! mixed with Findings, which Mortise works out from files.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::environment::{Credentials, Environment};

/// The status the actuator reports, for the app or for one component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Up,
    Down,
    OutOfService,
    Unknown,
}

impl Status {
    fn parse(text: &str) -> Self {
        match text {
            "UP" => Status::Up,
            "DOWN" => Status::Down,
            "OUT_OF_SERVICE" => Status::OutOfService,
            _ => Status::Unknown,
        }
    }
}

/// One health indicator of the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub status: Status,
    /// The details the app gives, as it wrote them, in its order.
    pub details: Vec<(String, String)>,
}

/// What `info` says. Every part is absent when the app does not give it.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Info {
    pub environment: Option<String>,
    pub laravel: Option<String>,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub php_version: Option<String>,
    pub php_sapi: Option<String>,
    /// Bytes. A limit of -1 means there is none.
    pub memory_usage: Option<i64>,
    pub memory_peak: Option<i64>,
    pub memory_limit: Option<i64>,
}

/// One route of the app, from `mappings`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    pub methods: Vec<String>,
    /// Without a leading slash, as Laravel has it.
    pub uri: String,
    pub name: Option<String>,
    /// `Modules\Blog\Http\Controllers\BlogController@index`, or `Closure`.
    pub action: String,
    pub middleware: Vec<String>,
}

/// One scheduled task of the app, from `scheduledtasks`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// The description, else the command, else the class of the job.
    pub target: String,
    /// The cron expression. `None` for a task that runs more than once a minute.
    pub expression: Option<String>,
    /// For a task that runs more than once a minute: how often, in milliseconds.
    pub interval_ms: Option<u64>,
    pub timezone: Option<String>,
    /// The next run, as the app wrote it: ISO 8601 in the task's timezone.
    pub next: Option<String>,
}

/// What a running app answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Runtime {
    pub status: Status,
    /// The components of `health`. `None` when the app hides them.
    pub components: Option<Vec<Component>>,
    /// The endpoints the app exposes, from the links at the actuator's base path.
    pub exposed: Vec<String>,
    /// `None` when `info` is not exposed.
    pub info: Option<Info>,
    /// `None` when `mappings` is not exposed.
    pub routes: Option<Vec<Route>>,
    /// `None` when `scheduledtasks` is not exposed.
    pub tasks: Option<Vec<Task>>,
}

impl Runtime {
    /// True when the app exposes this endpoint.
    pub fn exposes(&self, endpoint: &str) -> bool {
        self.exposed.iter().any(|exposed| exposed == endpoint)
    }

    /// The endpoints Mortise reads beside `health` that the app does not expose.
    pub fn not_exposed(&self) -> Vec<&'static str> {
        ["info", "mappings", "scheduledtasks"]
            .into_iter()
            .filter(|endpoint| !self.exposes(endpoint))
            .collect()
    }
}

/// What asking an environment gave. "No answer" and "Not authorized" are Mortise's
/// words for the connection, not statuses of the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Answer {
    /// Nothing came back: a refused connection, a timeout, a name that does not resolve.
    NoAnswer {
        url: String,
        message: String,
    },
    /// The app answered 401 or 403.
    NotAuthorized {
        url: String,
        status: u16,
    },
    /// The app answered with a status the actuator does not use, such as 500 or 404.
    Unexpected {
        url: String,
        status: u16,
    },
    Answered(Box<Runtime>),
}

/// The secret that goes with [`Credentials`]: a token, a password or a header value.
pub type Secret = Option<String>;

/// A response of the app: its status and its body.
struct Reply {
    status: u16,
    body: String,
}

/// One connection to an environment: an agent, and the credentials every request carries.
struct Client<'a> {
    agent: ureq::Agent,
    environment: &'a Environment,
    secret: &'a Secret,
}

impl<'a> Client<'a> {
    fn new(environment: &'a Environment, secret: &'a Secret) -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            // A 503 from health is an answer, not a failed request.
            .http_status_as_error(false)
            // A redirect is not followed. It could take a custom header with the secret to
            // another host, or ask for a path Mortise does not read. It comes back as an
            // unexpected answer.
            .max_redirects(0)
            // The certificates this computer trusts, so that a local site with its own
            // certificate authority works as it does in the browser.
            .tls_config(
                ureq::tls::TlsConfig::builder()
                    .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                    .build(),
            )
            .build();
        Self {
            agent: ureq::Agent::new_with_config(config),
            environment,
            secret,
        }
    }

    fn get(&self, url: &str) -> Result<Reply, ureq::Error> {
        // Laravel answers errors as JSON only when it is asked for JSON.
        let mut request = self.agent.get(url).header("Accept", "application/json");
        let secret = self.secret.as_deref().unwrap_or_default();
        match &self.environment.credentials {
            Credentials::None => {}
            Credentials::Bearer => {
                request = request.header("Authorization", format!("Bearer {secret}"));
            }
            Credentials::Basic { user } => {
                let pair = base64(format!("{user}:{secret}").as_bytes());
                request = request.header("Authorization", format!("Basic {pair}"));
            }
            Credentials::Header { name } => request = request.header(name.as_str(), secret),
        }
        let mut response = request.call()?;
        let status = response.status().as_u16();
        let body = response.body_mut().read_to_string().unwrap_or_default();
        Ok(Reply { status, body })
    }

    /// The JSON an endpoint answers, or the answer that says why there is none.
    fn fetch(&self, endpoint: &str) -> Result<Value, Answer> {
        let url = self.environment.url(endpoint);
        let reply = match self.get(&url) {
            Ok(reply) => reply,
            Err(error) => {
                return Err(Answer::NoAnswer {
                    url,
                    message: reason(&error),
                });
            }
        };
        match reply.status {
            // 503: the app is down or out of service, and says so in the body.
            200 | 503 => Ok(serde_json::from_str(&reply.body).unwrap_or(Value::Null)),
            status @ (401 | 403) => Err(Answer::NotAuthorized { url, status }),
            status => Err(Answer::Unexpected { url, status }),
        }
    }

    /// The status of the app and of its components, from `health`.
    fn health(&self) -> Result<(Status, Option<Vec<Component>>), Answer> {
        let health = self.fetch("health")?;
        let status = |value: &Value| {
            value
                .get("status")
                .and_then(Value::as_str)
                .map_or(Status::Unknown, Status::parse)
        };
        let components = health
            .get("components")
            .and_then(Value::as_object)
            .map(|components| {
                components
                    .iter()
                    .map(|(name, component)| Component {
                        name: name.clone(),
                        status: status(component),
                        details: component
                            .get("details")
                            .and_then(Value::as_object)
                            .map(|details| {
                                details
                                    .iter()
                                    .map(|(key, value)| (key.clone(), plain(value)))
                                    .collect()
                            })
                            .unwrap_or_default(),
                    })
                    .collect()
            });
        Ok((status(&health), components))
    }

    fn everything(&self) -> Result<Runtime, Answer> {
        // What is exposed: the links at the base path. An endpoint that is not listed
        // has no route in the app.
        let exposed: Vec<String> = self
            .fetch("")?
            .get("_links")
            .and_then(Value::as_object)
            .map(|links| {
                links
                    .keys()
                    .filter(|key| *key != "self" && !key.contains('-'))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        let (status, components) = self.health()?;
        let mut runtime = Runtime {
            status,
            components,
            exposed,
            info: None,
            routes: None,
            tasks: None,
        };
        if runtime.exposes("info") {
            runtime.info = Some(read_info(&self.fetch("info")?));
        }
        if runtime.exposes("mappings") {
            runtime.routes = Some(read_routes(&self.fetch("mappings")?));
        }
        if runtime.exposes("scheduledtasks") {
            runtime.tasks = Some(read_tasks(&self.fetch("scheduledtasks")?));
        }
        Ok(runtime)
    }
}

/// How long Mortise waits for one endpoint.
const TIMEOUT: Duration = Duration::from_secs(10);

/// Why nothing came back, in a few words.
fn reason(error: &ureq::Error) -> String {
    use std::io::ErrorKind;
    let timeout = || format!("no answer in {} s", TIMEOUT.as_secs());
    let text = match error {
        ureq::Error::Timeout(_) => return timeout(),
        ureq::Error::HostNotFound => return "the host was not found".to_owned(),
        ureq::Error::ConnectionFailed => return "connection failed".to_owned(),
        ureq::Error::BadUri(_) => return "the URL is not valid".to_owned(),
        ureq::Error::Io(error) => match error.kind() {
            ErrorKind::ConnectionRefused => return "connection refused".to_owned(),
            ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted => {
                return "the connection was closed".to_owned();
            }
            ErrorKind::TimedOut => return timeout(),
            // WSAHOST_NOT_FOUND and WSANO_DATA: the name does not resolve.
            #[cfg(windows)]
            _ if matches!(error.raw_os_error(), Some(11001 | 11004)) => {
                return "the host was not found".to_owned();
            }
            _ => error.to_string(),
        },
        other => other.to_string(),
    };
    // What is left comes as the text of the operating system or of the TLS library.
    if text.contains("lookup address") {
        "the host was not found".to_owned()
    } else if text.contains("invalid peer certificate") {
        "the certificate is not trusted on this computer".to_owned()
    } else {
        text
    }
}

/// Asks an environment for everything Mortise reads: what is exposed, `health`,
/// and `info`, `mappings` and `scheduledtasks` where they are exposed.
///
/// This waits for the app, for at most 10 s an endpoint: call it from a worker thread.
pub fn ask(environment: &Environment, secret: &Secret) -> Answer {
    match Client::new(environment, secret).everything() {
        Ok(runtime) => Answer::Answered(Box::new(runtime)),
        Err(answer) => answer,
    }
}

/// Asks an environment for `health` only, which is what the regular check does.
/// The rest is taken over from `earlier`, the last full answer.
pub fn ask_status(environment: &Environment, secret: &Secret, earlier: &Runtime) -> Answer {
    match Client::new(environment, secret).health() {
        Ok((status, components)) => Answer::Answered(Box::new(Runtime {
            status,
            components,
            ..earlier.clone()
        })),
        Err(answer) => answer,
    }
}

/// A JSON value as the app wrote it, without quotes around a string.
fn plain(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        // The certificate check of the operating system said no.
        other if other.to_string().starts_with("invalid peer certificate") => {
            "the certificate is not trusted on this computer".to_owned()
        }
        other => other.to_string(),
    }
}

fn text(value: &Value, path: &[&str]) -> Option<String> {
    path.iter()
        .try_fold(value, |value, key| value.get(key))?
        .as_str()
        .map(str::to_owned)
}

fn number(value: &Value, path: &[&str]) -> Option<i64> {
    path.iter()
        .try_fold(value, |value, key| value.get(key))?
        .as_i64()
}

fn read_info(info: &Value) -> Info {
    Info {
        environment: text(info, &["app", "environment"]),
        laravel: text(info, &["app", "laravel"]),
        git_branch: text(info, &["git", "branch"]),
        git_commit: text(info, &["git", "commit", "id"]),
        php_version: text(info, &["php", "version"]),
        php_sapi: text(info, &["php", "sapi"]),
        memory_usage: number(info, &["process", "memory", "usage"]),
        memory_peak: number(info, &["process", "memory", "peak"]),
        memory_limit: number(info, &["process", "memory", "limit"]),
    }
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn read_routes(mappings: &Value) -> Vec<Route> {
    mappings
        .get("routes")
        .and_then(Value::as_array)
        .map(|routes| {
            routes
                .iter()
                .map(|route| Route {
                    methods: strings(route.get("methods")),
                    uri: text(route, &["uri"]).unwrap_or_default(),
                    name: text(route, &["name"]),
                    action: text(route, &["action"]).unwrap_or_default(),
                    middleware: strings(route.get("middleware")),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn read_tasks(tasks: &Value) -> Vec<Task> {
    let list = |key: &str| {
        tasks
            .get(key)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let cron = list("cron").into_iter().map(|task| Task {
        target: text(&task, &["runnable", "target"]).unwrap_or_default(),
        expression: text(&task, &["expression"]),
        interval_ms: None,
        timezone: text(&task, &["timezone"]),
        next: text(&task, &["nextExecution", "time"]),
    });
    let fixed = list("fixedRate").into_iter().map(|task| Task {
        target: text(&task, &["runnable", "target"]).unwrap_or_default(),
        expression: None,
        interval_ms: task.get("interval").and_then(Value::as_u64),
        timezone: None,
        next: None,
    });
    cron.chain(fixed).collect()
}

/// Base64, for HTTP basic.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let group = u32::from(chunk[0]) << 16
            | u32::from(*chunk.get(1).unwrap_or(&0)) << 8
            | u32::from(*chunk.get(2).unwrap_or(&0));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[(group >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_known_values() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"mortise:secret"), "bW9ydGlzZTpzZWNyZXQ=");
    }
}
