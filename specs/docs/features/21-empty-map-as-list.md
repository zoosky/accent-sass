# The empty map as a list

**Landed. `spec/core_functions/list` is down from 9 failures to 1.** All
three sections are done; the fixture left is `join/error/named`, which item
24 covers. Measured on master after items 16, 17, 20 and 23 landed: 243
failures before, 234 after, nothing regressed.

The nine include one outside this item's area. A review pointed out that all
three sections apply verbatim to `Value::ArgList`, which is a list too: the
same arms now match it, and that closed
`spec/libsass-closed-issues/issue_1269`, the fourth of item 24's
arglist-separator fixtures. The other three are a different defect -- an
arglist built from a splatted list should keep *that list's* separator -- and
are still open under item 24.

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

`Value::separator` now returns `ListSeparator::Undecided` when the map is
empty. The enum already had the variant and `as_str` already mapped it to a
space.

`list.join` also hard-coded `ListSeparator::Comma` for a map on either side
rather than asking the value, so the four `join/empty/map` fixtures needed
that call site as well as the predicate.

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

`Value::as_list` at line 476 already converts a map to its pair list and
`Value::separator` gives the separator, so `append` now uses both. The same
arm matches `Value::ArgList`, which fell through to the scalar case in
exactly the same way: `list.append(args(1, 2), 3)` gave `1, 2 3` where
dart-sass gives `1, 2, 3`. The other list builtins were audited:
`list.length` was already right, and `list.join` needed the separator half of
the same fix on both of its operands.

## 3. An empty map does not equal an empty list

`list/utils/empty_map/same_as_empty_list`

`PartialEq for Value` at line 98 compared a map only against another map,
and `not_equals` at line 428 fell through to `s != other`, so
`$empty-map == ()` was `false`. dart-sass makes the empty map equal to any
empty list. Both directions are now handled in `PartialEq`, and `not_equals`
has the case too: it is written out rather than derived from `eq`, so fixing
one alone would have left `!=` disagreeing with `==`. A test pins all four
combinations.

An empty arglist is the same value as the empty map, and both directions are
handled. It is *not* equal to `()`, though, which is the one pairing where an
arglist's separator counts: it is comma-separated and `()` is undecided.
dart-sass agrees, and a test pins that too, because it looks like an
inconsistency and is not.

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

- ~~`spec/core_functions/list` drops from 9 failures to 1, the one left
  being `join/error/named`, which item 24 covers.~~ Done.
- ~~`spec/core_functions/map` and `spec/values/lists` do not regress.~~
  Done: 243 to 234, nine fixtures, none newly failing anywhere in the
  suite.
