import assert from "node:assert/strict";
import test from "node:test";
import { AdapterError, VIEW_LIMITS, type ViewerFixture } from "./adapter";
import { createMolstarAdapter } from "./molstarAdapter";

const fixture: ViewerFixture = {
  id: "water", kind: "molecule", format: "sdf", text: "local fixture", description: "water",
  atoms: [{ id: 1, element: "O", x: 0, y: 0, z: 0 }, { id: 2, element: "H", x: 1, y: 0, z: 0 }],
  bonds: [{ from: 1, to: 2 }],
};

function installDom() {
  class Element {
    isConnected = true;
    style: any = { cssText: "" };
    dataset: Record<string, string> = {};
    children: Element[] = [];
    append(child: Element) { this.children.push(child); }
    remove() { this.isConnected = false; }
  }
  class LocalFile { constructor(public parts: unknown[], public name: string, public options: unknown) {} }
  Object.assign(globalThis, { window: {}, document: { createElement: () => new Element() }, HTMLElement: Element, File: LocalFile });
  return Element;
}

test("fails closed during SSR before importing Mol*", async () => {
  const previous = { window: globalThis.window, document: globalThis.document, File: globalThis.File };
  // @ts-expect-error deliberate SSR simulation
  delete globalThis.window; delete globalThis.document; delete globalThis.File;
  let imports = 0;
  await assert.rejects(createMolstarAdapter(async () => { imports++; throw new Error(); }).mount({} as HTMLElement, fixture, { labelsVisible: false, reducedMotion: true }),
    (error: unknown) => error instanceof AdapterError && error.code === "renderer-unavailable");
  assert.equal(imports, 0);
  Object.assign(globalThis, previous);
});

test("uses local File input, bounds resize, selects, labels, snapshots and disposes", async () => {
  const Element = installDom();
  const calls: any[] = [];
  const fakeViewer: any = {
    plugin: {
      managers: { structure: { hierarchy: { current: { structures: [{ components: [{ cell: "component" }] }] } } }, animation: { stop: async () => calls.push("stop") } },
      builders: { structure: { representation: { addRepresentation: async (...args: any[]) => { calls.push(["label", ...args]); return { ref: "label-1" }; } } } },
      state: { data: { updateCellState: (...args: any[]) => calls.push(["label-state", ...args]) } },
    },
    loadFiles: async (files: any[]) => calls.push(["files", files]),
    structureInteractivity: (value: any) => calls.push(["select", value]),
    handleResize: () => calls.push("resize"),
    dispose: () => calls.push("dispose"),
  };
  const adapter = createMolstarAdapter(async () => ({ Viewer: { create: async (_host, options) => { calls.push(["create", options]); return fakeViewer; } } }));
  const host = new Element() as unknown as HTMLElement;
  const session = await adapter.mount(host, fixture, { labelsVisible: true, reducedMotion: true });
  const createOptions = calls.find((call) => Array.isArray(call) && call[0] === "create")[1];
  assert.equal(createOptions.volumeStreamingDisabled, true);
  assert.equal(createOptions.layoutShowRemoteState, false);
  assert.equal(createOptions.pluginStateServer, "");
  assert.equal(calls.find((call) => Array.isArray(call) && call[0] === "files")[1][0].name, "water.sdf");
  await session.select([2, 1, 2]);
  await session.resize(Infinity, 50_000, 9);
  assert.deepEqual(session.snapshot(), { candidate: "molstar", fixture: "water", selectedAtomIds: [1, 2], labelsVisible: true, width: 1, height: VIEW_LIMITS.maxHeight, dpr: VIEW_LIMITS.maxDpr, status: "ready" });
  await session.setLabels(false);
  await session.dispose(); await session.dispose();
  assert.equal(session.snapshot().status, "disposed");
  assert.equal(calls.filter((call) => call === "dispose").length, 1);
});

test("rejects unsupported formats and oversized local input before import", async () => {
  installDom();
  let imports = 0;
  const adapter = createMolstarAdapter(async () => { imports++; throw new Error("should not import"); });
  const host = new HTMLElement();
  await assert.rejects(adapter.mount(host, { ...fixture, kind: "trajectory", format: "dcd" }, { labelsVisible: false, reducedMotion: false }),
    (error: unknown) => error instanceof AdapterError && error.code === "unsupported-fixture");
  await assert.rejects(adapter.mount(host, { ...fixture, text: "x".repeat(8 * 1024 * 1024 + 1) }, { labelsVisible: false, reducedMotion: false }),
    (error: unknown) => error instanceof AdapterError && error.code === "resource-limit");
  assert.equal(imports, 0);
});
