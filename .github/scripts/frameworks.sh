#!/usr/bin/env bash
#
# Compile a corpus of real-world CSS frameworks with accent-sass and with dart-sass
# and compare the results.
#
# The frameworks are the ones that exercise the modern colour APIs this fork
# implements, so a regression in colour handling shows up here even when the
# unit tests stay green.
#
# A difference in a *colour value* fails the run. Differences in rule grouping
# and ordering are reported but tolerated: they are accent-sass's own
# long-standing behaviour, and canonicalising both sides (see NORMALISE below)
# shows they do not change what the stylesheet means.
#
# It also compiles `foundation-functions.scss`, beside this script, which calls
# every public Sass function Foundation documents directly. Each line of its
# output is one function's result, so any difference there fails the run.
#
# Usage: .github/scripts/frameworks.sh
#   ACCENT_SASS  path to the accent-sass binary (default ./target/release/accent-sass)
#   SASS         path to the dart-sass binary   (default ./dart-sass/sass)
#   NORMALISE    path to lightningcss, optional; when set, also reports the
#                diff after canonicalising both outputs through it
#   WORK       working directory               (default ./framework-corpus)

set -uo pipefail

ACCENT_SASS=${ACCENT_SASS:-./target/release/accent-sass}
SASS=${SASS:-./dart-sass/sass}
WORK=${WORK:-framework-corpus}
NORMALISE=${NORMALISE:-}

# name|npm spec|entry point relative to node_modules|load path relative to node_modules ("-" for none)
FRAMEWORKS=(
  "bulma|bulma@1.0.4|bulma/bulma.scss|-"
  "pico|@picocss/pico@2.1.1|@picocss/pico/scss/pico.scss|-"
  "foundation|foundation-sites@6.9.0|foundation-sites/assets/foundation.scss|foundation-sites/scss"
  "uswds|@uswds/uswds@3.13.0|USWDS_ENTRY|@uswds/uswds/packages"
)

# The corpus is built inside $WORK, so resolve every tool path before moving
# there -- CI passes them relative to the repository root.
abspath() {
  case "$1" in
    /*) printf '%s' "$1" ;;
    *)  printf '%s/%s' "$PWD" "$1" ;;
  esac
}
ACCENT_SASS=$(abspath "$ACCENT_SASS")
SASS=$(abspath "$SASS")
FOUNDATION_PROBE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/foundation-functions.scss
if [ -n "$NORMALISE" ]; then
  NORMALISE=$(abspath "$NORMALISE")
fi

mkdir -p "$WORK"
cd "$WORK" || exit 1
[ -f package.json ] || npm init -y >/dev/null 2>&1

specs=()
for entry in "${FRAMEWORKS[@]}"; do
  IFS='|' read -r _ spec _ _ <<<"$entry"
  specs+=("$spec")
done
echo "Installing: ${specs[*]}"
npm install --no-audit --no-fund --silent "${specs[@]}" >/dev/null 2>&1 || {
  echo "::error::failed to install the framework corpus"
  exit 1
}

# USWDS is consumed through a `@use`, not by compiling a file inside the package.
printf '@use "uswds";\n' > uswds-entry.scss

status=0
summary=""

for entry in "${FRAMEWORKS[@]}"; do
  IFS='|' read -r name spec input loadpath <<<"$entry"

  if [ "$input" = "USWDS_ENTRY" ]; then
    input="uswds-entry.scss"
  else
    input="node_modules/$input"
  fi

  accent_args=()
  sass_args=()
  if [ "$loadpath" != "-" ]; then
    accent_args+=(-I "node_modules/$loadpath")
    sass_args+=("--load-path=node_modules/$loadpath")
  fi

  if ! "$ACCENT_SASS" "${accent_args[@]}" "$input" > "$name-accent-sass.css" 2>"$name-accent-sass.err"; then
    echo "::error::accent-sass failed to compile $name"
    head -n 20 "$name-accent-sass.err"
    status=1
    continue
  fi
  "$SASS" --quiet "${sass_args[@]}" "$input" > "$name-sass.css" 2>/dev/null

  differing=$(diff "$name-accent-sass.css" "$name-sass.css" | grep -c '^[<>]')
  colours=$(diff "$name-accent-sass.css" "$name-sass.css" \
    | grep '^[<>]' | grep -cE 'rgb\(|hsl\(|#[0-9a-fA-F]{3,8}\b')

  # Canonicalising both sides collapses formatting-only differences, so what
  # survives is a difference in meaning. lightningcss rejects some legacy CSS
  # outright (Foundation ships IE media-query hacks), so a failed or empty
  # canonicalisation is reported as such -- never as agreement, which is what
  # diffing two empty files would otherwise claim.
  normalised="n/a"
  if [ -n "$NORMALISE" ]; then
    if "$NORMALISE" --minify "$name-accent-sass.css" 2>/dev/null | sed 's/}/}\n/g' > "$name-accent-sass.norm" \
      && "$NORMALISE" --minify "$name-sass.css" 2>/dev/null | sed 's/}/}\n/g' > "$name-sass.norm" \
      && [ -s "$name-accent-sass.norm" ] && [ -s "$name-sass.norm" ]; then
      normalised=$(diff "$name-accent-sass.norm" "$name-sass.norm" | grep -c '^[<>]')
    else
      normalised="not parseable"
    fi
  fi

  echo "$name: $differing differing lines ($colours colour-bearing, $normalised after canonicalisation)"
  summary="$summary| \`$name\` | $differing | $colours | $normalised |"$'\n'

  # USWDS loads the same module with several different `@use ... with (...)`
  # configurations. dart-sass instantiates it once per configuration and
  # accent-sass caches one instance per URL, so whole blocks legitimately
  # differ; the
  # colour check would report those as colour differences. Reported, not gated.
  if [ "$name" != "uswds" ] && [ "$colours" -ne 0 ]; then
    echo "::error::$name: colour values diverge from dart-sass"
    diff "$name-accent-sass.css" "$name-sass.css" | grep '^[<>]' \
      | grep -E 'rgb\(|hsl\(|#[0-9a-fA-F]{3,8}\b' | head -n 20
    status=1
  fi
done

# Foundation's documented functions, called one by one. Unlike a whole
# framework, this output has no rule grouping or ordering to tolerate: each
# line is a single function's result, so every differing line is a real
# divergence and fails the run.
probe_args=(-I node_modules/foundation-sites/scss)
if ! "$ACCENT_SASS" "${probe_args[@]}" "$FOUNDATION_PROBE" > foundation-functions-accent-sass.css 2>foundation-functions-accent-sass.err; then
  echo "::error::accent-sass failed to compile the Foundation function probe"
  head -n 20 foundation-functions-accent-sass.err
  status=1
elif ! "$SASS" --quiet --load-path=node_modules/foundation-sites/scss "$FOUNDATION_PROBE" > foundation-functions-sass.css 2>foundation-functions-sass.err; then
  # A dart-sass or Foundation bump that drops a probed function lands here,
  # so show why rather than only that it failed.
  echo "::error::dart-sass failed to compile the Foundation function probe"
  head -n 20 foundation-functions-sass.err
  status=1
else
  results=$(grep -c ': ' foundation-functions-sass.css)
  probe_differing=$(diff foundation-functions-accent-sass.css foundation-functions-sass.css | grep -c '^[<>]')
  echo "foundation functions: $probe_differing differing lines across $results results"
  summary="$summary| \`foundation\` functions | $probe_differing | - | - |"$'\n'
  if [ "$probe_differing" -ne 0 ]; then
    echo "::error::Foundation function results diverge from dart-sass (< accent-sass, > dart-sass)"
    diff foundation-functions-accent-sass.css foundation-functions-sass.css | grep '^[<>]' | head -n 20
    status=1
  fi
fi

if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  {
    echo "### Framework corpus vs dart-sass"
    echo
    echo "| framework | differing lines | colour-bearing | canonicalised |"
    echo "|---|---:|---:|---:|"
    printf '%s' "$summary"
  } >> "$GITHUB_STEP_SUMMARY"
fi

exit $status
