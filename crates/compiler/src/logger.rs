use codemap::SpanLoc;
use std::fmt::{self, Debug};

/// A trait to allow replacing logging mechanisms
pub trait Logger: Debug {
    /// Logs message from a [`@debug`](https://sass-lang.com/documentation/at-rules/debug/)
    /// statement
    fn debug(&self, location: SpanLoc, message: &str);

    /// Logs message from a [`@warn`](https://sass-lang.com/documentation/at-rules/warn/)
    /// statement
    fn warn(&self, location: SpanLoc, message: &str);

    /// Logs a warning about a deprecated feature the stylesheet uses.
    ///
    /// The default hands the message and its location to [`Logger::warn`], so
    /// a logger written before deprecation warnings existed still receives
    /// them. [`DeprecationWarning::formatted`] has the full text dart-sass
    /// prints, source frame included.
    fn deprecation(&self, warning: &DeprecationWarning) {
        self.warn(warning.location().clone(), warning.message());
    }

    /// Reports that `count` deprecation warnings were left out because the
    /// same deprecation had already been reported five times.
    ///
    /// Called once, at the end of a compilation, and only when something was
    /// left out; [`crate::Options::verbose`] reports every warning instead.
    /// The default does nothing.
    fn repetitive_deprecations_omitted(&self, count: usize) {
        let _ = count;
    }
}

/// A deprecated feature, named the way dart-sass names it.
///
/// More deprecations will be added, so match with a wildcard arm.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Deprecation {
    /// A selector with a leading, trailing or repeated combinator, such as
    /// `a >` with declarations of its own, which dart-sass will reject in
    /// 2.0.0.
    BogusCombinators,
}

impl Deprecation {
    /// The id dart-sass prints in brackets after `DEPRECATION WARNING`.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::BogusCombinators => "bogus-combinators",
        }
    }
}

impl fmt::Display for Deprecation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

/// A warning about a deprecated feature, with where it happened.
#[derive(Clone, Debug)]
pub struct DeprecationWarning {
    deprecation: Deprecation,
    message: String,
    location: SpanLoc,
    formatted: String,
}

impl DeprecationWarning {
    /// Builds a warning from its parts: `frame` is the source frame drawn
    /// under the message, and `trace` the location line written below it.
    pub(crate) fn new(
        deprecation: Deprecation,
        message: String,
        location: SpanLoc,
        frame: &str,
        trace: &str,
    ) -> Self {
        let formatted = format!(
            "DEPRECATION WARNING [{}]: {}\n\n{}\n    {}",
            deprecation.id(),
            message,
            frame,
            trace
        );

        Self {
            deprecation,
            message,
            location,
            formatted,
        }
    }

    /// Which deprecation this is.
    #[must_use]
    pub const fn deprecation(&self) -> Deprecation {
        self.deprecation
    }

    /// The message alone, with no banner or source frame. It can run over
    /// several lines.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The span the warning is about.
    #[must_use]
    pub const fn location(&self) -> &SpanLoc {
        &self.location
    }

    /// The warning as dart-sass prints it: a `DEPRECATION WARNING [id]:`
    /// banner, the message, a source frame marking every span involved, and
    /// a location line. There is no trailing newline.
    ///
    /// dart-sass writes a stack trace where this writes one location line,
    /// and that line names `root stylesheet` only when the warning comes from
    /// the entry stylesheet outside any mixin or function.
    #[must_use]
    pub fn formatted(&self) -> &str {
        &self.formatted
    }
}

/// Logs events to standard error, through [`eprintln!`]
#[derive(Debug)]
pub struct StdLogger;

impl Logger for StdLogger {
    #[inline]
    fn debug(&self, location: SpanLoc, message: &str) {
        eprintln!(
            "{}:{} DEBUG: {}",
            location.file.name(),
            location.begin.line + 1,
            message
        );
    }

    #[inline]
    fn warn(&self, location: SpanLoc, message: &str) {
        eprintln!(
            "Warning: {}\n    ./{}:{}:{}",
            message,
            location.file.name(),
            location.begin.line + 1,
            location.begin.column + 1
        );
    }

    /// Writes [`DeprecationWarning::formatted`] and a blank line, as dart-sass
    /// does.
    #[inline]
    fn deprecation(&self, warning: &DeprecationWarning) {
        eprintln!("{}\n", warning.formatted());
    }

    #[inline]
    fn repetitive_deprecations_omitted(&self, count: usize) {
        eprintln!(
            "WARNING: {count} repetitive deprecation warnings omitted.\nRun in verbose mode to see all warnings.\n"
        );
    }
}

/// Discards all logs
#[derive(Debug)]
pub struct NullLogger;

impl Logger for NullLogger {
    #[inline]
    fn debug(&self, _location: SpanLoc, _message: &str) {}

    #[inline]
    fn warn(&self, _location: SpanLoc, _message: &str) {}

    #[inline]
    fn deprecation(&self, _warning: &DeprecationWarning) {}
}
