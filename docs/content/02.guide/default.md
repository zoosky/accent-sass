---
title: Usage guide
template: docs
lead: >-
  How to compile Sass with accent-sass, whichever way you are running it.
menu:
  visible: true
  order: 2
description: >-
  Install accent-sass and compile Sass from the command line, from Rust, at
  build time, or in a browser through WebAssembly.
---

`accent-sass` compiles [Sass](https://sass-lang.com/documentation/) to CSS. It
is one compiler with four front ends, and they share everything except how you
call them:

| You want | Read |
|---|---|
| A `sass` replacement on the command line | [Command line](/guide/command-line) |
| To compile stylesheets from a Rust program | [Rust library](/guide/rust) |
| CSS baked into your binary at compile time | [Rust library](/guide/rust#compile-sass-at-build-time) |
| To compile Sass in a browser or an editor | [In a browser](/guide/browser) |

Start with [Install](/guide/install).

## What it is compatible with

dart-sass is the reference implementation, currently **1.104.1**. A deviation
from it is treated as a bug, not as a dialect, with two exceptions that are
written down: error message wording, and the spans errors point at.

If you are migrating from libsass, expect the same changes you would make to
adopt dart-sass; there is no third dialect here. If you are migrating from
dart-sass, expect your stylesheets to compile unchanged. Bulma, Pico,
Foundation and USWDS are compiled with both engines on every commit, and a
single differing colour value fails the build.

[Compatibility](/reference/compatibility) has the numbers and the known gaps.

## A first compile

```scss
// input.scss
$brand: #bada55;

.button {
  background: $brand;
  border: 1px solid color.adjust($brand, $lightness: -10%);

  &:hover {
    background: color.adjust($brand, $lightness: 8%);
  }
}
```

```sh
accent-sass input.scss
```

```css
.button {
  background: #bada55;
  border: 1px solid #a8c932;
}
.button:hover {
  background: #c5e173;
}
```

That is the whole idea. Everything else is about where the source comes from
and where the CSS goes.
