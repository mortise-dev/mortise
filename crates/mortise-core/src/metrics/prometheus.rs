//! The Prometheus text format (version 0.0.4), as the `prometheus` endpoint of the actuator writes it.
//!
//! [`samples`] reads an answer in one pass, line by line. A sample borrows its names and its label
//! values from the text. Only a label value with an escape is copied. Nothing holds all samples.

use std::borrow::Cow;

/// The kind of a family, from its `# TYPE` line. A family of another kind is skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Counter,
    Gauge,
    Histogram,
}

/// Which number of its family a sample is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Part {
    /// The value of a counter or a gauge.
    Value,
    /// `_bucket`, with its `le` bound. `+Inf` is `f64::INFINITY`.
    Bucket(f64),
    Sum,
    Count,
}

/// One line of an answer.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Sample<'a> {
    /// The name on the `# TYPE` line, such as `http_server_request_duration_seconds`.
    pub family: &'a str,
    pub kind: Kind,
    pub part: Part,
    /// The labels in the order written, without `le`. A value is copied only when it has an escape.
    pub labels: Vec<(&'a str, Cow<'a, str>)>,
    pub value: f64,
}

impl Sample<'_> {
    /// The value of the label `name`, with its escapes read. `None` when the sample has no such label.
    pub(crate) fn label(&self, name: &str) -> Option<&str> {
        self.labels
            .iter()
            .find(|(label, _)| *label == name)
            .map(|(_, value)| value.as_ref())
    }
}

/// The samples of an answer, in its order. A line that cannot be read is skipped.
pub(crate) fn samples(text: &str) -> impl Iterator<Item = Sample<'_>> {
    // The family of the last `# TYPE` line. `None` before the first one and after one of a kind
    // that is not read.
    let mut family: Option<(&str, Kind)> = None;
    // `lines` ends a line at `\n` and drops the `\r` of `\r\n`.
    text.lines().filter_map(move |line| {
        let line = line.trim_start();
        if let Some(comment) = line.strip_prefix('#') {
            if let Some(found) = type_line(comment) {
                family = found;
            }
            return None;
        }
        let (name, kind) = family?;
        sample(line, name, kind)
    })
}

/// `true` when the text has a `# TYPE` line, of any kind. An answer with none is not metrics, such
/// as a sign-in page served with 200. An answer with only families that are not read still has one.
pub(crate) fn typed(text: &str) -> bool {
    text.lines().any(|line| {
        line.trim_start()
            .strip_prefix('#')
            .is_some_and(|comment| type_line(comment).is_some())
    })
}

/// Reads `TYPE name kind`, the text after `#`. `None` when the comment is not a `TYPE` line, such as
/// `HELP`. `Some(None)` when it names a kind that is not read.
fn type_line(comment: &str) -> Option<Option<(&str, Kind)>> {
    let mut words = comment.split_ascii_whitespace();
    if words.next() != Some("TYPE") {
        return None;
    }
    let Some(name) = words.next() else {
        return Some(None);
    };
    let kind = match words.next() {
        Some("counter") => Kind::Counter,
        Some("gauge") => Kind::Gauge,
        Some("histogram") => Kind::Histogram,
        _ => return Some(None),
    };
    Some(Some((name, kind)))
}

/// Reads a sample line of the family `family`. `None` when the line belongs to no part of the family
/// or cannot be read.
fn sample<'a>(line: &'a str, family: &'a str, kind: Kind) -> Option<Sample<'a>> {
    let end = line.find(['{', ' ', '\t']).unwrap_or(line.len());
    let (name, rest) = line.split_at(end);
    // The name decides first, so a line of another family costs no more than this.
    let part = match kind {
        Kind::Counter | Kind::Gauge => (name == family).then_some(Part::Value)?,
        Kind::Histogram => match name.strip_prefix(family)? {
            // The bound comes from `le` below.
            "_bucket" => Part::Bucket(0.0),
            "_sum" => Part::Sum,
            "_count" => Part::Count,
            _ => return None,
        },
    };
    let mut labels = Vec::new();
    let mut le = None;
    let rest = match rest.strip_prefix('{') {
        Some(inside) => read_labels(inside, &mut labels, &mut le)?,
        None => rest,
    };
    // A timestamp, a whole number of milliseconds, may follow the value. It is not used. Anything
    // else after the value makes the line one that cannot be read.
    let mut fields = rest.split_ascii_whitespace();
    let value = fields.next()?.parse().ok()?;
    if let Some(timestamp) = fields.next() {
        timestamp.parse::<i64>().ok()?;
        if fields.next().is_some() {
            return None;
        }
    }
    let part = match part {
        Part::Bucket(_) => Part::Bucket(le?.parse().ok()?),
        part => part,
    };
    Some(Sample {
        family,
        kind,
        part,
        labels,
        value,
    })
}

/// Reads the labels after `{` into `labels`, and the value of `le` apart. Gives the text after `}`.
/// `None` when a label has no name or no `=`, a value has no quotes, a value is followed by
/// neither `,` nor `}`, or the closing `}` is missing.
fn read_labels<'a>(
    mut rest: &'a str,
    labels: &mut Vec<(&'a str, Cow<'a, str>)>,
    le: &mut Option<Cow<'a, str>>,
) -> Option<&'a str> {
    loop {
        rest = rest.trim_start_matches([' ', '\t']);
        if let Some(after) = rest.strip_prefix('}') {
            return Some(after);
        }
        let (name, after) = rest.split_once('=')?;
        let name = name.trim_end();
        if !is_label_name(name) {
            return None;
        }
        let (value, after) = label_value(after.trim_start().strip_prefix('"')?)?;
        if name == "le" {
            *le = Some(value);
        } else {
            labels.push((name, value));
        }
        let after = after.trim_start_matches([' ', '\t']);
        rest = match after.strip_prefix(',') {
            Some(after) => after,
            None if after.starts_with('}') => after,
            None => return None,
        };
    }
}

/// `[a-zA-Z_][a-zA-Z0-9_]*`, the names the format allows.
fn is_label_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// Reads a label value after its opening quote, up to the closing quote. Gives the value and the
/// text after the quote. `None` when the quote is not closed on the line.
///
/// `\\`, `\"` and `\n` are escapes. Any other backslash stays as written.
fn label_value(text: &str) -> Option<(Cow<'_, str>, &str)> {
    let bytes = text.as_bytes();
    // Filled only once a backslash is met. `copied` is where the text not yet copied starts.
    let mut owned: Option<String> = None;
    let mut copied = 0;
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'"' => {
                let value = match owned {
                    None => Cow::Borrowed(&text[..at]),
                    Some(mut value) => {
                        value.push_str(&text[copied..at]);
                        Cow::Owned(value)
                    }
                };
                return Some((value, &text[at + 1..]));
            }
            b'\\' => {
                let value = owned.get_or_insert_with(String::new);
                value.push_str(&text[copied..at]);
                let (read, width) = match bytes.get(at + 1) {
                    Some(b'\\') => ('\\', 2),
                    Some(b'"') => ('"', 2),
                    Some(b'n') => ('\n', 2),
                    _ => ('\\', 1),
                };
                value.push(read);
                at += width;
                copied = at;
            }
            _ => at += 1,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const REQUESTS: &str = "http_server_request_duration_seconds";

    #[test]
    fn reads_the_recorded_answers() {
        let busy = include_str!("../../fixtures/actuator/prometheus-busy.txt");
        let read: Vec<Sample<'_>> = samples(busy).collect();

        let job = read
            .iter()
            .find(|s| s.family == "messaging_process_duration_seconds")
            .unwrap();
        assert_eq!(
            job.label("laravel_job_name"),
            Some(r"Modules\Blog\Jobs\PublishPost")
        );

        let blog: Vec<&Sample<'_>> = read
            .iter()
            .filter(|s| s.family == REQUESTS && s.label("http_route") == Some("/blog"))
            .collect();
        let last: Vec<(Part, f64)> = blog[blog.len() - 3..]
            .iter()
            .map(|s| (s.part, s.value))
            .collect();
        assert_eq!(
            last,
            [
                (Part::Bucket(f64::INFINITY), 20.0),
                (Part::Sum, 0.244_769_096_374_511_72),
                (Part::Count, 20.0),
            ]
        );
        assert_eq!(blog[0].part, Part::Bucket(0.005));
        assert_eq!(blog[0].label("le"), None);
        assert_eq!(blog[0].label("http_response_status_code"), Some("200"));

        let exceptions = read
            .iter()
            .find(|s| s.family == "laravel_exceptions_total")
            .unwrap();
        assert_eq!(
            (exceptions.kind, exceptions.part, exceptions.value),
            (Kind::Counter, Part::Value, 3.0)
        );
        assert_eq!(exceptions.label("exception_type"), Some("RuntimeException"));
        let scripts = read
            .iter()
            .find(|s| s.family == "php_opcache_scripts")
            .unwrap();
        assert_eq!(
            (scripts.kind, scripts.part, scripts.value),
            (Kind::Gauge, Part::Value, 1246.0)
        );

        let request_lines = busy.lines().filter(|l| l.starts_with(REQUESTS)).count();
        let request_samples = read.iter().filter(|s| s.family == REQUESTS).count();
        assert_eq!(request_samples, request_lines);

        let quiet = include_str!("../../fixtures/actuator/prometheus-quiet.txt");
        let quiet_lines = quiet.lines().filter(|l| !l.starts_with('#')).count();
        let quiet: Vec<Sample<'_>> = samples(quiet).collect();
        assert_eq!(quiet.len(), quiet_lines);
        assert!(
            quiet
                .iter()
                .all(|s| s.part == Part::Value && s.kind != Kind::Histogram)
        );

        let empty = include_str!("../../fixtures/actuator/prometheus-empty.txt");
        assert_eq!(samples(empty).count(), 0);
    }

    #[test]
    fn skips_what_it_cannot_read() {
        let text = r#"# HELP a A histogram.
# TYPE a histogram
a_bucket{x="1",le="+Inf"} 2
a_sum{x="1"} 0.5
# TYPE s summary
s{quantile="0.5"} 1
s_sum 1
s_count 1
# TYPE b histogram
untyped 9
b_count{x="a\"b\nc\\d\e"} 4 1700000000000
b_sum{x="no value"}
b_sum{x="open} 5
b_sum{x="a" y="b"} 5
b_sum{,x="a"} 5
b_count{x="last" , } 6
b_count{x="bad time"} 8 soon
b_count{x="extra"} 8 1700000000000 more
# TYPE c counter
c 7
"#;
        let sample = |family, kind, part, labels: &[(&'static str, &'static str)], value| Sample {
            family,
            kind,
            part,
            labels: labels
                .iter()
                .map(|&(name, value)| (name, Cow::Borrowed(value)))
                .collect(),
            value,
        };
        let expected = vec![
            sample(
                "a",
                Kind::Histogram,
                Part::Bucket(f64::INFINITY),
                &[("x", "1")],
                2.0,
            ),
            sample("a", Kind::Histogram, Part::Sum, &[("x", "1")], 0.5),
            sample(
                "b",
                Kind::Histogram,
                Part::Count,
                &[("x", "a\"b\nc\\d\\e")],
                4.0,
            ),
            sample("b", Kind::Histogram, Part::Count, &[("x", "last")], 6.0),
            sample("c", Kind::Counter, Part::Value, &[], 7.0),
        ];
        assert_eq!(samples(text).collect::<Vec<_>>(), expected);
        let crlf = text.replace('\n', "\r\n");
        assert_eq!(samples(&crlf).collect::<Vec<_>>(), expected);
    }
}
