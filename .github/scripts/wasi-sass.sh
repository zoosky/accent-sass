#!/usr/bin/env bash
#
# Present the wasm32-wasip1 build to the sass-spec runner as if it were a
# native binary, so `npm run sass-spec -- --command <this script>` works.
#
# Not used by CI: a full run takes about twenty minutes against three for the
# native build, because every one of the 14,218 tests pays wasmtime's startup.
# It exists so the measurement in specs/docs/features/13-wasm-wasi.md can be
# reproduced and re-taken.
#
# The runner sets the working directory to each test's own directory and passes
# an absolute --load-path. A guest sees neither: the cwd needs a preopen, and an
# absolute host path is meaningless inside the sandbox. So the load path is
# preopened under a short guest alias and the argument rewritten to match. A
# deep guest path fails where a short one works, which is why the alias is
# `/loadpath` rather than the host path repeated.
#
# Usage:
#   WASMTIME=/path/to/wasmtime \
#   WASM=target/wasm32-wasip1/release/accent-sass.wasm \
#   npm run sass-spec -- --impl=dart-sass --command ../.github/scripts/wasi-sass.sh \
#     --trim-errors --ignore-warning-diffs --ignore-error-diffs

set -uo pipefail

: "${WASMTIME:?set WASMTIME to the wasmtime binary}"
: "${WASM:?set WASM to the wasm32-wasip1 module}"

args=()
preopens=(--dir=.)
root=""

for arg in "$@"; do
  case "$arg" in
    --load-path=*)
      root=${arg#--load-path=}
      preopens+=("--dir=${root}::/spec")
      args+=("--load-path=/spec")
      ;;
    *.scss|*.sass|*.css)
      # The entry point, relative to the test's own directory. Rewrite it to a
      # path under the mapped spec root, so a `@use "../shared"` resolves
      # inside that preopen instead of escaping the one covering the cwd.
      if [ -n "$root" ] && [ "${PWD#"$root"/}" != "$PWD" ]; then
        args+=("/spec/${PWD#"$root"/}/$arg")
      else
        args+=("$arg")
      fi
      ;;
    *)
      args+=("$arg")
      ;;
  esac
done

exec "$WASMTIME" run "${preopens[@]}" "$WASM" "${args[@]}"
