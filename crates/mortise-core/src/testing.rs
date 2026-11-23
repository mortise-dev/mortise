//! Helpers for tests: a small actuator that answers on a local port with what it is
//! told to, and the fake artisan of the `fake-artisan` binary.
//!
//! The bodies of the actuator are those of the tests of `nwidart/laravel-actuator` 1.0.0.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// What the stub answers for one path.
#[derive(Debug, Clone)]
pub struct Reply {
    pub status: u16,
    pub body: String,
    /// The `Location` header, for a redirect.
    pub location: Option<String>,
}

impl Reply {
    pub fn ok(body: &str) -> Self {
        Self {
            status: 200,
            body: body.to_owned(),
            location: None,
        }
    }

    pub fn status(status: u16, body: &str) -> Self {
        Self {
            status,
            body: body.to_owned(),
            location: None,
        }
    }

    /// A 302 that sends the client to another address.
    pub fn redirect(location: &str) -> Self {
        Self {
            status: 302,
            body: String::new(),
            location: Some(location.to_owned()),
        }
    }
}

#[derive(Default)]
struct Routes {
    replies: BTreeMap<String, Reply>,
    /// A header every request must carry, as `name: value` in lower case.
    required_header: Option<String>,
    /// The paths that were asked for, in order.
    asked: Vec<String>,
}

/// An actuator that listens on `127.0.0.1` until it is dropped.
pub struct StubActuator {
    pub base_url: String,
    routes: Arc<Mutex<Routes>>,
}

impl StubActuator {
    /// Starts a stub that answers 404 for every path.
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a local port");
        let base_url = format!("http://{}", listener.local_addr().expect("an address"));
        let routes = Arc::new(Mutex::new(Routes::default()));
        let shared = Arc::clone(&routes);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                serve(stream, &shared);
            }
        });
        Self { base_url, routes }
    }

    /// A stub that answers as an app with every endpoint exposed and everything up.
    pub fn healthy() -> Self {
        let stub = Self::start();
        stub.expose(&["health", "info", "mappings", "scheduledtasks"]);
        stub.set(
            "/actuator/health",
            Reply::ok(
                r#"{"status":"UP","components":{
                "ping":{"status":"UP"},
                "readinessState":{"status":"UP"},
                "diskSpace":{"status":"UP","details":{"total":494384795648,"free":213212008448,"threshold":10485760,"path":"/var/www","exists":true}},
                "db":{"status":"UP","details":{"connection":"mysql","database":"MySQL","validationQuery":"select 1"}},
                "cache":{"status":"UP","details":{"store":"redis"}}},
                "groups":["liveness","readiness"]}"#,
            ),
        );
        stub.set(
            "/actuator/info",
            Reply::ok(
                r#"{"app":{"name":"Laravel","environment":"local","laravel":"13.2.0","timezone":"UTC","locale":"en"},
                "git":{"branch":"feature/tracking-code","commit":{"id":"49689af"}},
                "php":{"version":"8.4.27","sapi":"fpm-fcgi"},
                "os":{"name":"Linux","version":"6.8.0","arch":"x86_64"},
                "process":{"pid":42,"memory":{"usage":39845888,"peak":42991616,"limit":536870912}}}"#,
            ),
        );
        stub.set(
            "/actuator/mappings",
            Reply::ok(
                r#"{"routes":[
                {"methods":["GET","HEAD"],"uri":"api/checkout/orders/{order}","name":"api.checkout.orders.show","action":"Modules\\Checkout\\Http\\Controllers\\OrderController@show","middleware":["api","auth:sanctum"],"domain":null},
                {"methods":["POST"],"uri":"api/checkout/orders","name":"api.checkout.orders.store","action":"Modules\\Checkout\\Http\\Controllers\\OrderController@store","middleware":["api","auth:sanctum"],"domain":null},
                {"methods":["GET","HEAD"],"uri":"billing/invoices/{invoice}","name":"billing.invoices.show","action":"Modules\\Billing\\Http\\Controllers\\InvoiceController@show","middleware":["web","auth"],"domain":null},
                {"methods":["GET","HEAD"],"uri":"up","name":null,"action":"Closure","middleware":[],"domain":null},
                {"methods":["GET","HEAD"],"uri":"actuator/health/{path?}","name":"actuator.health","action":"Nwidart\\Actuator\\Http\\Controllers\\HealthController","middleware":[],"domain":null}
                ]}"#,
            ),
        );
        stub.set(
            "/actuator/scheduledtasks",
            Reply::ok(
                r#"{"cron":[
                {"runnable":{"target":"php artisan billing:send-invoices"},"expression":"0 6 * * *","timezone":"UTC","nextExecution":{"time":"2026-11-23T06:00:00+00:00"}},
                {"runnable":{"target":"Modules\\Inventory\\Jobs\\SyncStock"},"expression":"0 * * * *","timezone":"UTC","nextExecution":{"time":"2026-11-22T15:00:00+00:00"}}
                ],"fixedDelay":[],"fixedRate":[{"runnable":{"target":"Poll shipment tracking"},"initialDelay":0,"interval":10000}],"custom":[]}"#,
            ),
        );
        stub
    }

    /// Sets the discovery links at the base path to these endpoints.
    pub fn expose(&self, endpoints: &[&str]) {
        let mut links = vec![format!(
            r#""self":{{"href":"{}/actuator","templated":false}}"#,
            self.base_url
        )];
        for endpoint in endpoints {
            links.push(format!(
                r#""{endpoint}":{{"href":"{}/actuator/{endpoint}","templated":false}}"#,
                self.base_url
            ));
            if *endpoint == "health" {
                links.push(format!(
                    r#""health-path":{{"href":"{}/actuator/health/{{*path}}","templated":true}}"#,
                    self.base_url
                ));
            }
        }
        self.set(
            "/actuator",
            Reply::ok(&format!(r#"{{"_links":{{{}}}}}"#, links.join(","))),
        );
    }

    /// Sets what the stub answers for a path.
    pub fn set(&self, path: &str, reply: Reply) {
        self.routes
            .lock()
            .expect("the routes")
            .replies
            .insert(path.to_owned(), reply);
    }

    /// What the stub answers for a path now.
    pub fn reply(&self, path: &str) -> Option<Reply> {
        self.routes
            .lock()
            .expect("the routes")
            .replies
            .get(path)
            .cloned()
    }

    /// Stops answering for a path: it is 404 again, as an endpoint that is not exposed.
    pub fn remove(&self, path: &str) {
        self.routes.lock().expect("the routes").replies.remove(path);
    }

    /// From now on every request must carry this header, or it is answered 401.
    pub fn require_header(&self, name: &str, value: &str) {
        let header = format!("{}: {value}", name.to_lowercase());
        self.routes.lock().expect("the routes").required_header = Some(header);
    }

    /// The paths that were asked for so far, in order.
    pub fn asked(&self) -> Vec<String> {
        self.routes.lock().expect("the routes").asked.clone()
    }
}

fn serve(mut stream: TcpStream, routes: &Mutex<Routes>) {
    let mut reader = BufReader::new(stream.try_clone().expect("a stream"));
    let mut request = String::new();
    if reader.read_line(&mut request).is_err() {
        return;
    }
    let path = request.split_whitespace().nth(1).unwrap_or("/").to_owned();
    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.push(format!("{}: {}", name.trim().to_lowercase(), value.trim()));
        }
    }

    let reply = {
        let mut routes = routes.lock().expect("the routes");
        routes.asked.push(path.clone());
        let authorized = routes
            .required_header
            .as_ref()
            .is_none_or(|required| headers.contains(required));
        if !authorized {
            Reply::status(401, r#"{"message":"Unauthenticated."}"#)
        } else {
            routes.replies.get(&path).cloned().unwrap_or_else(|| {
                let message = format!(
                    "The route {} could not be found.",
                    path.trim_start_matches('/')
                );
                Reply::status(404, &format!(r#"{{"message":"{message}"}}"#))
            })
        }
    };
    let location = reply
        .location
        .map(|location| format!("Location: {location}\r\n"))
        .unwrap_or_default();
    let response = format!(
        "HTTP/1.1 {} X\r\n{location}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        reply.status,
        reply.body.len(),
        reply.body
    );
    let _ = stream.write_all(response.as_bytes());
}

/// The folder of the recorded package output that the fake artisan replays.
const RECORDINGS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/output");

/// One recorded command of the package, as `fixtures/output/*.json` holds it.
#[derive(serde::Deserialize)]
struct Recording {
    /// The artisan words without `--no-interaction` and `--ansi`.
    args: Vec<String>,
    exit: u8,
    stdout: String,
    stderr: String,
}

/// The fake artisan.
///
/// First it takes the words `hold` and `ignore-term`, in any order. `hold` waits
/// 60 seconds after the output. `ignore-term` ignores SIGTERM on Unix and does
/// nothing on Windows.
///
/// Then the first remaining word picks the case.
/// `--version` prints the version with the number in green and ends with 0.
/// `fail` prints the error of a stopped Docker service on stderr and ends with 1.
/// `wait` sleeps for 60 seconds. `flood` prints 2 MiB of short lines on stderr and
/// one line of 2 MiB with no newline on stdout, and ends with 0.
/// `ask` asks for a name and ends with 1 when its input ends.
/// `orphan` starts a copy of itself with `wait` on the same pipes, prints its pid
/// and ends with 0. `orphan-wait` does the same, then sleeps for 60 seconds.
/// `orphan-quiet` is `orphan-wait` with a child that ignores SIGTERM and has
/// closed its pipes, so only a kill of the group ends it.
/// Any other words replay the recording in `fixtures/output` whose `args` they are,
/// without `--no-interaction` and `--ansi`. With no recording, they are a command
/// artisan does not know.
pub fn fake_artisan_main() -> ExitCode {
    let mut words: Vec<String> = std::env::args().skip(1).collect();
    let mut hold = false;
    loop {
        match words.first().map(String::as_str) {
            Some("hold") => hold = true,
            Some("ignore-term") => {
                #[cfg(unix)]
                {
                    words.remove(0);
                    let mut rest = Vec::new();
                    if hold {
                        rest.push("hold".to_string());
                    }
                    rest.extend(words);
                    return ignore_term(&rest);
                }
            }
            _ => break,
        }
        words.remove(0);
    }
    let code = fake_case(&words);
    if hold {
        std::thread::sleep(Duration::from_secs(60));
    }
    code
}

/// Starts the fake again with `rest`, through a shell that ignores SIGTERM. An
/// ignored signal stays ignored across `exec`. This is the only shell Mortise runs,
/// in a test helper, with no text of a user. It returns only when `exec` fails.
#[cfg(unix)]
fn ignore_term(rest: &[String]) -> ExitCode {
    use std::os::unix::process::CommandExt;
    let Ok(itself) = std::env::current_exe() else {
        return ExitCode::from(2);
    };
    let error = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(r#"trap '' TERM; exec "$0" "$@""#)
        .arg(itself)
        .args(rest)
        .exec();
    eprintln!("{error}");
    ExitCode::from(2)
}

/// Runs the case that `words` name. See [`fake_artisan_main`].
fn fake_case(words: &[String]) -> ExitCode {
    match words.first().map(String::as_str) {
        Some("--version") => {
            println!("Laravel Framework \x1b[32m13.2.0\x1b[39m");
            ExitCode::SUCCESS
        }
        Some("fail") => {
            eprintln!("service \"app\" is not running");
            ExitCode::from(1)
        }
        Some("flood") => {
            const SIZE: usize = 2 * 1024 * 1024;
            let lines = "Laravel Framework 13.2.0\n".repeat(SIZE / 25);
            // A reader that stops reading would block one of these writes.
            let _ = std::io::stderr().lock().write_all(lines.as_bytes());
            let _ = std::io::stdout().lock().write_all(&vec![b'x'; SIZE]);
            ExitCode::SUCCESS
        }
        Some("wait") => {
            std::thread::sleep(Duration::from_secs(60));
            ExitCode::SUCCESS
        }
        Some("ask") => {
            println!("Name of the new module?");
            let _ = std::io::stdout().flush();
            let mut answer = String::new();
            match std::io::stdin().read_line(&mut answer) {
                Ok(read) if read > 0 => ExitCode::SUCCESS,
                _ => {
                    eprintln!("Aborted.");
                    ExitCode::from(1)
                }
            }
        }
        Some(case @ ("orphan" | "orphan-wait" | "orphan-quiet")) => {
            let quiet = case == "orphan-quiet";
            let child = std::env::current_exe().and_then(|itself| {
                let mut child = std::process::Command::new(itself);
                child.stdin(std::process::Stdio::null());
                if quiet {
                    child
                        .args(["ignore-term", "wait"])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null());
                } else {
                    child.arg("wait");
                }
                child.spawn()
            });
            let Ok(child) = child else {
                eprintln!("the orphan did not start");
                return ExitCode::from(2);
            };
            println!("{}", child.id());
            if case != "orphan" {
                std::thread::sleep(Duration::from_secs(60));
            }
            ExitCode::SUCCESS
        }
        _ => replay(words),
    }
}

/// Prints the recording whose `args` are `words` without `--no-interaction` and
/// `--ansi`, and ends with its code.
fn replay(words: &[String]) -> ExitCode {
    let words: Vec<&str> = words
        .iter()
        .map(String::as_str)
        .filter(|word| !matches!(*word, "--no-interaction" | "--ansi"))
        .collect();
    let recordings = std::fs::read_dir(RECORDINGS)
        .into_iter()
        .flatten()
        .flatten();
    for entry in recordings {
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let Some(recording) = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Recording>(&bytes).ok())
        else {
            continue;
        };
        if recording.args == words {
            let _ = std::io::stdout().write_all(recording.stdout.as_bytes());
            let _ = std::io::stdout().flush();
            let _ = std::io::stderr().write_all(recording.stderr.as_bytes());
            return ExitCode::from(recording.exit);
        }
    }
    eprintln!(
        "Command \"{}\" is not defined.",
        words.first().copied().unwrap_or_default()
    );
    ExitCode::from(1)
}
