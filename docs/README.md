# The project website

<https://zoosky.github.io/accent-sass>

The landing page, the usage guide, the reference, the changelog, and a demo
that compiles Bulma and USWDS in your browser. Built with [Accent CMS](https://accentcms.dev),
the project this compiler is maintained alongside.

The circularity is the point: Accent compiles this theme's `main.scss` with
accent-sass, so the stylesheet you read the site through is produced by the
compiler the site documents.

## Build it locally

From the repository root:

```sh
# The site
accent build --config docs/config.yaml --output docs/output --clean

# What the demo page loads, which Accent does not build -- see docs/demo/README.md
wasm-pack build crates/lib --release --target web --out-name index \
  --out-dir ../../docs/demo/vendor/pkg -- \
  --no-default-features --features wasm-exports,random
node .github/scripts/demo-bundle.mjs
cp docs/demo/app.js docs/demo/worker.js docs/demo/styles.css docs/output/demo/
cp -R docs/demo/vendor docs/output/demo/vendor

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
| `content/demo/default.md` | The demo page, which names the theme's `demo` template |
| `../CHANGELOG.md` | The changelog page, at `/changelog`, through `content.mounts` in `config.yaml` |
| `themes/sass/` | The theme: templates, stylesheet, and the header script |
| `demo/` | What the demo page loads: its scripts, its stylesheet, and the generated `vendor/` |
| `output/` | **Generated.** What gets deployed |

## The changelog is mounted, not copied

`config.yaml` mounts the repository's `CHANGELOG.md` at `/changelog` as a
single-file mount. Accent reads the file on every build, takes the page title
from its `# Changelog` heading, and renders it with the `docs` template, so the
page cannot drift from the file. A relative link inside `CHANGELOG.md` would
resolve against `/changelog` on the site, so links there are absolute.

## Why the demo's assets are copied rather than placed in `media/`

The demo page is rendered by Accent like any other page, from
`content/demo/default.md` and the theme's `demo` template, so it carries the
site's header, footer, search and theme toggle. What it loads is not content:
`app.js`, `worker.js`, `styles.css`, and 2.8 MB of generated wasm and
framework JSON that is deliberately not committed. Accent copies `media/`
verbatim to `/media/...`, which would put those files at `/media/demo`, away
from the page. So the workflow builds the site, builds the demo's inputs, and
copies them beside the rendered page.

## The theme

Derived from [accent-proust](https://github.com/zoosky/accent-proust)'s Proust
theme, so the family shares one look: Accent's palette, dark authored first
with light derived from it, a toggle in the header, and search. The playground
template and its stylesheet block were dropped -- this site's interactive page
is `/demo`, rendered by the `demo` template, with the page's own stylesheet
scoped under `.demo` so it cannot restyle the shell around it.
