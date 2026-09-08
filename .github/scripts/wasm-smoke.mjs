// Compile one stylesheet with the wasm-pack package in crates/lib/pkg.
//
// The build job it belongs to was green for two releases while shipping a
// module with no compiler in it: `wasm-exports` is not a default feature, so
// wasm-bindgen exported nothing and every symbol was dead code. A size check
// would have caught that, but only running the thing proves the binding works,
// so this compiles a stylesheet and compares the output.
//
// Usage: node .github/scripts/wasm-smoke.mjs [package directory]
//   The default package directory is crates/lib/pkg, relative to the
//   repository root.

import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const pkg = resolve(root, process.argv[2] ?? "crates/lib/pkg");

const { initSync, from_string } = await import(
  pathToFileURL(join(pkg, "index.js")).href
);
initSync(readFileSync(join(pkg, "index_bg.wasm")));

const input = "$c: 2.5rem;\na {\n  b: calc(#{$c} - 0.5rem);\n}\n";
const want = "a {\n  b: calc(2.5rem - 0.5rem);\n}\n";
const got = from_string(input);

if (got !== want) {
  console.error(`from_string returned ${JSON.stringify(got)}`);
  console.error(`                want ${JSON.stringify(want)}`);
  process.exit(1);
}

console.log(`from_string compiles: ok (${readFileSync(join(pkg, "index_bg.wasm")).length} bytes of wasm)`);
