// Build the framework source bundles the browser demo compiles.
//
// The demo compiles Bulma and USWDS in the browser, which means it must hold
// their whole Sass trees in memory before it starts: `Fs::read` is
// synchronous, so the compiler cannot fetch a dependency at the moment it
// resolves an import. This script turns each framework's `node_modules` tree
// into one JSON object of path to source, which the page fetches once and
// passes to `compileString` as `files`.
//
// Paths in a bundle are rooted at the framework rather than at
// `node_modules`, so the demo's load paths stay short:
//
//   bulma/sass/_index.scss   ->  @use "bulma/sass"
//   uswds/uswds/_index.scss  ->  @use "uswds"    with loadPaths ["uswds"]
//
// Usage: node .github/scripts/demo-bundle.mjs [output directory]
//   The default output directory is docs/demo/vendor/frameworks.

import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const outDir = resolve(root, process.argv[2] ?? "docs/demo/vendor/frameworks");

// The versions the `frameworks` CI job compiles, so the demo shows the same
// stylesheets the parity numbers were measured against. Keep these in step
// with `.github/scripts/frameworks.sh`.
const FRAMEWORKS = [
  {
    name: "bulma",
    spec: "bulma@1.0.4",
    // Copied from node_modules/bulma into the bundle as bulma/...
    from: "bulma",
    to: "bulma",
    loadPaths: [],
  },
  {
    name: "uswds",
    spec: "@uswds/uswds@3.13.0",
    from: "@uswds/uswds/packages",
    to: "uswds",
    loadPaths: ["uswds"],
  },
];

// Only stylesheets matter. A framework package also ships compiled CSS, JS,
// fonts and images, which together are tens of megabytes and none of which
// the compiler reads.
const EXTENSIONS = [".scss", ".sass"];

/** Lists every stylesheet under `dir`, recursively. */
function stylesheets(dir) {
  const found = [];

  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);

    if (entry.isDirectory()) {
      found.push(...stylesheets(path));
    } else if (EXTENSIONS.some((ext) => entry.name.endsWith(ext))) {
      found.push(path);
    }
  }

  return found;
}

const work = resolve(root, "target/demo-corpus");
mkdirSync(work, { recursive: true });

// `npm install` needs a package.json to install into, and a bare one keeps npm
// from walking up and touching anything else.
writeFileSync(
  join(work, "package.json"),
  JSON.stringify({ name: "demo-corpus", private: true, version: "0.0.0" }, null, 2),
);

const specs = FRAMEWORKS.map((f) => f.spec);
console.log(`installing ${specs.join(" ")}`);
execFileSync("npm", ["install", "--no-audit", "--no-fund", "--silent", ...specs], {
  cwd: work,
  stdio: "inherit",
});

rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

const manifest = [];

for (const framework of FRAMEWORKS) {
  const source = join(work, "node_modules", framework.from);
  const files = {};
  let bytes = 0;

  for (const path of stylesheets(source)) {
    const key = join(framework.to, relative(source, path));
    const contents = readFileSync(path, "utf8");

    files[key] = contents;
    bytes += Buffer.byteLength(contents, "utf8");
  }

  const count = Object.keys(files).length;
  if (count === 0) {
    throw new Error(`no stylesheets found for ${framework.name} under ${source}`);
  }

  const bundlePath = join(outDir, `${framework.name}.json`);
  writeFileSync(bundlePath, JSON.stringify(files));

  // The version npm actually resolved, rather than the spec, so the page
  // reports what it is really compiling.
  const pkg = JSON.parse(
    readFileSync(join(work, "node_modules", framework.spec.replace(/@[^@/]+$/, ""), "package.json"), "utf8"),
  );

  manifest.push({
    name: framework.name,
    version: pkg.version,
    // No `entry` here: the page and `demo-check.mjs` each build their own
    // configured entry point, so a copy in the manifest would be a third
    // declaration that nothing executes.
    loadPaths: framework.loadPaths,
    files: count,
    bytes,
    bundle: `frameworks/${framework.name}.json`,
    bundleBytes: statSync(bundlePath).size,
  });

  console.log(
    `${framework.name} ${pkg.version}: ${count} stylesheets, ${(bytes / 1024).toFixed(0)} KiB of Sass` +
      ` -> ${(statSync(bundlePath).size / 1024).toFixed(0)} KiB of JSON`,
  );
}

writeFileSync(join(outDir, "manifest.json"), JSON.stringify(manifest, null, 2));
console.log(`wrote ${relative(root, outDir)}/manifest.json`);
