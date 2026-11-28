//! What the tests of the command line tool share. Not every test file uses all of it.
#![allow(dead_code)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

pub const TOKEN: &str = "mortise_01jd8k2x9q4m_secretsecretsecret";

pub fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/shop-api")
}

pub fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// A copy of shop-api that a test may change. It is in no git repository.
pub fn copy_of_fixture() -> tempfile::TempDir {
    let app = tempfile::tempdir().unwrap();
    copy_dir(&fixture(), app.path());
    app
}

/// One request the stub took.
pub struct Request {
    pub line: String,
    /// Names in lower case.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

/// A cloud that gives these answers, one per connection, in order: status, extra header
/// lines and body. Gives its base URL and the requests it takes.
pub fn stub(answers: Vec<(u16, &'static str, &'static str)>) -> (String, Receiver<Request>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let (sent, taken) = channel();
    std::thread::spawn(move || {
        for (status, headers, body) in answers {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let mut fields = Vec::new();
            loop {
                let mut field = String::new();
                reader.read_line(&mut field).unwrap();
                let Some((name, value)) = field.trim_end().split_once(':') else {
                    break;
                };
                fields.push((name.to_lowercase(), value.trim().to_owned()));
            }
            let length = fields
                .iter()
                .find(|(name, _)| name == "content-length")
                .map_or(0, |(_, value)| value.parse().unwrap());
            let mut content = vec![0; length];
            reader.read_exact(&mut content).unwrap();
            write!(
                stream,
                "HTTP/1.1 {status} Answer\r\nContent-Type: application/json\r\n{headers}\
                 Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
            let _ = sent.send(Request {
                line: line.trim_end().to_owned(),
                headers: fields,
                body: content,
            });
        }
    });
    (url, taken)
}

/// Every request the stub took, waiting a little for each.
pub fn requests(taken: &Receiver<Request>) -> Vec<Request> {
    std::iter::from_fn(|| taken.recv_timeout(Duration::from_secs(2)).ok()).collect()
}

/// `mortise <command>` with these variables, and none of the variables of the CI or the
/// proxy of the computer that runs the tests. The cloud is a closed port unless `vars`
/// names one.
pub fn mortise(command: &str, args: &[&str], vars: &[(&str, &str)]) -> Output {
    let mut run = Command::new(env!("CARGO_BIN_EXE_mortise"));
    run.arg(command).args(args);
    for (name, _) in std::env::vars() {
        let proxy = name.to_ascii_lowercase().ends_with("_proxy");
        if name.starts_with("GITHUB_") || name.starts_with("MORTISE_") || proxy {
            run.env_remove(name);
        }
    }
    run.env("MORTISE_CLOUD_URL", "http://127.0.0.1:9");
    run.envs(vars.iter().copied()).output().unwrap()
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

/// Runs git in `folder` and fails the test when git does.
pub fn git(folder: &Path, args: &[&str]) {
    git_at(folder, None, args);
}

/// As [`git`]. With `time`, in seconds since 1970, a commit gets that date.
pub fn git_at(folder: &Path, time: Option<i64>, args: &[&str]) {
    let mut command = Command::new("git");
    command
        .current_dir(folder)
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        // Without the settings of this computer, such as its hooks.
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null");
    if let Some(time) = time {
        let date = format!("{time} +0000");
        command
            .env("GIT_AUTHOR_DATE", &date)
            .env("GIT_COMMITTER_DATE", &date);
    }
    let output = command.output().unwrap();
    assert!(output.status.success(), "git {args:?}: {}", stderr(&output));
}

/// What git prints for `args` in `folder`, without the line end.
pub fn git_says(folder: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(folder)
        .args(args)
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}
