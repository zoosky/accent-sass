# The browser demo

A page that compiles Bulma and USWDS from source in the browser, using the
WebAssembly build of this compiler.

It is published as part of the [documentation site](../README.md), at `/demo`.
The page itself is rendered by Accent CMS like every other page on the site:
`../content/demo/default.md` names the theme's `demo` template, which holds the
interface markup inside the site's header, footer, search and theme toggle.

This directory holds what that page loads and Accent does not build. The Pages
workflow builds the site, builds `vendor/`, and copies these files beside the
rendered page.

## What is committed and what is built

| Path | What | Built by |
|---|---|---|
| `app.js` | The interface: presets, theme controls, result views | committed |
| `worker.js` | The compiler, off the main thread | committed |
| `styles.css` | The page's styling, scoped under `.demo` | committed |
| `vendor/pkg/` | 1.72 MB wasm, 0.62 MB gzipped | `wasm-pack` |
| `vendor/frameworks/*.json` | 1.16 MB of JSON | `.github/scripts/demo-bundle.mjs` |

`vendor/` is ignored and rebuilt on every deploy rather than committed. The
module changes whenever the compiler changes, so committing it would add a new
copy of a binary to the history on each release.

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

# 4. Build the site, which renders the page, and put the demo's files beside it
accent build --config docs/config.yaml --output docs/output --clean
cp docs/demo/app.js docs/demo/worker.js docs/demo/styles.css docs/output/demo/
cp -R docs/demo/vendor docs/output/demo/vendor

# 5. Serve it under the site's path prefix. Opening the file directly does not
#    work: ES modules and the wasm fetch both need an http origin.
accent serve-static --dir docs/output --base-path /accent-sass --port 4470 --no-tls
```

Then open <http://127.0.0.1:4470/accent-sass/demo/>.

`wasm-exports` is not a default feature. Without it wasm-bindgen exports
nothing, every symbol is dead code, and the page loads a module with no
compiler in it.

## How it fits together

- The `demo` template owns the markup. `app.js` finds every element by id, so
  the ids in the template are its contract with the script.
- `worker.js` owns the compiler. USWDS takes over a second to compile, which
  would freeze the page if it ran on the main thread, and the framework
  bundles are parsed there too so the first paint is not blocked.
- `app.js` owns the interface: the presets, the theme controls that rewrite
  the `@use … with (…)` block, and the result views.
- `styles.css` loads after the theme's stylesheet. Every rule is scoped under
  `.demo`, and its colours are the theme's tokens, so the page follows the
  header's theme toggle and cannot restyle the shell around it.
- The compiler resolves imports against a file map the page supplies, because
  `Fs::read` is synchronous and an import therefore cannot be fetched while
  compiling. Each framework bundle is loaded in full before its first compile.

## Keeping it honest

`demo-check.mjs` compiles each framework in the manifest through the built
package, with the same entry point and load paths the page uses. The bundle
layout, the entry stylesheet and the JavaScript API are each checked
elsewhere; nothing else checks that the three agree, and a mismatch shows up
only as a blank page.
