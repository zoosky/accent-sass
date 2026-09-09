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
#   2. `--load-path` resolves against a preopened directory, with no importer
#      bridge. The partial is reachable *only* through the load path, so the
#      check fails if load-path handling breaks under WASI,
#   3. an import that reaches outside every preopen fails with the compiler's
#      own error rather than a panic or a silent miss. That is the sandbox
#      doing its job, and it is the property the whole target is for.
#
# The expected output deliberately avoids anything under active development.
# A `calc()` here would turn this job red for a deliberate calc change and
# point at WASI, which would be the wrong place to look.
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
printf '$v: 2px;\n' > "$work/site/_partial.scss"
printf '@use "partial";\na {\n  b: partial.$v + 1px;\n}\n' > "$work/site/main.scss"
printf '$w: 4px;\n' > "$work/shared/_x.scss"
# Reachable only through --load-path: there is no `x` beside the entry point.
printf '@use "x";\na {\n  b: x.$w;\n}\n' > "$work/site/via-load-path.scss"
# Reachable only by leaving the entry point's directory.
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

# 1. One file and its neighbour, with arithmetic that has one answer.
got=$("$WASMTIME" run --dir=. "$WASM" site/main.scss 2>&1)
check "compiles a stylesheet" $'a {\n  b: 3px;\n}' "$got"

# 2. A partial that exists nowhere but the load path, so the check fails if
#    --load-path stops being honoured rather than passing on a fallback.
got=$("$WASMTIME" run --dir=. "$WASM" --load-path=shared site/via-load-path.scss 2>&1)
check "resolves @use through a load path" $'a {\n  b: 4px;\n}' "$got"

# 2b. The same file with the load path removed must fail, or check 2 proves
#     nothing: it would pass on any resolution that happened to find `x`.
got=$("$WASMTIME" run --dir=. "$WASM" site/via-load-path.scss 2>&1 | head -n 1)
check "needs that load path" "Error: Can't find stylesheet to import." "$got"

# 3. The same import with a preopen that does not cover the target. The
#    compiler must say so in its own words; a panic or an empty success here
#    would mean the sandbox is not being reported to the user.
got=$("$WASMTIME" run --dir=site "$WASM" site/escape.scss 2>&1 | head -n 1)
check "reports an import outside the sandbox" "Error: Can't find stylesheet to import." "$got"

exit "$fail"
