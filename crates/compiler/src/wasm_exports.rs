//! A JavaScript API for the browser build.
//!
//! `wasm32-unknown-unknown` plus wasm-bindgen, for a playground, a
//! documentation demo, or client-side theme editing: anywhere that compiles
//! Sass with no filesystem and no process. See
//! `specs/docs/features/12-wasm-browser-package.md`.
//!
//! # What a browser build cannot do, and what replaces it
//!
//! [`StdFs`](crate::StdFs) calls `std::fs`, which on this target compiles and
//! then fails at runtime. A package that only exposed
//! [`from_string`](crate::from_string) could therefore compile a single file
//! and nothing else: every `@use` failed with `Can't find stylesheet to
//! import.` [`compile_string`] and [`compile`] take the stylesheet tree from
//! the caller instead, as a `files` map, and resolve imports against it
//! through [`MemoryFs`].
//!
//! **The caller must have every dependency in memory before it compiles.**
//! [`Fs::read`](crate::Fs::read) is synchronous, so an importer cannot
//! `fetch`, cannot `await`, and cannot reach the File System Access API. An
//! editor loads the theme's files into the map first, then compiles. Making
//! imports async would mean an async evaluator, which is a rewrite rather
//! than a binding change.
//!
//! # The option names follow dart-sass
//!
//! Where dart-sass's JavaScript API has a name for a knob, this uses that
//! name, so a caller that knows one knows the other: `style`, `syntax`,
//! `loadPaths`, `charset`, `alertAscii`, and `url` for the entry point's
//! name. `quiet` is the exception, because dart-sass's JavaScript API
//! silences warnings by passing `Logger.silent` rather than by a flag; its
//! command line calls it `--quiet`, and so does this project's.
//!
//! Two spellings are worth knowing: dart-sass's `Syntax` calls the indented
//! syntax `indented`, not `sass`, and `alertAscii` is the inverse of this
//! crate's [`Options::unicode_error_messages`](crate::Options::unicode_error_messages).
//! Both spellings of the indented syntax are accepted here.
//!
//! # Errors
//!
//! A failed compile throws a real `Error`, so `instanceof Error` holds and a
//! stack trace survives. Its `message` is the Sass message on its own, and it
//! carries `formatted` (the block the command-line compiler prints, with the
//! source line and caret), plus `file`, `line` and `column` for an editor
//! that wants to underline the offending characters. Line and column are
//! 1-based, matching what the formatted block shows.

use codemap::SpanLoc;
use js_sys::{Array, Function, JsString, Map, Object, Reflect};
use wasm_bindgen::prelude::*;

use crate::{
    Error as SassError, ErrorKind, Fs, InputSyntax, Logger, MemoryFs, Options, OutputStyle,
    from_path, from_string, from_string_with_file_name,
};

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &'static str = r#"
/** A stylesheet tree the compiler resolves imports against. Paths are virtual
 * and normalized: "a/b.scss" and "./a/b.scss" name the same file. */
export type SassFiles = Record<string, string> | Map<string, string>;

/** An `@warn` or `@debug` reported while compiling. */
export interface SassLogEvent {
  type: "warn" | "debug";
  message: string;
  file: string;
  /** 1-based. */
  line: number;
  /** 1-based. */
  column: number;
}

export interface CompileOptions {
  /** Output style. Defaults to "expanded". */
  style?: "expanded" | "compressed";
  /** Syntax of the entry point only; imported files always infer their own.
   * Defaults to the extension of `url`, else "scss". */
  syntax?: "scss" | "indented" | "sass" | "css";
  /** Paths searched when a relative import does not resolve. */
  loadPaths?: string[];
  /** The stylesheet tree to resolve imports against. */
  files?: SassFiles;
  /** Virtual path of the source, which relative imports resolve against.
   * Defaults to "stdin". `compile()` ignores it and uses its path argument. */
  url?: string;
  /** Emit `@charset` or a byte-order mark for non-ASCII output. Defaults to true. */
  charset?: boolean;
  /** Restrict error messages to ASCII. Defaults to false. */
  alertAscii?: boolean;
  /** Silence `@warn`, `@debug` and deprecation warnings. Defaults to false. */
  quiet?: boolean;
  /** Called for each `@warn` and `@debug`. Exceptions it throws are ignored. */
  logger?: (event: SassLogEvent) => void;
}

export interface CompileResult {
  css: string;
  /** The files the compile actually read, in the order it read them. */
  loadedUrls: string[];
}

export interface SassException extends Error {
  /** The message with no span or source context. */
  message: string;
  /** The full block the command-line compiler prints. */
  formatted: string;
  file: string;
  /** 1-based. */
  line: number;
  /** 1-based. */
  column: number;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "CompileOptions")]
    pub type CompileOptions;

    #[wasm_bindgen(typescript_type = "CompileResult")]
    pub type CompileResult;
}

/// Compiles a stylesheet held in a string.
///
/// Imports resolve against `options.files`, relative to `options.url`. See the
/// [module documentation](self) for the option names and the synchronous
/// filesystem constraint.
///
/// # Errors
///
/// Throws a `SassException` when the stylesheet does not compile, or a
/// `TypeError` when an option has the wrong shape.
#[wasm_bindgen(js_name = compileString)]
pub fn compile_string(
    source: String,
    options: Option<CompileOptions>,
) -> Result<CompileResult, JsValue> {
    let config = Config::parse(options.as_ref())?;
    let logger = JsLogger::new(config.logger);
    let url = config.url.unwrap_or_else(|| "stdin".to_owned());

    let built = config
        .base
        .build(&config.fs, &logger, &config.load_paths, config.syntax);

    finish(from_string_with_file_name(source, &url, &built), &config.fs)
}

/// Compiles the stylesheet stored at `path` in `options.files`.
///
/// The syntax of the entry point is inferred from its extension unless
/// `options.syntax` overrides it, and relative imports resolve against the
/// file's own directory. `options.url` is ignored, because `path` already
/// names the entry point.
///
/// # Errors
///
/// Throws a `SassException` when the file is missing or does not compile, or a
/// `TypeError` when an option has the wrong shape.
#[wasm_bindgen(js_name = compile)]
pub fn compile(path: String, options: Option<CompileOptions>) -> Result<CompileResult, JsValue> {
    let config = Config::parse(options.as_ref())?;
    let logger = JsLogger::new(config.logger);

    let built = config
        .base
        .build(&config.fs, &logger, &config.load_paths, config.syntax);

    finish(from_path(&path, &built), &config.fs)
}

/// Compiles a single stylesheet with the default options.
///
/// This is the package's original export, kept so that callers written
/// against it keep working. It cannot resolve `@use` or `@import`, because it
/// has no filesystem to resolve them against; use [`compile_string`] for
/// anything with dependencies.
///
/// # Errors
///
/// Throws the error's formatted block as a string, which is what it has
/// always thrown. [`compile_string`] throws a structured error instead.
#[wasm_bindgen(js_name = from_string)]
pub fn from_string_js(input: String) -> Result<String, String> {
    from_string(input, &Options::default()).map_err(|e| e.to_string())
}

/// Turns a finished compile into a `CompileResult`, or throws.
fn finish(result: crate::Result<String>, fs: &MemoryFs) -> Result<CompileResult, JsValue> {
    match result {
        Ok(css) => {
            let loaded = fs
                .loaded_paths()
                .iter()
                .map(|p| JsValue::from_str(&p.to_string_lossy()))
                .collect::<Array>();

            let out = Object::new();
            Reflect::set(&out, &JsValue::from_str("css"), &JsValue::from_str(&css))?;
            Reflect::set(&out, &JsValue::from_str("loadedUrls"), &loaded)?;

            Ok(CompileResult::from(JsValue::from(out)))
        }
        Err(e) => Err(to_js_error(*e)),
    }
}

/// Builds the `SassException` thrown for a failed compile.
///
/// The formatted block is read before the error is consumed for its structured
/// fields, so a caller gets both rather than having to parse one out of the
/// other.
fn to_js_error(error: SassError) -> JsValue {
    let formatted = error.to_string();

    let (message, file, line, column) = match error.kind() {
        ErrorKind::ParseError { message, loc, .. } => {
            let (file, line, column) = span_parts(&loc);
            (message, file, line, column)
        }
        // A missing entry point or a non-UTF-8 file has no span to report.
        // Give the caller the same shape anyway, so it never has to branch.
        ErrorKind::IoError(io) => (io.to_string(), String::new(), 0, 0),
        ErrorKind::FromUtf8Error(message) => (message, String::new(), 0, 0),
    };

    let js = js_sys::Error::new(&message);
    let value = JsValue::from(js);

    // These are best-effort: `Reflect::set` on a fresh Error cannot fail, and
    // there is nothing useful to do if it somehow did, since the error itself
    // is what is being reported.
    let _ = Reflect::set(
        &value,
        &JsValue::from_str("formatted"),
        &JsValue::from_str(&formatted),
    );
    let _ = Reflect::set(
        &value,
        &JsValue::from_str("file"),
        &JsValue::from_str(&file),
    );
    let _ = Reflect::set(&value, &JsValue::from_str("line"), &JsValue::from(line));
    let _ = Reflect::set(&value, &JsValue::from_str("column"), &JsValue::from(column));

    value
}

/// Returns a location's file, line and column, with line and column 1-based.
///
/// [`SpanLoc`] counts both from zero; the formatted error block and every
/// editor count from one.
fn span_parts(loc: &SpanLoc) -> (String, u32, u32) {
    (
        loc.file.name().to_owned(),
        loc.begin.line as u32 + 1,
        loc.begin.column as u32 + 1,
    )
}

/// The options for one compile, owned so that borrowing `Options` can be built
/// from them.
///
/// [`Options`](crate::Options) borrows its filesystem and logger, and a
/// wasm-bindgen type cannot carry a lifetime, so the owned values live here
/// and the borrowing `Options` is built inside the exported function.
struct Config {
    base: BaseOptions,
    fs: MemoryFs,
    load_paths: Vec<String>,
    syntax: Option<InputSyntax>,
    url: Option<String>,
    logger: Option<Function>,
}

/// The scalar options, separated so they can be applied after the borrows.
struct BaseOptions {
    style: OutputStyle,
    charset: bool,
    unicode_error_messages: bool,
    quiet: bool,
}

impl BaseOptions {
    /// Builds the borrowing [`Options`] for one compile.
    fn build<'a>(
        &self,
        fs: &'a dyn Fs,
        logger: &'a dyn Logger,
        load_paths: &[String],
        syntax: Option<InputSyntax>,
    ) -> Options<'a> {
        let mut options = Options::default()
            .fs(fs)
            .logger(logger)
            .style(self.style)
            .allows_charset(self.charset)
            .unicode_error_messages(self.unicode_error_messages)
            .quiet(self.quiet)
            .load_paths(load_paths);

        if let Some(syntax) = syntax {
            options = options.input_syntax(syntax);
        }

        options
    }
}

impl Config {
    /// Reads a JavaScript options object.
    ///
    /// Every field is optional, and `undefined` or `null` means "use the
    /// default" rather than being an error, so a caller can pass a partially
    /// filled object without checking each field first.
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` when a field is present but has the wrong type,
    /// or when `style` or `syntax` is not one of the values it allows.
    fn parse(options: Option<&CompileOptions>) -> Result<Self, JsValue> {
        let Some(options) = options.map(|value| -> &JsValue { value.as_ref() }) else {
            return Ok(Self::defaults());
        };

        if options.is_undefined() || options.is_null() {
            return Ok(Self::defaults());
        }

        if !options.is_object() {
            return Err(type_error("options must be an object"));
        }

        let mut config = Self::defaults();

        if let Some(value) = get(options, "style")? {
            config.base.style = match require_string(&value, "style")?.as_str() {
                "expanded" => OutputStyle::Expanded,
                "compressed" => OutputStyle::Compressed,
                other => {
                    return Err(type_error(&format!(
                        "style must be \"expanded\" or \"compressed\", got {other:?}"
                    )));
                }
            };
        }

        if let Some(value) = get(options, "syntax")? {
            config.syntax = Some(match require_string(&value, "syntax")?.as_str() {
                "scss" => InputSyntax::Scss,
                // dart-sass spells the indented syntax "indented"; "sass" is
                // accepted because that is what the file extension is called.
                "indented" | "sass" => InputSyntax::Sass,
                "css" => InputSyntax::Css,
                other => {
                    return Err(type_error(&format!(
                        "syntax must be \"scss\", \"indented\" or \"css\", got {other:?}"
                    )));
                }
            });
        }

        if let Some(value) = get(options, "loadPaths")? {
            let array: Array = value
                .dyn_into()
                .map_err(|_| type_error("loadPaths must be an array of strings"))?;

            for entry in array.iter() {
                config
                    .load_paths
                    .push(require_string(&entry, "loadPaths entry")?);
            }
        }

        if let Some(value) = get(options, "files")? {
            read_files(&value, &mut config.fs)?;
        }

        if let Some(value) = get(options, "url")? {
            config.url = Some(require_string(&value, "url")?);
        }

        if let Some(value) = get(options, "charset")? {
            config.base.charset = require_bool(&value, "charset")?;
        }

        if let Some(value) = get(options, "alertAscii")? {
            // `alertAscii` is the inverse of `unicode_error_messages`.
            config.base.unicode_error_messages = !require_bool(&value, "alertAscii")?;
        }

        if let Some(value) = get(options, "quiet")? {
            config.base.quiet = require_bool(&value, "quiet")?;
        }

        if let Some(value) = get(options, "logger")? {
            config.logger = Some(
                value
                    .dyn_into()
                    .map_err(|_| type_error("logger must be a function"))?,
            );
        }

        Ok(config)
    }

    /// The configuration used when no options are passed.
    fn defaults() -> Self {
        Self {
            base: BaseOptions {
                style: OutputStyle::Expanded,
                charset: true,
                unicode_error_messages: true,
                quiet: false,
            },
            fs: MemoryFs::new(),
            load_paths: Vec::new(),
            syntax: None,
            url: None,
            logger: None,
        }
    }
}

/// Reads a `files` value into the filesystem.
///
/// Accepts a `Map` or a plain object, because both are natural ways to hold a
/// path-to-source table in JavaScript and neither is obviously the one a
/// caller will reach for.
///
/// # Errors
///
/// Returns a `TypeError` when the value is neither, or when any key or value
/// is not a string.
fn read_files(value: &JsValue, fs: &mut MemoryFs) -> Result<(), JsValue> {
    if let Some(map) = value.dyn_ref::<Map>() {
        let entries = map.entries();
        // `Map::entries` yields [key, value] pairs through the iterator
        // protocol, so step it rather than indexing.
        let iterator = js_sys::try_iter(&entries)?
            .ok_or_else(|| type_error("files must be a Map or an object"))?;

        for entry in iterator {
            let pair: Array = entry?
                .dyn_into()
                .map_err(|_| type_error("files entries must be [path, source] pairs"))?;

            insert_file(fs, &pair.get(0), &pair.get(1))?;
        }

        return Ok(());
    }

    // `is_object` is true for arrays and functions as well as plain objects.
    // Letting an array through would store a file named `0` whose contents are
    // the first element, and the compile would then fail with `Can't find
    // stylesheet to import.` -- an error pointing anywhere but at the mistake.
    if Array::is_array(value) || value.is_function() {
        return Err(type_error("files must be a Map or an object"));
    }

    if value.is_object() {
        for entry in Object::entries(&Object::from(value.clone())).iter() {
            let pair: Array = entry
                .dyn_into()
                .map_err(|_| type_error("files entries must be [path, source] pairs"))?;

            insert_file(fs, &pair.get(0), &pair.get(1))?;
        }

        return Ok(());
    }

    Err(type_error("files must be a Map or an object"))
}

/// Inserts one `files` entry, checking that both halves are strings.
fn insert_file(fs: &mut MemoryFs, path: &JsValue, source: &JsValue) -> Result<(), JsValue> {
    let path = require_string(path, "files key")?;
    let source = require_string(source, "files value")?;

    fs.insert(path, source);

    Ok(())
}

/// Reads a property, treating `undefined` and `null` as absent.
fn get(object: &JsValue, key: &str) -> Result<Option<JsValue>, JsValue> {
    let value = Reflect::get(object, &JsValue::from_str(key))?;

    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }

    Ok(Some(value))
}

/// Requires a value to be a string.
///
/// # Errors
///
/// Returns a `TypeError` naming `field` when it is not.
fn require_string(value: &JsValue, field: &str) -> Result<String, JsValue> {
    value
        .dyn_ref::<JsString>()
        .map(String::from)
        .ok_or_else(|| type_error(&format!("{field} must be a string")))
}

/// Requires a value to be a boolean.
///
/// # Errors
///
/// Returns a `TypeError` naming `field` when it is not.
fn require_bool(value: &JsValue, field: &str) -> Result<bool, JsValue> {
    value
        .as_bool()
        .ok_or_else(|| type_error(&format!("{field} must be a boolean")))
}

/// Builds a `TypeError` to throw for a malformed option.
fn type_error(message: &str) -> JsValue {
    JsValue::from(js_sys::TypeError::new(message))
}

/// A [`Logger`] that forwards `@warn` and `@debug` to a JavaScript callback.
///
/// With no callback it discards everything, which is what a browser wants by
/// default: there is no standard error to write to, and USWDS alone emits
/// several kilobytes of `@warn` on every compile.
struct JsLogger {
    callback: Option<Function>,
}

impl JsLogger {
    const fn new(callback: Option<Function>) -> Self {
        Self { callback }
    }

    /// Calls the callback with one event object.
    ///
    /// An exception thrown by the callback is swallowed, because [`Logger`]
    /// cannot report one: its methods return nothing, and a warning must not
    /// be able to fail a compile that would otherwise have succeeded.
    fn emit(&self, kind: &str, location: &SpanLoc, message: &str) {
        let Some(callback) = &self.callback else {
            return;
        };

        let (file, line, column) = span_parts(location);

        let event = Object::new();
        let _ = Reflect::set(&event, &JsValue::from_str("type"), &JsValue::from_str(kind));
        let _ = Reflect::set(
            &event,
            &JsValue::from_str("message"),
            &JsValue::from_str(message),
        );
        let _ = Reflect::set(
            &event,
            &JsValue::from_str("file"),
            &JsValue::from_str(&file),
        );
        let _ = Reflect::set(&event, &JsValue::from_str("line"), &JsValue::from(line));
        let _ = Reflect::set(&event, &JsValue::from_str("column"), &JsValue::from(column));

        let _ = callback.call1(&JsValue::NULL, &event);
    }
}

impl Logger for JsLogger {
    fn debug(&self, location: SpanLoc, message: &str) {
        self.emit("debug", &location, message);
    }

    fn warn(&self, location: SpanLoc, message: &str) {
        self.emit("warn", &location, message);
    }
}

impl std::fmt::Debug for JsLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsLogger")
            .field("callback", &self.callback.is_some())
            .finish()
    }
}
