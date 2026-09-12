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
// preopen is reported as the compiler's own error rather than a trap, that an
// options handle survives the boundary and changes the output, and that bad
// input is a status rather than a panic. Anything about what CSS the compiler
// produces belongs in the Rust test suite.
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
  accent_sass_options_new,
  accent_sass_options_free,
  accent_sass_options_set_style,
  accent_sass_options_set_syntax,
  accent_sass_options_add_load_path,
  accent_sass_options_set_quiet,
  accent_sass_compile_string_with_options,
  accent_sass_compile_path_with_options,
} = instance.exports;

// The ABI's status words and enum values, as the module documents them.
const OK = 0;
const REJECTED = 1;
const STYLE_COMPRESSED = 1;
const SYNTAX_INDENTED = 1;

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

/** Call one of the `_with_options` exports with a string argument. */
function callWith(fn, input, options) {
  const { ptr, len } = put(input);
  const result = take(fn(ptr, len, options));
  accent_sass_dealloc(ptr, len);
  return result;
}

/** Append a load path to a handle, copying it into the guest first. */
function addLoadPath(options, path) {
  const { ptr, len } = put(path);
  const status = accent_sass_options_add_load_path(options, ptr, len);
  accent_sass_dealloc(ptr, len);
  return status;
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

// 8. An options handle crosses the boundary and changes the output. Compressed
//    output is the cheapest knob to see from outside: same input, one line.
const options = accent_sass_options_new();
check("sets the output style", OK, accent_sass_options_set_style(options, STYLE_COMPRESSED));
check("silences the logger", OK, accent_sass_options_set_quiet(options, 1));
got = callWith(
  accent_sass_compile_string_with_options,
  '@warn "noisy";\na {\n  b: 1px + 2px;\n}\n',
  options,
);
check("compiles compressed", "0|a{b:3px}", `${got.status}|${got.body}`);

// 9. The same handle again. A configuration that survives one call only would
//    be no use to a host compiling a theme.
got = callWith(accent_sass_compile_path_with_options, "/work/site/main.scss", options);
check("reuses a handle for a path compile", "0|a{b:3px}", `${got.status}|${got.body}`);

// 10. A load path, which under WASI is a guest path and so needs a preopen.
//     The input has no file of its own, so nothing but the load path can
//     resolve the import.
check("adds a load path", OK, addLoadPath(options, "/work/site"));
got = callWith(
  accent_sass_compile_string_with_options,
  '@use "partial";\na {\n  b: partial.$v + 1px;\n}\n',
  options,
);
check("resolves @use through a load path", "0|a{b:3px}", `${got.status}|${got.body}`);

// 11. The indented syntax, which is not valid SCSS. If the option were
//     ignored this would be a parse error rather than a pass.
check("sets the syntax", OK, accent_sass_options_set_syntax(options, SYNTAX_INDENTED));
got = callWith(accent_sass_compile_string_with_options, "a\n  b: 1px + 2px\n", options);
check("parses the indented syntax", "0|a{b:3px}", `${got.status}|${got.body}`);

// 12. A value the ABI does not define is refused rather than rounded to a
//     default, and the refusal leaves the handle as it was.
check("refuses an undefined style", REJECTED, accent_sass_options_set_style(options, 7));
check("refuses an undefined syntax", REJECTED, accent_sass_options_set_syntax(options, 7));
got = callWith(accent_sass_compile_string_with_options, "a\n  b: 1px + 2px\n", options);
check("keeps the settings a refusal did not change", "0|a{b:3px}", `${got.status}|${got.body}`);

accent_sass_options_free(options);

// 13. A null handle is a refusal with a message, not a trap and not a silent
//     compile with defaults.
check("refuses a null handle", REJECTED, accent_sass_options_set_quiet(0, 1));
got = callWith(accent_sass_compile_string_with_options, "a { b: 1px; }", 0);
check(
  "refuses to compile without a handle",
  "1|Error: null options handle.",
  `${got.status}|${got.body}`,
);

// 14. Entropy. `random()` and `unique-id()` are the only builtins that need
//     it, and they are the only part of the module that can fail for want of
//     a random source while everything above still passes. On this target it
//     arrives through WASI's `random_get`, with no opt-in backend, which is
//     what makes it worth running: the browser target needs a feature to get
//     one, so the two targets reach entropy by different routes.
const entropy = "a {\n  b: random();\n  c: unique-id();\n}\n";
const roll1 = call(accent_sass_compile_string, entropy);
const roll2 = call(accent_sass_compile_string, entropy);
check("compiles a stylesheet that needs entropy", 0, roll1.status);
const drawn = Number((roll1.body.match(/b: ([0-9.]+);/) ?? [])[1]);
check("random() returns a number in [0, 1)", true, drawn >= 0 && drawn < 1);
check("unique-id() returns twelve alphanumerics", true, /c: id-[A-Za-z0-9]{12};/.test(roll1.body));
// A stubbed or failing source is likeliest to return a constant, which one
// plausible-looking value cannot tell apart from a working one.
check("two draws differ", true, roll1.body !== roll2.body);

await rm(work, { recursive: true, force: true });
process.exit(failed);
