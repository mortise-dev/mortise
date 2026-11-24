//! Interface text of the Metrics tab (section 10 of the specification).

use std::time::Duration;

use mortise_core::{Range, Under};

pub const NOT_CONNECTED_TITLE: &str = "Metrics are not connected";

pub fn not_connected(app: &str) -> String {
    format!(
        "Metrics come from the actuator of the running app. Connect an environment for {app} to \
         read requests, response time and errors per module."
    )
}

pub const CONNECT_ENVIRONMENT: &str = "Connect an environment";

/// `Metrics are not exposed on local`
pub fn not_exposed_title(environment: &str) -> String {
    format!("Metrics are not exposed on {environment}")
}

pub const NOT_EXPOSED: &str =
    "Add prometheus to exposure.include in config/actuator.php, next to health.";

/// Not in the spec: before the first read of metrics ends.
pub fn reading(environment: &str) -> String {
    format!("Reading metrics from {environment}")
}

/// Not in the spec: the app answers 200 with no text.
pub fn off_title(environment: &str) -> String {
    format!("Metrics are off on {environment}")
}

/// Not in the spec.
pub const OFF: &str =
    "The app answers with no metrics. Set METRICS_ENABLED=true in its .env to turn them on.";

/// Not in the spec: the app answers 503.
pub fn unavailable_title(environment: &str) -> String {
    format!("Metrics cannot be read on {environment}")
}

/// Not in the spec. `line` is the first line of the answer, as the app wrote it.
pub fn unavailable(environment: &str, url: &str, line: &str) -> String {
    let path = crate::strings::path_of(url);
    format!(
        "{environment} answered 503 for {path}: \"{line}\" Check the metrics store of the app, \
         then try again."
    )
}

/// Not in the spec: the app answers 200 with text that has no `# TYPE` line.
pub fn not_metrics_title(environment: &str) -> String {
    format!("The answer of {environment} is not metrics")
}

/// Not in the spec.
pub fn not_metrics(environment: &str) -> String {
    format!(
        "{environment} answered prometheus with text that holds no metrics. Check that the \
         address points to the actuator of the app."
    )
}

/// Not in the spec: the answer is over the limit Mortise reads.
pub fn too_large_title(environment: &str) -> String {
    format!("The metrics of {environment} are too large")
}

/// Not in the spec. `limit` is in bytes.
pub fn too_large(limit: u64) -> String {
    format!(
        "The answer is larger than {} MB, so Mortise does not read it. Export fewer series from \
         the app.",
        limit / (1024 * 1024)
    )
}

/// Not in the spec: the answer has more series than a read keeps.
pub fn too_many_title(environment: &str) -> String {
    format!("The metrics of {environment} have too many series")
}

/// Not in the spec. `limit` is the most numbers a read keeps.
pub fn too_many(limit: usize) -> String {
    format!(
        "Mortise keeps at most {} figures per read. Export fewer series from the app.",
        crate::strings::count(limit as u64)
    )
}

pub const COL_REQUESTS: &str = "Requests/min";
pub const COL_JOBS: &str = "Jobs/min";

/// Not in the spec: a figure before two reads.
pub const NO_RATE: &str = "\u{2013}";

/// Not in the spec: under the table before two reads.
pub const ONE_READ: &str = "A rate needs two reads. The second read comes 15 s after the first.";

/// Not in the spec: under the table when `mappings` is not exposed.
pub fn not_placed(environment: &str) -> String {
    format!(
        "Requests without a module label are not placed in a module, because mappings is not \
         exposed on {environment}."
    )
}

/// A rate per minute: one decimal under 10, else a whole number. Zero is `0`.
pub fn per_minute(rate: f64) -> String {
    if rate == 0.0 {
        return "0".to_owned();
    }
    let tenths = (rate * 10.0).round() / 10.0;
    if tenths < 10.0 {
        format!("{tenths:.1}")
    } else {
        crate::strings::count(rate.round() as u64)
    }
}

/// The label of a range in the range switch.
pub fn range(range: Range) -> &'static str {
    match range {
        Range::FifteenMinutes => "15 min",
        Range::Hour => "1 hour",
        Range::Day => "24 hours",
    }
}

/// A duration in whole seconds: `15 s`.
fn seconds(time: Duration) -> String {
    format!("{} s", time.as_secs())
}

/// `read every 15 s, last read 8 s ago`
pub fn read_every(pace: Duration, ago: Duration) -> String {
    format!(
        "read every {}, last read {} ago",
        seconds(pace),
        seconds(ago)
    )
}

pub const COL_UNDER_95: &str = "95% under";
pub const COL_ERRORS: &str = "Errors";
pub const COL_DATABASE: &str = "Database time";
pub const COL_LAST_HOUR: &str = "Last hour";

/// Not in the spec: the first column when one module lists its routes.
pub const COL_ROUTE: &str = "Route";

/// Not in the spec: the database time of an app whose series have no `module` label.
pub const UNKNOWN: &str = "unknown";

/// Not in the spec: the tooltip of "unknown".
pub const NEEDS_NEWER_PACKAGE: &str = "Needs a newer nwidart/laravel-modules";

/// A route of a module: `GET /billing/invoices/{invoice}`.
pub fn route(method: &str, route: Option<&str>) -> String {
    match route {
        Some(route) => format!("{method} {route}"),
        // Not in the spec: a request with a `module` label and no route.
        None => format!("{method} without a route"),
    }
}

/// The title of the chart for everything in the app.
pub const CHART_ALL: &str = "Requests per minute, all modules";

/// The title of the chart for a module or a route: `Requests per minute, Billing`.
pub fn chart_title(row: &str) -> String {
    format!("Requests per minute, {row}")
}

/// The note of the chart box: `842 now`.
pub fn now(rate: f64) -> String {
    format!("{} now", per_minute(rate))
}

/// The accessible name of the chart: `Requests per minute, all modules, last hour`.
pub fn chart_name(title: &str, range: Range) -> String {
    let words = match range {
        Range::FifteenMinutes => "last 15 min",
        Range::Hour => "last hour",
        Range::Day => "last 24 hours",
    };
    format!("{title}, {words}")
}

/// Not in the spec: the text summary of the chart. `now` is `None` when the newest read gave no
/// point of the line.
pub fn summary(now: Option<f64>, highest: f64, lowest: f64) -> String {
    let (highest, lowest) = (per_minute(highest), per_minute(lowest));
    match now {
        Some(now) => format!(
            "Now {}, highest {highest}, lowest {lowest} requests per minute.",
            per_minute(now)
        ),
        None => format!("Highest {highest}, lowest {lowest} requests per minute."),
    }
}

/// Not in the spec: the summary of a chart with no point.
pub const NO_RATE_YET: &str = "No rate yet.";

/// Not in the spec: under the left end of the time axis.
pub fn range_ago(range: Range) -> &'static str {
    match range {
        Range::FifteenMinutes => "15 min ago",
        Range::Hour => "1 hour ago",
        Range::Day => "24 hours ago",
    }
}

/// Not in the spec: under the right end of the time axis.
pub const NOW: &str = "now";

/// Not in the spec: the accessible name of the sparkline of a row.
pub fn spark_name(row: &str) -> String {
    format!("Requests per minute of {row}, last hour")
}

/// The 95% figure: whole milliseconds under 1 s, else seconds with at most one decimal.
pub fn under_95(under: Under) -> String {
    let time = |seconds: f64| {
        if seconds < 1.0 {
            format!("{} ms", (seconds * 1000.0).round())
        } else {
            let tenths = (seconds * 10.0).round() / 10.0;
            if tenths.fract() == 0.0 {
                format!("{tenths:.0} s")
            } else {
                format!("{tenths:.1} s")
            }
        }
    };
    match under {
        Under::Bound(bound) => time(bound),
        Under::Above(bound) => format!("over {}", time(bound)),
    }
}

/// The share of errors, 0 to 1, as a percent with one decimal: `2.4%`.
pub fn errors(share: f64) -> String {
    format!("{:.1}%", share * 100.0)
}

/// The database time against the request time, 0 to 1 or more, as a whole percent: `31%`.
pub fn database(share: f64) -> String {
    format!("{:.0}%", share * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figures_read_as_s7_3_writes_them() {
        assert_eq!(per_minute(0.0), "0");
        assert_eq!(per_minute(0.5), "0.5");
        assert_eq!(per_minute(4.0), "4.0");
        assert_eq!(per_minute(9.96), "10");
        assert_eq!(per_minute(366.4), "366");
        assert_eq!(per_minute(1_234.0), "1,234");
        assert_eq!(
            read_every(Duration::from_secs(15), Duration::from_millis(8_900)),
            "read every 15 s, last read 8 s ago"
        );
        assert_eq!(under_95(Under::Bound(0.1)), "100 ms");
        assert_eq!(under_95(Under::Bound(0.005)), "5 ms");
        assert_eq!(under_95(Under::Bound(2.5)), "2.5 s");
        assert_eq!(under_95(Under::Bound(1.0)), "1 s");
        assert_eq!(under_95(Under::Above(10.0)), "over 10 s");
        assert_eq!(errors(0.024), "2.4%");
        assert_eq!(errors(0.0), "0.0%");
        assert_eq!(database(0.31), "31%");
        assert_eq!(database(1.25), "125%");
    }
}
