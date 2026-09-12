// The demo's interface.
//
// The compiler itself lives in `worker.js`; this file decides what to send it
// and how to show what comes back. The one idea worth knowing is that the
// theme controls do not patch the compiled CSS -- they rewrite the
// `@use ... with (...)` block and recompile the framework from source, which
// is what a project configuring Bulma or USWDS actually does.

const $ = (id) => document.getElementById(id);

const els = {
  source: $("source"),
  compile: $("compile"),
  auto: $("auto"),
  controls: $("controls"),
  controlGrid: $("control-grid"),
  controlsNote: $("controls-note"),
  reset: $("reset"),
  presetNote: $("preset-note"),
  status: $("status"),
  statusText: $("status-text"),
  stats: $("stats"),
  preview: $("preview"),
  css: $("css-output").querySelector("code"),
  warnings: $("warnings-output"),
  files: $("files-output"),
  badgeWarnings: $("badge-warnings"),
  badgeFiles: $("badge-files"),
  error: $("error"),
  errorTitle: $("error-title"),
  errorBody: $("error-body").querySelector("code"),
  factSize: $("fact-size"),
  factVersion: $("fact-version"),
};

// USWDS colour settings take the name of a token from its own palette, not a
// CSS colour. Every value here was compiled against USWDS 3.13.0 before being
// offered, because an unknown token name fails the whole build.
const USWDS_COLORS = [
  "blue-60v",
  "indigo-50v",
  "cyan-30v",
  "mint-40v",
  "green-cool-50v",
  "gold-30v",
  "red-50v",
  "magenta-50v",
  "violet-60v",
];

// USWDS resolves fonts and icons against these settings. Pointing them at the
// published package means the preview renders with the real typeface and
// icons; leaving them at their defaults would give a frame full of missing
// assets.
const USWDS_CDN = "https://cdn.jsdelivr.net/npm/@uswds/uswds@3.13.0/dist";

const PRESETS = {
  bulma: {
    label: "Bulma",
    note: "Bulma 1.0.4: 78 stylesheets. The whole framework is recompiled from source each time you move a control.",
    controls: [
      { name: "primary", label: "$primary", type: "color", value: "#00d1b2" },
      { name: "link", label: "$link", type: "color", value: "#485fc7" },
      {
        name: "family",
        label: "$family-primary",
        type: "select",
        // Each value is a Sass font stack, with the family names quoted
        // individually. Quoting the stack as a whole would make it one family
        // name that matches nothing: Bulma writes `$family-primary` straight
        // into the `.is-family-primary` helper, where the quotes survive.
        value: '"Inter", system-ui, sans-serif',
        options: [
          '"Inter", system-ui, sans-serif',
          '"Georgia", serif',
          "ui-monospace, monospace",
        ],
      },
      {
        name: "radius",
        label: "$radius",
        type: "select",
        value: "6px",
        options: ["0", "4px", "6px", "12px", "999px"],
      },
      { name: "hue", label: "$scheme-h", type: "number", value: "221" },
    ],
    entry: (t) =>
      `// Bulma 1.0.4, compiled from source in this tab.\n` +
      `// Every value below is one of Bulma's own !default variables.\n` +
      `@use "bulma/sass" with (\n` +
      `  $primary: ${t.primary},\n` +
      `  $link: ${t.link},\n` +
      // Parenthesised: inside `@use ... with (...)` a bare comma separates
      // arguments, so an unwrapped font stack is read as several arguments
      // rather than one list.
      `  $family-primary: (${t.family}),\n` +
      `  $radius: ${t.radius},\n` +
      `  $scheme-h: ${t.hue}\n` +
      `);\n`,
    preview: bulmaMarkup,
  },

  uswds: {
    label: "USWDS",
    note: "USWDS 3.13.0: 605 stylesheets, about 800 KiB of Sass, compiled to roughly 33,700 lines of CSS. Expect a few seconds.",
    controls: [
      {
        name: "primary",
        label: "$theme-color-primary",
        type: "select",
        value: "blue-60v",
        options: USWDS_COLORS,
      },
      {
        name: "secondary",
        label: "$theme-color-secondary",
        type: "select",
        value: "red-50v",
        options: USWDS_COLORS,
      },
    ],
    entry: (t) =>
      `// USWDS 3.13.0, compiled from source in this tab.\n` +
      `// Configure uswds-core first, then forward the framework: that is the\n` +
      `// order USWDS's own theme file uses.\n` +
      `@use "uswds-core" with (\n` +
      `  $theme-color-primary: "${t.primary}",\n` +
      `  $theme-color-secondary: "${t.secondary}",\n` +
      `  $theme-font-path: "${USWDS_CDN}/fonts",\n` +
      `  $theme-image-path: "${USWDS_CDN}/img"\n` +
      `);\n` +
      `@forward "uswds";\n`,
    preview: uswdsMarkup,
  },

  playground: {
    label: "Playground",
    note: "One stylesheet, no dependencies. Modern module functions, loops and colour maths.",
    controls: [],
    entry: () =>
      `@use "sass:color";\n` +
      `@use "sass:math";\n\n` +
      `$brand: #4b3bd4;\n` +
      `$steps: 5;\n\n` +
      `.swatches {\n` +
      `  display: grid;\n` +
      `  grid-template-columns: repeat(#{$steps}, 1fr);\n` +
      `  gap: math.div(24px, 2);\n\n` +
      `  @for $i from 1 through $steps {\n` +
      `    .step-#{$i} {\n` +
      `      $shift: ($i - math.ceil(math.div($steps, 2))) * 14%;\n` +
      `      background: color.adjust($brand, $lightness: $shift);\n` +
      `      // Relative colour syntax and calc() survive to the output.\n` +
      `      border-bottom: 4px solid color.adjust($brand, $hue: $i * 24deg);\n` +
      `      padding: calc(0.5rem + #{$i} * 2px);\n` +
      `    }\n` +
      `  }\n` +
      `}\n\n` +
      `.note {\n` +
      `  color: color.adjust($brand, $lightness: -18%);\n` +
      `  font: 500 14px/1.5 system-ui, sans-serif;\n` +
      `}\n`,
    preview: playgroundMarkup,
  },
};

/** Sample markup for the Bulma preview. */
function bulmaMarkup() {
  return `
    <nav class="navbar is-primary" role="navigation">
      <div class="navbar-brand">
        <span class="navbar-item has-text-weight-bold">Bulma</span>
      </div>
    </nav>
    <section class="section">
      <div class="container">
        <h1 class="title">Compiled in this tab</h1>
        <p class="subtitle">Move a control and the framework rebuilds.</p>
        <div class="buttons">
          <button class="button is-primary">Primary</button>
          <button class="button is-link">Link</button>
          <button class="button is-primary is-light">Light</button>
          <button class="button">Default</button>
        </div>
        <div class="notification is-primary">
          A notification, coloured by <code>$primary</code>.
        </div>
        <div class="columns">
          <div class="column">
            <div class="card">
              <div class="card-content">
                <p class="title is-5">Card</p>
                <p>Corner radius comes from <code>$radius</code>.</p>
                <div class="tags">
                  <span class="tag is-primary">primary</span>
                  <span class="tag is-link">link</span>
                </div>
              </div>
            </div>
          </div>
          <div class="column">
            <label class="label">Field</label>
            <div class="control">
              <input class="input" type="text" placeholder="Type here" />
            </div>
            <progress class="progress is-primary" value="60" max="100">60%</progress>
          </div>
        </div>
      </div>
    </section>`;
}

/** Sample markup for the USWDS preview. */
function uswdsMarkup() {
  return `
    <header class="usa-header usa-header--basic">
      <div class="usa-nav-container">
        <div class="usa-navbar">
          <div class="usa-logo"><em class="usa-logo__text">Design system</em></div>
        </div>
      </div>
    </header>
    <main class="usa-section">
      <div class="grid-container">
        <h1>Compiled in this tab</h1>
        <p class="usa-intro">
          The colour settings below are USWDS palette tokens, resolved by its
          own Sass functions.
        </p>
        <div class="usa-button-group">
          <button class="usa-button" type="button">Primary</button>
          <button class="usa-button usa-button--secondary" type="button">Secondary</button>
          <button class="usa-button usa-button--outline" type="button">Outline</button>
          <button class="usa-button usa-button--base" type="button">Base</button>
        </div>
        <div class="usa-alert usa-alert--info margin-top-3">
          <div class="usa-alert__body">
            <h2 class="usa-alert__heading">Informative status</h2>
            <p class="usa-alert__text">An alert, themed from the palette.</p>
          </div>
        </div>
        <div class="usa-alert usa-alert--success margin-top-2">
          <div class="usa-alert__body">
            <p class="usa-alert__text">A success message.</p>
          </div>
        </div>
        <div class="usa-card grid-col-12 margin-top-3">
          <div class="usa-card__container">
            <div class="usa-card__header"><h3 class="usa-card__heading">Card</h3></div>
            <div class="usa-card__body"><p>Typography is loaded from the published package.</p></div>
            <div class="usa-card__footer">
              <button class="usa-button" type="button">Act</button>
            </div>
          </div>
        </div>
        <form class="usa-form margin-top-3">
          <label class="usa-label" for="demo-input">Text input</label>
          <input class="usa-input" id="demo-input" type="text" />
          <div class="usa-checkbox margin-top-2">
            <input class="usa-checkbox__input" id="demo-check" type="checkbox" checked />
            <label class="usa-checkbox__label" for="demo-check">A checkbox</label>
          </div>
        </form>
      </div>
    </main>`;
}

/** Sample markup for the playground preview. */
function playgroundMarkup() {
  return `
    <div style="padding:24px;font:14px system-ui,sans-serif">
      <p class="note">Five steps, generated by an @for loop.</p>
      <div class="swatches">
        <div class="step-1">1</div>
        <div class="step-2">2</div>
        <div class="step-3">3</div>
        <div class="step-4">4</div>
        <div class="step-5">5</div>
      </div>
    </div>`;
}

const state = {
  preset: "bulma",
  tokens: {},
  /** Set once the source is edited by hand, which stops the controls driving it. */
  manual: false,
  /** Framework metadata from the generated manifest, keyed by name. */
  manifest: new Map(),
  busy: false,
  /** A compile requested while another was running. */
  pending: false,
  nextId: 1,
};

const worker = new Worker(new URL("./worker.js", import.meta.url), {
  type: "module",
});

const waiting = new Map();

worker.onmessage = (event) => {
  const resolve = waiting.get(event.data.id);
  if (resolve) {
    waiting.delete(event.data.id);
    resolve(event.data);
  }
};

worker.onerror = (event) => {
  showError(
    "The compiler could not start",
    event.message ??
      "The worker failed to load. This page needs an http origin and a browser that supports module workers.",
  );

  // Release anything waiting on a reply that will never arrive. Without this
  // the page locks: `state.busy` stays true, so every later compile returns at
  // its first line, and the Compile button stays disabled forever.
  for (const [id, resolve] of waiting) {
    waiting.delete(id);
    resolve({
      ok: false,
      message: "the compiler worker stopped",
      formatted: null,
      line: null,
      column: null,
    });
  }

  state.busy = false;
  state.pending = false;
  els.compile.disabled = false;
};

/** Sends one compile to the worker and resolves with its reply. */
function sendCompile(source, framework, preset) {
  const id = state.nextId++;

  return new Promise((resolve) => {
    waiting.set(id, resolve);
    worker.postMessage({
      id,
      source,
      bundleUrl: framework ? `vendor/${framework.bundle}` : null,
      loadPaths: framework ? framework.loadPaths : [],
    });
  }).then((reply) => ({ ...reply, preset }));
}

/** Renders the theme controls for the current preset. */
function renderControls() {
  const preset = PRESETS[state.preset];
  els.controlGrid.replaceChildren();

  if (preset.controls.length === 0) {
    els.controls.hidden = true;
    return;
  }

  els.controls.hidden = false;

  for (const control of preset.controls) {
    const wrapper = document.createElement("div");
    wrapper.className = "control";

    const label = document.createElement("label");
    label.textContent = control.label;
    label.htmlFor = `control-${control.name}`;

    let input;
    if (control.type === "select") {
      input = document.createElement("select");
      for (const option of control.options) {
        const el = document.createElement("option");
        el.value = option;
        el.textContent = option;
        input.append(el);
      }
    } else {
      input = document.createElement("input");
      input.type = control.type === "color" ? "color" : "text";
      if (control.type === "number") input.inputMode = "numeric";
    }

    input.id = `control-${control.name}`;
    input.value = state.tokens[control.name];
    input.addEventListener("input", () => {
      state.tokens[control.name] = input.value;
      applyTokens();
    });

    wrapper.append(label, input);
    els.controlGrid.append(wrapper);
  }
}

/** Rewrites the source from the current control values, then compiles. */
function applyTokens() {
  if (state.manual) return;

  els.source.value = PRESETS[state.preset].entry(state.tokens);
  compile();
}

/** Switches preset, resetting its controls and source. */
function selectPreset(name) {
  state.preset = name;
  state.manual = false;
  els.controlsNote.hidden = true;

  const preset = PRESETS[name];
  state.tokens = Object.fromEntries(
    preset.controls.map((control) => [control.name, control.value]),
  );

  for (const button of document.querySelectorAll(".preset")) {
    button.setAttribute(
      "aria-current",
      String(button.dataset.preset === name),
    );
  }

  const framework = state.manifest.get(name);
  els.presetNote.textContent = framework
    ? `${preset.note} Bundle: ${(framework.bundleBytes / 1024).toFixed(0)} KiB of JSON.`
    : preset.note;

  renderControls();
  els.source.value = preset.entry(state.tokens);
  compile();
}

/** Compiles the current source and renders the result. */
async function compile() {
  if (state.busy) {
    state.pending = true;
    return;
  }

  state.busy = true;
  state.pending = false;
  setStatus("working", "Compiling…");
  els.compile.disabled = true;

  const preset = state.preset;
  const framework = state.manifest.get(preset);

  // A framework preset with no manifest entry means the bundle never loaded.
  // Compiling anyway would resolve no imports and report `Can't find
  // stylesheet to import.` against the user's own source, which points at the
  // wrong thing entirely.
  if (!framework && PRESETS[preset].preview !== playgroundMarkup) {
    state.busy = false;
    els.compile.disabled = false;
    showError(
      "The framework bundle is missing",
      `vendor/frameworks/manifest.json did not load, so ${PRESETS[preset].label}'s ` +
        `stylesheets are not available. Run \`node .github/scripts/demo-bundle.mjs\` ` +
        `and serve the page over http.`,
      "bundle missing",
    );
    return;
  }

  const reply = await sendCompile(els.source.value, framework, preset);

  state.busy = false;
  els.compile.disabled = false;

  // A newer compile is already queued, so this result is stale. Rendering it
  // would paint one preset's CSS into another preset's markup, and on USWDS
  // the correction is seconds away rather than a frame.
  if (state.pending) {
    compile();
    return;
  }

  if (reply.ok) {
    showResult(reply);
  } else {
    showError(
      reply.line ? `Error at line ${reply.line}:${reply.column}` : "Error",
      reply.formatted ?? reply.message,
      reply.message,
    );
  }
}

/** Shows a successful compile. */
function showResult(reply) {
  els.error.hidden = true;

  const bytes = new Blob([reply.css]).size;
  const lines = reply.css.split("\n").length;

  setStatus("ok", "Compiled");
  els.stats.textContent =
    `${reply.ms.toFixed(0)} ms · ${lines.toLocaleString()} lines · ` +
    `${(bytes / 1024).toFixed(0)} KiB`;

  els.css.textContent = reply.css;

  // `loadedUrls` counts reads rather than distinct files, so a stylesheet
  // consulted twice appears twice. Show both numbers rather than implying one.
  const distinct = new Set(reply.loadedUrls);
  els.badgeFiles.textContent = String(distinct.size);
  els.files.replaceChildren();
  if (distinct.size === 0) {
    els.files.append(empty("No imports: this stylesheet stands alone."));
  } else {
    const heading = document.createElement("p");
    heading.className = "log-where";
    heading.textContent = `${distinct.size} files, ${reply.loadedUrls.length} reads`;
    els.files.append(heading);
    for (const url of distinct) {
      const row = document.createElement("div");
      row.className = "log-entry";
      row.textContent = url;
      els.files.append(row);
    }
  }

  els.badgeWarnings.textContent = String(reply.warnings.length);
  els.warnings.replaceChildren();
  if (reply.warnings.length === 0) {
    els.warnings.append(empty("No @warn or @debug from this compile."));
  } else {
    for (const warning of reply.warnings) {
      const row = document.createElement("div");
      row.className = "log-entry";

      const where = document.createElement("div");
      where.className = "log-where";
      where.textContent = `${warning.type} · ${warning.file}:${warning.line}:${warning.column}`;

      const body = document.createElement("div");
      body.textContent = warning.message;

      row.append(where, body);
      els.warnings.append(row);
    }
  }

  renderPreview(reply.css, reply.preset ?? state.preset);
}

/** Puts the compiled CSS and the markup of the preset it came from into the frame. */
function renderPreview(css, preset) {
  const markup = PRESETS[preset].preview();
  els.preview.srcdoc =
    `<!doctype html><html><head><meta charset="utf-8">` +
    `<style>${css}</style></head><body>${markup}</body></html>`;
}

/** Shows a failed compile, keeping the last good preview on screen. */
function showError(title, body, short) {
  setStatus("error", short ?? title);
  els.stats.textContent = "";
  els.error.hidden = false;
  els.errorTitle.textContent = title;
  els.errorBody.textContent = body;
}

function setStatus(stateName, text) {
  els.status.dataset.state = stateName;
  els.statusText.textContent = text;
}

function empty(text) {
  const el = document.createElement("p");
  el.className = "log-empty";
  el.textContent = text;
  return el;
}

// Wiring -------------------------------------------------------------------

for (const button of document.querySelectorAll(".preset")) {
  button.addEventListener("click", () => selectPreset(button.dataset.preset));
}

for (const tab of document.querySelectorAll(".tab")) {
  tab.addEventListener("click", () => {
    for (const other of document.querySelectorAll(".tab")) {
      other.setAttribute("aria-selected", String(other === tab));
    }
    for (const view of document.querySelectorAll(".view")) {
      view.hidden = view.dataset.view !== tab.dataset.tab;
    }
  });
}

els.compile.addEventListener("click", () => compile());
els.reset.addEventListener("click", () => selectPreset(state.preset));

let debounce;
els.source.addEventListener("input", () => {
  // Editing by hand wins over the controls: regenerating the source from them
  // would discard what was just typed.
  if (!state.manual) {
    state.manual = true;
    els.controlsNote.hidden = PRESETS[state.preset].controls.length === 0;
  }

  if (!els.auto.checked) return;

  clearTimeout(debounce);
  debounce = setTimeout(() => compile(), 500);
});

// Startup ------------------------------------------------------------------

document.querySelector('.tab[data-tab="preview"]').setAttribute("aria-selected", "true");

/** Reports the module's transfer size, which is the number that matters. */
async function reportModuleSize() {
  try {
    const response = await fetch("vendor/pkg/index_bg.wasm", { method: "HEAD" });
    const length = response.headers.get("content-encoding")
      ? null
      : response.headers.get("content-length");

    els.factSize.textContent = length
      ? `${(Number(length) / 1024 / 1024).toFixed(2)} MB wasm`
      : "about 0.6 MB compressed";
  } catch {
    els.factSize.textContent = "about 0.6 MB compressed";
  }
}

async function start() {
  reportModuleSize();

  try {
    const response = await fetch("vendor/frameworks/manifest.json");
    if (response.ok) {
      for (const framework of await response.json()) {
        state.manifest.set(framework.name, framework);
        const meta = document.getElementById(`meta-${framework.name}`);
        if (meta) {
          meta.textContent = `${framework.version} · ${framework.files} files`;
        }
      }
    }
  } catch {
    // The playground still works without the bundles; the framework presets
    // will report a failed fetch when they are chosen.
  }

  selectPreset("bulma");
}

start();
