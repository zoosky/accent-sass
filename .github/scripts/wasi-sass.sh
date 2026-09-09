#!/usr/bin/env bash
#
# Present the wasm32-wasip1 build to the sass-spec runner as if it were a
# native binary, so `npm run sass-spec -- --command <this script>` works.
#
# Not used by CI: a full run takes about twenty-three minutes against three for
# the native build, because every one of the 14,218 tests pays wasmtime's
# startup. It exists so the measurement in
# specs/docs/features/13-wasm-wasi.md can be reproduced and re-taken.
#
# Two things have to be translated, and both were learned the hard way:
#
#   * The runner passes an absolute --load-path. A guest has no such path, so
#     each one is preopened under a guest alias and the argument rewritten.
#   * The runner sets the working directory to the test's own directory and
#     passes the entry point relative to it. Preopening only that directory is
#     what a first attempt did, and every fixture starting `@use '../x'` then
#     failed with `Can't find stylesheet to import.` -- 44 of them, which read
#     as compiler differences and were not. The entry point is rewritten to a
#     path under the mapped spec root instead, so a relative import resolves
#     inside a preopen rather than escaping one.
#
# Anything this script cannot translate is a hard error. A silent fallback to
# the cwd-relative form would reproduce those 44 phantom failures, and they
# look exactly like a compiler that cannot find a file.
#
# Usage:
#   WASMTIME=/path/to/wasmtime \
#   WASM=target/wasm32-wasip1/release/accent-sass.wasm \
#   npm run sass-spec -- --impl=dart-sass --command ../.github/scripts/wasi-sass.sh \
#     --trim-errors --ignore-warning-diffs --ignore-error-diffs

set -uo pipefail

: "${WASMTIME:?set WASMTIME to the wasmtime binary}"
: "${WASM:?set WASM to the wasm32-wasip1 module}"

die() { echo "wasi-sass: $*" >&2; exit 78; }

args=()
preopens=(--dir=.)
roots=()
aliases=()
n=0

for arg in "$@"; do
  case "$arg" in
    --load-path=*)
      host=${arg#--load-path=}
      host=${host%/}
      # `--load-path` may be repeated, so each one gets an alias of its own
      # rather than sharing a name and colliding in the preopen table.
      guest="/loadpath$n"
      preopens+=("--dir=${host}::${guest}")
      args+=("--load-path=${guest}")
      roots+=("$host")
      aliases+=("$guest")
      n=$((n + 1))
      ;;
    *.scss|*.sass|*.css)
      entry=$arg
      rewritten=""
      for i in "${!roots[@]}"; do
        root=${roots[$i]}
        if [ "${PWD#"$root"/}" != "$PWD" ]; then
          rewritten="${aliases[$i]}/${PWD#"$root"/}/$entry"
          break
        fi
      done
      # The runner always passes --load-path before the entry point and always
      # runs from inside the spec tree. If either stops being true, say so:
      # the fallback would silently measure the wrong thing.
      [ -n "$rewritten" ] \
        || die "no --load-path covers $PWD; cannot place '$entry' inside a preopen"
      args+=("$rewritten")
      ;;
    *)
      args+=("$arg")
      ;;
  esac
done

exec "$WASMTIME" run "${preopens[@]}" "$WASM" "${args[@]}"
