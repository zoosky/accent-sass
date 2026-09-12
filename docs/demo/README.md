# The browser demo

A page that compiles Bulma and USWDS from source in the browser, using the
WebAssembly build of this compiler.

It is published as part of the [documentation site](../README.md), at `/demo`.
The workflow builds the site with Accent CMS, builds this page's wasm and
framework bundles, and copies the second into the first -- the page is not
rendered by the CMS and has no template, which is why it carries its own
stylesheet and its own link back to the site.

That back-link is relative (`../`), so it lands on the documentation site when
the page is served inside it. Serving this directory on its own for
development, the link points above the server root and goes nowhere; nothing
else on the page depends on it.

## What is committed and what is built

Everything in this directory is source except `vendor/`, which is ignored.
`vendor/` holds the two large generated inputs:

| Path | Built by | Size |
|---|---|---:|
| `vendor/pkg/` | `wasm-pack` | 1.69 MB wasm, 0.60 MB gzipped |
| `vendor/frameworks/*.json` | `.github/scripts/demo-bundle.mjs` | 1.16 MB of JSON |

They are rebuilt on every deploy rather than committed. The module changes
whenever the compiler changes, so committing it would add a new copy of a
binary to the history on each release.

## Build it locally

From the repository root:

```bash
# 1. The compiler, as a browser package
wasm-pack build crates/lib --release --target web --out-name index \
  --out-dir ../../docs/demo/vendor/pkg -- \
  --no-default-features --features wasm-exports,random

# 2. The framework sources, as JSON bundles
node .github/scripts/demo-bundle.mjs

# 3. Check that the page's own compiles still work
node .github/scripts/demo-check.mjs

# 4. Serve it. Opening index.html from the filesystem does not work:
#    ES modules and the wasm fetch both need an http origin.
python3 -m http.server -d docs/demo 8000
```

`wasm-exports` is not a default feature. Without it wasm-bindgen exports
nothing, every symbol is dead code, and the page loads a module with no
compiler in it.

## How it fits together

- `worker.js` owns the compiler. USWDS takes over a second to compile, which
  would freeze the page if it ran on the main thread, and the framework
  bundles are parsed there too so the first paint is not blocked.
- `app.js` owns the interface: the presets, the theme controls that rewrite
  the `@use … with (…)` block, and the result views.
- The compiler resolves imports against a file map the page supplies, because
  `Fs::read` is synchronous and an import therefore cannot be fetched while
  compiling. Each framework bundle is loaded in full before its first compile.

## Keeping it honest

`demo-check.mjs` compiles each framework in the manifest through the built
package, with the same entry point and load paths the page uses. The bundle
layout, the entry stylesheet and the JavaScript API are each checked
elsewhere; nothing else checks that the three agree, and a mismatch shows up
only as a blank page.
