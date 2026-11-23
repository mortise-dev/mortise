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

/// The fake artisan: it reads its first argument only.
///
/// `--version` prints the version with the number in green and ends with 0.
/// `fail` prints the error of a stopped Docker service on stderr and ends with 1.
/// `wait` sleeps for 60 seconds. Anything else is a command artisan does not know.
pub fn fake_artisan_main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("--version") => {
            println!("Laravel Framework \x1b[32m13.2.0\x1b[39m");
            ExitCode::SUCCESS
        }
        Some("fail") => {
            eprintln!("service \"app\" is not running");
            ExitCode::from(1)
        }
        Some("wait") => {
            std::thread::sleep(Duration::from_secs(60));
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("Command \"{}\" is not defined.", other.unwrap_or_default());
            ExitCode::from(1)
        }
    }
}
