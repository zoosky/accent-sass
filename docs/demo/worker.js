// The compiler, off the main thread.
//
// USWDS takes over a second to compile, which would freeze the page if it ran
// on the main thread: no spinner, no typing, no scrolling. The worker also
// keeps the framework bundles here, so parsing 882 KiB of JSON does not block
// the first paint either.
//
// A module worker is required, because the wasm-pack `web` target is an ES
// module. Every browser that supports WebAssembly threads-free modules
// supports it.

import init, { compileString } from "./vendor/pkg/index.js";

/** Resolves once the wasm module is instantiated. */
const ready = init();

/** Framework bundles, keyed by URL, fetched at most once each. */
const bundles = new Map();

/**
 * Fetches and parses a framework bundle, reusing the parsed object.
 *
 * The promise rather than the value is cached, so two compiles that start
 * before the first fetch lands share it instead of fetching twice.
 */
function bundle(url) {
  if (!bundles.has(url)) {
    bundles.set(
      url,
      fetch(url).then((response) => {
        if (!response.ok) {
          throw new Error(`could not load ${url}: ${response.status}`);
        }
        return response.json();
      }),
    );
  }

  return bundles.get(url);
}

self.onmessage = async (event) => {
  const { id, source, bundleUrl, loadPaths, style } = event.data;

  try {
    await ready;

    const files = bundleUrl ? await bundle(bundleUrl) : {};
    const warnings = [];

    // `performance.now()` rather than `Date.now()`: a sub-millisecond compile
    // is the interesting case for the playground.
    const started = performance.now();
    const result = compileString(source, {
      files,
      loadPaths: loadPaths ?? [],
      style: style ?? "expanded",
      url: "demo.scss",
      logger: (warning) => warnings.push(warning),
    });
    const ms = performance.now() - started;

    self.postMessage({
      id,
      ok: true,
      css: result.css,
      loadedUrls: result.loadedUrls,
      warnings,
      ms,
    });
  } catch (error) {
    // A Sass failure carries the structured fields; anything else (a failed
    // fetch, a missing bundle) only has a message.
    self.postMessage({
      id,
      ok: false,
      message: error?.message ?? String(error),
      formatted: error?.formatted ?? null,
      file: error?.file ?? null,
      line: error?.line ?? null,
      column: error?.column ?? null,
    });
  }
};
