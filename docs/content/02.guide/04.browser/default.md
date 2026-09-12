---
title: In a browser
template: docs
lead: >-
  Compiling Sass client side, including a whole design system, with the
  WebAssembly build.
menu:
  visible: true
  order: 4
description: >-
  Compile Sass in a browser with the accent-sass WebAssembly build: the files
  map, load paths, theme configuration, and the synchronous filesystem
  constraint.
---

The compiler builds to `wasm32-unknown-unknown` and exposes a JavaScript API
shaped after dart-sass's. The [demo](/demo/) compiles Bulma and USWDS from
source in the page, with nothing sent anywhere.

Build the package first -- see [Install](/guide/install#the-browser-package).

## Compile a single stylesheet

```js
import init, { compileString } from "./pkg/index.js";

await init();

const { css } = compileString("a { b: calc(1rem - 2px); }");
```

## Compile something with imports

There is no filesystem in a browser, so you supply one. `files` is a map of
path to source, and the compiler resolves `@use`, `@forward` and `@import`
against it:

```js
const files = {
  "theme/_colors.scss": "$brand: #bada55 !default;",
  "theme/_index.scss": '@forward "colors";',
};

const { css, loadedUrls } = compileString(
  '@use "theme" with ($brand: rebeccapurple);\na { color: theme.$brand; }',
  { files },
);
```

`loadedUrls` reports the files the compile read. It counts reads rather than
distinct files, so a stylesheet consulted twice appears twice.

> [!WARNING]
> **Every file must be in `files` before you call.** Reads are synchronous: the
> compiler asks for a file and gets bytes back, with nothing to await. An
> importer cannot `fetch`, cannot `await`, and cannot reach the File System
> Access API. Load your theme's stylesheets into the map first, then compile.

## Compile a whole framework

A framework is just a large `files` map plus a load path. This is what the demo
does with USWDS -- 605 stylesheets, about 800 KiB of Sass, compiled to roughly
33,700 lines of CSS in a few seconds:

```js
const files = await fetch("uswds.json").then((r) => r.json());

const { css } = compileString(
  '@use "uswds-core" with ($theme-color-primary: "red-50v");\n@forward "uswds";\n',
  { files, loadPaths: ["uswds"] },
);
```

Configuring a framework through `@use ... with (...)` recompiles it from
source, so you get the stylesheet the framework would have produced rather than
overrides layered on top.

> [!NOTE]
> Inside `@use ... with (...)` a bare comma separates *arguments*. A list value
> needs parentheses: `$family: ("Inter", system-ui, sans-serif)`. Quoting the
> whole stack instead parses, but produces one font name that matches nothing.

## Keep the page responsive

USWDS takes a few seconds. Run the compiler in a worker so typing and scrolling
are not blocked:

```js
// worker.js
import init, { compileString } from "./pkg/index.js";
const ready = init();

self.onmessage = async ({ data }) => {
  await ready;
  try {
    const result = compileString(data.source, { files: data.files });
    self.postMessage({ ok: true, css: result.css });
  } catch (error) {
    self.postMessage({ ok: false, message: error.message, formatted: error.formatted });
  }
};
```

A module worker is required, because the package is an ES module.

## Show errors where they happened

A failed compile throws a real `Error`, so `instanceof Error` holds. Its
`message` is the Sass message alone, and it carries the formatted block plus
the position an editor needs to underline:

```js
try {
  compileString("a { color: ; }");
} catch (error) {
  console.error(error.message);   // Expected expression.
  console.error(error.formatted); // ...with the source line and a caret
  console.error(error.line, error.column); // 1-based
}
```

## Capture warnings

```js
compileString(source, {
  files,
  logger: (event) => console.warn(`${event.type} ${event.file}:${event.line}`, event.message),
});
```

Pass `quiet: true` instead to silence them. [The JavaScript
API](/reference/javascript-api) lists every option.

## Serving it

The package is an ES module and `init()` fetches the `.wasm`, so a browser
refuses both over `file:`. Serve the directory over HTTP.
