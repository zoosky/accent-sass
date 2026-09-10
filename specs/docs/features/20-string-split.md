# string.split

**Landed. `spec/core_functions/string` is down from 8 failures to 1, and the
suite from 284 to 277.** The one left is `private_use_character`, which needs
the escaping fix item 24 records as well; this document said so before the
change and the measurement bears it out.

8 sass-spec failures in `spec/core_functions/string`, every one of them in
`string.split` and every one of them in the same 35-line function. The rest
of the string module is clear.

Measured 2026-09-10 against master (`31361d7`), the pinned sass-spec
revision `4a9eea66`, and dart-sass 1.103.1 run as `npx -y sass@1.103.1`.

The function is `str_split` in
`crates/compiler/src/builtin/functions/string.rs:128`. It has three
defects, and the eight fixtures are combinations of them.

| input | accent-sass | dart-sass 1.103.1 |
|---|---|---|
| `string.split("Helvetica", "")` | `["", "H", ..., "a", ""]` | `["H", ..., "a"]` |
| `string.split("", "/")` | `[""]`, printed `["",]` | `[]` |
| `string.split(abc, "")` | `["", "a", "b", "c", ""]` | `[a, b, c]` |
| `string.split("a b c", " ")` | `["a", "b", "c"]` | agrees |

## 1. An empty separator is not a Rust empty pattern

`split/{single,empty_separator,both_empty,double_width_character,private_use_character,unquoted_string}`

Rust's `str::split("")` matches at every character boundary *including the
two ends*, so it yields an empty string before the first character and
another after the last. Dart's `String.split("")` yields the characters
alone. The compiler hands the separator straight to `split` and `splitn`,
so every split on an empty separator comes back with two extra elements.

Special-case the empty separator and split into characters. Sass counts
Unicode code points, not UTF-16 units, so `chars()` is the right iterator:
the `double_width_character` fixture splits a two-element string whose
first character sits outside the basic multilingual plane, and expects two
elements back -- which `chars()` gives and a byte- or UTF-16-based split
does not.

## 2. An empty string is not an empty list

`split/{empty_string,empty}`

`"".split("/")` in Rust yields one empty string; dart-sass returns an empty
list. The `empty` fixture also checks that the empty result still reports
`comma` as its separator, so the empty list is comma-separated and
bracketed rather than undecided.

The `$limit` check has to stay in front of this: `string.split("", ",", 0)`
is `$limit: Must be 1 or greater, was 0.` in dart-sass, not an empty list.

## 3. The result is always quoted

`split/unquoted_string`

Both `map` closures built `Value::String(s, QuoteKind::Quoted)`. dart-sass
gives each piece the quotedness of the input string, so
`string.split(abc, "")` returns `[a, b, c]` unquoted. The input's
`QuoteKind` now carries through instead.

## What section 1 does not fix on its own

`split/private_use_character` needs one more thing. It splits `"\E000"`
and expects `["\e000"]` with no `@charset` line, because dart-sass writes
private-use characters back as escapes in expanded mode. This compiler
emits the character itself and then adds `@charset "UTF-8"` because the
output is no longer ASCII. That is a serializer difference, not a split
one -- `spec/libsass-closed-issues/issue_1231` fails the same way with no
`split` in sight -- and item 24 records it. Both changes are needed for
this fixture to pass.

## Testing

- Ground truth: `spec/core_functions/string/split.hrx` at the pinned
  revision.
- Scoped spec run: `spec/core_functions/string`.
- Add regression tests to `crates/lib/tests/` with `test!`: an empty
  separator, an empty string, an unquoted input, a two-code-unit character,
  and the `$limit` path with an empty separator, which no fixture covers.
- Verify every new expectation against dart-sass 1.103.1 with
  `npx -y sass@1.103.1`, never bare `npx sass`.

## Acceptance criteria

- ~~`spec/core_functions/string` drops from 8 failures to 1, the one left
  being `private_use_character`, which also needs item 24's escaping fix.~~
  Done.
- With that fix too, the area is clear.
- ~~No other area moves: `str_split` has one caller.~~ Done: 284 to 277,
  seven fixtures, none newly failing.
