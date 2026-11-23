//! Interface text of Runtime (section 10 of the specification).

pub const STATUS: &str = "Status";
pub const UP: &str = "Up";
pub const DOWN: &str = "Down";
pub const OUT_OF_SERVICE: &str = "Out of service";
pub const RUNTIME_UNKNOWN: &str = "Unknown";
pub const NO_ANSWER: &str = "No answer";
pub const NOT_AUTHORIZED: &str = "Not authorized";
pub const COPY_LINE: &str = "Copy line";
/// Not in the spec: what the toast says after "Copy line".
pub const COPIED_LINE: &str = "Copied the line.";
pub const REQUIRE_ACTUATOR: &str = "composer require nwidart/laravel-actuator";
pub const EXPOSURE_LINE: &str =
    "'exposure' => [\n    'include' => ['health', 'info', 'mappings', 'scheduledtasks'],\n],";
pub const ENVIRONMENT: &str = "Environment";
pub const MEMORY: &str = "Memory";
pub const COL_TASK: &str = "Task";
pub const COL_SCHEDULE: &str = "Schedule";
pub const COL_NEXT_RUN: &str = "Next run";
pub const COL_METHOD: &str = "Method";
pub const COL_URI: &str = "URI";
pub const COL_NAME: &str = "Name";
pub const COL_ACTION: &str = "Action";
pub const COL_MIDDLEWARE: &str = "Middleware";
pub const NO_FIXED_TIME: &str = "no fixed time";
/// Not in the spec: the status line before the first answer is there.
pub const ASKING: &str = "asking";

/// `Runtime on local`
pub fn runtime_on(environment: &str) -> String {
    format!("Runtime on {environment}")
}

pub fn not_exposed(endpoints: &[&str], environment: &str) -> String {
    let (verb, them) = if endpoints.len() == 1 {
        ("is", "it")
    } else {
        ("are", "them")
    };
    format!(
        "{} {verb} not exposed on {environment}. Only health is exposed by default. Add {them} \
         to exposure.include in config/actuator.php.",
        crate::strings::and_list(endpoints)
    )
}

pub const COMPONENTS_HIDDEN: &str = "Components are hidden. The app shows them only when \
    health.show_details allows it, and that is never by default. Set it to when_authorized or \
    always in config/actuator.php.";

pub fn out_of_service(component: &str) -> String {
    if component == "readinessState" {
        format!(
            "{component} is out of service. The app reports this while it is in maintenance mode."
        )
    } else {
        format!("{component} is out of service.")
    }
}

/// What a test in the environment form gave, when the app answered with a status.
/// `http://shop.test/actuator answered 301 and sends to https://shop.test/actuator.`
pub fn tested(url: &str, status: u16, location: Option<&str>) -> String {
    match location {
        // Not in the spec: a redirect, which Mortise does not follow.
        Some(location) => format!("{url} answered {status} and sends to {location}."),
        None => format!("{url} answered {status}."),
    }
}

/// `4 up`
pub fn n_up(n: usize) -> String {
    format!("{n} up")
}

/// `4 other components`
pub fn other_components(n: usize) -> String {
    match n {
        1 => "1 other component".to_owned(),
        n => format!("{n} other components"),
    }
}

/// `198.6 GB free of 460.4 GB`
pub fn disk(free: f64, total: f64) -> String {
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    format!("{:.1} GB free of {:.1} GB", free / GB, total / GB)
}

/// `38 MB now, 41 MB peak, limit 512 MB`
pub fn memory(usage: Option<i64>, peak: Option<i64>, limit: Option<i64>) -> String {
    let mb = |bytes: i64| format!("{} MB", bytes / (1024 * 1024));
    let mut parts = Vec::new();
    if let Some(usage) = usage {
        parts.push(format!("{} now", mb(usage)));
    }
    if let Some(peak) = peak {
        parts.push(format!("{} peak", mb(peak)));
    }
    match limit {
        Some(limit) if limit > 0 => parts.push(format!("limit {}", mb(limit))),
        // PHP writes -1 for a process without a limit.
        Some(_) => parts.push("no limit".to_owned()),
        None => {}
    }
    parts.join(", ")
}

/// Days since 1970 of a date in the calendar.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// When a scheduled task runs next: `14:10 UTC`, `tomorrow 06:00 UTC`, or the date.
/// `next` is what the app wrote, ISO 8601 with an offset: `2026-11-23T06:00:00+00:00`.
pub fn next_run(next: Option<&str>, timezone: Option<&str>, now: i64) -> String {
    let Some(next) = next else {
        return NO_FIXED_TIME.to_owned();
    };
    let parsed = (|| {
        let number = |from: usize, to: usize| next.get(from..to)?.parse::<i64>().ok();
        let (year, month, day) = (number(0, 4)?, number(5, 7)?, number(8, 10)?);
        let (hour, minute) = (number(11, 13)?, number(14, 16)?);
        // The offset of the task's timezone, in seconds.
        let offset = match next.get(19..) {
            Some("Z") | Some("") | None => 0,
            Some(offset) => {
                let sign = if offset.starts_with('-') { -1 } else { 1 };
                let hours = offset.get(1..3)?.parse::<i64>().ok()?;
                let minutes = offset
                    .get(4..6)
                    .and_then(|m| m.parse::<i64>().ok())
                    .unwrap_or(0);
                sign * (hours * 3600 + minutes * 60)
            }
        };
        Some((year, month, day, hour, minute, offset))
    })();
    let Some((year, month, day, hour, minute, offset)) = parsed else {
        // As the app wrote it.
        return next.to_owned();
    };
    let zone = match (timezone, offset) {
        (Some(zone), _) => zone.to_owned(),
        (None, 0) => "UTC".to_owned(),
        (None, _) => next.get(19..).unwrap_or_default().to_owned(),
    };
    let time = format!("{hour:02}:{minute:02} {zone}");
    // Today, in the timezone of the task.
    let today = (now + offset).div_euclid(86_400);
    match days_from_civil(year, month, day) - today {
        0 => time,
        1 => format!("tomorrow {time}"),
        _ => format!("{year:04}-{month:02}-{day:02} {time}"),
    }
}
pub const CREDENTIALS_HINT: &str = "what the app's middleware asks for";
pub const NO_CREDENTIALS: &str =
    "Mortise sends no credentials. Right for a local app whose actuator is open.";
pub const TOKEN: &str = "Token";
pub const TOKEN_HINT: &str = "sent as Authorization: Bearer";
pub const USER_NAME: &str = "User name";
pub const PASSWORD: &str = "Password";
pub const HEADER_NAME: &str = "Header name";
pub const HEADER_VALUE: &str = "Header value";
pub const KEPT_IN_STORE: &str =
    "Kept in the credential store of this computer, not in mortise.json and not in the app.";
/// Not in the spec as a sentence. The spec says that Mortise says so and keeps the
/// secret only until it closes.
pub const NO_STORE: &str =
    "No credential store is running on this computer. The secret is kept until Mortise closes.";
pub const SAVED_NOT_SHOWN: &str = "Saved. Not shown again.";
pub const REMOVE: &str = "Remove";
pub const BASE_URL_HINT: &str = "from APP_URL in .env";
pub const ACTUATOR_PATH: &str = "Actuator path";

/// `Exposed: health, info, mappings, scheduledtasks.`
pub fn exposed(endpoints: &[String]) -> String {
    format!("Exposed: {}.", endpoints.join(", "))
}

#[cfg(test)]
mod runtime_tests {
    use super::*;
    use crate::strings::{and_list, answered_unexpected, every};
    use crate::summary::strings::routes_of_module;

    #[test]
    fn the_next_run_is_said_relative_to_today_in_the_zone_of_the_task() {
        // 2026-11-22 13:00:00 UTC
        let now = 1_795_352_400;
        assert_eq!(
            next_run(Some("2026-11-22T14:10:00+00:00"), Some("UTC"), now),
            "14:10 UTC"
        );
        assert_eq!(
            next_run(Some("2026-11-23T06:00:00+00:00"), Some("UTC"), now),
            "tomorrow 06:00 UTC"
        );
        assert_eq!(
            next_run(Some("2026-12-01T03:00:00+00:00"), Some("UTC"), now),
            "2026-12-01 03:00 UTC"
        );
        // 23:30 in Brussels on the 22nd is still today there.
        assert_eq!(
            next_run(
                Some("2026-11-22T23:30:00+01:00"),
                Some("Europe/Brussels"),
                now
            ),
            "23:30 Europe/Brussels"
        );
        // At 23:30 UTC it is already the 23rd in Brussels.
        assert_eq!(
            next_run(
                Some("2026-11-23T06:00:00+01:00"),
                None,
                now + 10 * 3600 + 1800
            ),
            "06:00 +01:00"
        );
        assert_eq!(next_run(None, None, now), "no fixed time");
        assert_eq!(next_run(Some("soon"), None, now), "soon");
    }

    #[test]
    fn figures_are_written_as_in_the_drawings() {
        assert_eq!(
            disk(213_212_008_448.0, 494_384_795_648.0),
            "198.6 GB free of 460.4 GB"
        );
        assert_eq!(
            memory(Some(39_845_888), Some(42_991_616), Some(536_870_912)),
            "38 MB now, 41 MB peak, limit 512 MB"
        );
        assert_eq!(
            memory(Some(39_845_888), None, Some(-1)),
            "38 MB now, no limit"
        );
        assert_eq!(every(10_000), "every 10 s");
        assert_eq!(routes_of_module(18, 12, 6), "18 (12 API, 6 web)");
        assert_eq!(routes_of_module(3, 0, 0), "3");
        assert_eq!(routes_of_module(5, 2, 0), "5 (2 API, 3 other)");
        assert_eq!(
            and_list(&["info", "mappings", "scheduledtasks"]),
            "info, mappings and scheduledtasks"
        );
        assert_eq!(
            not_exposed(&["mappings"], "local"),
            "mappings is not exposed on local. Only health is exposed by default. Add it to \
             exposure.include in config/actuator.php."
        );
        assert_eq!(
            answered_unexpected("local", "https://shop-api.test/actuator/health", 500, None),
            "local answered 500 for /actuator/health. The actuator configuration has an error. \
             Read the app log."
        );
        assert_eq!(
            answered_unexpected(
                "local",
                "http://shop-api.test/actuator",
                301,
                Some("https://shop-api.test/actuator")
            ),
            "local answered 301 for /actuator and sends to https://shop-api.test/actuator. \
             Correct the base URL or the actuator path in Settings to match that address."
        );
    }
}
