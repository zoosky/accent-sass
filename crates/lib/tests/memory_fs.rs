//! Resolution of `@use`, `@forward` and `@import` through [`MemoryFs`].
//!
//! These are plain `#[test]` functions rather than the `test!` macro used
//! elsewhere in this directory, because the macro compiles with
//! `Options::default()` and the whole point here is a non-default filesystem.
//!
//! What is being tested is the resolver's contract with [`accent_sass::Fs`],
//! not the evaluator: partials, index files, load paths, and the path
//! normalization that decides whether a module is instantiated once or twice.
//! A browser build depends on every one of these, because there is no
//! `std::fs` underneath it to paper over a mistake.

use accent_sass::{MemoryFs, Options, from_path, from_string};

/// Compiles `input` as a virtual entry point named `stdin`.
fn compile(fs: &MemoryFs, input: &str) -> String {
    from_string(input.to_owned(), &Options::default().fs(fs)).expect("compile failed")
}

#[test]
fn use_resolves_a_partial() {
    let mut fs = MemoryFs::new();
    fs.insert("_colors.scss", "$brand: #bada55;");

    assert_eq!(
        compile(&fs, "@use \"colors\";\na { color: colors.$brand; }"),
        "a {\n  color: #bada55;\n}\n"
    );
}

#[test]
fn use_resolves_an_index_file_in_a_directory() {
    let mut fs = MemoryFs::new();
    fs.insert("lib/_index.scss", ".from-index { color: red; }");

    assert_eq!(
        compile(&fs, "@use \"lib\";"),
        ".from-index {\n  color: red;\n}\n"
    );
}

#[test]
fn use_resolves_through_a_load_path() {
    let mut fs = MemoryFs::new();
    fs.insert("vendor/framework/_index.scss", ".f { color: green; }");

    let options = Options::default().fs(&fs).load_path("vendor");
    let css = from_string("@use \"framework\";".to_owned(), &options).expect("compile failed");

    assert_eq!(css, ".f {\n  color: green;\n}\n");
}

#[test]
fn imports_resolve_relative_to_the_importing_file() {
    let mut fs = MemoryFs::new();
    fs.insert("a/b/_leaf.scss", ".leaf { color: blue; }");
    fs.insert("a/_mid.scss", "@use \"b/leaf\";");

    assert_eq!(
        compile(&fs, "@use \"a/mid\";"),
        ".leaf {\n  color: blue;\n}\n"
    );
}

#[test]
fn forwarded_defaults_can_be_configured_with_use_with() {
    // This is the shape every real framework is themed through: an index file
    // forwards a settings partial, and the entry point overrides one of its
    // `!default` values. Bulma and USWDS are both configured this way.
    let mut fs = MemoryFs::new();
    fs.insert("lib/_vars.scss", "$c: red !default;");
    fs.insert("lib/_index.scss", "@forward \"vars\";");

    assert_eq!(
        compile(&fs, "@use \"lib\" with ($c: blue);\na { color: lib.$c; }"),
        "a {\n  color: blue;\n}\n"
    );
}

#[test]
fn from_path_compiles_an_entry_point_in_the_filesystem() {
    let mut fs = MemoryFs::new();
    fs.insert("theme/_colors.scss", "$brand: #bada55;");
    fs.insert(
        "theme/index.scss",
        "@use \"colors\";\na { color: colors.$brand; }",
    );

    let css = from_path("theme/index.scss", &Options::default().fs(&fs)).expect("compile failed");

    assert_eq!(css, "a {\n  color: #bada55;\n}\n");
}

#[test]
fn a_module_used_twice_is_only_loaded_once() {
    // Path normalization is what makes this hold: `lib/_shared.scss` reached
    // as `shared` and as `./shared` must canonicalize to one path, or the
    // module is instantiated twice and its CSS is emitted twice.
    let mut fs = MemoryFs::new();
    fs.insert("_shared.scss", ".shared { color: red; }");
    fs.insert("_one.scss", "@use \"shared\";");
    fs.insert("_two.scss", "@use \"./shared\";");

    assert_eq!(
        compile(&fs, "@use \"one\";\n@use \"two\";"),
        ".shared {\n  color: red;\n}\n"
    );
}

#[test]
fn loaded_paths_record_what_the_compile_touched() {
    let mut fs = MemoryFs::new();
    fs.insert("_colors.scss", "$brand: red;");
    fs.insert("_index.scss", "@use \"colors\";");

    let _ = compile(&fs, "@use \"index\";");

    let loaded = fs.loaded_paths();
    assert!(
        loaded.iter().any(|p| p.ends_with("_colors.scss")),
        "expected _colors.scss among {loaded:?}"
    );
    assert!(
        loaded.iter().any(|p| p.ends_with("_index.scss")),
        "expected _index.scss among {loaded:?}"
    );
}

#[test]
fn clearing_loaded_paths_scopes_them_to_one_compile() {
    let mut fs = MemoryFs::new();
    fs.insert("_colors.scss", "$brand: red;");

    let _ = compile(&fs, "@use \"colors\";");
    assert!(!fs.loaded_paths().is_empty());

    fs.clear_loaded_paths();
    assert!(fs.loaded_paths().is_empty());
}

#[test]
fn a_missing_import_reports_the_usual_error() {
    let fs = MemoryFs::new();
    let err = from_string("@use \"nope\";".to_owned(), &Options::default().fs(&fs))
        .expect_err("expected the import to fail");

    assert!(
        err.to_string().contains("Can't find stylesheet to import."),
        "unexpected error: {err}"
    );
}

#[test]
fn an_empty_filesystem_resolves_nothing() {
    let fs = MemoryFs::new();

    assert!(fs.is_empty());
    assert_eq!(fs.len(), 0);
    assert_eq!(compile(&fs, "a { color: red; }"), "a {\n  color: red;\n}\n");
}
