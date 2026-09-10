# The empty map as a list

8 sass-spec failures in `spec/core_functions/list`. Sass says a map *is* a
list of key-value pairs and the empty map *is* the empty list; this
compiler keeps `Value::Map` and `Value::List` apart in three places where
the language does not.

Measured 2026-09-10 against master (`31361d7`), the pinned sass-spec
revision `4a9eea66`, and dart-sass 1.103.1 run as `npx -y sass@1.103.1`.

The fixtures build their empty map with `map.remove((a: b), a)` -- see
`spec/core_functions/list/_utils.scss`, which says in a comment that the
result "*should* be treated as identical to the literal `()`" -- so a
literal `()` will not reproduce any of this. That is why these eight
survived every earlier item: the shape only appears when a map arrives
through a function.

```scss
$empty-map: map.remove((a: b), a);
a {
  eq: $empty-map == ();
  sep: list.separator($empty-map);
  join: meta.inspect(list.join($empty-map, (1 2 3)));
  app: meta.inspect(list.append((c: d, e: f), g));
}
```

| | dart-sass 1.103.1 | accent-sass |
|---|---|---|
| `eq` | `true` | `false` |
| `sep` | `space` | `comma` |
| `join` | `1 2 3` | `1, 2, 3` |
| `app` | `c d, e f, g` | `(c: d, e: f) g` |

## 1. An empty map's separator is comma, not undecided

`list/separator/empty/map`, `list/join/empty/map/{first,second}/*`

`Value::separator` at `crates/compiler/src/value/mod.rs:485` returns
`ListSeparator::Comma` for every map:

```rust
Value::Map(..) | Value::ArgList(..) => ListSeparator::Comma,
```

A non-empty map is comma-separated, but an empty one has no separator to
speak of, and dart-sass reports it as undecided -- which `list.separator`
prints as `space` and which lets the *other* operand of a `join` decide.
That is the whole of the four `join/empty/map` failures: joining an empty
map with `1 2 3` must give `1 2 3`, and today the map's comma wins.

Return `ListSeparator::Undecided` when the map is empty. The enum already
has the variant and `as_str` already maps it to a space.

## 2. `list.append` does not see a map as a list

`list/append/map/{empty,non_empty}`

`append` at `crates/compiler/src/builtin/functions/list.rs:101` destructures
only `Value::List`:

```rust
let (mut list, sep, brackets) = match args.get_err(0, "list")? {
    Value::List(v, sep, b) => (v, sep, b),
    v => (vec![v], ListSeparator::Undecided, Brackets::None),
};
```

A map falls to the second arm and becomes a one-element list *containing a
map*, so `list.append((c: d, e: f), g)` returns `(c: d, e: f) g` and then
fails to serialize with `(c: d, e: f) isn't a valid CSS value.`

`Value::as_list` at line 476 already converts a map to its pair list, and
`Value::separator` gives the separator; use both here rather than matching
on the variant. Audit the other list builtins for the same pattern while
you are in the file -- `list.length` and `list.join` already agree with
dart-sass on a map, so the pattern is not uniform.

## 3. An empty map does not equal an empty list

`list/utils/empty_map/same_as_empty_list`

`PartialEq for Value` at line 98 compares a map only against another map,
and `not_equals` at line 428 falls through to `s != other`, so
`$empty-map == ()` is `false`. dart-sass makes the empty map equal to any
empty list. Add that case to both, and keep them consistent: `not_equals`
is not derived from `eq`, so a fix to one alone leaves `!=` disagreeing
with `==`.

## Testing

- Ground truth: `spec/core_functions/list/{separator,join/empty,append,utils}.hrx`
  and `spec/core_functions/list/_utils.scss` at the pinned revision.
- Scoped spec runs: `spec/core_functions/list`, `spec/core_functions/map`,
  and `spec/values/lists`.
- Add regression tests to `crates/lib/tests/` with `test!`, all of them
  building the map through `map.remove` rather than as a literal, plus the
  `==`/`!=` pair from section 3.
- Verify every new expectation against dart-sass 1.103.1 with
  `npx -y sass@1.103.1`, never bare `npx sass`.

## Acceptance criteria

- `spec/core_functions/list` drops from 9 failures to 1, the one left being
  `join/error/named`, which item 24 covers.
- `spec/core_functions/map` and `spec/values/lists` do not regress.
  Section 1 changes what every empty map reports, and section 3 changes
  equality, which maps and lists both reach.
