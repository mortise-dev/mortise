//! Interface text of the Metrics tab (section 10 of the specification).

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rate_has_one_decimal_under_10() {
        assert_eq!(per_minute(0.0), "0");
        assert_eq!(per_minute(0.5), "0.5");
        assert_eq!(per_minute(4.0), "4.0");
        assert_eq!(per_minute(9.96), "10");
        assert_eq!(per_minute(366.4), "366");
        assert_eq!(per_minute(1_234.0), "1,234");
    }
}
