//! The `accent-sass` command-line binary.
//!
//! The binary is a drop-in for the `sass` executable: every flag it implements
//! takes dart-sass's name and values. That contract is load-bearing rather
//! than cosmetic -- the sass-spec runner, `.github/scripts/frameworks.sh` and
//! the WASI smoke scripts all drive this binary directly -- so the command
//! line has no subcommands, and verification is the `--check` flag rather than
//! a `verify` subcommand that would make a bare file argument ambiguous.
//!
//! See `specs/docs/features/29-command-line-check.md`.

use std::{
    fs,
    io::{ErrorKind, Read, Write, stdin, stdout},
    path::Path,
    process::ExitCode,
    str,
};

use clap::{Arg, ArgAction, ArgMatches, Command, ValueEnum, builder::PossibleValue, value_parser};

use accent_sass::{InputSyntax, Options, OutputStyle, from_path, from_string};

/// The stylesheet compiled, and matched the output file if one was named.
const EXIT_OK: u8 = 0;

/// The stylesheet did not compile, or a file could not be read or written.
const EXIT_ERROR: u8 = 1;

/// Under `--check`, the output file is stale or missing.
///
/// Distinct from [`EXIT_ERROR`] on purpose: "your Sass is broken" and "your CSS
/// is out of date" call for different responses from whoever reads the log, and
/// one code for both would make the log lie about which happened. clap exits 2
/// for a usage error, so 3 is the first code free to mean this.
const EXIT_STALE: u8 = 3;

/// Flags dart-sass implements, this binary parses, and neither can be silent
/// about.
///
/// Each would change what the program does if it worked, so accepting one
/// without a word would be a lie: a `--watch` that compiles once and exits
/// looks exactly like a watcher that missed every change. dart-sass's other
/// unimplemented flags -- source map suppression, error CSS, colour, precision,
/// verbosity -- already describe what this binary does, and pass without
/// comment.
const UNIMPLEMENTED: [(&str, &str); 6] = [
    ("UPDATE", "--update"),
    ("WATCH", "--watch"),
    ("POLL", "--poll"),
    ("INTERACTIVE", "--interactive"),
    ("EMBED_SOURCES", "--embed-sources"),
    ("EMBED_SOURCE_MAP", "--embed-source-map"),
];

/// How much whitespace the compiler leaves in the CSS it writes.
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum Style {
    Expanded,
    Compressed,
}

impl ValueEnum for Style {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Expanded, Self::Compressed]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Expanded => PossibleValue::new("expanded"),
            Self::Compressed => PossibleValue::new("compressed"),
        })
    }
}

/// How a source map links back to the files it was built from.
///
/// Parsed for command-line compatibility only; this binary writes no source
/// maps.
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum SourceMapUrls {
    Relative,
    Absolute,
}

impl ValueEnum for SourceMapUrls {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Relative, Self::Absolute]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Relative => PossibleValue::new("relative"),
            Self::Absolute => PossibleValue::new("absolute"),
        })
    }
}

/// Build the argument parser.
///
/// Every argument that does not take a value is declared with
/// [`ArgAction::SetTrue`]. Leaving the action off makes clap default to
/// `ArgAction::Set`, which takes one -- and a flag that takes a value silently
/// eats the file name after it, which is how ten of these once behaved.
fn cli() -> Command {
    Command::new("accent-sass")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A Sass compiler written purely in Rust")
        .disable_version_flag(true)
        .arg(
            Arg::new("version")
                .action(ArgAction::Version)
                .long("version")
                .short('v')
                .global(true)
        )
        .arg(
            Arg::new("STDIN")
                .action(ArgAction::SetTrue)
                .long("stdin")
                .help("Read the stylesheet from stdin"),
        )
        .arg(
            Arg::new("INDENTED")
                .action(ArgAction::SetTrue)
                .long("indented")
                .help("Use the indented syntax for the entry point, whatever its name"),
        )
        .arg(
            Arg::new("CHECK")
                .action(ArgAction::SetTrue)
                .long("check")
                .help("Compile without writing. With an OUTPUT, verify it is up to date"),
        )
        .arg(
            Arg::new("LOAD_PATH")
                .short('I')
                .long("load-path")
                .help("A path to use when resolving imports. May be passed multiple times.")
                .action(ArgAction::Append)
                .value_parser(value_parser!(String))
                .num_args(1)
        )
        .arg(
            Arg::new("STYLE")
                // this is required for compatibility with ruby sass
                .short_alias('t')
                .short('s')
                .long("style")
                .help("Minified or expanded output")
                .default_value("expanded")
                .ignore_case(true)
                .num_args(1)
                .value_parser(value_parser!(Style)),
        )
        .arg(
            Arg::new("NO_CHARSET")
                .action(ArgAction::SetTrue)
                .long("no-charset")
                .help("Don't emit a @charset or BOM for CSS with non-ASCII characters."),
        )
        .arg(
            Arg::new("UPDATE")
                .action(ArgAction::SetTrue)
                .long("update")
                .hide(true)
                .help("Only compile out-of-date stylesheets."),
        )
        .arg(
            Arg::new("NO_ERROR_CSS")
                .action(ArgAction::SetTrue)
                .long("no-error-css")
                .hide(true)
                .help("When an error occurs, don't emit a stylesheet describing it."),
        )
        // Source maps
        .arg(
            Arg::new("NO_SOURCE_MAP")
                .action(ArgAction::SetTrue)
                .long("no-source-map")
                .hide(true)
                .help("Whether to generate source maps."),
        )
        .arg(
            Arg::new("SOURCE_MAP_URLS")
                .long("source-map-urls")
                .hide(true)
                .help("How to link from source maps to source files.")
                .default_value("relative")
                .ignore_case(true)
                .num_args(1)
                .value_parser(value_parser!(SourceMapUrls)),
        )
        .arg(
            Arg::new("EMBED_SOURCES")
                .action(ArgAction::SetTrue)
                .long("embed-sources")
                .hide(true)
                .help("Embed source file contents in source maps."),
        )
        .arg(
            Arg::new("EMBED_SOURCE_MAP")
                .action(ArgAction::SetTrue)
                .long("embed-source-map")
                .hide(true)
                .help("Embed source map contents in CSS."),
        )
        // Other
        .arg(
            Arg::new("WATCH")
                .action(ArgAction::SetTrue)
                .long("watch")
                .hide(true)
                .help("Watch stylesheets and recompile when they change."),
        )
        .arg(
            Arg::new("POLL")
                .action(ArgAction::SetTrue)
                .long("poll")
                .hide(true)
                .help("Manually check for changes rather than using a native watcher. Only valid with --watch.")
                .requires("WATCH"),
        )
        .arg(
            Arg::new("NO_STOP_ON_ERROR")
                .action(ArgAction::SetTrue)
                .long("no-stop-on-error")
                .hide(true)
                .help("Continue to compile more files after error is encountered.")
        )
        .arg(
            Arg::new("INTERACTIVE")
                .action(ArgAction::SetTrue)
                .short('i')
                .long("interactive")
                .hide(true)
                .help("Run an interactive SassScript shell.")
        )
        .arg(
            Arg::new("NO_COLOR")
                .short('c')
                .action(ArgAction::SetTrue)
                .long("no-color")
                .hide(true)
                .help("Whether to use terminal colors for messages.")
        )
        .arg(
            Arg::new("VERBOSE")
                .action(ArgAction::SetTrue)
                .long("verbose")
                .help("Print all deprecation warnings even when they're repetitive.")
        )
        .arg(
            Arg::new("NO_UNICODE")
                .action(ArgAction::SetTrue)
                .long("no-unicode")
                .help("Whether to use Unicode characters for messages.")
        )
        .arg(
            Arg::new("QUIET")
                .action(ArgAction::SetTrue)
                .short('q')
                .long("quiet")
                .help("Don't print warnings."),
        )
        .arg(
            Arg::new("INPUT")
                .value_parser(value_parser!(String))
                .required_unless_present("STDIN")
                .help("The stylesheet to compile. With --stdin, the CSS file to write"),
        )
        .arg(
            // With `--stdin` the stylesheet arrives on standard input and the
            // lone positional is the destination, as in dart-sass, so a second
            // one means the command line was misread. Without this conflict it
            // binds here and is silently ignored.
            Arg::new("OUTPUT")
                .conflicts_with("STDIN")
                .help("Output CSS file")
        )

        // Hidden, legacy arguments
        .arg(
            Arg::new("PRECISION")
                .long("precision")
                .hide(true)
                .num_args(1)
        )
}

fn main() -> ExitCode {
    let matches = cli().get_matches();

    warn_about_unimplemented(&matches);

    let load_paths = matches
        .get_many::<String>("LOAD_PATH")
        .map_or_else(Vec::new, |vals| vals.map(Path::new).collect());

    let style = match &matches.get_one::<Style>("STYLE").unwrap() {
        Style::Expanded => OutputStyle::Expanded,
        Style::Compressed => OutputStyle::Compressed,
    };

    let mut options = Options::default()
        .load_paths(&load_paths)
        .style(style)
        .quiet(matches.get_flag("QUIET"))
        .verbose(matches.get_flag("VERBOSE"))
        .unicode_error_messages(!matches.get_flag("NO_UNICODE"))
        .allows_charset(!matches.get_flag("NO_CHARSET"));

    // Standard input has no extension to infer from, which is the whole reason
    // the flag exists; for a named file it overrides what the extension says,
    // as it does in dart-sass.
    if matches.get_flag("INDENTED") {
        options = options.input_syntax(InputSyntax::Sass);
    }

    // With `--stdin` the stylesheet comes from standard input and the lone
    // positional names the destination, as in dart-sass. Reading it as the
    // input instead is not a harmless mix-up: `--stdin --check app.css` then
    // compiles app.css, never reads standard input, finds no output file left
    // to compare against, and reports success having verified nothing.
    let (input, output) = if matches.get_flag("STDIN") {
        (None, matches.get_one::<String>("INPUT").map(String::as_str))
    } else {
        (
            matches.get_one::<String>("INPUT").map(String::as_str),
            matches.get_one::<String>("OUTPUT").map(String::as_str),
        )
    };

    // Compile before touching the output file. Opening it first, as this once
    // did, means a stylesheet that stops compiling takes the last good CSS with
    // it: the file is truncated and then nothing is written.
    let css = match compile(input, &options) {
        Ok(css) => css,
        Err(code) => return ExitCode::from(code),
    };

    if matches.get_flag("CHECK") {
        return check(&css, output);
    }

    let written = match output {
        Some(path) => fs::write(path, &css),
        None => stdout().write_all(css.as_bytes()),
    };

    if let Err(e) = written {
        eprintln!("Error: {e}");
        return ExitCode::from(EXIT_ERROR);
    }

    ExitCode::from(EXIT_OK)
}

/// Say so for each unimplemented flag the command line carries.
///
/// `--quiet` does not silence these. `-q` covers what the *stylesheet* says --
/// `@warn`, `@debug`, deprecations -- and these report that the command line
/// asked for something it will not get, which is not the stylesheet's to hide.
fn warn_about_unimplemented(matches: &ArgMatches) {
    for (id, flag) in UNIMPLEMENTED {
        if matches.get_flag(id) {
            eprintln!("Warning: {flag} is not implemented, and is ignored.");
        }
    }
}

/// Compile the entry point, reading standard input when `input` is `None`.
///
/// The error case is the process exit code to use, the message having already
/// been printed: a compile error goes to standard error in the compiler's own
/// words, unchanged, because sass-spec compares that text byte for byte.
fn compile(input: Option<&str>, options: &Options) -> Result<String, u8> {
    let result = if let Some(name) = input {
        from_path(name, options)
    } else {
        let mut buffer = String::new();
        if let Err(e) = stdin().read_to_string(&mut buffer) {
            eprintln!("Error: {e}");
            return Err(EXIT_ERROR);
        }
        from_string(buffer, options)
    };

    result.map_err(|e| {
        eprintln!("{e}");
        EXIT_ERROR
    })
}

/// Report whether the compiled CSS is what the output file already holds.
///
/// Writes nothing, which is the point of the mode: it reports whether a build
/// is current without being able to change the answer. With no output file
/// there is nothing to compare against, and the compile having already
/// succeeded is the whole check.
fn check(css: &str, output: Option<&str>) -> ExitCode {
    let Some(path) = output else {
        return ExitCode::from(EXIT_OK);
    };

    // Bytes rather than text. A file that is not valid UTF-8 cannot be what
    // this compile produced, so it is stale; `read_to_string` calls it an I/O
    // failure and exits 1, collapsing the distinction between a broken
    // stylesheet and an out-of-date file that the separate codes exist to draw.
    let existing = match fs::read(path) {
        Ok(existing) => existing,
        // Under `--check`, absent means the build has not run rather than that
        // anything failed. A file that exists and cannot be read is a real I/O
        // failure, and keeps the error code.
        // Still stale rather than an error: a file that is not there means the
        // build has not run, which is exactly what 3 says. Only the advice needs
        // care. This binary writes a file but does not create the directory
        // above it, so telling someone to drop `--check` when the parent is
        // missing sends them to a run that fails with `No such file or
        // directory`, and the advice would be provably false.
        Err(e) if e.kind() == ErrorKind::NotFound => {
            match Path::new(path).parent() {
                Some(parent) if !parent.as_os_str().is_empty() && !parent.is_dir() => {
                    eprintln!(
                        "{path} does not exist, and neither does {}.",
                        parent.display()
                    );
                }
                _ => eprintln!("{path} does not exist. Run without --check to create it."),
            }
            return ExitCode::from(EXIT_STALE);
        }
        Err(e) => {
            eprintln!("Error: cannot read {path}: {e}");
            return ExitCode::from(EXIT_ERROR);
        }
    };

    if existing == css.as_bytes() {
        return ExitCode::from(EXIT_OK);
    }

    eprintln!("{path} is out of date.");
    match str::from_utf8(&existing) {
        Ok(existing) => report_difference(existing, css),
        Err(_) => eprintln!("  {path} is not valid UTF-8, so it cannot be this stylesheet's CSS"),
    }

    ExitCode::from(EXIT_STALE)
}

/// Say where the file on disk and the compiled CSS part company.
fn report_difference(existing: &str, css: &str) {
    match first_difference(existing, css) {
        Some((number, on_disk, compiled)) => {
            eprintln!("  first difference on line {number}:");
            eprintln!("    on disk:  {}", quoted(on_disk));
            eprintln!("    compiled: {}", quoted(compiled));
            // Trailing whitespace, a tab where spaces were, a stray byte-order
            // mark: each prints as nothing, so without a word here the two lines
            // above are the same picture and the one diagnostic this mode exists
            // for tells the reader nothing.
            if let (Some(on_disk), Some(compiled)) = (on_disk, compiled)
                && on_disk.trim() == compiled.trim()
            {
                eprintln!("  they differ only in whitespace.");
            }
        }
        // `str::lines` strips a carriage return along with the newline, so two
        // files differing only in their line endings have no differing line.
        // Blaming a trailing newline for that sends a Windows checkout, where
        // `core.autocrlf` makes it the usual case, hunting for the wrong thing.
        None if existing.contains('\r') != css.contains('\r') => {
            eprintln!("  every line matches; the files differ in their line endings");
        }
        None => eprintln!("  every line matches; the files differ in a trailing newline"),
    }
}

/// Quote a line so that trailing whitespace has a visible boundary.
///
/// The quotes are the point: `  color: red; ` and `  color: red;` are the same
/// picture without them.
fn quoted(line: Option<&str>) -> String {
    match line {
        Some(line) => format!("\"{line}\""),
        None => "<end of file>".to_owned(),
    }
}

/// The 1-based number of the first line that differs, and both versions of it.
///
/// `None` where one side runs out first; `None` for the whole result means
/// every line matched, which for two strings known to differ leaves a trailing
/// newline or a line-ending difference -- `str::lines` strips a carriage return
/// along with the newline, so CRLF and LF text compare equal line by line.
///
/// Diffing the whole file is deliberately not attempted. The mode answers
/// whether the output is current, and one line separates a stale build from a
/// wrong one; a full diff is what `diff` is for.
fn first_difference<'a>(
    expected: &'a str,
    actual: &'a str,
) -> Option<(usize, Option<&'a str>, Option<&'a str>)> {
    let mut expected = expected.lines();
    let mut actual = actual.lines();
    let mut number = 0;

    loop {
        number += 1;
        match (expected.next(), actual.next()) {
            (None, None) => return None,
            (left, right) if left == right => {}
            (left, right) => return Some((number, left, right)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{cli, first_difference};

    #[test]
    fn cli_definition_is_valid() {
        cli().debug_assert();
    }

    #[test]
    fn identical_text_has_no_first_difference() {
        assert_eq!(first_difference("a\nb\n", "a\nb\n"), None);
    }

    #[test]
    fn a_trailing_newline_alone_has_no_first_difference() {
        assert_eq!(first_difference("a\nb\n", "a\nb"), None);
    }

    #[test]
    fn a_changed_line_is_reported_with_both_versions() {
        assert_eq!(
            first_difference("a\nb\nc\n", "a\nB\nc\n"),
            Some((2, Some("b"), Some("B")))
        );
    }

    #[test]
    fn a_missing_line_is_reported_against_nothing() {
        assert_eq!(
            first_difference("a\nb\n", "a\n"),
            Some((2, Some("b"), None))
        );
        assert_eq!(
            first_difference("a\n", "a\nb\n"),
            Some((2, None, Some("b")))
        );
    }
}
