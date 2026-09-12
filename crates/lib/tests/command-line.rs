//! Tests for the `accent-sass` binary, run as a process.
//!
//! The rest of this directory calls the library through the `test!` and
//! `error!` macros, which cannot see argument parsing, an exit status, or a
//! file left on disk. Every defect these tests cover lives in one of those
//! three, so they drive the built binary instead.
//!
//! See `specs/docs/features/29-command-line-check.md`.
#![cfg(feature = "commandline")]

use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

use tempfile::TempDir;

/// The binary cargo built for this test run.
const BIN: &str = env!("CARGO_BIN_EXE_accent-sass");

/// Compiled from `a { color: red; }`, which every fixture here starts from.
const RED: &str = "a {\n  color: red;\n}\n";

/// Run the binary in `dir`, optionally feeding it standard input.
fn run(dir: &Path, args: &[&str], input: Option<&str>) -> Output {
    let mut child = Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to run the accent-sass binary");

    if let Some(input) = input {
        child
            .stdin
            .as_mut()
            .expect("stdin is piped when there is input")
            .write_all(input.as_bytes())
            .expect("failed to write to the child's stdin");
    }

    // `wait_with_output` drops the child's stdin first, so the compiler sees
    // end of file rather than waiting for more.
    child
        .wait_with_output()
        .expect("failed to collect the child's output")
}

fn code(output: &Output) -> i32 {
    output
        .status
        .code()
        .expect("the binary was killed by a signal")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout is not UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr is not UTF-8")
}

/// A directory holding `good.scss`, which compiles, and `bad.scss`, which does
/// not.
fn fixture() -> TempDir {
    let dir = tempfile::tempdir().expect("failed to create a temporary directory");
    fs::write(dir.path().join("good.scss"), "a { color: red; }\n").unwrap();
    fs::write(dir.path().join("bad.scss"), "a { color: ; }\n").unwrap();
    dir
}

#[test]
fn compiles_a_file_to_stdout() {
    let dir = fixture();
    let output = run(dir.path(), &["good.scss"], None);

    assert_eq!(code(&output), 0);
    assert_eq!(stdout(&output), RED);
}

#[test]
fn compiles_a_file_to_an_output_file() {
    let dir = fixture();
    let output = run(dir.path(), &["good.scss", "out.css"], None);

    assert_eq!(code(&output), 0);
    assert_eq!(stdout(&output), "");
    assert_eq!(fs::read_to_string(dir.path().join("out.css")).unwrap(), RED);
}

/// Criterion 1: a switch must not consume the argument after it.
///
/// All ten were declared without an action, so clap defaulted to
/// `ArgAction::Set` and each read the file name as its own value, leaving no
/// input file and exiting 2.
#[test]
fn unimplemented_flags_do_not_swallow_the_input() {
    let dir = fixture();

    for flag in [
        "--update",
        "--no-error-css",
        "--no-source-map",
        "--embed-sources",
        "--embed-source-map",
        "--watch",
        "--no-stop-on-error",
        "--interactive",
    ] {
        let output = run(dir.path(), &[flag, "good.scss"], None);

        assert_eq!(code(&output), 0, "{flag} did not compile the stylesheet");
        assert_eq!(stdout(&output), RED, "{flag} changed the CSS");
    }
}

/// `--poll` requires `--watch`, so it is the one flag that cannot be passed
/// alone.
#[test]
fn poll_does_not_swallow_the_input_either() {
    let dir = fixture();
    let output = run(dir.path(), &["--watch", "--poll", "good.scss"], None);

    assert_eq!(code(&output), 0);
    assert_eq!(stdout(&output), RED);
}

/// A flag that would change what the program does says it is ignored; one that
/// already describes what the program does stays silent.
#[test]
fn only_the_flags_that_would_change_behaviour_warn() {
    let dir = fixture();

    let output = run(dir.path(), &["--watch", "good.scss"], None);
    assert_eq!(
        stderr(&output),
        "Warning: --watch is not implemented, and is ignored.\n"
    );

    let output = run(dir.path(), &["--no-source-map", "good.scss"], None);
    assert_eq!(stderr(&output), "");
}

/// Criterion 2: standard input is the one route with no extension to infer
/// from, and the flag exists to serve it.
#[test]
fn indented_switches_the_syntax_of_stdin() {
    let dir = fixture();
    let output = run(
        dir.path(),
        &["--stdin", "--indented"],
        Some("a\n  color: red\n"),
    );

    assert_eq!(code(&output), 0, "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), RED);
}

#[test]
fn stdin_without_indented_is_scss() {
    let dir = fixture();
    let output = run(dir.path(), &["--stdin"], Some("a { color: red; }\n"));

    assert_eq!(code(&output), 0);
    assert_eq!(stdout(&output), RED);
}

/// `--indented` overrides the extension for the entry point, as in dart-sass.
#[test]
fn indented_overrides_the_extension() {
    let dir = fixture();
    fs::write(dir.path().join("indented.scss"), "a\n  color: red\n").unwrap();

    let output = run(dir.path(), &["--indented", "indented.scss"], None);

    assert_eq!(code(&output), 0, "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), RED);
}

/// Criterion 3: the output file was opened with `truncate(true)` before the
/// compile ran, so a stylesheet that stopped compiling took the last good CSS
/// with it.
#[test]
fn a_failed_compile_leaves_the_output_file_alone() {
    let dir = fixture();
    let out = dir.path().join("out.css");

    assert_eq!(code(&run(dir.path(), &["good.scss", "out.css"], None)), 0);
    assert_eq!(fs::read_to_string(&out).unwrap(), RED);

    let output = run(dir.path(), &["bad.scss", "out.css"], None);

    assert_eq!(code(&output), 1);
    assert!(stderr(&output).starts_with("Error: Expected expression."));
    assert_eq!(
        fs::read_to_string(&out).unwrap(),
        RED,
        "a failed compile must not touch the output file"
    );
}

#[test]
fn a_missing_input_leaves_the_output_file_alone() {
    let dir = fixture();
    let out = dir.path().join("out.css");

    assert_eq!(code(&run(dir.path(), &["good.scss", "out.css"], None)), 0);

    let output = run(dir.path(), &["nonexistent.scss", "out.css"], None);

    assert_eq!(code(&output), 1);
    assert_eq!(fs::read_to_string(&out).unwrap(), RED);
}

/// Criterion 5, and criterion 4 for the no-output case.
#[test]
fn check_without_an_output_verifies_only_that_it_compiles() {
    let dir = fixture();

    let output = run(dir.path(), &["--check", "good.scss"], None);
    assert_eq!(code(&output), 0);
    assert_eq!(stdout(&output), "", "--check must not write the CSS");

    let output = run(dir.path(), &["--check", "bad.scss"], None);
    assert_eq!(code(&output), 1);
    assert!(stderr(&output).starts_with("Error: Expected expression."));
}

#[test]
fn check_accepts_a_current_output_file() {
    let dir = fixture();
    fs::write(dir.path().join("out.css"), RED).unwrap();

    let output = run(dir.path(), &["--check", "good.scss", "out.css"], None);

    assert_eq!(code(&output), 0, "stderr: {}", stderr(&output));
    assert_eq!(stderr(&output), "");
}

#[test]
fn check_rejects_a_stale_output_file() {
    let dir = fixture();
    let stale = "a {\n  color: blue;\n}\n";
    fs::write(dir.path().join("out.css"), stale).unwrap();

    let output = run(dir.path(), &["--check", "good.scss", "out.css"], None);

    assert_eq!(code(&output), 3);
    let stderr = stderr(&output);
    assert!(stderr.contains("out.css is out of date."), "{stderr}");
    assert!(stderr.contains("first difference on line 2"), "{stderr}");
    assert!(stderr.contains("on disk:    color: blue;"), "{stderr}");
    assert!(stderr.contains("compiled:   color: red;"), "{stderr}");

    // Criterion 4: reporting the file as stale must not fix it.
    assert_eq!(
        fs::read_to_string(dir.path().join("out.css")).unwrap(),
        stale,
        "--check must not write the output file"
    );
}

#[test]
fn check_reports_a_missing_output_file_as_stale() {
    let dir = fixture();

    let output = run(dir.path(), &["--check", "good.scss", "out.css"], None);

    assert_eq!(code(&output), 3);
    assert!(stderr(&output).contains("out.css does not exist."));
    assert!(
        !dir.path().join("out.css").exists(),
        "--check must not create the output file"
    );
}

/// A stylesheet that does not compile is exit 1 even when the output file is
/// also stale: the compile never got far enough to compare anything.
#[test]
fn check_reports_a_compile_error_rather_than_staleness() {
    let dir = fixture();
    fs::write(dir.path().join("out.css"), "a {\n  color: blue;\n}\n").unwrap();

    let output = run(dir.path(), &["--check", "bad.scss", "out.css"], None);

    assert_eq!(code(&output), 1);
}

#[test]
fn check_compares_compressed_output_too() {
    let dir = fixture();
    fs::write(dir.path().join("out.css"), "a{color:red}").unwrap();

    let output = run(
        dir.path(),
        &["--check", "--style", "compressed", "good.scss", "out.css"],
        None,
    );

    assert_eq!(code(&output), 0, "stderr: {}", stderr(&output));
}

/// A wrong command line stays clap's exit 2, which is why staleness is 3.
#[test]
fn an_unknown_flag_is_a_usage_error() {
    let dir = fixture();
    let output = run(dir.path(), &["--bogus", "good.scss"], None);

    assert_eq!(code(&output), 2);
}
