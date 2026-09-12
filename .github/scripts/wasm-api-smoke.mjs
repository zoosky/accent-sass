// Exercise the browser package's JavaScript API in crates/lib/pkg.
//
// `wasm-smoke.mjs` proves the module has a compiler in it at all. This proves
// the API around that compiler works: options, the supplied filesystem, the
// logger callback and the structured error. None of it can be covered by
// `cargo test`, because the bindings only exist on wasm32-unknown-unknown.
//
// The filesystem is the part worth running rather than trusting. A browser
// build has no `std::fs` underneath it, so if `files` stopped being wired
// through, every multi-file compile would fail -- which is exactly the state
// the package shipped in before this API existed.
//
// Usage: node .github/scripts/wasm-api-smoke.mjs [package directory]
//   The default package directory is crates/lib/pkg, relative to the
//   repository root.

import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const pkg = resolve(root, process.argv[2] ?? "crates/lib/pkg");

const { initSync, compile, compileString, from_string } = await import(
  pathToFileURL(join(pkg, "index.js")).href
);
initSync(readFileSync(join(pkg, "index_bg.wasm")));

let failures = 0;

function check(name, got, want) {
  if (got === want) {
    console.log(`ok   ${name}`);
    return;
  }
  failures += 1;
  console.error(`FAIL ${name}`);
  console.error(`     got  ${JSON.stringify(got)}`);
  console.error(`     want ${JSON.stringify(want)}`);
}

function checkThat(name, condition, detail) {
  if (condition) {
    console.log(`ok   ${name}`);
    return;
  }
  failures += 1;
  console.error(`FAIL ${name}`);
  if (detail !== undefined) console.error(`     ${detail}`);
}

// --- the original export still works -------------------------------------

check(
  "from_string still compiles",
  from_string("a { b: calc(1rem - 2px); }"),
  "a {\n  b: calc(1rem - 2px);\n}\n",
);

// --- compileString with no options ---------------------------------------

check(
  "compileString with no options",
  compileString("a { color: red; }").css,
  "a {\n  color: red;\n}\n",
);

// --- a multi-file tree through `files` -----------------------------------
//
// A partial, an index file and a nested relative import: the three shapes a
// real framework relies on.

const files = {
  "theme/_colors.scss": "$brand: #bada55 !default;",
  "theme/_index.scss": '@forward "colors";',
  "theme/parts/_button.scss":
    '@use "../colors";\n.btn { color: colors.$brand; }',
};

check(
  "compileString resolves a partial through an index file",
  compileString('@use "theme";\na { color: theme.$brand; }', { files }).css,
  "a {\n  color: #bada55;\n}\n",
);

check(
  "compileString resolves a nested relative import",
  compileString('@use "theme/parts/button";', { files }).css,
  ".btn {\n  color: #bada55;\n}\n",
);

// The pattern every framework is themed through.
check(
  "@use ... with overrides a forwarded default",
  compileString('@use "theme" with ($brand: blue);\na { color: theme.$brand; }', {
    files,
  }).css,
  "a {\n  color: blue;\n}\n",
);

// A Map is accepted as well as a plain object.
check(
  "files accepts a Map",
  compileString('@use "theme";\na { color: theme.$brand; }', {
    files: new Map(Object.entries(files)),
  }).css,
  "a {\n  color: #bada55;\n}\n",
);

// --- loadPaths ------------------------------------------------------------

check(
  "loadPaths resolves a module outside the entry's directory",
  compileString('@use "framework";', {
    files: { "vendor/framework/_index.scss": ".f { color: green; }" },
    loadPaths: ["vendor"],
  }).css,
  ".f {\n  color: green;\n}\n",
);

// --- url ------------------------------------------------------------------

check(
  "url sets the directory relative imports resolve against",
  compileString('@use "colors";\na { color: colors.$brand; }', {
    files,
    url: "theme/app.scss",
  }).css,
  "a {\n  color: #bada55;\n}\n",
);

// --- compile(path) --------------------------------------------------------

check(
  "compile reads the entry point from files",
  compile("theme/parts/_button.scss", { files }).css,
  ".btn {\n  color: #bada55;\n}\n",
);

// --- loadedUrls -----------------------------------------------------------

const loaded = compileString('@use "theme";', { files }).loadedUrls;
checkThat(
  "loadedUrls reports the files the compile read",
  loaded.includes("theme/_index.scss") && loaded.includes("theme/_colors.scss"),
  `got ${JSON.stringify(loaded)}`,
);

// --- style ----------------------------------------------------------------

check(
  "style: compressed",
  compileString("a { color: red; }", { style: "compressed" }).css,
  "a{color:red}",
);

// --- syntax ---------------------------------------------------------------

check(
  "syntax: indented",
  compileString("a\n  color: red", { syntax: "indented" }).css,
  "a {\n  color: red;\n}\n",
);

// --- logger ---------------------------------------------------------------

const events = [];
compileString('@warn "careful";\n@debug "looking";\na { color: red; }', {
  logger: (event) => events.push(event),
});
// The message arrives as its text, the way dart-sass 1.104.0 reports it:
// `@warn "careful"` hands the logger `careful` rather than `"careful"`. This
// assertion carried the old quoted form and is the line that caught the
// change, which is what it was written for. Only a top-level string is
// unwrapped, so a string nested in a list still arrives quoted.
checkThat(
  "logger receives @warn and @debug",
  events.length === 2 &&
    events[0].type === "warn" &&
    events[0].message === "careful" &&
    events[1].type === "debug" &&
    events[1].message === "looking" &&
    events[0].line === 1,
  `got ${JSON.stringify(events)}`,
);

const quietEvents = [];
compileString('@warn "careful";\na { color: red; }', {
  quiet: true,
  logger: (event) => quietEvents.push(event),
});
check("quiet silences the logger", quietEvents.length, 0);

// A throwing logger must not fail a compile that would otherwise succeed.
check(
  "a throwing logger does not fail the compile",
  compileString('@warn "careful";\na { color: red; }', {
    logger: () => {
      throw new Error("logger blew up");
    },
  }).css,
  "a {\n  color: red;\n}\n",
);

// --- structured errors ----------------------------------------------------

let thrown;
try {
  compileString("a { color: ; }");
} catch (e) {
  thrown = e;
}
checkThat("a failed compile throws", thrown !== undefined);
checkThat("the error is an Error", thrown instanceof Error, String(thrown));
checkThat(
  "the error carries message, file, line and column",
  typeof thrown?.message === "string" &&
    thrown.message.length > 0 &&
    !thrown.message.includes("\n") &&
    typeof thrown?.file === "string" &&
    thrown?.line === 1 &&
    typeof thrown?.column === "number",
  `message=${JSON.stringify(thrown?.message)} file=${JSON.stringify(thrown?.file)} line=${thrown?.line} column=${thrown?.column}`,
);
checkThat(
  "the error keeps the formatted block",
  typeof thrown?.formatted === "string" && thrown.formatted.includes("╷"),
  JSON.stringify(thrown?.formatted),
);

let missing;
try {
  compileString('@use "nope";');
} catch (e) {
  missing = e;
}
checkThat(
  "a missing import reports the usual message",
  missing?.message.includes("Can't find stylesheet to import."),
  JSON.stringify(missing?.message),
);

// --- malformed options ----------------------------------------------------

let badOption;
try {
  compileString("a { color: red; }", { style: "nonsense" });
} catch (e) {
  badOption = e;
}
checkThat(
  "an unknown style is a TypeError",
  badOption instanceof TypeError,
  String(badOption),
);

// `is_object()` is true for arrays, so without an explicit check this stored a
// file named "0" and failed later with `Can't find stylesheet to import.`,
// which points at the stylesheet rather than at the malformed option.
let badFiles;
try {
  compileString('@use "theme";', { files: ["theme/_colors.scss"] });
} catch (e) {
  badFiles = e;
}
checkThat(
  "an array as files is a TypeError",
  badFiles instanceof TypeError,
  String(badFiles),
);

if (failures > 0) {
  console.error(`\n${failures} check(s) failed`);
  process.exit(1);
}

console.log(`\nall checks passed`);
