#!/usr/bin/env bash
#
# Compile stylesheets with the wasm32-wasip1 build, under a real runtime.
#
# The point is the runtime. A WASI build that only compiles proves nothing --
# every filesystem call in it succeeds at compile time and can still fail at
# run time, and the browser package shipped an empty module for two releases
# for want of exactly this check.
#
# Three things are checked, and the third is the interesting one:
#
#   1. a single file compiles, and the output matches the native build's,
#   2. `@use` resolves against a preopened directory, with no importer bridge,
#   3. an import that reaches outside every preopen fails with the compiler's
#      own error rather than a panic or a silent miss. That is the sandbox
#      doing its job, and it is the property the whole target is for.
#
# Usage: .github/scripts/wasi-smoke.sh
#   WASMTIME  path to the wasmtime binary  (default: wasmtime)
#   WASM      path to the built module
#             (default: target/wasm32-wasip1/release/accent-sass.wasm)

set -uo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
WASMTIME=${WASMTIME:-wasmtime}
WASM=${WASM:-$ROOT/target/wasm32-wasip1/release/accent-sass.wasm}

command -v "$WASMTIME" >/dev/null 2>&1 || [ -x "$WASMTIME" ] \
  || { echo "error: no wasmtime at '$WASMTIME'" >&2; exit 1; }
[ -f "$WASM" ] || { echo "error: no module at '$WASM'" >&2; exit 1; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

mkdir -p "$work/site" "$work/shared"
printf '$v: 2.5rem;\n' > "$work/site/_partial.scss"
printf '@use "partial";\na {\n  b: calc(#{partial.$v} - 0.5rem);\n}\n' > "$work/site/main.scss"
printf '$w: 2px;\n' > "$work/shared/_x.scss"
printf '@use "../shared/x";\na {\n  b: x.$w;\n}\n' > "$work/site/escape.scss"

fail=0
check() { # name expected actual
  if [ "$2" = "$3" ]; then
    echo "  ok    $1"
  else
    echo "  FAIL  $1"
    echo "        expected: $(printf '%s' "$2" | head -c 200)"
    echo "        actual:   $(printf '%s' "$3" | head -c 200)"
    fail=1
  fi
}

cd "$work" || exit 1

# 1. One file, and the same answer the native build gives.
want=$'a {\n  b: calc(2.5rem - 0.5rem);\n}'
got=$("$WASMTIME" run --dir=. "$WASM" site/main.scss 2>&1)
check "compiles a stylesheet" "$want" "$got"

# 2. `@use` through a preopen, from a directory that is not the entry point's.
got=$("$WASMTIME" run --dir=. "$WASM" --load-path=shared site/escape.scss 2>&1)
check "resolves @use through a preopen" $'a {\n  b: 2px;\n}' "$got"

# 3. The same import with a preopen that does not cover the target. The
#    compiler must say so in its own words; a panic or an empty success here
#    would mean the sandbox is not being reported to the user.
got=$("$WASMTIME" run --dir=site "$WASM" site/escape.scss 2>&1 | head -n 1)
check "reports an import outside the sandbox" "Error: Can't find stylesheet to import." "$got"

exit "$fail"
