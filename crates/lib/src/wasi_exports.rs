//! A C ABI for embedding the compiler in a WebAssembly host.
//!
//! `wasm32-wasip1` has two shapes: a *command* module, which the host runs
//! like a process, and a *reactor* module, which the host instantiates once
//! and then calls into. The command shape is what `accent-sass.wasm` is; this
//! module is the reactor shape, for a plugin host that wants to compile many
//! stylesheets inside one instance without paying process startup for each.
//!
//! It is also what makes the artifact measurable. A `cdylib` that exports
//! nothing is dead code from the linker's point of view, so its size says
//! nothing about the compiler in it -- the browser package shipped a module
//! with no compiler in it for two releases for exactly that reason. These
//! exports pin the compiler into the module, so the number on the disk is the
//! number an embedder would carry.
//!
//! # The ABI
//!
//! Every string crosses the boundary as UTF-8 bytes in the module's linear
//! memory, because that is all a WebAssembly host can pass. A host:
//!
//! 1. calls [`accent_sass_alloc`] and writes the input into the returned
//!    offset,
//! 2. calls [`accent_sass_compile_string`] or [`accent_sass_compile_path`],
//!    which returns the offset of a [`CompileResult`],
//! 3. reads three 32-bit little-endian words at that offset -- status,
//!    pointer, length -- and decodes the bytes they point at,
//! 4. calls [`accent_sass_result_free`], and [`accent_sass_dealloc`] for its
//!    own input buffer.
//!
//! `status` is 0 when the bytes are CSS, 1 when they are a compile error
//! formatted the way the command-line binary formats it, and 2 when the input
//! was not UTF-8.
//!
//! [`accent_sass_compile_path`] resolves `@use` and `@import` against the
//! host's preopened directories through [`StdFs`](crate::StdFs), so a
//! stylesheet tree works with no importer callback. Paths outside every
//! preopen fail as a missing file, which is what a sandbox denial looks like
//! from inside the guest.
//!
//! # Options
//!
//! The two functions above compile with the compiler's defaults: expanded
//! output, the syntax inferred from the file extension, no load paths, and
//! warnings on. A host that wants anything else builds an options handle,
//! sets what it needs, and calls the `_with_options` twin:
//!
//! 1. [`accent_sass_options_new`] returns a handle,
//! 2. the setters fill it in, each returning a status word,
//! 3. [`accent_sass_compile_string_with_options`] or
//!    [`accent_sass_compile_path_with_options`] compiles with it,
//! 4. [`accent_sass_options_free`] releases it.
//!
//! A handle is reusable: set it up once and compile a whole theme through it.
//! It carries no borrowed state, so nothing about it expires between calls.
//!
//! The names follow dart-sass's JavaScript API wherever the two have the same
//! knob, so an embedder that knows one knows the other, and so this ABI and
//! the browser binding do not drift apart. The browser binding has the same
//! gap open; see `specs/docs/features/12-wasm-browser-package.md`.
//!
//! | this ABI | dart-sass | values |
//! |---|---|---|
//! | [`accent_sass_options_set_style`] | `style` | 0 `expanded`, 1 `compressed` |
//! | [`accent_sass_options_set_syntax`] | `syntax` | 0 `scss`, 1 `indented`, 2 `css` |
//! | [`accent_sass_options_add_load_path`] | `loadPaths` | one path per call |
//! | [`accent_sass_options_set_charset`] | `charset` | non-zero is true, default true |
//! | [`accent_sass_options_set_alert_ascii`] | `alertAscii` | non-zero is true, default false |
//! | [`accent_sass_options_set_quiet`] | -- | non-zero is true, default false |
//!
//! `quiet` has no name in dart-sass's JavaScript API, which silences warnings
//! by passing `Logger.silent`; dart-sass's command line calls it `--quiet`,
//! and so does this project's. It stops `@warn` and `@debug` as well as
//! deprecation warnings from reaching the logger.
//!
//! Load paths are *guest* paths. Under WASI a path is only readable if a
//! preopen covers it, so a host that wants `--load-path=/shared` must map a
//! directory onto that name when it instantiates the module. A load path no
//! preopen covers is not an error by itself; it simply never resolves an
//! import, and the compile fails with `Can't find stylesheet to import.`
//!
//! Setting the syntax overrides the extension for the entry point only.
//! Stylesheets pulled in by `@use`, `@import` and `@forward` always have
//! their syntax inferred from their own names, which is dart-sass's rule.
//!
//! # Panics
//!
//! Both release profiles set `panic = 'abort'`, so a panic in the compiler
//! reaches the host as a trap and leaves the instance unusable. A host that
//! compiles untrusted input should be ready to throw the instance away rather
//! than assume it can keep calling in.

use std::{
    alloc::{Layout, alloc, dealloc},
    path::PathBuf,
};

use accent_sass_compiler::{InputSyntax, Options, OutputStyle, from_path, from_string};

/// A call did what was asked.
pub const ACCENT_SASS_OK: u32 = 0;

/// A call was refused: a null handle, or a value this ABI does not define.
///
/// [`CompileResult::status`] uses the same word for a compile that produced
/// an error rather than CSS.
pub const ACCENT_SASS_REJECTED: u32 = 1;

/// The bytes handed across the boundary were not UTF-8.
pub const ACCENT_SASS_NOT_UTF8: u32 = 2;

/// [`accent_sass_options_set_style`]: each selector and declaration on its
/// own line. The default.
pub const ACCENT_SASS_STYLE_EXPANDED: u32 = 0;

/// [`accent_sass_options_set_style`]: the whole stylesheet on one line, with
/// every removable character removed.
pub const ACCENT_SASS_STYLE_COMPRESSED: u32 = 1;

/// [`accent_sass_options_set_syntax`]: the CSS-superset SCSS syntax.
pub const ACCENT_SASS_SYNTAX_SCSS: u32 = 0;

/// [`accent_sass_options_set_syntax`]: the whitespace-sensitive indented
/// syntax. dart-sass calls this value `indented`, not `sass`.
pub const ACCENT_SASS_SYNTAX_INDENTED: u32 = 1;

/// [`accent_sass_options_set_syntax`]: plain CSS, which rejects Sass
/// features.
pub const ACCENT_SASS_SYNTAX_CSS: u32 = 2;

/// What a compile produced: a status and a byte buffer in linear memory.
///
/// The layout is three 32-bit words on `wasm32`, in this order, and a host
/// reads it straight out of the module's memory. It is returned by pointer
/// because a WebAssembly function can only return one scalar.
#[repr(C)]
#[derive(Debug)]
pub struct CompileResult {
    /// 0: the buffer is CSS. 1: it is a formatted compile error. 2: the input
    /// was not valid UTF-8, and the buffer says so.
    pub status: u32,
    /// Offset of the result bytes in linear memory, or null when `len` is 0.
    pub ptr: *mut u8,
    /// Length of the result in bytes.
    pub len: usize,
}

/// Reserve `len` bytes in linear memory and return the offset.
///
/// Returns null for `len == 0`, which needs no allocation. The host must hand
/// the same `len` back to [`accent_sass_dealloc`]; the buffer is exactly the
/// size asked for, not rounded up.
#[unsafe(no_mangle)]
pub extern "C" fn accent_sass_alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }

    match Layout::from_size_align(len, 1) {
        // SAFETY: `len` is non-zero, so the layout has a non-zero size.
        Ok(layout) => unsafe { alloc(layout) },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Release a buffer obtained from [`accent_sass_alloc`].
///
/// # Safety
///
/// `ptr` must have come from [`accent_sass_alloc`] with the same `len`, and
/// must not have been freed already. A null `ptr` or a zero `len` is ignored.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accent_sass_dealloc(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }

    if let Ok(layout) = Layout::from_size_align(len, 1) {
        // SAFETY: the caller guarantees the pointer came from this allocator
        // with this layout.
        unsafe { dealloc(ptr, layout) }
    }
}

/// An owned compile configuration, held by the host across calls.
///
/// [`Options`] borrows its filesystem and logger, so it cannot outlive the
/// call that builds it and cannot be handed to a host as a pointer. This is
/// the owned half: plain values the setters write, which
/// [`CompileOptions::as_options`] turns into an [`Options`] for the duration
/// of one compile.
///
/// The layout is **not** part of the ABI. A host holds the pointer
/// [`accent_sass_options_new`] returns and never reads through it.
#[derive(Debug)]
pub struct CompileOptions {
    style: OutputStyle,
    syntax: Option<InputSyntax>,
    load_paths: Vec<PathBuf>,
    charset: bool,
    alert_ascii: bool,
    quiet: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        // These mirror `Options::default()`, and `alert_ascii` is the inverse
        // of its `unicode_error_messages`. dart-sass defaults `alertAscii` to
        // false as well, so the two agree.
        Self {
            style: OutputStyle::Expanded,
            syntax: None,
            load_paths: Vec::new(),
            charset: true,
            alert_ascii: false,
            quiet: false,
        }
    }
}

impl CompileOptions {
    /// Build the borrowing [`Options`] the compiler takes.
    fn as_options(&self) -> Options<'_> {
        let options = Options::default()
            .style(self.style)
            .quiet(self.quiet)
            .allows_charset(self.charset)
            .unicode_error_messages(!self.alert_ascii)
            .load_paths(&self.load_paths);

        match self.syntax {
            Some(syntax) => options.input_syntax(syntax),
            None => options,
        }
    }
}

/// Create an options handle carrying the compiler's defaults.
///
/// The handle is owned by the host until it calls
/// [`accent_sass_options_free`]. It never returns null: allocation failure
/// aborts the instance rather than reporting.
#[unsafe(no_mangle)]
pub extern "C" fn accent_sass_options_new() -> *mut CompileOptions {
    Box::into_raw(Box::new(CompileOptions::default()))
}

/// Release an options handle.
///
/// # Safety
///
/// `options` must have come from [`accent_sass_options_new`] and must not
/// have been freed already. Null is ignored.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accent_sass_options_free(options: *mut CompileOptions) {
    if options.is_null() {
        return;
    }

    // SAFETY: the caller guarantees the pointer came from
    // `accent_sass_options_new`, which built it with `Box::into_raw`.
    drop(unsafe { Box::from_raw(options) });
}

/// Choose the output style: [`ACCENT_SASS_STYLE_EXPANDED`] or
/// [`ACCENT_SASS_STYLE_COMPRESSED`].
///
/// Returns [`ACCENT_SASS_OK`], or [`ACCENT_SASS_REJECTED`] for a null handle
/// or a value this ABI does not define. A rejected call changes nothing, so a
/// host that ignores the status keeps the previous style rather than a
/// silently invented one.
///
/// # Safety
///
/// `options` must be a live handle from [`accent_sass_options_new`], or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accent_sass_options_set_style(
    options: *mut CompileOptions,
    style: u32,
) -> u32 {
    let style = match style {
        ACCENT_SASS_STYLE_EXPANDED => OutputStyle::Expanded,
        ACCENT_SASS_STYLE_COMPRESSED => OutputStyle::Compressed,
        _ => return ACCENT_SASS_REJECTED,
    };

    // SAFETY: the caller guarantees the handle is live; null is checked.
    unsafe { with(options, |options| options.style = style) }
}

/// Force the entry point's syntax: [`ACCENT_SASS_SYNTAX_SCSS`],
/// [`ACCENT_SASS_SYNTAX_INDENTED`] or [`ACCENT_SASS_SYNTAX_CSS`].
///
/// Without this the syntax comes from the file extension, and from SCSS for a
/// stylesheet compiled out of memory. Imported stylesheets always infer their
/// own, so this reaches the entry point only.
///
/// Returns [`ACCENT_SASS_OK`], or [`ACCENT_SASS_REJECTED`] for a null handle
/// or an undefined value.
///
/// # Safety
///
/// `options` must be a live handle from [`accent_sass_options_new`], or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accent_sass_options_set_syntax(
    options: *mut CompileOptions,
    syntax: u32,
) -> u32 {
    let syntax = match syntax {
        ACCENT_SASS_SYNTAX_SCSS => InputSyntax::Scss,
        ACCENT_SASS_SYNTAX_INDENTED => InputSyntax::Sass,
        ACCENT_SASS_SYNTAX_CSS => InputSyntax::Css,
        _ => return ACCENT_SASS_REJECTED,
    };

    // SAFETY: the caller guarantees the handle is live; null is checked.
    unsafe { with(options, |options| options.syntax = Some(syntax)) }
}

/// Append one load path, as UTF-8 bytes in linear memory.
///
/// Load paths are searched in the order they are added, and only after a
/// relative resolution fails, which is dart-sass's rule. The path is a guest
/// path: under WASI it resolves only if a preopen covers it.
///
/// Returns [`ACCENT_SASS_OK`], [`ACCENT_SASS_REJECTED`] for a null handle, or
/// [`ACCENT_SASS_NOT_UTF8`] for bytes that are not UTF-8. An empty path is
/// accepted, because the compiler treats it as the working directory.
///
/// # Safety
///
/// `options` must be a live handle from [`accent_sass_options_new`], or null.
/// `ptr` must point at `len` readable bytes, or be null with `len` 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accent_sass_options_add_load_path(
    options: *mut CompileOptions,
    ptr: *const u8,
    len: usize,
) -> u32 {
    // SAFETY: the caller guarantees the range is readable for `len` bytes.
    let path = match unsafe { borrow(ptr, len) } {
        Ok(path) => PathBuf::from(path),
        Err(_) => return ACCENT_SASS_NOT_UTF8,
    };

    // SAFETY: the caller guarantees the handle is live; null is checked.
    unsafe { with(options, |options| options.load_paths.push(path)) }
}

/// Whether the compiler may emit `@charset` or a byte-order mark for
/// non-ASCII output. Non-zero is true, and true is the default.
///
/// Returns [`ACCENT_SASS_OK`], or [`ACCENT_SASS_REJECTED`] for a null handle.
///
/// # Safety
///
/// `options` must be a live handle from [`accent_sass_options_new`], or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accent_sass_options_set_charset(
    options: *mut CompileOptions,
    charset: u32,
) -> u32 {
    // SAFETY: the caller guarantees the handle is live; null is checked.
    unsafe { with(options, |options| options.charset = charset != 0) }
}

/// Whether messages stay inside ASCII. Non-zero is true; false, the default,
/// lets the compiler draw spans with non-ASCII characters.
///
/// This is dart-sass's `alertAscii`, and it is the inverse of this crate's
/// [`Options::unicode_error_messages`]. It does not affect the CSS.
///
/// Returns [`ACCENT_SASS_OK`], or [`ACCENT_SASS_REJECTED`] for a null handle.
///
/// # Safety
///
/// `options` must be a live handle from [`accent_sass_options_new`], or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accent_sass_options_set_alert_ascii(
    options: *mut CompileOptions,
    alert_ascii: u32,
) -> u32 {
    // SAFETY: the caller guarantees the handle is live; null is checked.
    unsafe { with(options, |options| options.alert_ascii = alert_ascii != 0) }
}

/// Silence the logger: `@warn`, `@debug` and deprecation warnings. Non-zero
/// is true; false is the default.
///
/// It matters more here than natively. The logger writes to standard output,
/// which in a reactor module is whatever file descriptor the host wired up at
/// instantiation -- often nothing, sometimes the host's own stream. A host
/// that does not want a guest writing there turns this on.
///
/// Returns [`ACCENT_SASS_OK`], or [`ACCENT_SASS_REJECTED`] for a null handle.
///
/// # Safety
///
/// `options` must be a live handle from [`accent_sass_options_new`], or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accent_sass_options_set_quiet(
    options: *mut CompileOptions,
    quiet: u32,
) -> u32 {
    // SAFETY: the caller guarantees the handle is live; null is checked.
    unsafe { with(options, |options| options.quiet = quiet != 0) }
}

/// Run `f` against a handle, rejecting null.
///
/// # Safety
///
/// `options` must be a live handle from [`accent_sass_options_new`], or null.
unsafe fn with<F: FnOnce(&mut CompileOptions)>(options: *mut CompileOptions, f: F) -> u32 {
    if options.is_null() {
        return ACCENT_SASS_REJECTED;
    }

    // SAFETY: the caller guarantees the pointer is a live handle, and the ABI
    // is single-threaded, so no other reference to it exists.
    f(unsafe { &mut *options });

    ACCENT_SASS_OK
}

/// Compile a stylesheet held in memory, with default options.
///
/// The input is not a file, so relative `@use` and `@import` resolve against
/// the working directory rather than against the stylesheet. Use
/// [`accent_sass_compile_path`] for a stylesheet that has neighbours, or
/// [`accent_sass_compile_string_with_options`] to add a load path.
///
/// # Safety
///
/// `ptr` must point at `len` readable bytes, or be null with `len` 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accent_sass_compile_string(
    ptr: *const u8,
    len: usize,
) -> *mut CompileResult {
    // SAFETY: the caller guarantees the range is readable for `len` bytes.
    unsafe { compile_string(ptr, len, &CompileOptions::default()) }
}

/// Compile a stylesheet held in memory with a host-supplied configuration.
///
/// `options` is a handle from [`accent_sass_options_new`]; it is read, not
/// consumed, so one handle drives as many compiles as the host likes. A null
/// handle gives status [`ACCENT_SASS_REJECTED`] and a message, rather than
/// silently compiling with defaults -- a host that lost its handle wanted the
/// options it set.
///
/// # Safety
///
/// `ptr` must point at `len` readable bytes, or be null with `len` 0.
/// `options` must be a live handle from [`accent_sass_options_new`], or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accent_sass_compile_string_with_options(
    ptr: *const u8,
    len: usize,
    options: *const CompileOptions,
) -> *mut CompileResult {
    if options.is_null() {
        return result(ACCENT_SASS_REJECTED, NULL_OPTIONS.to_owned());
    }

    // SAFETY: the caller guarantees both the range and the handle.
    unsafe { compile_string(ptr, len, &*options) }
}

/// Compile a stylesheet from a path, reading it through the host's preopened
/// directories.
///
/// The path is interpreted by the guest's filesystem, so it must be reachable
/// from a preopen. One that is not fails the way a missing file fails, with
/// status 1 and the compiler's own message.
///
/// # Safety
///
/// `ptr` must point at `len` readable bytes, or be null with `len` 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accent_sass_compile_path(
    ptr: *const u8,
    len: usize,
) -> *mut CompileResult {
    // SAFETY: the caller guarantees the range is readable for `len` bytes.
    unsafe { compile_path(ptr, len, &CompileOptions::default()) }
}

/// Compile a stylesheet from a path with a host-supplied configuration.
///
/// Load paths set on the handle are guest paths, so each one needs a preopen
/// covering it; see the module documentation.
///
/// # Safety
///
/// `ptr` must point at `len` readable bytes, or be null with `len` 0.
/// `options` must be a live handle from [`accent_sass_options_new`], or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accent_sass_compile_path_with_options(
    ptr: *const u8,
    len: usize,
    options: *const CompileOptions,
) -> *mut CompileResult {
    if options.is_null() {
        return result(ACCENT_SASS_REJECTED, NULL_OPTIONS.to_owned());
    }

    // SAFETY: the caller guarantees both the range and the handle.
    unsafe { compile_path(ptr, len, &*options) }
}

/// What a compile reports when the host passes a null options handle.
///
/// It is prefixed the way the compiler prefixes its own errors so a host that
/// only prints the body reads the same shape either way.
const NULL_OPTIONS: &str = "Error: null options handle.";

/// Compile bytes as a stylesheet. Shared by the two string entry points.
///
/// # Safety
///
/// `ptr` must point at `len` readable bytes, or be null with `len` 0.
unsafe fn compile_string(
    ptr: *const u8,
    len: usize,
    options: &CompileOptions,
) -> *mut CompileResult {
    // SAFETY: the caller guarantees the range is readable for `len` bytes.
    match unsafe { borrow(ptr, len) } {
        Ok(input) => match from_string(input.to_owned(), &options.as_options()) {
            Ok(css) => result(ACCENT_SASS_OK, css),
            Err(e) => result(ACCENT_SASS_REJECTED, e.to_string()),
        },
        Err(message) => result(ACCENT_SASS_NOT_UTF8, message),
    }
}

/// Compile bytes as a path. Shared by the two path entry points.
///
/// # Safety
///
/// `ptr` must point at `len` readable bytes, or be null with `len` 0.
unsafe fn compile_path(ptr: *const u8, len: usize, options: &CompileOptions) -> *mut CompileResult {
    // SAFETY: the caller guarantees the range is readable for `len` bytes.
    match unsafe { borrow(ptr, len) } {
        Ok(path) => match from_path(path, &options.as_options()) {
            Ok(css) => result(ACCENT_SASS_OK, css),
            Err(e) => result(ACCENT_SASS_REJECTED, e.to_string()),
        },
        Err(message) => result(ACCENT_SASS_NOT_UTF8, message),
    }
}

/// Release a [`CompileResult`] and the bytes it points at.
///
/// # Safety
///
/// `res` must be a pointer returned by one of the compile functions and not
/// yet freed. Null is ignored.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accent_sass_result_free(res: *mut CompileResult) {
    if res.is_null() {
        return;
    }

    // SAFETY: the caller guarantees the pointer came from `result`, which
    // built it with `Box::into_raw`.
    let res = unsafe { Box::from_raw(res) };

    if !res.ptr.is_null() && res.len != 0 {
        // SAFETY: `result` leaks a boxed slice of exactly `len` bytes.
        drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(res.ptr, res.len)) });
    }
}

/// Read a host-supplied byte range as a string.
///
/// # Safety
///
/// `ptr` must point at `len` readable bytes, or be null with `len` 0.
unsafe fn borrow<'a>(ptr: *const u8, len: usize) -> Result<&'a str, String> {
    if ptr.is_null() || len == 0 {
        return Ok("");
    }

    // SAFETY: the caller guarantees the range is readable and stays alive for
    // the duration of the call.
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };

    std::str::from_utf8(bytes).map_err(|e| e.to_string())
}

/// Leak `body` into linear memory and describe it with a leaked
/// [`CompileResult`], for the host to read and then free.
fn result(status: u32, body: String) -> *mut CompileResult {
    let bytes = body.into_bytes().into_boxed_slice();
    let len = bytes.len();
    let ptr = if len == 0 {
        drop(bytes);
        std::ptr::null_mut()
    } else {
        Box::into_raw(bytes).cast::<u8>()
    };

    Box::into_raw(Box::new(CompileResult { status, ptr, len }))
}

#[cfg(test)]
mod tests {
    //! The ABI is exercised natively here because the memory handling is the
    //! part that can go wrong quietly: a leaked result or a mismatched free
    //! shows up as a growing instance, not as a failed compile. The
    //! WebAssembly side of it is covered by
    //! `.github/scripts/wasi-lib-smoke.mjs`, which runs the real module.

    use super::*;

    /// Copy `input` in the way a host would, call `f`, and take the result
    /// apart, freeing everything on the way out.
    fn round_trip(
        f: unsafe extern "C" fn(*const u8, usize) -> *mut CompileResult,
        input: &str,
    ) -> (u32, String) {
        let len = input.len();
        let ptr = accent_sass_alloc(len);
        // SAFETY: `ptr` is `len` writable bytes from the allocator above.
        unsafe { std::ptr::copy_nonoverlapping(input.as_ptr(), ptr, len) };

        // SAFETY: the range was just written and stays alive for the call.
        let res = unsafe { f(ptr, len) };
        // SAFETY: `res` is the pointer the compile function just returned.
        let (status, body) = unsafe {
            let body = if (*res).len == 0 {
                String::new()
            } else {
                String::from_utf8(std::slice::from_raw_parts((*res).ptr, (*res).len).to_vec())
                    .unwrap()
            };
            ((*res).status, body)
        };

        // SAFETY: both pointers came from this module and are freed once.
        unsafe {
            accent_sass_result_free(res);
            accent_sass_dealloc(ptr, len);
        }

        (status, body)
    }

    /// The same round trip, driven through a host-supplied options handle.
    fn round_trip_with(
        f: unsafe extern "C" fn(*const u8, usize, *const CompileOptions) -> *mut CompileResult,
        input: &str,
        options: *const CompileOptions,
    ) -> (u32, String) {
        let len = input.len();
        let ptr = accent_sass_alloc(len);
        // SAFETY: `ptr` is `len` writable bytes from the allocator above.
        unsafe { std::ptr::copy_nonoverlapping(input.as_ptr(), ptr, len) };

        // SAFETY: the range was just written and stays alive for the call.
        let res = unsafe { f(ptr, len, options) };
        // SAFETY: `res` is the pointer the compile function just returned.
        let (status, body) = unsafe {
            let body = if (*res).len == 0 {
                String::new()
            } else {
                String::from_utf8(std::slice::from_raw_parts((*res).ptr, (*res).len).to_vec())
                    .unwrap()
            };
            ((*res).status, body)
        };

        // SAFETY: both pointers came from this module and are freed once.
        unsafe {
            accent_sass_result_free(res);
            accent_sass_dealloc(ptr, len);
        }

        (status, body)
    }

    /// Hand a path to a setter the way a host does, as bytes in memory.
    fn add_load_path(options: *mut CompileOptions, path: &str) -> u32 {
        let len = path.len();
        let ptr = accent_sass_alloc(len);
        // SAFETY: `ptr` is `len` writable bytes from the allocator above.
        unsafe { std::ptr::copy_nonoverlapping(path.as_ptr(), ptr, len) };

        // SAFETY: the range was just written, and `options` is a live handle.
        let status = unsafe { accent_sass_options_add_load_path(options, ptr, len) };
        // SAFETY: the pointer came from this module and is freed once.
        unsafe { accent_sass_dealloc(ptr, len) };

        status
    }

    #[test]
    fn compiles_a_string() {
        let (status, body) = round_trip(accent_sass_compile_string, "a {\n  b: 1px + 2px;\n}\n");
        assert_eq!(status, 0);
        assert_eq!(body, "a {\n  b: 3px;\n}\n");
    }

    #[test]
    fn reports_a_compile_error() {
        let (status, body) = round_trip(accent_sass_compile_string, "a { b: 1px + ; }");
        assert_eq!(status, 1);
        assert!(body.starts_with("Error: "), "{body}");
    }

    #[test]
    fn reports_a_missing_file() {
        let (status, body) =
            round_trip(accent_sass_compile_path, "there-is-no-such-stylesheet.scss");
        assert_eq!(status, 1);
        assert!(body.starts_with("Error: "), "{body}");
    }

    #[test]
    fn rejects_invalid_utf8() {
        let input = [0x61_u8, 0xff, 0x7b, 0x7d];
        let ptr = accent_sass_alloc(input.len());
        // SAFETY: `ptr` is `input.len()` writable bytes.
        unsafe { std::ptr::copy_nonoverlapping(input.as_ptr(), ptr, input.len()) };
        // SAFETY: the range was just written.
        let res = unsafe { accent_sass_compile_string(ptr, input.len()) };
        // SAFETY: `res` came from the call above.
        unsafe {
            assert_eq!((*res).status, 2);
            accent_sass_result_free(res);
            accent_sass_dealloc(ptr, input.len());
        }
    }

    #[test]
    fn empty_input_compiles_to_nothing() {
        // A null pointer with a zero length is the shape a host reaches when
        // it allocates for an empty string, so it must not be a trap.
        // SAFETY: null with length 0 is explicitly allowed.
        let res = unsafe { accent_sass_compile_string(std::ptr::null(), 0) };
        // SAFETY: `res` came from the call above.
        unsafe {
            assert_eq!((*res).status, 0);
            assert_eq!((*res).len, 0);
            accent_sass_result_free(res);
        }
    }

    #[test]
    fn compresses_when_asked() {
        let options = accent_sass_options_new();
        // SAFETY: `options` is the handle just created.
        unsafe {
            assert_eq!(
                accent_sass_options_set_style(options, ACCENT_SASS_STYLE_COMPRESSED),
                ACCENT_SASS_OK
            );
        }

        let (status, body) = round_trip_with(
            accent_sass_compile_string_with_options,
            "a {\n  b: 1px + 2px;\n}\n",
            options,
        );
        assert_eq!(status, ACCENT_SASS_OK);
        assert_eq!(body, "a{b:3px}");

        // SAFETY: the handle is live and freed once.
        unsafe { accent_sass_options_free(options) };
    }

    #[test]
    fn parses_the_indented_syntax_when_asked() {
        // Indented input is not valid SCSS, so this fails without the option
        // and the test would pass for the wrong reason if the option were
        // ignored.
        let input = "a\n  b: 1px + 2px\n";

        let options = accent_sass_options_new();
        let (status, _) = round_trip_with(accent_sass_compile_string_with_options, input, options);
        assert_eq!(
            status, ACCENT_SASS_REJECTED,
            "indented input parsed as SCSS"
        );

        // SAFETY: `options` is the handle created above.
        unsafe {
            assert_eq!(
                accent_sass_options_set_syntax(options, ACCENT_SASS_SYNTAX_INDENTED),
                ACCENT_SASS_OK
            );
        }

        let (status, body) =
            round_trip_with(accent_sass_compile_string_with_options, input, options);
        assert_eq!(status, ACCENT_SASS_OK);
        assert_eq!(body, "a {\n  b: 3px;\n}\n");

        // SAFETY: the handle is live and freed once.
        unsafe { accent_sass_options_free(options) };
    }

    #[test]
    fn rejects_plain_css_input_when_the_syntax_says_css() {
        let options = accent_sass_options_new();
        // SAFETY: `options` is the handle just created.
        unsafe {
            assert_eq!(
                accent_sass_options_set_syntax(options, ACCENT_SASS_SYNTAX_CSS),
                ACCENT_SASS_OK
            );
        }

        let (status, body) = round_trip_with(
            accent_sass_compile_string_with_options,
            "$a: 1px;\nb {\n  c: $a;\n}\n",
            options,
        );
        assert_eq!(status, ACCENT_SASS_REJECTED);
        assert!(body.starts_with("Error: "), "{body}");

        // SAFETY: the handle is live and freed once.
        unsafe { accent_sass_options_free(options) };
    }

    #[test]
    fn resolves_an_import_through_a_load_path() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("_shared.scss"), "$v: 2px;\n").unwrap();

        let input = "@use \"shared\";\na {\n  b: shared.$v + 1px;\n}\n";

        // Without the load path the import cannot resolve, so the compile has
        // to fail before it can pass for the right reason.
        let options = accent_sass_options_new();
        let (status, _) = round_trip_with(accent_sass_compile_string_with_options, input, options);
        assert_eq!(status, ACCENT_SASS_REJECTED);

        assert_eq!(
            add_load_path(options, dir.path().to_str().unwrap()),
            ACCENT_SASS_OK
        );

        let (status, body) =
            round_trip_with(accent_sass_compile_string_with_options, input, options);
        assert_eq!(status, ACCENT_SASS_OK, "{body}");
        assert_eq!(body, "a {\n  b: 3px;\n}\n");

        // SAFETY: the handle is live and freed once.
        unsafe { accent_sass_options_free(options) };
    }

    #[test]
    fn load_paths_apply_to_a_path_compile_too() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("_shared.scss"), "$v: 2px;\n").unwrap();
        let entry = dir.path().join("entry.scss");
        std::fs::write(&entry, "@use \"shared\";\na {\n  b: shared.$v + 1px;\n}\n").unwrap();

        let options = accent_sass_options_new();
        assert_eq!(
            add_load_path(options, dir.path().to_str().unwrap()),
            ACCENT_SASS_OK
        );

        let (status, body) = round_trip_with(
            accent_sass_compile_path_with_options,
            entry.to_str().unwrap(),
            options,
        );
        assert_eq!(status, ACCENT_SASS_OK, "{body}");
        assert_eq!(body, "a {\n  b: 3px;\n}\n");

        // SAFETY: the handle is live and freed once.
        unsafe { accent_sass_options_free(options) };
    }

    #[test]
    fn charset_can_be_turned_off() {
        let input = "a {\n  b: \"\u{e9}\";\n}\n";

        let options = accent_sass_options_new();
        let (_, with_charset) =
            round_trip_with(accent_sass_compile_string_with_options, input, options);
        assert!(with_charset.starts_with("@charset"), "{with_charset}");

        // SAFETY: `options` is the handle created above.
        unsafe {
            assert_eq!(accent_sass_options_set_charset(options, 0), ACCENT_SASS_OK);
        }

        let (_, without) = round_trip_with(accent_sass_compile_string_with_options, input, options);
        assert!(!without.starts_with("@charset"), "{without}");

        // SAFETY: the handle is live and freed once.
        unsafe { accent_sass_options_free(options) };
    }

    #[test]
    fn alert_ascii_keeps_error_messages_inside_ascii() {
        let input = "a { b: 1px + ; }";

        let options = accent_sass_options_new();
        let (_, unicode) = round_trip_with(accent_sass_compile_string_with_options, input, options);
        assert!(!unicode.is_ascii(), "{unicode}");

        // SAFETY: `options` is the handle created above.
        unsafe {
            assert_eq!(
                accent_sass_options_set_alert_ascii(options, 1),
                ACCENT_SASS_OK
            );
        }

        let (_, ascii) = round_trip_with(accent_sass_compile_string_with_options, input, options);
        assert!(ascii.is_ascii(), "{ascii}");

        // SAFETY: the handle is live and freed once.
        unsafe { accent_sass_options_free(options) };
    }

    #[test]
    fn an_undefined_value_is_refused_and_changes_nothing() {
        let options = accent_sass_options_new();
        // SAFETY: `options` is the handle just created.
        unsafe {
            assert_eq!(
                accent_sass_options_set_style(options, ACCENT_SASS_STYLE_COMPRESSED),
                ACCENT_SASS_OK
            );
            assert_eq!(
                accent_sass_options_set_style(options, 7),
                ACCENT_SASS_REJECTED
            );
            assert_eq!(
                accent_sass_options_set_syntax(options, 7),
                ACCENT_SASS_REJECTED
            );
        }

        // The refused call must not have reset the style to a default.
        let (status, body) = round_trip_with(
            accent_sass_compile_string_with_options,
            "a {\n  b: 1px + 2px;\n}\n",
            options,
        );
        assert_eq!(status, ACCENT_SASS_OK);
        assert_eq!(body, "a{b:3px}");

        // SAFETY: the handle is live and freed once.
        unsafe { accent_sass_options_free(options) };
    }

    #[test]
    fn a_null_handle_is_refused_rather_than_ignored() {
        // SAFETY: null is explicitly allowed by every one of these.
        unsafe {
            assert_eq!(
                accent_sass_options_set_style(std::ptr::null_mut(), ACCENT_SASS_STYLE_EXPANDED),
                ACCENT_SASS_REJECTED
            );
            assert_eq!(
                accent_sass_options_set_syntax(std::ptr::null_mut(), ACCENT_SASS_SYNTAX_SCSS),
                ACCENT_SASS_REJECTED
            );
            assert_eq!(
                accent_sass_options_set_charset(std::ptr::null_mut(), 1),
                ACCENT_SASS_REJECTED
            );
            assert_eq!(
                accent_sass_options_set_alert_ascii(std::ptr::null_mut(), 1),
                ACCENT_SASS_REJECTED
            );
            assert_eq!(
                accent_sass_options_set_quiet(std::ptr::null_mut(), 1),
                ACCENT_SASS_REJECTED
            );
            assert_eq!(
                add_load_path(std::ptr::null_mut(), "/tmp"),
                ACCENT_SASS_REJECTED
            );
            accent_sass_options_free(std::ptr::null_mut());
        }

        // A compile with no handle says so rather than quietly using defaults.
        let (status, body) = round_trip_with(
            accent_sass_compile_string_with_options,
            "a { b: 1px; }",
            std::ptr::null(),
        );
        assert_eq!(status, ACCENT_SASS_REJECTED);
        assert_eq!(body, NULL_OPTIONS);
    }

    #[test]
    fn a_load_path_that_is_not_utf8_is_refused() {
        let options = accent_sass_options_new();
        let input = [0x2f_u8, 0xff];
        let ptr = accent_sass_alloc(input.len());
        // SAFETY: `ptr` is `input.len()` writable bytes.
        unsafe { std::ptr::copy_nonoverlapping(input.as_ptr(), ptr, input.len()) };
        // SAFETY: the range was just written, and the handle is live.
        unsafe {
            assert_eq!(
                accent_sass_options_add_load_path(options, ptr, input.len()),
                ACCENT_SASS_NOT_UTF8
            );
            accent_sass_dealloc(ptr, input.len());
            accent_sass_options_free(options);
        }
    }

    #[test]
    fn one_handle_drives_many_compiles() {
        // The reactor shape exists to compile more than once per instance, so
        // a handle that only survives one call would defeat the point.
        let options = accent_sass_options_new();
        // SAFETY: `options` is the handle just created.
        unsafe {
            accent_sass_options_set_style(options, ACCENT_SASS_STYLE_COMPRESSED);
            accent_sass_options_set_quiet(options, 1);
        }

        for _ in 0..3 {
            let (status, body) = round_trip_with(
                accent_sass_compile_string_with_options,
                "@warn \"noisy\";\na {\n  b: 1px + 2px;\n}\n",
                options,
            );
            assert_eq!(status, ACCENT_SASS_OK);
            assert_eq!(body, "a{b:3px}");
        }

        // SAFETY: the handle is live and freed once.
        unsafe { accent_sass_options_free(options) };
    }

    #[test]
    fn zero_sized_allocations_are_null_and_freeing_null_is_a_no_op() {
        assert!(accent_sass_alloc(0).is_null());
        // SAFETY: both calls are documented as ignoring null.
        unsafe {
            accent_sass_dealloc(std::ptr::null_mut(), 0);
            accent_sass_result_free(std::ptr::null_mut());
        }
    }
}
