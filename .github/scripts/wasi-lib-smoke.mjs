// Exercise the wasm32-wasip1 *reactor* build through its C ABI.
//
// `wasi-smoke.sh` runs the command module: the host starts it like a process
// and reads stdout. This one instantiates the library build instead, calls
// into it the way a plugin host would, and compiles more than once in the same
// instance -- which is the reason the reactor shape exists at all.
//
// The checks are the ABI's contract, not the compiler's: that a string
// crosses the boundary and comes back as CSS, that `@use` resolves through a
// preopened directory with no importer bridge, that a read outside every
// preopen is reported as the compiler's own error rather than a trap, and
// that bad input is a status rather than a panic. Anything about what CSS the
// compiler produces belongs in the Rust test suite.
//
// Node's WASI is used rather than wasmtime because the CLI cannot write bytes
// into the guest's memory, and every call here needs to.
//
// Usage: node .github/scripts/wasi-lib-smoke.mjs
//   WASM  path to the module
//         (default: target/wasm32-wasip1/small/accent_sass.wasm)

import { readFile, mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { WASI } from "node:wasi";

const root = path.resolve(import.meta.dirname, "../..");
const wasmPath =
  process.env.WASM ?? path.join(root, "target/wasm32-wasip1/small/accent_sass.wasm");

const work = await mkdtemp(path.join(tmpdir(), "accent-sass-wasi-"));
await mkdir(path.join(work, "site"));
await mkdir(path.join(work, "shared"));
await writeFile(path.join(work, "site/_partial.scss"), "$v: 2px;\n");
await writeFile(
  path.join(work, "site/main.scss"),
  '@use "partial";\na {\n  b: partial.$v + 1px;\n}\n',
);
await writeFile(path.join(work, "shared/_x.scss"), "$w: 4px;\n");
// Reachable only by leaving the `/site` preopen.
await writeFile(
  path.join(work, "site/escape.scss"),
  '@use "../shared/x";\na {\n  b: x.$w;\n}\n',
);

const wasi = new WASI({
  version: "preview1",
  args: [],
  env: {},
  // `/work` sees the whole tree; `/site` sees one directory of it, so a
  // stylesheet opened through `/site` cannot reach `shared`.
  preopens: { "/work": work, "/site": path.join(work, "site") },
});

const module = await WebAssembly.compile(await readFile(wasmPath));
const instance = await WebAssembly.instantiate(module, wasi.getImportObject());
wasi.initialize(instance);

const {
  memory,
  accent_sass_alloc,
  accent_sass_dealloc,
  accent_sass_compile_string,
  accent_sass_compile_path,
  accent_sass_result_free,
} = instance.exports;

const encoder = new TextEncoder();
const decoder = new TextDecoder();

// The buffer is replaced whenever linear memory grows, so it is read fresh on
// every access rather than captured once.
const bytes = () => new Uint8Array(memory.buffer);
const words = () => new DataView(memory.buffer);

/** Copy `input` into the guest and return the borrowed range. */
function put(input) {
  const encoded = input instanceof Uint8Array ? input : encoder.encode(input);
  const ptr = accent_sass_alloc(encoded.length);
  bytes().set(encoded, ptr);
  return { ptr, len: encoded.length };
}

/** Read a CompileResult -- status, pointer, length -- and free it. */
function take(resultPtr) {
  const view = words();
  const status = view.getUint32(resultPtr, true);
  const ptr = view.getUint32(resultPtr + 4, true);
  const len = view.getUint32(resultPtr + 8, true);
  const body = len === 0 ? "" : decoder.decode(bytes().slice(ptr, ptr + len));
  accent_sass_result_free(resultPtr);
  return { status, body };
}

/** Call one of the compile exports with a string argument. */
function call(fn, input) {
  const { ptr, len } = put(input);
  const result = take(fn(ptr, len));
  accent_sass_dealloc(ptr, len);
  return result;
}

let failed = 0;
function check(name, expected, actual) {
  if (expected === actual) {
    console.log(`  ok    ${name}`);
  } else {
    console.log(`  FAIL  ${name}`);
    console.log(`        expected: ${JSON.stringify(expected).slice(0, 200)}`);
    console.log(`        actual:   ${JSON.stringify(actual).slice(0, 200)}`);
    failed = 1;
  }
}

// 1. A stylesheet in memory, with arithmetic that has one answer.
let got = call(accent_sass_compile_string, "a {\n  b: 1px + 2px;\n}\n");
check("compiles a string", "0|a {\n  b: 3px;\n}\n", `${got.status}|${got.body}`);

// 2. The same instance again. A reactor that only works once is no use to the
//    plugin host this build is for.
got = call(accent_sass_compile_string, "a {\n  b: 1px + 2px;\n}\n");
check("compiles a second time in the same instance", "0|a {\n  b: 3px;\n}\n", `${got.status}|${got.body}`);

// 3. A file and its partial, read through a preopened directory. No importer
//    callback, no filesystem written for the host: StdFs is the whole bridge.
got = call(accent_sass_compile_path, "/work/site/main.scss");
check("resolves @use through a preopen", "0|a {\n  b: 3px;\n}\n", `${got.status}|${got.body}`);

// 4. The same tree seen through a narrower preopen. The import leaves it, and
//    the compiler must say so in its own words rather than trap.
got = call(accent_sass_compile_path, "/site/escape.scss");
check(
  "reports an import outside the sandbox",
  "1|Error: Can't find stylesheet to import.",
  `${got.status}|${got.body.split("\n")[0]}`,
);

// 5. A path no preopen covers at all.
got = call(accent_sass_compile_path, "/nowhere/main.scss");
check("reports an unreachable path", 1, got.status);

// 6. A compile error is a status, not a trap, and carries the formatted block.
got = call(accent_sass_compile_string, "a { b: 1px + ; }");
check("reports a compile error", "1|true", `${got.status}|${got.body.startsWith("Error: ")}`);

// 7. Bytes that are not UTF-8 are rejected without reaching the parser.
got = call(accent_sass_compile_string, new Uint8Array([0x61, 0xff, 0x7b, 0x7d]));
check("rejects invalid UTF-8", 2, got.status);

await rm(work, { recursive: true, force: true });
process.exit(failed);
