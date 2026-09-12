//! An in-memory [`Fs`] for compiling a stylesheet tree that is not on disk.
//!
//! The compiler resolves `@use`, `@forward` and `@import` through the [`Fs`]
//! trait, and the default [`StdFs`](crate::StdFs) answers those lookups from
//! `std::fs`. That is the wrong seam in two places that matter:
//!
//! - **A browser.** On `wasm32-unknown-unknown` the `std::fs` calls compile
//!   and then fail at runtime, so a browser build can only ever compile a
//!   single file. Every import fails with `Can't find stylesheet to import.`
//! - **A host that already holds the sources.** A CMS that keeps theme
//!   stylesheets in a database would otherwise have to write them to a
//!   temporary directory just so the compiler can read them back.
//!
//! `MemoryFs` closes both: you insert the sources you have, then compile
//! against them.
//!
//! # Reads are synchronous, so load everything first
//!
//! [`Fs::read`] returns the bytes directly, with no way to await. A caller
//! therefore cannot fetch a dependency when the compiler asks for it: every
//! file a compile might touch must be inserted *before* the compile starts.
//! This is a property of the trait, not of this type, and it is the reason a
//! browser embedder loads a whole framework into memory up front rather than
//! resolving imports lazily over the network.
//!
//! # Paths are normalized, and there is no working directory
//!
//! Every path is normalized lexically before it is stored or looked up: `.`
//! segments are dropped and `..` pops the preceding segment. Nothing touches
//! a real filesystem, so a relative path is not resolved against a process
//! working directory -- `a/b.scss` and `./a/b.scss` name the same file, and
//! `/a/b.scss` is a different one.
//!
//! Normalization matters beyond tidiness: the compiler canonicalizes a
//! resolved import before using it as the key of its module cache. Were two
//! spellings of one path to survive, the same stylesheet would be parsed and
//! executed twice, and a module that is supposed to be instantiated once
//! would not be.
//!
//! # Directories are implied by the files in them
//!
//! There is no way to create an empty directory, because the compiler never
//! needs one. Inserting `a/b/c.scss` makes `a` and `a/b` report `true` from
//! [`Fs::is_dir`], which is what the resolver needs in order to look for
//! `a/b/index.scss` and `a/b/_index.scss`.
//!
//! # Example
//!
//! ```
//! # use accent_sass_compiler as accent_sass;
//! use accent_sass::{MemoryFs, Options, from_path};
//!
//! let mut fs = MemoryFs::new();
//! fs.insert("theme/_colors.scss", "$brand: #bada55;");
//! fs.insert("theme/index.scss", "@use \"colors\";\na { color: colors.$brand; }");
//!
//! let css = from_path("theme/index.scss", &Options::default().fs(&fs))?;
//! assert_eq!(css, "a {\n  color: #bada55;\n}\n");
//! # Ok::<(), Box<accent_sass::Error>>(())
//! ```

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt,
    io::{self, Error, ErrorKind},
    path::{Component, Path, PathBuf},
};

use crate::Fs;

/// A [`Fs`] backed by a map of path to file contents.
///
/// See the [module documentation](self) for the constraints that come with
/// it: reads are synchronous, so every file must be inserted before the
/// compile begins.
///
/// Cloning copies every stored file, so prefer to build one and pass it by
/// reference.
#[derive(Clone, Default)]
pub struct MemoryFs {
    /// The stored files, keyed by normalized path.
    files: HashMap<PathBuf, Vec<u8>>,
    /// Every ancestor directory of every stored file, so that
    /// [`Fs::is_dir`] can answer without walking the map.
    dirs: HashSet<PathBuf>,
    /// The paths the compiler has actually read, in the order it read them.
    ///
    /// This is behind a [`RefCell`] because [`Fs::read`] takes `&self`. It is
    /// a record of what a compile touched, which an embedder uses to report
    /// the loaded files and to decide what to cache.
    loaded: RefCell<Vec<PathBuf>>,
}

impl MemoryFs {
    /// Creates a filesystem with no files in it.
    ///
    /// Compiling against it behaves like [`NullFs`](crate::NullFs): every
    /// import fails to resolve.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a file, replacing whatever was at that path.
    ///
    /// The path is normalized, so `./a.scss` and `a.scss` are the same entry.
    /// Every ancestor directory of the path starts reporting `true` from
    /// [`Fs::is_dir`].
    ///
    /// The contents may be any bytes, but the compiler rejects a file that is
    /// not valid UTF-8 when it reads it, not when you insert it.
    #[inline]
    pub fn insert<P: AsRef<Path>, C: Into<Vec<u8>>>(&mut self, path: P, contents: C) {
        let path = normalize(path.as_ref());

        let mut parent = path.parent();
        while let Some(dir) = parent {
            // `Path::parent` of a bare file name is the empty path, which is
            // not a directory anyone can name. Stop rather than store it.
            if dir.as_os_str().is_empty() {
                break;
            }
            // An ancestor already recorded means every ancestor above it is
            // recorded too, so there is nothing left to walk.
            if !self.dirs.insert(dir.to_path_buf()) {
                break;
            }
            parent = dir.parent();
        }

        self.files.insert(path, contents.into());
    }

    /// Returns the number of files stored.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns `true` when no files are stored.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Returns the paths that have been read, in the order they were read.
    ///
    /// A path appears once per read, so a stylesheet the compiler consults
    /// twice appears twice. Use it to report which files a compile depended
    /// on -- to invalidate a cache, or to show a user what a framework
    /// actually pulled in.
    #[must_use]
    #[inline]
    pub fn loaded_paths(&self) -> Vec<PathBuf> {
        self.loaded.borrow().clone()
    }

    /// Forgets the record of which paths have been read.
    ///
    /// Call it between compiles when you reuse one filesystem, so that
    /// [`loaded_paths`](Self::loaded_paths) describes the latest compile
    /// rather than every compile so far.
    #[inline]
    pub fn clear_loaded_paths(&self) {
        self.loaded.borrow_mut().clear();
    }
}

/// Normalizes a path lexically, without consulting any real filesystem.
///
/// Drops `.` segments and resolves `..` against the preceding segment. A
/// leading `..` that has nothing to pop is kept, because discarding it would
/// silently turn a path that escapes the tree into one that does not.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                // Only pop a segment that `..` can legitimately cancel. Popping
                // a root or a `..` already kept would change which file the
                // path names.
                match out.components().next_back() {
                    Some(Component::Normal(_)) => {
                        out.pop();
                    }
                    _ => out.push(component),
                }
            }
            other => out.push(other),
        }
    }

    out
}

impl Fs for MemoryFs {
    #[inline]
    fn is_dir(&self, path: &Path) -> bool {
        self.dirs.contains(&normalize(path))
    }

    #[inline]
    fn is_file(&self, path: &Path) -> bool {
        self.files.contains_key(&normalize(path))
    }

    /// Reads a stored file.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::NotFound`] when no file is stored at the path.
    /// The compiler turns that into `Can't find stylesheet to import.` at the
    /// `@use` or `@import` that asked for it.
    #[inline]
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let path = normalize(path);

        match self.files.get(&path) {
            Some(contents) => {
                self.loaded.borrow_mut().push(path);
                Ok(contents.clone())
            }
            None => Err(Error::new(
                ErrorKind::NotFound,
                format!("no such file in MemoryFs: {}", path.display()),
            )),
        }
    }

    /// Normalizes the path.
    ///
    /// The compiler uses the result as the identity of a loaded module, so
    /// this must map every spelling of one file onto a single path. It never
    /// fails: an unknown path normalizes like any other, and the read that
    /// follows is what reports that the file is missing.
    #[inline]
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        Ok(normalize(path))
    }
}

/// Prints the file count rather than the files.
///
/// [`Fs`] requires [`Debug`](fmt::Debug), and a derived one would dump every
/// stylesheet in the tree -- megabytes, for a framework -- into any log line
/// that formats the options.
impl fmt::Debug for MemoryFs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryFs")
            .field("files", &self.files.len())
            .field("dirs", &self.dirs.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    //! The normalization rules are unit-tested here because they decide module
    //! identity: two spellings of one path that survive normalization make the
    //! compiler execute a module twice. Resolution through a whole compile is
    //! covered by `crates/lib/tests/memory_fs.rs`.

    use super::*;

    #[test]
    fn normalizes_current_dir_segments() {
        assert_eq!(
            normalize(Path::new("./a/./b.scss")),
            PathBuf::from("a/b.scss")
        );
    }

    #[test]
    fn normalizes_parent_dir_segments() {
        assert_eq!(
            normalize(Path::new("a/b/../c.scss")),
            PathBuf::from("a/c.scss")
        );
    }

    #[test]
    fn keeps_leading_parent_dir() {
        assert_eq!(
            normalize(Path::new("../a.scss")),
            PathBuf::from("../a.scss")
        );
    }

    #[test]
    fn insert_and_lookup_agree_across_spellings() {
        let mut fs = MemoryFs::new();
        fs.insert("./a/b.scss", "x");

        assert!(fs.is_file(Path::new("a/b.scss")));
        assert!(fs.is_file(Path::new("./a/b.scss")));
        assert!(fs.is_file(Path::new("a/c/../b.scss")));
    }

    #[test]
    fn ancestors_become_directories() {
        let mut fs = MemoryFs::new();
        fs.insert("a/b/c.scss", "x");

        assert!(fs.is_dir(Path::new("a")));
        assert!(fs.is_dir(Path::new("a/b")));
        assert!(!fs.is_dir(Path::new("a/b/c.scss")));
        assert!(!fs.is_dir(Path::new("nope")));
    }

    #[test]
    fn bare_file_name_creates_no_directory() {
        let mut fs = MemoryFs::new();
        fs.insert("a.scss", "x");

        assert!(fs.is_file(Path::new("a.scss")));
        assert!(!fs.is_dir(Path::new("")));
    }

    #[test]
    fn read_records_loaded_paths() {
        let mut fs = MemoryFs::new();
        fs.insert("a.scss", "x");

        assert!(fs.loaded_paths().is_empty());
        assert_eq!(fs.read(Path::new("./a.scss")).unwrap(), b"x");
        assert_eq!(fs.loaded_paths(), vec![PathBuf::from("a.scss")]);

        fs.clear_loaded_paths();
        assert!(fs.loaded_paths().is_empty());
    }

    #[test]
    fn read_of_a_missing_file_is_not_found() {
        let fs = MemoryFs::new();
        let err = fs.read(Path::new("nope.scss")).unwrap_err();

        assert_eq!(err.kind(), ErrorKind::NotFound);
    }
}
