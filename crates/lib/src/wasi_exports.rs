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
//! # Panics
//!
//! Both release profiles set `panic = 'abort'`, so a panic in the compiler
//! reaches the host as a trap and leaves the instance unusable. A host that
//! compiles untrusted input should be ready to throw the instance away rather
//! than assume it can keep calling in.

use std::alloc::{Layout, alloc, dealloc};

use accent_sass_compiler::{Options, from_path, from_string};

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

/// Compile a stylesheet held in memory, with default options.
///
/// The input is not a file, so relative `@use` and `@import` resolve against
/// the working directory rather than against the stylesheet. Use
/// [`accent_sass_compile_path`] for a stylesheet that has neighbours.
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
    let input = unsafe { borrow(ptr, len) };

    match input {
        Ok(input) => match from_string(input.to_owned(), &Options::default()) {
            Ok(css) => result(0, css),
            Err(e) => result(1, e.to_string()),
        },
        Err(message) => result(2, message),
    }
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
    let path = unsafe { borrow(ptr, len) };

    match path {
        Ok(path) => match from_path(path, &Options::default()) {
            Ok(css) => result(0, css),
            Err(e) => result(1, e.to_string()),
        },
        Err(message) => result(2, message),
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
    fn zero_sized_allocations_are_null_and_freeing_null_is_a_no_op() {
        assert!(accent_sass_alloc(0).is_null());
        // SAFETY: both calls are documented as ignoring null.
        unsafe {
            accent_sass_dealloc(std::ptr::null_mut(), 0);
            accent_sass_result_free(std::ptr::null_mut());
        }
    }
}
