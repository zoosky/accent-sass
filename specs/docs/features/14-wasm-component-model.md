# A Component Model build, and why not yet

`wasm32-wasip2` plus a WIT interface, so Accent could load this compiler
through its plugin runtime instead of linking it.

**This item unlocks no sass-spec fixtures, and it is open in the sense that it
is written down, not in the sense that it is queued.** It exists so the option
is recorded with its costs attached, and so the next person to propose it
starts from the argument rather than from scratch.

## The recommendation

Do not build this without one of the triggers below. Accent links this
compiler natively today, pins it by revision, and gates every bump on four
frameworks compiling byte-identically. A component boundary would add
marshalling to every compile and a WIT interface to maintain, and would buy
nothing that the current arrangement does not already provide.

## What already exists

- Accent embeds wasmtime and runs Component-Model components for its plugin
  system, so the host side is built and in production.
- The compiler is portable: no time, thread or process APIs, and `Fs` is a
  three-method trait, so nothing in it resists a component boundary.
- `wasm32-wasip1` compiles today ([13](13-wasm-wasi.md)). `wasm32-wasip2` and
  the component tooling were not exercised; treat that as unmeasured rather
  than as working.

## What it would cost

Stated as expectations, not measurements, because nothing here has been built:

- **A WIT interface.** `compile-string(source, options) -> result<string,
  error>` is the easy half. The filesystem is the hard half: `Fs` would become
  a WIT resource the host implements, and every `is-file` and `read` during an
  `@use` graph walk becomes a call across the boundary.
- **Marshalling on every compile.** Source in, CSS out, plus the import
  traffic above. Against a direct library call this is pure overhead; how much
  is unknown and would need measuring before anyone commits.
- **A second distribution path.** The component becomes an artifact to build,
  version, sign and pin, alongside the git revision Accent already pins.

## The triggers that would justify it

Build this if one of these becomes true, and say which in the pull request:

1. **Accent needs to compile Sass it does not trust.** A hosted service
   compiling stylesheets from users who are not the site owner. The sandbox
   then buys something a native link cannot: a compile that cannot read the
   filesystem, cannot exhaust memory beyond a limit, and cannot outrun a
   deadline. This is the strongest trigger, and it is a product decision
   rather than a technical one.
2. **A theme needs to pin its own compiler version.** If themes ever ship
   against incompatible compiler behaviour, a component per theme decouples
   them from Accent's single pinned revision. Nothing today asks for that.
3. **A third party wants to embed the compiler in a Component-Model host.**
   External demand, which no one has expressed.

Performance is not a trigger. The native link is faster than any component
boundary; nobody should build this expecting a speedup.

## If it goes ahead

- Start from [13](13-wasm-wasi.md). WASI is the same portability work with a
  fraction of the surface, and it settles the path questions a component would
  otherwise hit for the first time inside a WIT resource.
- Define the WIT interface before writing any Rust, and review it as an
  interface -- it is a compatibility surface, and unlike this crate's Rust API
  it will have a host on the other side that Accent ships separately.
- Measure a compile through the boundary against the native path on the same
  input, and put both numbers in the pull request. If the overhead is large
  enough to change how Accent builds a site, that is a finding worth having
  before the interface is finalised, not after.

## Acceptance criteria

There are none until a trigger fires. When one does, the first deliverable is
a measurement of the boundary cost and a WIT draft, not an implementation.
