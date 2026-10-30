# accent-sass (zoosky fork) - Claude Code guidelines

These are the working rules shared across Zoosky's repositories, copied here
so a session in this repo follows them without needing the Accent checkout
open. Rules that are specific to Accent CMS -- its `specs/` tracker, feature
IDs, editions and licensing, `site-docs/` pages, the 500-line file rule --
deliberately did **not** come across. Do not apply them here.

## Ground rules

**These must always be followed.**

1. **Never push directly to `master`.** Every change goes through a pull
   request.
2. **Create a branch first.** Use `feature/...`, `fix/...` or `chore/...`.
3. **Run the quality gates before committing.** See "Quality gates" below for
   this repository's commands. They differ from Accent's.
4. **Open a pull request for review.**
5. **Wait for CI.** Pull requests must pass before merging.
6. **No emojis in the codebase.**
7. **Test code before shipping it.** A claim that something works needs a run
   behind it, not an inference.
8. **Never commit debugging leftovers** -- `dbg!`, stray `println!`, commented-out
   experiments.
9. **Never add `Claude`, `Generated with Claude Code`, `Co-Authored-By: Claude`
   or any other AI attribution** to the codebase, commit messages, pull
   requests or issues. This file is the one place such mentions belong,
   because it is addressed to the assistant.
10. **Write self-documenting code.** Every module, struct, enum, trait and
    public function gets a doc comment (`///`, `//!`) explaining its purpose
    and responsibility -- the "why" -- plus error conditions and edge cases.
    Applies to new code; existing code is not rewritten for this alone.
11. **Admit and stop when a URL is unreachable.** When a URL comes up -- an
    upstream issue, a release page, a spec -- **actually fetch it** before
    citing it. If the fetch fails for any reason, say so plainly and ask how
    to proceed. Never fabricate content, version numbers, changelog entries,
    API shapes or repository metadata from training data or inference. An
    unverified claim about an external source is worse than a visible
    blocker.
12. **This repository is a fork. Treat it as the canonical source.** See
    "Fork operating rules" below.
13. **Model selection when Fable is the session model.** When the session runs
    on Fable (Mythos-class), pick the best-suited model per delegated task
    rather than letting every subagent inherit the expensive session model:
    `haiku` for mechanical lookups, `sonnet` for routine search and coding,
    `opus`/Fable only for the hardest reasoning, review or judgment. Keep
    Fable for orchestration and final synthesis.
14. **Write in the Google developer documentation style**
    (<https://developers.google.com/style>): second person, active voice,
    present tense, sentence case headings, plain language, and the fewest
    words that stay accurate. This covers commit messages, pull request
    bodies, code comments and this file.

    **Concise is not terse.** A pull request body records why a change was
    made, what was measured, and what was deliberately not done; that
    reasoning is the artifact. Cut the padding around an argument, never the
    argument. A finding stated in one sentence instead of three is better; a
    finding omitted is not.

    Applies to text written from now on. Existing documents are not rewritten
    for style alone.

## Workflow for every change

```bash
# 1. Branch (never work on master)
git checkout -b fix/my-change

# 2. Change, then run the gates
cargo fmt --all -- --check
cargo +1.96.1 clippy --features=macro,wasi-exports --all-targets -- -D warnings
cargo +stable  clippy --features=macro,wasi-exports --all-targets -- -D warnings
cargo test --features=macro,wasi-exports

# 3. Commit and push
git add . && git commit -m "Describe the change"
git push -u origin fix/my-change

# 4. Open a pull request
```

## Session completion

**Work is not complete until `git push` succeeds.**

1. Run the quality gates if code changed.
2. Push to the remote. `git status` must show the branch up to date with
   origin.
3. Verify everything is committed and pushed.
4. Hand off: say what was done, what was measured, and what is left.

Never stop before pushing -- that strands the work locally. Never say "ready
to push when you are"; push. If the push fails, resolve it and retry.

## Quality gates

The gating jobs pin Rust **1.96.1**, the MSRV normalised across the Accent
crates; the integration jobs use `stable`. The commands CI runs:

```bash
cargo fmt --all -- --check
cargo clippy --features=macro,wasi-exports --all-targets -- -D warnings
cargo test --features=macro,wasi-exports
```

Clippy gates on **two** toolchains, the MSRV and current stable, and both must
pass. Pinning the lint gate to the MSRV alone left CI blind to every lint added
after it, so a contributor on stable once saw sixteen errors CI called clean.
Run it locally the way CI does:

```bash
cargo +1.96.1 clippy --features=macro,wasi-exports --all-targets -- -D warnings
cargo +stable  clippy --features=macro,wasi-exports --all-targets -- -D warnings
```

`wasi-exports` is in the feature set so the WebAssembly C ABI in
`crates/lib/src/wasi_exports.rs` is linted and tested on the host. The `wasi`
job is what exercises it on the target.

Clippy reads `rust-version`, so raising the MSRV can turn lints on. Going from
1.85 to 1.96 made `collapsible_if` suggest let chains, which need 1.88: 23
findings appeared with no code change.

`cargo test` does not need the `sass-spec` submodule; the crate keeps its own
test suite deliberately separate from the spec.

CI jobs in `.github/workflows/tests.yml`:

| Job | Gates? | What it does |
|---|---|---|
| `tests`, `fmt` | yes | the commands above, on the MSRV |
| `clippy` | yes | the command above, on **both** the MSRV and stable |
| `wasi` | yes | builds both `wasm32-wasip1` artifacts and runs them: the command module under wasmtime, the library module through a Node host that calls its C ABI |
| `bootstrap` | advisory | compiles Bootstrap 5.0.2 with both engines; fails only on a colour-value difference |
| `frameworks` | yes | compiles Bulma, Pico, Foundation and USWDS with both engines via `.github/scripts/frameworks.sh`; fails on a colour-value difference. It also compiles `.github/scripts/foundation-functions.scss`, which calls every Sass function Foundation documents, and fails on any difference in those results |
| `sass-spec` | advisory | runs the official spec suite and publishes the tallies |

`.gitignore` ignores `*.sh` and `*.s[ac]ss` repository-wide, with
exceptions for `.github/scripts/*.sh` and `.github/scripts/*.scss`.

## Testing conventions

Tests live in `crates/lib/tests/` and use the `test!` and `error!` macros: an
input string and the exact expected output or first error line.

**dart-sass is the reference implementation, currently 1.107.0.** When a
change alters existing expectations, verify each one against the real
dart-sass binary before updating it. Re-baselining a test to whatever the new
code prints turns a regression into a passing test; that is how wrong
behaviour gets frozen into the suite. Where output deliberately differs from
dart-sass, say so in a comment next to the test with the reason.

## Moving the reference to a new dart-sass release

1. **Read what the release changes.** Compare the tags with
   `gh api repos/sass/dart-sass/compare/<old>...<new>` and read the commits
   that touch `lib/`, not only the changelog. A release can leave the
   language alone: 1.105.2 fixed only the Node.js embedded host, which this
   project does not have.
2. **Pin sass-spec to its last commit before the release, not to its main.**
   sass-spec lands fixtures together with the dart-sass change they test,
   often before a release ships that change. A day after 1.105.2,
   `58295762b` changed 29 fixture files for an escape change no release
   carried yet. Pinning past it grades the compiler against behaviour the
   reference does not have.
3. **Measure at both pins.** Run the suite at the old pin and the new one,
   and diff the `Failure:` lines of the two runs, not only the tallies. CI's
   Linux runner reports two fewer passing fixtures than macOS.

   ```bash
   git submodule update --init sass-spec
   cargo build --release
   cd sass-spec && npm install --no-audit --no-fund
   npm run sass-spec -- --impl=dart-sass --command '../target/release/accent-sass' \
     --trim-errors --ignore-warning-diffs --ignore-error-diffs
   ```

   Then run `.github/scripts/frameworks.sh` with `SASS` set to the new
   release binary.
4. **Update every place that names the current reference.**
   `git grep -n <old version> -- ':!sass-spec'` finds them: the `bootstrap`
   and `frameworks` jobs, `README.md`, this file, the site under `docs/`, and
   `specs/docs/features/README.md`. Leave the mentions that record what was
   checked against an older release: released changelog sections, item
   documents and test comments.
5. **Record the move** in an item under `specs/docs/features/` and in the
   changelog's `[Unreleased]` section.

A fresh checkout of the `sass-spec` submodule shows about 239 HRX files as
modified. They are committed with CRLF line endings, which its
`.gitattributes` forbids. Their bytes match the committed blobs, so the
runner reads them unchanged; ignore them.

## Documentation

- The site mounts `CHANGELOG.md` at `/changelog`, so a relative link in it
  resolves against that path and breaks. Link to repository files with
  absolute `https://github.com/zoosky/accent-sass/blob/master/...` URLs.
- The sass-spec tally and the framework measurement are quoted in
  `README.md`, `docs/content/default.md` and
  `docs/content/03.reference/04.compatibility/default.md`. Update all three
  when you re-measure.

## Fork operating rules

This repository began as a fork of `connorskees/grass`, whose last release
was 0.13.4 in August 2024. **The owner decided on 2026-09-04 that no return
to upstream is planned.** Upstream is the origin of this code, not a
destination for it, and that changes how the rules below read: decisions are
judged on what is right for this project, never on what keeps a merge back
cheap.

- **This project is canonical.** Accent pins a revision of this repository,
  never upstream and never crates.io.
- **Judge changes on their own merit.** There is no rebase budget to protect,
  so a change that is right for the codebase is not weighed against how far
  it moves from upstream. The 2024 style edition reformatting is the first
  decision taken on this basis.
- **No rebase cadence.** There is no obligation to track upstream. If
  something useful lands there, cherry-pick it deliberately and say why in
  the commit.
- **No upstream contribution obligation.** The owner decided on 2026-09-02
  not to file the pull request against upstream issue #105; that decision
  stands and needs no revisiting.
- **Preserve history on any merge you do make.** Use merge commits rather
  than squash or rebase when merging a long-lived branch, so the parentage
  stays readable.
- **Accent pins a release, not master's head.** The pin names the commit a
  release tag names, so the pin and the published version are the same code;
  every pin since `v0.15.0` has been a tag. Bump it after a release that
  changes what Accent builds, once every CI job, `frameworks` included, is
  green on the tagged commit itself and not merely on the pull request head.
  A merge that changes nothing under `crates/` waits for the next release,
  and so does a release that changes no compiler code: `v0.16.1`, a packaging
  release, was never pinned. Accent records each bump in the comment above
  `accent-sass` in its `Cargo.toml`. `specs/epics/e043-accent-sass-fork.md`,
  closed on 2026-09-05, holds the history before that.
