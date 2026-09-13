---
title: Sass, in Rust
template: home
menu:
  visible: false
  order: 0
description: >-
  accent-sass is a Sass compiler written in Rust, at parity with dart-sass
  1.104.1. It compiles Bulma, Pico, Foundation and USWDS, runs as a library or
  a binary, and compiles to WebAssembly so the same engine runs in a browser.

# `lead` above is one of Accent's own page fields, so home.html.jinja reads it
# as `page.lead`. Everything below is read through `page.custom.*`. A landing
# page is a layout with slots rather than a document, so its copy lives here as
# data instead of as body prose -- which also means the wording can change
# without touching a template.

eyebrow: "v0.16.0 - MIT - dart-sass 1.104.1"
headline: "Sass,"
headline_accent: "in Rust."
lead: >-
  Compile Sass to CSS with no Node, no libsass and no subprocess. A library, a
  command-line binary, and a WebAssembly module that compiles a whole design
  system in a browser tab.

pipeline_title: Source in, CSS out, checked against the reference
pipeline_lead: >-
  The first three stages are what any Sass compiler does. The fourth is why
  this one can claim parity rather than resemblance: every build compiles four
  real frameworks with both engines and fails on a single differing colour
  value.

stages:
  - name: parse
    description: >-
      Source text to an AST, in SCSS, the indented syntax, or plain CSS. The
      syntax follows the file extension unless you override it.
  - name: evaluate
    description: >-
      Modules, mixins, functions and colour maths. `@use` and `@forward`
      resolve through a filesystem you can replace.
  - name: serialize
    description: >-
      Expanded or compressed CSS, with `@charset` handling that matches the
      reference implementation.
  - name: verify
    description: >-
      Bulma, Pico, Foundation and USWDS compiled with both engines on every
      commit. A differing colour value fails the build.

figures_title: The reference implementation is the test
figures_lead: >-
  dart-sass 1.104.1 is the reference, and a deviation from it is a bug rather
  than a dialect. The numbers below come from the official spec suite and from
  the framework corpus CI compiles on every commit.

figures:
  - value: "14,147"
    tone: green
    label: sass-spec tests passing of 14,266
  - value: "0"
    tone: cyan
    label: lines differing across four frameworks
  - value: "32,781"
    tone: gold
    label: lines of USWDS CSS, identical to dart-sass
  - value: "0.62"
    tone: coral
    label: MB of WebAssembly, gzipped

features_title: What the compiler actually promises
features_lead: >-
  A short list, because each item is something the code is arranged around
  rather than something it happens to do.

features:
  - title: Parity is the contract
    body: >-
      dart-sass is the reference implementation, currently 1.104.1. Where
      output deliberately differs, the reason is written next to the test. A
      re-baselined expectation that nobody checked against the reference is how
      wrong behaviour gets frozen, so it is not allowed.
  - title: Real frameworks, every commit
    body: >-
      Bulma, Pico, Foundation and USWDS compile with both engines in CI, and
      any differing colour value fails the job. All four compile
      byte-identically to dart-sass; for USWDS that is 32,781 lines on both
      sides, comments included.
  - title: The filesystem is a seam
    body: >-
      <code>Options::fs</code> takes any <code>Fs</code> implementation, so a
      host that already holds its stylesheets never writes them to disk.
      <code>MemoryFs</code> ships as one, and is what the browser build
      resolves imports through.
  - title: It runs in a browser
    body: >-
      A WebAssembly build exposes <code>compileString</code> and
      <code>compile</code> with dart-sass's option names. The demo on this site
      compiles USWDS -- 605 stylesheets -- client side, with nothing sent
      anywhere.
  - title: Compile Sass at build time
    body: >-
      <code>accent_sass::include!</code> compiles a stylesheet into your binary
      during <code>cargo build</code>, tracked for incremental rebuilds. No
      asset pipeline, no build script, no runtime cost.
  - title: A C ABI for plugin hosts
    body: >-
      The <code>wasi-exports</code> feature exposes a C ABI for
      <code>wasm32-wasip1</code>, so a host can instantiate the compiler once
      and compile many stylesheets without paying process startup for each.

band_title: Watch it compile a design system, live
band_lead: >-
  The demo holds Bulma and USWDS in memory and compiles them in your browser.
  Change a theme token and the whole framework rebuilds from source -- not a
  layer of CSS pasted on top.
---

## Why a Rust implementation

dart-sass is the reference implementation and it is written in Dart. A Rust
program that wants Sass has three options: shell out to a Node or Dart process,
bind to the deprecated libsass, or use a native port. The first adds a runtime
to your deployment; the second stopped at a dialect of Sass that has since
moved on.

This is the third. It is a fork of
[`connorskees/grass`](https://github.com/connorskees/grass), whose last release
was 0.13.4 in August 2024, and it carries the modern Dart Sass features
upstream has not released -- the colour spaces, `color.channel`, the module
system corners, the calculation long tail -- so that a Rust program can build
Bulma or USWDS without another toolchain in the picture.

Parity with dart-sass is the goal rather than an aspiration to approximate it.
Where the two cannot agree, the difference is written down rather than emulated
silently; [Compatibility](/reference/compatibility) is the list.
