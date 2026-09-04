# Plain CSS

`spec/css/plain` was the deepest unclaimed area on the roadmap: 51
failures, 45 of them "rejects valid input". Measured 2026-09-04 against
the pinned sass-spec revision `4a9eea66` and the dart-sass 1.103.1
binary, under the roadmap's standard flags.

**CSS nesting passthrough has landed** (`zoosky/accent-sass` #27), which
closed 27 of the 51:

```
236 runs, 212 passing, 24 failures, 0 todo, 0 ignored, 0 errors
```

The 24 that remain are four independent defects, none of them about
nesting. They are described below so the next contributor can pick one
without re-deriving the split.

## Landed: CSS nesting passthrough

A `.css` file may nest style rules. Sass does not resolve that nesting --
it is CSS nesting, which the browser resolves -- so the rule keeps its own
selector and stays nested in the output:

```css
/* plain.css */
a {b {c: d}}
```

```css
/* output */
a {
  b {
    c: d;
  }
}
```

The rules that fall out of that, all of which dart-sass 1.103.1 applies
and this fork now applies too:

- `&` is the CSS nesting selector, not Sass's parent selector. It is
  written out unresolved, may appear anywhere in a compound selector
  (`.b&.c`), and may not carry a suffix (`&b` is an error).
- An at-rule directly inside the outermost rule still bubbles out of it,
  the way it does in Sass. Once nesting has been passed through, at-rules
  stay where they were written and media queries are not merged: the
  stylesheet already needs a browser that supports nesting.
- A leading combinator is fine in a nested rule and an error at the top
  level, where there is no enclosing rule to be relative to. A trailing
  combinator is an error anywhere.
- A plain CSS rule that uses `&` and is loaded into a Sass rule through
  `@import` or `meta.load-css` is nested under that rule rather than
  merged into it, so the `&` still reaches the browser.

## 1. The CSS `@function` rule

`spec/css/plain/function/**` -- 8 failures.

```css
@function --a(--b <color>) {result: c}
```

gives "This at-rule isn't allowed in plain CSS."; dart-sass passes the
rule through. The same feature accounts for `spec/css/function` (16) and
`spec/directives/function` (15) on the roadmap, so it is worth doing as
its own item across all three areas rather than for these 8 alone.

Note the `result` descriptor is parsed specially: dart-sass's
`_declarationOrBuffer` treats `result:` inside a plain CSS `@function`
like a custom property, taking the rest of the value verbatim instead of
as SassScript.

## 2. Whitespace in `@import ... supports(...)`

`spec/css/plain/import/whitespace/supports/**` -- 13 failures, 11 of them
in the indented syntax.

```scss
@import "a.css" supports(
    a: b)
```

gives "Expected expression."; dart-sass accepts a newline after the open
paren, around `and`, around `not`, and before the closing paren. This is
a parser gap in the import-modifier path, not in `@supports` itself.

## 3. The CSS `if()` function in plain CSS

`spec/css/plain/if` -- 1 failure.

```css
a {b: if(css(1): c; css(2): d; else: e)}
```

gives `expected ")"`. `if()` with CSS-style conditions landed for Sass in
[02-css-if-function.md](02-css-if-function.md) (#13); the plain CSS
parser does not reach that code path.

## 4. `//` inside a plain CSS value

`spec/css/plain/slash/without_intermediate/no_whitespace` -- 1 failure.

```css
a {b: 1///bar}
```

gives "Silent comments aren't allowed in plain CSS.". There are no silent
comments in plain CSS, so `//` in a value position is just two slashes;
the value parser should not be looking for a comment there. The sibling
test with whitespace (`1/ / /bar`) already passes.

## Ground rules

The roadmap's [ground rules](README.md#ground-rules-for-every-item) apply:
verify every expectation against the dart-sass 1.103.1 binary, add
regression tests to `crates/lib/tests/`, and run both clippy toolchains
before committing.
