# The project website

<https://zoosky.github.io/accent-sass>

The landing page, the usage guide, the reference, and a demo that compiles
Bulma and USWDS in your browser. Built with [Accent CMS](https://accentcms.dev),
the project this compiler is maintained alongside.

The circularity is the point: Accent compiles this theme's `main.scss` with
accent-sass, so the stylesheet you read the site through is produced by the
compiler the site documents.

## Build it locally

From the repository root:

```sh
# The site
accent build --config docs/config.yaml --output docs/output --clean

# The demo it links to, which is built separately -- see docs/demo/README.md
wasm-pack build crates/lib --release --target web --out-name index \
  --out-dir ../../docs/demo/vendor/pkg -- \
  --no-default-features --features wasm-exports,random
node .github/scripts/demo-bundle.mjs
cp -R docs/demo docs/output/demo

# Serve the result the way Pages serves it, under the project path prefix
accent serve-static --dir docs/output --base-path /accent-sass --port 4470 --no-tls
```

Then open <http://127.0.0.1:4470/accent-sass/>. The path prefix is not a
mistake: `site.url` in `config.yaml` is the GitHub Pages project URL, so every
internal link in the build is written as `/accent-sass/...`, and serving
without the prefix gives you a working home page whose every link 404s.

`--base-path` is passed explicitly rather than left to its default. It does
default to the `site.base_path` derived from config -- but `serve-static`
looks for `config.yaml` in the current directory, and this project's lives in
`docs/`, so run from the repository root it finds none and mounts at `/`.

Or, while writing content, `accent serve --config docs/config.yaml` on
port 4460 with hot reload.

## Layout

| Path | What |
|---|---|
| `config.yaml` | Site configuration. The `site.url` path component prefixes every internal link |
| `content/` | The pages, as Markdown with frontmatter. Directory order (`02.`, `03.`) is menu order |
| `content/default.md` | The landing page, at the site root |
| `themes/sass/` | The theme: templates, stylesheet, and the header script |
| `demo/` | The browser demo, a hand-written static page with its own README |
| `output/` | **Generated.** What gets deployed |

## Why the demo is copied rather than placed in `media/`

Accent copies `media/` verbatim to `/media/...`, which would move the demo
from `/demo` to `/media/demo`. It is also 2.8 MB of generated wasm and
framework JSON that is deliberately not committed. So the workflow builds the
site, builds the demo, and copies the second into the first. The demo stays a
self-contained page that can be served on its own, and the site keeps the
`/demo` URL.

## The theme

Derived from [accent-proust](https://github.com/zoosky/accent-proust)'s Proust
theme, so the family shares one look: Accent's palette, dark authored first
with light derived from it, a toggle in the header, and search. The playground
template and its stylesheet block were dropped -- this site's interactive page
is `/demo`, which is a static page of its own rather than a template.
