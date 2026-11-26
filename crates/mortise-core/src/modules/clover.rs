//! Test coverage from a clover report, as PHPUnit writes it.

use std::path::{Path, PathBuf};

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};
use serde::{Deserialize, Serialize};

/// Covered and total statements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Coverage {
    pub covered: u64,
    pub statements: u64,
}

impl Coverage {
    /// The whole per cent, rounded: 54 for 54.4%. `None` when there is nothing to cover.
    pub fn percent(self) -> Option<u8> {
        (self.statements > 0)
            .then(|| ((self.covered as f64 / self.statements as f64) * 100.0).round() as u8)
    }

    pub(crate) fn add(&mut self, other: Coverage) {
        self.covered += other.covered;
        self.statements += other.statements;
    }
}

/// The coverage of one file in a report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileCoverage {
    /// The path as the report has it, with `/` between the parts. It is the path
    /// on the machine that ran the tests, which can be a container.
    pub(crate) path: String,
    pub(crate) coverage: Coverage,
}

/// Finds the clover report of an app: the file `phpunit.xml` names, or one of the
/// places where such a report is usually written.
pub(crate) fn find_report(root: &Path) -> Option<PathBuf> {
    let configured = ["phpunit.xml", "phpunit.xml.dist"]
        .iter()
        .filter_map(|name| std::fs::read(root.join(name)).ok())
        .find_map(|xml| report_in_phpunit_config(&xml))
        .map(|path| root.join(path));
    let usual = [
        "build/coverage/clover.xml",
        "build/logs/clover.xml",
        "coverage/clover.xml",
        "clover.xml",
    ];
    configured
        .into_iter()
        .chain(usual.iter().map(|path| root.join(path)))
        .find(|path| path.is_file())
}

/// The `outputFile` of `<coverage><report><clover>` in a PHPUnit config.
fn report_in_phpunit_config(xml: &[u8]) -> Option<String> {
    let mut reader = Reader::from_reader(xml);
    let mut inside: Vec<String> = Vec::new();
    loop {
        match reader.read_event().ok()? {
            Event::Start(tag) => inside.push(tag.name().as_ref().to_owned()),
            Event::End(_) => {
                inside.pop();
            }
            Event::Empty(tag) if tag.name().as_ref() == "clover" => {
                let in_report = inside.last().is_some_and(|parent| parent == "report");
                if in_report && let Some(file) = attribute(&tag, "outputFile") {
                    return Some(file);
                }
            }
            Event::Eof => return None,
            _ => {}
        }
    }
}

/// Reads the statements of every file in a clover report. A report that is not
/// valid XML gives what was read before the fault, and the fault. So does a file
/// without a `coverage` element, which is no clover report.
pub(crate) fn read_report(xml: &[u8]) -> (Vec<FileCoverage>, Option<String>) {
    let mut reader = Reader::from_reader(xml);
    let mut files = Vec::new();
    // The file being read, and how deep inside it the reader is.
    let mut file: Option<(String, usize)> = None;
    let mut clover = false;
    loop {
        let event = match reader.read_event() {
            Ok(Event::Eof) => break,
            Err(error) => return (files, Some(error.to_string())),
            Ok(event) => event,
        };
        match event {
            Event::Start(tag) if tag.name().as_ref() == "coverage" => clover = true,
            Event::Start(tag) if tag.name().as_ref() == "file" => {
                file = attribute(&tag, "name").map(|name| (name.replace('\\', "/"), 0));
            }
            Event::Start(_) => {
                if let Some((_, depth)) = &mut file {
                    *depth += 1;
                }
            }
            Event::End(tag) if tag.name().as_ref() == "file" => file = None,
            Event::End(_) => {
                if let Some((_, depth)) = &mut file {
                    *depth = depth.saturating_sub(1);
                }
            }
            // The metrics of the file itself, not those of a class inside it.
            Event::Empty(tag) if tag.name().as_ref() == "metrics" => {
                if let Some((path, 0)) = &file {
                    let number = |name: &str| {
                        attribute(&tag, name)
                            .and_then(|value| value.parse().ok())
                            .unwrap_or(0)
                    };
                    files.push(FileCoverage {
                        path: path.clone(),
                        coverage: Coverage {
                            covered: number("coveredstatements"),
                            statements: number("statements"),
                        },
                    });
                }
            }
            _ => {}
        }
    }
    let fault = (!clover).then(|| "it has no coverage element".to_owned());
    (files, fault)
}

fn attribute(tag: &BytesStart<'_>, name: &str) -> Option<String> {
    tag.attributes()
        .filter_map(Result::ok)
        .find(|attribute| attribute.key.as_ref() == name)
        .and_then(|attribute| attribute.normalized_value(XmlVersion::Implicit1_0).ok())
        .map(|value| value.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPORT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<coverage generated="1763800000">
  <project timestamp="1763800000">
    <file name="/var/www/html/Modules/Billing/app/Models/Invoice.php">
      <class name="Modules\Billing\Models\Invoice" namespace="global">
        <metrics complexity="3" methods="2" coveredmethods="1" statements="99" coveredstatements="1" elements="12" coveredelements="9"/>
      </class>
      <line num="12" type="stmt" count="1"/>
      <metrics loc="40" ncloc="30" classes="1" methods="2" coveredmethods="1" statements="10" coveredstatements="8" elements="12" coveredelements="9"/>
    </file>
    <package name="Modules\Checkout">
      <file name="C:\code\shop\Modules\Checkout\app\Services\Cart.php">
        <metrics loc="20" statements="4" coveredstatements="1"/>
      </file>
    </package>
    <metrics files="2" statements="14" coveredstatements="9"/>
  </project>
</coverage>"#;

    #[test]
    fn reads_the_statements_of_each_file_and_not_of_its_classes() {
        let (files, fault) = read_report(REPORT.as_bytes());
        assert_eq!(fault, None);
        assert_eq!(
            files,
            [
                FileCoverage {
                    path: "/var/www/html/Modules/Billing/app/Models/Invoice.php".into(),
                    coverage: Coverage {
                        covered: 8,
                        statements: 10
                    },
                },
                FileCoverage {
                    path: "C:/code/shop/Modules/Checkout/app/Services/Cart.php".into(),
                    coverage: Coverage {
                        covered: 1,
                        statements: 4
                    },
                },
            ]
        );
    }

    #[test]
    fn a_broken_report_gives_what_was_read_and_the_fault() {
        let broken = REPORT.replace("</file>\n    <package", "</fil>\n    <package");
        let (files, fault) = read_report(broken.as_bytes());
        assert_eq!(files.len(), 1);
        assert!(fault.is_some());
        for not_clover in [&b"not xml at all"[..], b"", b"<phpunit/>"] {
            let (files, fault) = read_report(not_clover);
            assert!(files.is_empty());
            assert_eq!(fault.as_deref(), Some("it has no coverage element"));
        }
    }

    #[test]
    fn per_cent_is_rounded_and_absent_without_statements() {
        assert_eq!(
            Coverage {
                covered: 8,
                statements: 10
            }
            .percent(),
            Some(80)
        );
        assert_eq!(
            Coverage {
                covered: 544,
                statements: 1000
            }
            .percent(),
            Some(54)
        );
        assert_eq!(
            Coverage {
                covered: 545,
                statements: 1000
            }
            .percent(),
            Some(55)
        );
        assert_eq!(Coverage::default().percent(), None);
    }

    #[test]
    fn the_report_named_in_phpunit_xml_comes_first() {
        let app = tempfile::tempdir().unwrap();
        let write = |path: &str, content: &str| {
            let path = app.path().join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        };
        assert_eq!(find_report(app.path()), None);

        write("build/logs/clover.xml", "<coverage/>");
        assert_eq!(
            find_report(app.path()),
            Some(app.path().join("build/logs/clover.xml"))
        );

        write(
            "phpunit.xml",
            r#"<phpunit><coverage><report><html outputDirectory="x"/><clover outputFile="storage/cov.xml"/></report></coverage></phpunit>"#,
        );
        // Named, but not written yet: the usual place still counts.
        assert_eq!(
            find_report(app.path()),
            Some(app.path().join("build/logs/clover.xml"))
        );
        write("storage/cov.xml", "<coverage/>");
        assert_eq!(
            find_report(app.path()),
            Some(app.path().join("storage/cov.xml"))
        );
    }
}
