//! Sending the push document to `POST /api/v1/snapshots`, and what the answer means
//! (`mortise-cloud/docs/spec/03-ingest-api.md`).

use std::io::Read;
use std::time::Duration;

use serde_json::Value;

/// How long one attempt may take.
const TIMEOUT: Duration = Duration::from_secs(30);
/// Attempts after the first one, for an outage or a rate limit.
const RETRIES: u32 = 3;
/// The longest wait that `Retry-After` may ask for.
const LONGEST_WAIT: u64 = 60;
/// The most of an answer that is read. An answer is a short JSON object.
const ANSWER_LIMIT: u64 = 64 * 1024;

/// What the cloud did with the document.
pub(crate) enum Outcome {
    /// `200` or `202`: the snapshot is queued, or was there already.
    Accepted {
        url: Option<String>,
        warnings: Vec<String>,
    },
    /// `402` or `403`: a billing state. The build goes on.
    NotTaken { status: u16, problem: Problem },
    /// No answer, a rate limit or a fault of the cloud, after every retry. The build goes on.
    Unreachable { reason: String },
    /// Any other answer: only the customer can fix it.
    Refused { status: u16, problem: Problem },
}

/// An error answer of the cloud (RFC 9457).
#[derive(Default)]
pub(crate) struct Problem {
    pub(crate) title: Option<String>,
    pub(crate) detail: Option<String>,
    /// The JSON pointer of each member that is wrong, and why.
    pub(crate) errors: Vec<(String, String)>,
}

impl Problem {
    fn read(body: &str) -> Self {
        let value: Value = serde_json::from_str(body).unwrap_or(Value::Null);
        let text = |value: &Value| value.as_str().map(str::to_owned);
        Self {
            title: text(&value["title"]),
            detail: text(&value["detail"]),
            errors: value["errors"]
                .as_array()
                .into_iter()
                .flatten()
                .take(20)
                .map(|error| {
                    (
                        text(&error["pointer"]).unwrap_or_default(),
                        text(&error["message"]).unwrap_or_default(),
                    )
                })
                .collect(),
        }
    }

    /// The title and the detail, as one text.
    pub(crate) fn summary(&self) -> String {
        [&self.title, &self.detail]
            .into_iter()
            .flatten()
            .map(|text| text.trim_end_matches('.').to_owned() + ".")
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Sends `body`, a gzipped push document, to `url` with `token`. Retries an outage and a
/// rate limit 3 times; `wait` waits between attempts.
pub(crate) fn send(url: &str, token: &str, body: &[u8], wait: &mut dyn FnMut(Duration)) -> Outcome {
    let agent = ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            // Every status is an answer to read, not an error.
            .http_status_as_error(false)
            // A redirect is not followed: it could take the token to another host.
            .max_redirects(0)
            // Plain http goes only to this computer (`push::endpoint`). A proxy from the
            // environment would see the token there, so none is used. An https proxy sees
            // only the host.
            .proxy(if url.starts_with("http://") {
                None
            } else {
                ureq::Proxy::try_from_env()
            })
            .user_agent(format!("mortise/{}", env!("CARGO_PKG_VERSION")))
            .tls_config(
                ureq::tls::TlsConfig::builder()
                    .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                    .build(),
            )
            .build(),
    );
    let mut attempt = 0;
    loop {
        let (reason, asked) = match agent
            .post(url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .header("Content-Encoding", "gzip")
            .header("Accept", "application/json")
            .send(body)
        {
            Ok(mut response) => {
                let status = response.status().as_u16();
                let asked = response
                    .headers()
                    .get("retry-after")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.trim().parse::<u64>().ok());
                let mut answer = String::new();
                // An answer that breaks off is read as far as it came.
                let _ = response
                    .body_mut()
                    .with_config()
                    .reader()
                    .take(ANSWER_LIMIT)
                    .read_to_string(&mut answer);
                match status {
                    200 | 202 => {
                        let value: Value = serde_json::from_str(&answer).unwrap_or(Value::Null);
                        return Outcome::Accepted {
                            url: value["url"].as_str().map(str::to_owned),
                            warnings: value["warnings"]
                                .as_array()
                                .into_iter()
                                .flatten()
                                .filter_map(|warning| warning.as_str().map(str::to_owned))
                                .collect(),
                        };
                    }
                    402 | 403 => {
                        return Outcome::NotTaken {
                            status,
                            problem: Problem::read(&answer),
                        };
                    }
                    429 | 500..=599 => (format!("HTTP {status}"), asked),
                    _ => {
                        return Outcome::Refused {
                            status,
                            problem: Problem::read(&answer),
                        };
                    }
                }
            }
            Err(error) => (error.to_string(), None),
        };
        if attempt == RETRIES {
            return Outcome::Unreachable { reason };
        }
        wait(delay(attempt, asked));
        attempt += 1;
    }
}

/// The wait before the retry after `attempt`: 2, 4, then 8 s, or what `Retry-After` asks,
/// up to 60 s.
fn delay(attempt: u32, asked: Option<u64>) -> Duration {
    Duration::from_secs(asked.map_or(2 << attempt, |asked| asked.min(LONGEST_WAIT)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_answer_is_tried_four_times_then_given_up() {
        // A port that was free a moment ago: nothing listens on it.
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let mut waits = Vec::new();
        let outcome = send(
            &format!("http://127.0.0.1:{port}/api/v1/snapshots"),
            "mortise_test",
            b"{}",
            &mut |wait| waits.push(wait.as_secs()),
        );
        assert!(matches!(outcome, Outcome::Unreachable { .. }));
        assert_eq!(waits, [2, 4, 8]);
        assert_eq!(delay(0, Some(600)), Duration::from_secs(60));
    }
}
