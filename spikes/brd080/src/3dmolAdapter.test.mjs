import assert from "node:assert/strict";
import test from "node:test";
import { mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { build } from "vite";

const outDir = await mkdtemp(join(tmpdir(), "brd080-3dmol-test-"));
await build({
  root: new URL("..", import.meta.url).pathname,
  configFile: false,
  logLevel: "silent",
  build: {
    outDir,
    emptyOutDir: true,
    minify: false,
    rollupOptions: {
      input: new URL("3dmolAdapter.ts", import.meta.url).pathname,
      external: ["3dmol"],
      preserveEntrySignatures: "strict",
      output: { entryFileNames: "adapter.mjs", format: "es" },
    },
  },
});
const { createThreeDMolAdapter } = await import(pathToFileURL(join(outDir, "adapter.mjs")));
const VIEW_LIMITS = { maxWidth: 1280, maxHeight: 960, maxDpr: 2 };

const fixture = Object.freeze({
  id: "water",
  kind: "molecule",
  format: "sdf",
  text: "water\nfixture\n",
  description: "synthetic molecule format probe",
  atoms: [
    { id: 10, element: "O", x: 0, y: 0, z: 0, label: "oxygen" },
    { id: 20, element: "H", x: 1, y: 0, z: 0 },
  ],
  bonds: [{ from: 10, to: 20, order: 1 }],
});

function harness({ createViewerReturnsNull = false, addModelThrows = false } = {}) {
  const calls = [];
  const children = [];
  const canvas = {
    style: {}, width: 0, height: 0,
    remove() { const index = children.indexOf(this); if (index >= 0) children.splice(index, 1); },
  };
  const host = {
    children,
    style: {},
    clientWidth: 500,
    clientHeight: 300,
    appendChild(child) { children.push(child); },
    querySelectorAll(selector) { return selector === "canvas" ? children.filter((child) => child === canvas) : []; },
  };
  const model = {
    setStyle(selection, style) { calls.push(["setStyle", selection, style]); },
    setClickable(selection, clickable, callback) { calls.push(["setClickable", selection, clickable]); this.click = callback; },
  };
  const viewer = {
    addModel(data, format) { calls.push(["addModel", data, format]); if (addModelThrows) throw new Error("parser"); return model; },
    addModelsAsFrames(data, format) { calls.push(["frames", data, format]); return model; },
    addUnitCell() { calls.push(["unitCell"]); },
    addVolumetricData(data, format, spec) { calls.push(["volume", format, spec.isoval]); },
    addLabel(text) { calls.push(["label", text]); },
    removeAllLabels() { calls.push(["removeLabels"]); },
    render() { calls.push(["render"]); },
    zoomTo() { calls.push(["zoomTo"]); },
    resize() {
      calls.push(["resize", host.style.width, host.style.height]);
      canvas.width = Math.round(Number.parseFloat(host.style.width) * globalThis.window.devicePixelRatio);
      canvas.height = Math.round(Number.parseFloat(host.style.height) * globalThis.window.devicePixelRatio);
    },
    stopAnimate() { calls.push(["stop"]); },
    clear() { calls.push(["clear"]); },
    getView() { return []; },
  };
  const load = async () => ({
    createViewer(receivedHost) {
      calls.push(["createViewer"]);
      if (createViewerReturnsNull) return null;
      receivedHost.appendChild(canvas);
      return viewer;
    },
  });
  return { host, canvas, model, viewer, calls, load };
}

function browser(testBody) {
  return async () => {
    const previousWindow = globalThis.window;
    const previousDocument = globalThis.document;
    globalThis.window = { devicePixelRatio: 3 };
    globalThis.document = {};
    try { await testBody(); } finally {
      if (previousWindow === undefined) delete globalThis.window; else globalThis.window = previousWindow;
      if (previousDocument === undefined) delete globalThis.document; else globalThis.document = previousDocument;
    }
  };
}

test("SSR rejects before dynamically loading 3Dmol", async () => {
  let loaded = false;
  const adapter = createThreeDMolAdapter(async () => { loaded = true; throw new Error("must not load"); });
  await assert.rejects(adapter.mount(harness().host, fixture, { labelsVisible: false, reducedMotion: true }),
    (error) => error.name === "AdapterError" && error.code === "renderer-unavailable");
  assert.equal(loaded, false);
});

test("local source and exact kind/format pairs fail closed", browser(async () => {
  const adapter = createThreeDMolAdapter(harness().load);
  await assert.rejects(adapter.mount(harness().host, { ...fixture, text: "https://example.invalid/a.sdf" }, { labelsVisible: false, reducedMotion: true }),
    (error) => error.code === "invalid-fixture");
  await assert.rejects(adapter.mount(harness().host, { ...fixture, format: "pdb" }, { labelsVisible: false, reducedMotion: true }),
    (error) => error.code === "unsupported-fixture");
}));

test("selection, labels, bounded resize, snapshot and idempotent disposal", browser(async () => {
  const h = harness();
  const session = await createThreeDMolAdapter(h.load).mount(h.host, fixture, { labelsVisible: true, reducedMotion: true });
  session.select([20, 10, 20]);
  session.resize(99_999, 99_999, 9);
  assert.deepEqual(session.snapshot(), {
    candidate: "3dmol", fixture: "water", selectedAtomIds: [10, 20], labelsVisible: true,
    width: VIEW_LIMITS.maxWidth, height: VIEW_LIMITS.maxHeight, dpr: VIEW_LIMITS.maxDpr, status: "ready",
  });
  assert.equal(h.canvas.width, VIEW_LIMITS.maxWidth * VIEW_LIMITS.maxDpr);
  assert.equal(h.canvas.height, VIEW_LIMITS.maxHeight * VIEW_LIMITS.maxDpr);
  const boundedResize = h.calls.filter(([name]) => name === "resize").at(-1);
  assert.equal(boundedResize[0], "resize");
  assert.ok(Math.abs(Number.parseFloat(boundedResize[1]) - VIEW_LIMITS.maxWidth * 2 / 3) < 1e-9);
  assert.equal(boundedResize[2], `${VIEW_LIMITS.maxHeight * 2 / 3}px`);
  assert.equal(h.host.style.width, `${VIEW_LIMITS.maxWidth}px`);
  assert.equal(h.calls.filter(([name]) => name === "label").length, 2);
  await assert.rejects(async () => session.select([999]), (error) => error.code === "invalid-fixture");
  session.dispose();
  session.dispose();
  assert.equal(h.host.children.length, 0);
  assert.equal(h.host.style.width, undefined);
  assert.equal(h.calls.filter(([name]) => name === "stop").length, 1);
  assert.equal(session.snapshot().status, "disposed");
  await assert.rejects(async () => session.setLabels(true), (error) => error.code === "renderer-failed");
}));

test("crystal, orbital and trajectory use their dedicated local APIs", browser(async () => {
  for (const [kind, format, expected] of [["crystal", "cif", "unitCell"], ["orbital", "cube", "volume"], ["trajectory", "xyz", "frames"]]) {
    const h = harness();
    const extra = kind === "orbital" ? { gridPointCount: 8 } : kind === "trajectory" ? { frameCount: 2 } : { unitCell: [5, 5, 5, 90, 90, 90] };
    const session = await createThreeDMolAdapter(h.load).mount(h.host, { ...fixture, kind, format, ...extra }, { labelsVisible: false, reducedMotion: true });
    assert.ok(h.calls.some(([name]) => name === expected));
    if (kind === "orbital") assert.deepEqual(h.calls.filter(([name]) => name === "volume").map((row) => row[2]), [0.05, -0.05]);
    session.dispose();
  }
}));

test("WebGL and parser failures are explicit and clean created canvas nodes", browser(async () => {
  const absent = harness({ createViewerReturnsNull: true });
  await assert.rejects(createThreeDMolAdapter(absent.load).mount(absent.host, fixture, { labelsVisible: false, reducedMotion: false }),
    (error) => error.code === "renderer-unavailable");

  const broken = harness({ addModelThrows: true });
  await assert.rejects(createThreeDMolAdapter(broken.load).mount(broken.host, fixture, { labelsVisible: false, reducedMotion: false }),
    (error) => error.code === "renderer-failed" && error.cause?.message === "parser");
  assert.equal(broken.host.children.length, 0);
  assert.ok(broken.calls.some(([name]) => name === "clear"));
}));
