#!/usr/bin/env bash
#
# Publish accent-sass: the three crates to crates.io, in dependency order, and
# the browser package to npm as @zoosky/accent-sass.
#
# The crate order is not a preference. Cargo resolves a `path` + `version`
# dependency against the registry when packaging, so a crate cannot be
# packaged until the one below it is live on the index:
#
#   1. accent_sass_compiler   (no workspace dependencies)
#   2. accent-sass-macro      (depends on accent_sass_compiler)
#   3. accent-sass            (depends on both)
#
# The npm package is the WebAssembly build for the `web` target, at the same
# version. It is built and smoke-tested before anything is published, and
# published after the crates.
#
# Usage:
#   .github/scripts/release.sh --dry-run             # verify only, publishes nothing
#   .github/scripts/release.sh                       # publish crates and npm, prompting once
#   .github/scripts/release.sh --npm-only --dry-run  # verify the npm package for a tagged version
#   .github/scripts/release.sh --npm-only            # publish only the npm package for a tagged version
#
#   MSRV=1.96.1   toolchain the gates run on (default: the rust-version field)
#   NO_TAG=1      skip creating the git tag
#
# See RELEASING.md for the surrounding checklist.

set -uo pipefail

DRY_RUN=0
NPM_ONLY=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --npm-only) NPM_ONLY=1 ;;
    -h|--help) sed -n '2,28p' "$0"; exit 0 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$ROOT" || exit 1

# Publish order. Directory names differ from package names, so carry both.
CRATES=(
  "accent_sass_compiler:crates/compiler"
  "accent-sass-macro:crates/accent-sass-macro"
  "accent-sass:crates/lib"
)

# The npm package. `wasm-pack --scope` turns the crate name into the scoped
# name. It is built under target/ rather than in wasm-pack's default
# crates/lib/pkg, which is ignored, easy to leave stale, and inside a crate.
NPM_SCOPE="zoosky"
NPM_PACKAGE="@$NPM_SCOPE/accent-sass"
NPM_DIR="target/npm/pkg"

# The compiler sources an npm-only release must share with the tag, so the
# package is the code the crates were published from. The README may differ:
# fixing how it names the package is a reason to publish.
RELEASED_SOURCES="crates/compiler crates/lib/src crates/lib/Cargo.toml Cargo.lock"

die() { echo "error: $*" >&2; exit 1; }
step() { printf '\n=== %s\n' "$*"; }

# --- Preconditions -----------------------------------------------------------

step "Preconditions"

# Git state is a release-time concern, not a packaging one. A real publish
# refuses to proceed; --dry-run only warns, so the dry run is usable from the
# branch that is changing the release itself -- which is when you most want it.
gitstate() { if [ "$DRY_RUN" -eq 1 ]; then echo "  warning: $*"; else die "$*"; fi; }

[ -z "$(git status --porcelain)" ] || gitstate "working tree is dirty; commit or stash first"

branch=$(git rev-parse --abbrev-ref HEAD)
[ "$branch" = "master" ] || gitstate "on branch '$branch'; release from master"

git fetch -q origin master
[ "$(git rev-parse HEAD)" = "$(git rev-parse origin/master)" ] \
  || gitstate "HEAD is not origin/master; pull or push first"

# One version across all three crates, or the `=` pins are inconsistent.
version=""
for entry in "${CRATES[@]}"; do
  dir="${entry#*:}"
  v=$(awk '/^\[package\]/{p=1} p&&/^version[[:space:]]*=/{print;exit}' "$dir/Cargo.toml" | cut -d'"' -f2)
  [ -n "$v" ] || die "no version in $dir/Cargo.toml"
  if [ -z "$version" ]; then version="$v"
  elif [ "$v" != "$version" ]; then die "version mismatch: $dir is $v, expected $version"
  fi
done
echo "  version:  $version (consistent across all three crates)"

# The pins *between these crates* must name that same version. Match only lines
# that declare a sibling crate as a dependency: a bare `version = ` grep also
# picks up wasm-bindgen, quote and clap, which have versions of their own.
siblings="accent_sass_compiler|accent-sass-macro|accent-sass"
pins=$(grep -rhE "^($siblings) = \{" crates/*/Cargo.toml \
       | grep -oE 'version = "=?[0-9]+\.[0-9]+\.[0-9]+"' \
       | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | sort -u)
[ -n "$pins" ] || die "found no interdependency pins to check"
if [ "$(echo "$pins" | wc -l | tr -d ' ')" != "1" ] || [ "$pins" != "$version" ]; then
  die "interdependency pins disagree with the package version: found [$(echo "$pins" | tr '\n' ' ')], expected $version"
fi
echo "  pins:     $(grep -rhcE "^($siblings) = \{" crates/*/Cargo.toml | paste -sd+ - | bc) sibling pins, all $version"

if [ "$NPM_ONLY" -eq 1 ]; then
  # --npm-only publishes a version whose crates are already out, such as one
  # released before npm joined the process, or one whose npm step failed.
  git rev-parse "v$version" >/dev/null 2>&1 \
    || die "--npm-only publishes a released version, and tag v$version does not exist"
  # shellcheck disable=SC2086 # RELEASED_SOURCES is a list of paths
  git diff --quiet "v$version" HEAD -- $RELEASED_SOURCES \
    || die "the compiler sources changed since v$version, so this build would not be the released code"
  echo "  tag:      v$version exists, and the compiler sources match it"
elif git rev-parse "v$version" >/dev/null 2>&1; then
  die "tag v$version already exists; bump the version first, or pass --npm-only to publish only the npm package"
fi

# The changelog must carry this version, not leave it under Unreleased.
grep -q "^## \[$version\]" CHANGELOG.md \
  || die "CHANGELOG.md has no '## [$version]' heading; move Unreleased into a dated release"
echo "  changelog: has a [$version] section"

if [ "$NPM_ONLY" -eq 0 ]; then
  MSRV=${MSRV:-$(awk -F'"' '/^rust-version/{print $2; exit}' crates/lib/Cargo.toml)}
  echo "  msrv:     $MSRV"
  rustup toolchain list 2>/dev/null | grep -q "^$MSRV" \
    || die "toolchain $MSRV is not installed (rustup toolchain install $MSRV)"

  # Check the components up front. Without this, a missing rustfmt surfaces below
  # as "cargo fmt reported differences", which names the wrong cause.
  for tc in "$MSRV" stable; do
    rustup toolchain list 2>/dev/null | grep -q "^$tc" \
      || die "toolchain $tc is not installed (rustup toolchain install $tc)"
    for component in rustfmt clippy; do
      rustup component list --toolchain "$tc" 2>/dev/null | grep -qE "^$component.*\(installed\)" \
        || die "$component is not installed for $tc (rustup component add --toolchain $tc $component)"
    done
  done
  echo "  components: rustfmt and clippy present on $MSRV and stable"
fi

# The npm tooling, checked before any gate runs, so a missing tool cannot stop
# the release after the crates are already live.
for tool in wasm-pack node npm; do
  command -v "$tool" >/dev/null 2>&1 || die "$tool is not installed; the npm package needs it"
done
rustup target list --installed 2>/dev/null | grep -qx wasm32-unknown-unknown \
  || die "the wasm32-unknown-unknown target is not installed (rustup target add wasm32-unknown-unknown)"
echo "  npm tools: wasm-pack, node, npm and the wasm32-unknown-unknown target present"

npm_user=$(npm whoami 2>/dev/null || true)
if [ -n "$npm_user" ]; then
  echo "  npm login: $npm_user"
elif [ "$DRY_RUN" -eq 1 ]; then
  echo "  warning: not logged in to npm; a real run needs 'npm login' first"
else
  die "not logged in to npm; run 'npm login' first"
fi

# npm refuses a version it already has, and finding that out after the crates
# are published would leave a release half done.
if [ "$(npm view "$NPM_PACKAGE@$version" version 2>/dev/null || true)" = "$version" ]; then
  die "npm already has $NPM_PACKAGE $version"
fi
echo "  npm:      $NPM_PACKAGE $version is not published yet"

# --- Gates -------------------------------------------------------------------

if [ "$NPM_ONLY" -eq 1 ]; then
  step "Gates"
  echo "  skipped: the crates were released from these compiler sources"
else
  step "Gates on $MSRV"

  cargo "+$MSRV" fmt --all -- --check || die "cargo fmt reported differences"
  echo "  fmt: clean"

  # Clippy on both toolchains, matching CI. The MSRV alone is not enough: it
  # cannot see lints added after it, which is how sixteen findings once sat in
  # the tree while every gate reported clean.
  # The feature set matches CI. `wasi-exports` belongs in it because the
  # WebAssembly C ABI is ordinary Rust on the host: left out, a leak or a
  # mismatched free in it is invisible to every gate here.
  for tc in "$MSRV" stable; do
    cargo "+$tc" clippy --features=macro,wasi-exports --all-targets -- -D warnings \
      || die "clippy failed on $tc"
    echo "  clippy ($tc): clean"
  done

  cargo "+$MSRV" test --features=macro,wasi-exports || die "tests failed"
  echo "  tests: pass"

  # A release is the moment to look at the advisory database. `cargo audit`
  # exits non-zero for a vulnerability and zero for a warning-level advisory,
  # such as a crate that is only unmaintained -- which is the behaviour wanted
  # here, since an unmaintained dev-dependency should be visible without
  # blocking a release.
  if command -v cargo-audit >/dev/null 2>&1; then
    cargo audit || die "cargo audit reported a vulnerability"
    echo "  audit: no vulnerabilities"
  else
    echo "  warning: cargo-audit is not installed, so advisories went unchecked"
    echo "           (cargo install cargo-audit)"
  fi
fi

# --- Package check -----------------------------------------------------------
#
# Only the first crate can be fully packaged before anything is published; the
# other two resolve their path+version dependencies against the index and fail
# until the crate below them is live. `--list` works for all three because it
# does not resolve the registry.

if [ "$NPM_ONLY" -eq 0 ]; then
  step "Package contents"

  for entry in "${CRATES[@]}"; do
    pkg="${entry%%:*}"
    # A dry run happens on a working branch, so it tolerates a dirty tree; a real
    # publish requires a clean one, enforced in the preconditions.
    #
    # Written as two explicit calls rather than an array of flags. bash 3.2 --
    # which is what macOS ships, and where this is most likely to be run -- treats
    # "${arr[@]}" on an EMPTY array as an unbound variable under `set -u`. The
    # array is empty exactly when this is a real publish, so that spelling failed
    # only on the path a dry run never exercises.
    #
    # Keep stderr: swallowing it turns "tree is dirty" into a misleading
    # "produced nothing".
    if [ "$DRY_RUN" -eq 1 ]; then
      listing=$(cargo package -p "$pkg" --list --allow-dirty 2>&1)
    else
      listing=$(cargo package -p "$pkg" --list 2>&1)
    fi
    [ $? -eq 0 ] || {
      echo "$listing" >&2
      die "cargo package --list failed for $pkg"
    }
    files=$(echo "$listing" | grep -c .)
    echo "$listing" | grep -qx 'README.md' \
      || die "$pkg would publish without a README (check its readme field and include list)"

    # A file git ignores has no business in a published crate: it is local build
    # output or a scratch file. `--allow-dirty` above hides it, so a dry run would
    # pass and then ship it. The real publish refuses it instead, naming the files
    # as "not yet committed", which is how `crates/lib/pkg/` from a default
    # `wasm-pack build` stopped the 0.16.0 release. Cargo writes the first three
    # names itself, and copies the workspace Cargo.lock in.
    dir="${entry#*:}"
    ignored=$(echo "$listing" \
      | grep -vxE '\.cargo_vcs_info\.json|Cargo\.toml\.orig|Cargo\.lock' \
      | while read -r file; do
          git check-ignore -q "$dir/$file" && echo "    $dir/$file"
        done)
    [ -z "$ignored" ] || {
      echo "$ignored" >&2
      die "$pkg would publish files git ignores (anchor its include patterns, or remove them)"
    }

    printf '  %-24s %3s files, README present, nothing ignored\n' "$pkg" "$files"
  done

  # `--list` only reads a file list; it never builds the packaged crate. A real
  # `cargo publish` does, so a dry run that skips it can still be followed by a
  # publish that fails during verification. The first crate can be verified now
  # because it has no workspace dependencies; the other two cannot until the one
  # below them is on the index, which is the same ordering constraint as above.
  if [ "$DRY_RUN" -eq 1 ]; then
    step "Verifying the first crate builds from its package"
    first="${CRATES[0]%%:*}"
    if cargo publish -p "$first" --dry-run --allow-dirty >/dev/null 2>&1; then
      echo "  $first: packages and builds"
    else
      echo "  $first: FAILED to build from its package" >&2
      cargo publish -p "$first" --dry-run --allow-dirty 2>&1 | tail -20 >&2
      die "the first crate does not build from its packaged form"
    fi
    echo "  (the other two cannot be verified until the crate below them is live)"
  fi
fi

# --- npm package -------------------------------------------------------------
#
# Built and checked on every run, before anything is published, so a package
# that does not build, does not load, or would ship the wrong files stops the
# release while nothing is live yet. The build is the one CI's Pages job ships,
# plus the scope.

step "npm package"

rm -rf "$NPM_DIR"
mkdir -p "$(dirname "$NPM_DIR")"
build_log="$(dirname "$NPM_DIR")/build.log"
# --out-dir is relative to the crate, crates/lib.
wasm-pack build crates/lib --release --target web --scope "$NPM_SCOPE" --out-name index \
  --out-dir "../../$NPM_DIR" -- --no-default-features --features wasm-exports,random \
  > "$build_log" 2>&1 || {
    tail -30 "$build_log" >&2
    die "wasm-pack build failed; the full log is $build_log"
  }
echo "  build: wasm-pack, web target, into $NPM_DIR"

manifest=$(node -e '
  const p = JSON.parse(require("fs").readFileSync(process.argv[1], "utf8"));
  console.log(p.name + " " + p.version);
' "$NPM_DIR/package.json") || die "could not read $NPM_DIR/package.json"
[ "$manifest" = "$NPM_PACKAGE $version" ] \
  || die "package.json names '$manifest', expected '$NPM_PACKAGE $version'"
echo "  package.json: $manifest"

# The two scripts CI runs against the Pages build: the module loads, and the
# JavaScript API compiles, resolves imports, reports errors and logs.
for smoke in wasm-smoke.mjs wasm-api-smoke.mjs; do
  out=$(node ".github/scripts/$smoke" "$NPM_DIR" 2>&1) || {
    echo "$out" >&2
    die "$smoke failed against $NPM_DIR"
  }
done
echo "  smoke: wasm-smoke.mjs and wasm-api-smoke.mjs pass"

# What `npm publish` would upload. wasm-pack's package.json lists the module
# files; npm adds the README and LICENSE.
pack=$(cd "$NPM_DIR" && npm pack --dry-run --json 2>/dev/null) || die "npm pack --dry-run failed in $NPM_DIR"
summary=$(node -e '
  const [p] = JSON.parse(process.argv[1]);
  const have = new Set(p.files.map((f) => f.path));
  const missing = ["README.md", "LICENSE", "package.json", "index.js", "index.d.ts", "index_bg.wasm"]
    .filter((f) => !have.has(f));
  if (missing.length) { console.error("missing: " + missing.join(", ")); process.exit(1); }
  console.log(p.files.length + " files, " + (p.size / 1024).toFixed(0) + " KiB packed");
' "$pack") || die "the npm package would publish without a required file"
echo "  contents: $summary"

if [ "$DRY_RUN" -eq 1 ]; then
  step "Dry run complete"
  echo "  Everything that can be checked before publishing passed."
  if [ "$NPM_ONLY" -eq 1 ]; then
    echo "  Re-run without --dry-run to publish $NPM_PACKAGE $version to npm."
  else
    echo "  Re-run without --dry-run to publish $version to crates.io and npm."
  fi
  exit 0
fi

# --- Publish -----------------------------------------------------------------

if [ "$NPM_ONLY" -eq 1 ]; then
  step "Publish $NPM_PACKAGE $version to npm"
  echo "  npm allows unpublishing only for 72 hours, and never reuses a version."
else
  step "Publish $version to crates.io and npm"
  echo "  This is irreversible: crates.io versions cannot be deleted, only yanked,"
  echo "  and npm never reuses a version."
fi
printf '  Type the version to confirm: '
read -r reply
[ "$reply" = "$version" ] || die "confirmation did not match; nothing published"

if [ "$NPM_ONLY" -eq 0 ]; then
  for entry in "${CRATES[@]}"; do
    pkg="${entry%%:*}"
    step "Publishing $pkg $version"
    cargo publish -p "$pkg" || die "publish failed for $pkg. Crates before it in the order are already live; fix the cause and re-run -- already-published crates will fail with 'already uploaded', which is safe to skip."

    # The index needs a moment before the next crate can resolve this one.
    if [ "$pkg" != "accent-sass" ]; then
      echo "  waiting for the index to carry $pkg $version"
      for _ in $(seq 1 30); do
        sleep 10
        if cargo search "$pkg" --limit 50 2>/dev/null | grep -q "^$pkg = \"$version\""; then
          echo "  index has it"
          break
        fi
      done
    fi
  done
fi

step "Publishing $NPM_PACKAGE $version to npm"
# A scoped package is private unless published with --access public, and a
# first publish has nothing earlier to inherit that from.
(cd "$NPM_DIR" && npm publish --access public) || {
  if [ "$NPM_ONLY" -eq 1 ]; then
    die "npm publish failed; fix the cause and re-run with --npm-only"
  else
    die "npm publish failed. The crates are live and the tag is not created; fix the cause, run release.sh --npm-only, then tag v$version"
  fi
}

# --- Tag ---------------------------------------------------------------------

if [ "$NPM_ONLY" -eq 1 ]; then
  step "Skipping tag (v$version already exists)"
elif [ "${NO_TAG:-0}" = "1" ]; then
  step "Skipping tag (NO_TAG=1)"
else
  step "Tagging v$version"
  git tag -a "v$version" -m "v$version"
  git push origin "v$version"
fi

step "Done"
if [ "$NPM_ONLY" -eq 1 ]; then
  echo "  Published $NPM_PACKAGE $version to npm."
else
  cat <<EOF
  Published $version:
$(for e in "${CRATES[@]}"; do echo "    ${e%%:*}"; done)
    $NPM_PACKAGE (npm)

  Next: bump Accent's pin. Accent depends on this project by git revision, not
  by version, so a crates.io release does not reach it. Per Accent's rule 21,
  bump only once the frameworks job is green on the revision being pinned.
EOF
fi
