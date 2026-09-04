#!/usr/bin/env bash
#
# Publish the three accent-sass crates to crates.io, in dependency order.
#
# The order is not a preference. Cargo resolves a `path` + `version`
# dependency against the registry when packaging, so a crate cannot be
# packaged until the one below it is live on the index:
#
#   1. accent_sass_compiler   (no workspace dependencies)
#   2. accent-sass-macro      (depends on accent_sass_compiler)
#   3. accent-sass            (depends on both)
#
# Usage:
#   .github/scripts/release.sh --dry-run      # verify only, publishes nothing
#   .github/scripts/release.sh                # publish, prompting once
#
#   MSRV=1.85.0   toolchain the gates run on (default: the rust-version field)
#   NO_TAG=1      skip creating the git tag
#
# See RELEASING.md for the surrounding checklist.

set -uo pipefail

DRY_RUN=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    -h|--help) sed -n '2,22p' "$0"; exit 0 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$ROOT"

# Publish order. Directory names differ from package names, so carry both.
CRATES=(
  "accent_sass_compiler:crates/compiler"
  "accent-sass-macro:crates/accent-sass-macro"
  "accent-sass:crates/lib"
)

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

if git rev-parse "v$version" >/dev/null 2>&1; then
  die "tag v$version already exists; bump the version first"
fi

# The changelog must carry this version, not leave it under Unreleased.
grep -q "^## \[$version\]" CHANGELOG.md \
  || die "CHANGELOG.md has no '## [$version]' heading; move Unreleased into a dated release"
echo "  changelog: has a [$version] section"

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

# --- Gates -------------------------------------------------------------------

step "Gates on $MSRV"

cargo "+$MSRV" fmt --all -- --check || die "cargo fmt reported differences"
echo "  fmt: clean"

# Clippy on both toolchains, matching CI. The MSRV alone is not enough: it
# cannot see lints added after 1.85, which is how sixteen findings once sat in
# the tree while every gate reported clean.
for tc in "$MSRV" stable; do
  cargo "+$tc" clippy --features=macro --all-targets -- -D warnings \
    || die "clippy failed on $tc"
  echo "  clippy ($tc): clean"
done

cargo "+$MSRV" test --features=macro || die "tests failed"
echo "  tests: pass"

# --- Package check -----------------------------------------------------------
#
# Only the first crate can be fully packaged before anything is published; the
# other two resolve their path+version dependencies against the index and fail
# until the crate below them is live. `--list` works for all three because it
# does not resolve the registry.

step "Package contents"

# A dry run is expected to happen on a working branch, so allow a dirty tree
# there. A real publish requires a clean one, enforced in the preconditions.
pkgflags=()
[ "$DRY_RUN" -eq 1 ] && pkgflags+=(--allow-dirty)

for entry in "${CRATES[@]}"; do
  pkg="${entry%%:*}"
  # Keep stderr: swallowing it turns "tree is dirty" into the useless and
  # misleading "produced nothing".
  listing=$(cargo package -p "$pkg" --list "${pkgflags[@]}" 2>&1) || {
    echo "$listing" >&2
    die "cargo package --list failed for $pkg"
  }
  files=$(echo "$listing" | grep -c .)
  echo "$listing" | grep -qx 'README.md' \
    || die "$pkg would publish without a README (check its readme field and include list)"
  printf '  %-24s %3s files, README present\n' "$pkg" "$files"
done

if [ "$DRY_RUN" -eq 1 ]; then
  step "Dry run complete"
  echo "  Everything that can be checked before publishing passed."
  echo "  Re-run without --dry-run to publish $version."
  exit 0
fi

# --- Publish -----------------------------------------------------------------

step "Publish $version to crates.io"
echo "  This is irreversible: crates.io versions cannot be deleted, only yanked."
printf '  Type the version to confirm: '
read -r reply
[ "$reply" = "$version" ] || die "confirmation did not match; nothing published"

for entry in "${CRATES[@]}"; do
  pkg="${entry%%:*}"
  step "Publishing $pkg $version"
  cargo publish -p "$pkg" || die "publish failed for $pkg. Crates before it in the order are already live; fix the cause and re-run -- already-published crates will fail with 'already uploaded', which is safe to skip."

  # The index needs a moment before the next crate can resolve this one.
  if [ "$pkg" != "accent-sass" ]; then
    echo "  waiting for the index to carry $pkg $version"
    for _ in $(seq 1 30); do
      sleep 10
      if cargo search "$pkg" 2>/dev/null | grep -q "^$pkg = \"$version\""; then
        echo "  index has it"
        break
      fi
    done
  fi
done

# --- Tag ---------------------------------------------------------------------

if [ "${NO_TAG:-0}" = "1" ]; then
  step "Skipping tag (NO_TAG=1)"
else
  step "Tagging v$version"
  git tag -a "v$version" -m "v$version"
  git push origin "v$version"
fi

step "Done"
cat <<EOF
  Published $version:
$(for e in "${CRATES[@]}"; do echo "    ${e%%:*}"; done)

  Next: bump Accent's pin. Accent depends on this project by git revision, not
  by version, so a crates.io release does not reach it. Per Accent's rule 21,
  bump only once the frameworks job is green on the revision being pinned.
EOF
