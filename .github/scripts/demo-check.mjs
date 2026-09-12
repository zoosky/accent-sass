// Compile the demo's own frameworks through the demo's own wasm package.
//
// The demo has three moving parts that can drift out of step: the bundle
// layout `demo-bundle.mjs` writes, the entry stylesheet and load paths the
// page sends, and the JavaScript API the module exposes. Each is checked
// somewhere else; nothing checks that the three agree, and a mismatch only
// shows up as a blank page.
//
// This runs what the page runs. It reads the same manifest the page reads,
// compiles each framework with the same entry and load paths, and fails if
// the output is not the size the framework is known to produce.
//
// Usage: node .github/scripts/demo-check.mjs [demo directory]
//   The default demo directory is docs/demo, relative to the repository root.

import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const demo = resolve(root, process.argv[2] ?? "docs/demo");
const vendor = join(demo, "vendor");

const { initSync, compileString } = await import(
  pathToFileURL(join(vendor, "pkg", "index.js")).href
);
initSync(readFileSync(join(vendor, "pkg", "index_bg.wasm")));

// The entry points the page sends, in their *configured* form.
//
// A bare `@use "bulma/sass"` would compile even if every variable the page
// offers had been renamed. The configured form is the one that breaks: an
// unknown name in `@use ... with (...)` is a hard error, so these must stay in
// step with the defaults in `docs/demo/app.js`.
const USWDS_CDN = "https://cdn.jsdelivr.net/npm/@uswds/uswds@3.13.0/dist";

const PRESETS = {
  bulma: {
    entry:
      '@use "bulma/sass" with (\n' +
      "  $primary: #00d1b2,\n" +
      "  $link: #485fc7,\n" +
      '  $family-primary: ("Inter", system-ui, sans-serif),\n' +
      "  $radius: 6px,\n" +
      "  $scheme-h: 221\n" +
      ");\n",
    // Bulma's whole stylesheet, which has been stable across 1.0.x.
    minLines: 20000,
  },
  uswds: {
    entry:
      '@use "uswds-core" with (\n' +
      '  $theme-color-primary: "blue-60v",\n' +
      '  $theme-color-secondary: "red-50v",\n' +
      `  $theme-font-path: "${USWDS_CDN}/fonts",\n` +
      `  $theme-image-path: "${USWDS_CDN}/img"\n` +
      ");\n" +
      '@forward "uswds";\n',
    // 33,684 lines, which is what dart-sass 1.104.0 produces for USWDS
    // 3.13.0. See specs/docs/features/27-uswds-parity.md.
    minLines: 33000,
  },
};

const manifest = JSON.parse(readFileSync(join(vendor, "frameworks", "manifest.json"), "utf8"));

let failures = 0;

for (const framework of manifest) {
  const preset = PRESETS[framework.name];
  if (!preset) {
    console.error(`FAIL ${framework.name}: no preset in demo-check.mjs`);
    failures += 1;
    continue;
  }

  const files = JSON.parse(readFileSync(join(vendor, framework.bundle), "utf8"));
  const warnings = [];

  const started = process.hrtime.bigint();
  let result;
  try {
    result = compileString(preset.entry, {
      files,
      loadPaths: framework.loadPaths,
      logger: (event) => warnings.push(event),
    });
  } catch (e) {
    console.error(`FAIL ${framework.name} ${framework.version}: ${e.message}`);
    if (e.formatted) console.error(e.formatted);
    failures += 1;
    continue;
  }
  const ms = Number(process.hrtime.bigint() - started) / 1e6;

  const lines = result.css.split("\n").length;
  const ok = lines >= preset.minLines;
  if (!ok) failures += 1;

  console.log(
    `${ok ? "ok  " : "FAIL"} ${framework.name} ${framework.version}: ` +
      `${lines} lines, ${(result.css.length / 1024).toFixed(0)} KiB CSS, ` +
      // `loadedUrls` counts reads, not distinct files, so a stylesheet the
      // compiler consults twice is counted twice and this exceeds the number
      // of files in the bundle.
      `${result.loadedUrls.length} reads across ${framework.files} files, ` +
      `${warnings.length} warnings, ${ms.toFixed(0)} ms`,
  );

  if (!ok) {
    console.error(`     expected at least ${preset.minLines} lines`);
  }
}

if (failures > 0) {
  console.error(`\n${failures} framework(s) failed`);
  process.exit(1);
}

console.log("\nthe demo compiles every framework it ships");
