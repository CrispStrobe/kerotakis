import { describe, expect, it, vi } from "vitest";
import { readFileSync } from "node:fs";
import { AdapterError, boundedViewport, validateFixture, type CandidateAdapter, type ViewerFixture } from "./adapter";
import { ComparisonController } from "./comparison";
import type { ThreeDMolLoader } from "./3dmolAdapter";

const fixture: ViewerFixture = { id: "water", kind: "molecule", format: "sdf", text: "fixture", description: "Water", atoms: [{ id: 0, element: "O", x: 0, y: 0, z: 0 }], bonds: [] };

// Updated host mock with nested DOM support for per-generation containers
const host = () => {
  const h: any = {
    children: [] as any[],
    style: { width: "", height: "" },
    clientWidth: 500,
    clientHeight: 300,
    getBoundingClientRect: () => ({ width: 500, height: 300 } as any),
    appendChild(c: any) {
      this.children.push(c);
      c.host = this;
      c.isConnected = true;
    },
    replaceChildren() { this.children.splice(0); },
    querySelectorAll(s: string) {
      if (s !== "canvas") return this.children;
      const canvases: any[] = [];
      const walk = (node: any) => {
        for (const child of node.children || []) {
          if (child.id && String(child.id).includes("canvas")) canvases.push(child);
          if (child.children) walk(child);
        }
      };
      walk(this);
      return canvases;
    },
  };
  return h as unknown as HTMLElement;
};

function deferred<T = void>() {
  let resolve!: (v: T) => void;
  let reject!: (e: unknown) => void;
  const promise = new Promise<T>((res, rej) => { resolve = res; reject = rej; });
  return { promise, resolve, reject };
}

// Ensure document mock for per-generation container path
function ensureDocument() {
  if (typeof (globalThis as any).document === "undefined" || typeof (globalThis as any).document.createElement !== "function") {
    (globalThis as any).document = {
      createElement(tag: string) {
        const el: any = {
          tagName: tag.toUpperCase(),
          children: [] as any[],
          style: { width: "", height: "" } as any,
          dataset: {} as any,
          clientWidth: 500,
          clientHeight: 300,
          getBoundingClientRect: () => ({ width: 500, height: 300 } as any),
          appendChild(c: any) { this.children.push(c); c.host = this; c.isConnected = true; },
          querySelectorAll(s: string) { return s === "canvas" ? this.children : []; },
          remove() {
            if ((this as any).host) {
              const h: any = (this as any).host;
              const idx = h.children.indexOf(this);
              if (idx >= 0) h.children.splice(idx, 1);
            }
            (this as any).isConnected = false;
          },
          isConnected: true,
        };
        return el as unknown as HTMLElement;
      },
    };
  }
  if (typeof (globalThis as any).window === "undefined") (globalThis as any).window = { devicePixelRatio: 1 };
  if (typeof (globalThis as any).devicePixelRatio === "undefined") (globalThis as any).devicePixelRatio = 1;
}

describe("comparison contract", () => {
  it("bounds hostile viewports", () => expect(boundedViewport(Infinity, 50_000, 9)).toEqual({ width: 1, height: 960, dpr: 2 }));
  it("rejects dangling bonds", () => expect(() => validateFixture({ ...fixture, bonds: [{ from: 0, to: 1 }] })).toThrow(AdapterError));
  it("rejects hostile coordinates, bond orders and unit-cell angles", () => {
    expect(() => validateFixture({ ...fixture, atoms: [{ ...fixture.atoms[0], x: 1_000_001 }] })).toThrow(/finite coordinates/);
    expect(() => validateFixture({ ...fixture, atoms: [...fixture.atoms, { id: 1, element: "H", x: 0, y: 0, z: 0 }], bonds: [{ from: 0, to: 1, order: 9 }] })).toThrow(/bond/);
    expect(() => validateFixture({ ...fixture, unitCell: [1, 1, 1, 90, 180, 90] })).toThrow(/Unit-cell/);
  });
  it("rejects oversized source input before a renderer sees it", () => expect(() => validateFixture({ ...fixture, text: "x".repeat(2_000_001) })).toThrow(/source-byte limit/));
  it("reports unsupported fixtures without mounting", async () => {
    ensureDocument();
    const updates = vi.fn();
    const adapter: CandidateAdapter = { id: "test", label: "Test", supports: () => false, mount: vi.fn() };
    const controller = new ComparisonController(host(), updates);
    await controller.show(adapter, fixture, true);
    expect(controller.status.state).toBe("unsupported");
    expect(adapter.mount).not.toHaveBeenCalled();
  });
  it("disposes the prior renderer before replacement", async () => {
    ensureDocument();
    const dispose = vi.fn();
    const adapter: CandidateAdapter = { id: "test", label: "Test", supports: () => true, mount: vi.fn(async () => ({ setLabels() {}, select() {}, resize() {}, snapshot: () => ({ candidate: "test", fixture: "water", selectedAtomIds: [], labelsVisible: false, width: 1, height: 1, dpr: 1, status: "ready" }), dispose })) };
    const controller = new ComparisonController(host(), () => {});
    await controller.show(adapter, fixture, false);
    await controller.show(adapter, fixture, false);
    expect(dispose).toHaveBeenCalledTimes(1);
  });

  it("keeps the semantic table, keyboard-native choices and reduced-motion control in the route", () => {
    const source = readFileSync(new URL("./App.svelte", import.meta.url), "utf8");
    const styles = readFileSync(new URL("./style.css", import.meta.url), "utf8");
    expect(source).toContain('id="semantic-view"');
    expect(source).toContain("<table>");
    expect(source).toContain('type="radio" name="candidate"');
    expect(source).toContain('id="reduce-motion"');
    expect(source).toContain("neither authoritative chemistry");
    expect(styles).toContain("prefers-reduced-motion: reduce");
  });

  // Lifecycle race regressions - deterministic pure Node with real controller

  it("stale 3dmol load does not dispose new canvas", async () => {
    ensureDocument();
    const h: any = host();
    const gates = [deferred<Awaited<ReturnType<ThreeDMolLoader>>>(), deferred<Awaited<ReturnType<ThreeDMolLoader>>>()];
    const started = [deferred<void>(), deferred<void>()];
    let n = 0;
    // Use real 3dmolAdapter with deferred loader
    const { createThreeDMolAdapter } = await import("./3dmolAdapter");
    function canvas(id: string) { return { id, style: {} as any, isConnected: true, remove() { if ((this as any).host) { const idx = (this as any).host.children.indexOf(this); if (idx >= 0) (this as any).host.children.splice(idx, 1); } } } as unknown as HTMLElement; }
    function moduleFor(id: string): Awaited<ReturnType<ThreeDMolLoader>> {
      return {
        createViewer(h: any) {
          const c: any = canvas(id);
          h.appendChild(c);
          const model = { setStyle() {}, setClickable() {} };
          return { addModel: () => model, addModelsAsFrames: () => model,
            addUnitCell() {}, addVolumetricData() {}, addLabel() {}, getView() {},
            resize() {}, render() {}, zoomTo() {}, stopAnimate() {}, clear() {}, removeAllLabels() {} };
        },
      };
    }
    const adapter = createThreeDMolAdapter(() => {
      const i = n++;
      started[i].resolve();
      return gates[i].promise;
    });
    const c = new ComparisonController(h, () => {});
    const old = c.show(adapter, { ...fixture, id: "old", description: "old" }, true);
    await started[0].promise;
    const current = c.show(adapter, { ...fixture, id: "current", description: "current" }, true);
    await started[1].promise;
    gates[1].resolve(moduleFor("current-canvas"));
    await current;
    expect(c.status.state).toBe("ready");
    expect(h.querySelectorAll("canvas").length).toBe(1);
    expect(h.querySelectorAll("canvas")[0].id).toBe("current-canvas");
    gates[0].resolve(moduleFor("old-canvas"));
    await old;
    expect(c.status.state).toBe("ready");
    expect(c.snapshot()?.fixture).toBe("current");
    expect(h.querySelectorAll("canvas").length).toBe(1);
    expect(h.querySelectorAll("canvas")[0].id).toBe("current-canvas");
  });

  it("stale show after dispose await does not clear current", async () => {
    ensureDocument();
    const h: any = host();
    const gate = deferred<void>();
    const started = deferred<void>();
    const mounts: string[] = [];
    const c = new ComparisonController(h, () => {});
    const initial: CandidateAdapter = {
      id: "initial",
      label: "initial",
      supports: () => true,
      mount: async (hostEl: any) => {
        const cont: any = document.createElement("div");
        const canvas: any = document.createElement("canvas");
        canvas.id = "session-canvas";
        cont.appendChild(canvas);
        hostEl.appendChild(cont);
        return {
          setLabels() {},
          select() {},
          resize() {},
          snapshot: () => ({ candidate: "stub", fixture: "initial", selectedAtomIds: [], labelsVisible: false, width: 500, height: 300, dpr: 1, status: "ready" as const }),
          dispose() { started.resolve(); return gate.promise; },
        } as any;
      },
    };
    await c.show(initial, fixture, true);
    const a: CandidateAdapter = {
      id: "a",
      label: "a",
      supports: () => true,
      mount: async (hostEl: any) => {
        mounts.push("stale");
        const cont: any = document.createElement("div");
        const canvas: any = document.createElement("canvas");
        canvas.id = "session-canvas";
        cont.appendChild(canvas);
        hostEl.appendChild(cont);
        return {
          setLabels() {},
          select() {},
          resize() {},
          snapshot: () => ({ candidate: "stub", fixture: "stale", selectedAtomIds: [], labelsVisible: false, width: 500, height: 300, dpr: 1, status: "ready" as const }),
          dispose() { try { cont.remove(); } catch {} },
        } as any;
      },
    };
    const b: CandidateAdapter = {
      id: "b",
      label: "b",
      supports: () => true,
      mount: async (hostEl: any) => {
        mounts.push("current");
        const cont: any = document.createElement("div");
        const canvas: any = document.createElement("canvas");
        canvas.id = "session-canvas";
        cont.appendChild(canvas);
        hostEl.appendChild(cont);
        return {
          setLabels() {},
          select() {},
          resize() {},
          snapshot: () => ({ candidate: "stub", fixture: "current", selectedAtomIds: [], labelsVisible: false, width: 500, height: 300, dpr: 1, status: "ready" as const }),
          dispose() { try { cont.remove(); } catch {} },
        } as any;
      },
    };
    const stale = c.show(a, { ...fixture, id: "stale", description: "stale" }, true);
    await started.promise;
    await c.show(b, { ...fixture, id: "current", description: "current" }, true);
    expect(c.snapshot()?.fixture).toBe("current");
    gate.resolve();
    await stale;
    expect(mounts).toEqual(["current"]);
    expect(c.snapshot()?.fixture).toBe("current");
    expect(h.querySelectorAll("canvas").length).toBe(1);
    expect(c.status.state).toBe("ready");
    expect(c.status.message).toMatch(/current is ready in b/);
  });

  it("stale resize does not publish old ready", async () => {
    ensureDocument();
    const h: any = host();
    const gate = deferred<void>();
    const started = deferred<void>();
    const updates: string[] = [];
    const c = new ComparisonController(h, (s) => updates.push(s.message));
    const a: CandidateAdapter = {
      id: "a",
      label: "a",
      supports: () => true,
      mount: async (hostEl: any) => {
        const cont: any = { children: [] as any[], style: {} as any, isConnected: true, remove() {} };
        hostEl.appendChild(cont);
        return {
          setLabels() {},
          select() {},
          resize() { started.resolve(); return gate.promise; },
          snapshot: () => ({ candidate: "stub", fixture: "old", selectedAtomIds: [], labelsVisible: false, width: 500, height: 300, dpr: 1, status: "ready" as const }),
          dispose() {},
        } as any;
      },
    };
    const b: CandidateAdapter = {
      id: "b",
      label: "b",
      supports: () => true,
      mount: async (hostEl: any) => {
        const cont: any = { children: [] as any[], style: {} as any, isConnected: true, remove() {} };
        hostEl.appendChild(cont);
        return {
          setLabels() {},
          select() {},
          resize() {},
          snapshot: () => ({ candidate: "stub", fixture: "current", selectedAtomIds: [], labelsVisible: false, width: 500, height: 300, dpr: 1, status: "ready" as const }),
          dispose() {},
        } as any;
      },
    };
    const old = c.show(a, { ...fixture, id: "old", description: "old" }, true);
    await started.promise;
    await c.show(b, { ...fixture, id: "current", description: "current" }, true);
    expect(c.snapshot()?.fixture).toBe("current");
    expect(c.status.message).toMatch(/current is ready in b/);
    gate.resolve();
    await old;
    expect(c.status.message).toMatch(/current is ready in b/);
    expect(c.snapshot()?.fixture).toBe("current");
  });

  it("failed mount does not leave detached session and cleans up", async () => {
    ensureDocument();
    const h: any = host();
    const c = new ComparisonController(h, () => {});
    const failing: CandidateAdapter = {
      id: "fail",
      label: "fail",
      supports: () => true,
      mount: async () => { throw new Error("mount failed"); },
    };
    await c.show(failing, fixture, true);
    expect(c.status.state).toBe("error");
    expect(c.snapshot()).toBeNull();
    expect(h.children.length).toBe(0);
    // Next show should succeed
    const ok: CandidateAdapter = {
      id: "ok",
      label: "ok",
      supports: () => true,
      mount: async (hostEl: any) => {
        const cont: any = { children: [] as any[], style: {} as any, isConnected: true, remove() { const idx = h.children.indexOf(this); if (idx >= 0) h.children.splice(idx, 1); } };
        hostEl.appendChild(cont);
        return {
          setLabels() {},
          select() {},
          resize() {},
          snapshot: () => ({ candidate: "ok", fixture: "water", selectedAtomIds: [], labelsVisible: false, width: 500, height: 300, dpr: 1, status: "ready" as const }),
          dispose() {},
        } as any;
      },
    };
    await c.show(ok, fixture, true);
    expect(c.status.state).toBe("ready");
    expect(c.snapshot()?.fixture).toBe("water");
  });
});


function sessionAdapter(name: string, hooks: {
  mounting?: ReturnType<typeof deferred<void>>;
  mounted?: ReturnType<typeof deferred<void>>;
  resize?: () => void | Promise<void>;
  dispose?: () => void | Promise<void>;
} = {}): CandidateAdapter {
  return {
    id: name, label: name, supports: () => true,
    async mount(container) {
      hooks.mounted?.resolve();
      if (hooks.mounting) await hooks.mounting.promise;
      const canvas = document.createElement("canvas");
      canvas.id = `${name}-canvas`;
      container.appendChild(canvas);
      return {
        setLabels() {}, select() {}, resize: hooks.resize ?? (() => {}),
        snapshot: () => ({ candidate: name, fixture: name, selectedAtomIds: [], labelsVisible: false,
          width: 500, height: 300, dpr: 1, status: "ready" as const }),
        dispose() { canvas.remove(); return hooks.dispose?.(); },
      };
    },
  };
}

describe("viewer error ownership", () => {
  it("publishes a current resize error before slow disposal and preserves a newer ready", async () => {
    ensureDocument();
    const h = host();
    const disposing = deferred<void>();
    const release = deferred<void>();
    const c = new ComparisonController(h, () => {});
    const old = c.show(sessionAdapter("old", {
      resize: () => Promise.reject(new Error("resize failure")),
      dispose: () => { disposing.resolve(); return release.promise; },
    }), { ...fixture, id: "old", description: "old" }, true);
    await disposing.promise;
    expect(c.status.state).toBe("error");
    await c.show(sessionAdapter("current"), { ...fixture, id: "current", description: "current" }, true);
    release.resolve();
    await old;
    expect(c.status.state).toBe("ready");
    expect(c.status.message).toMatch(/current is ready/);
    expect(c.snapshot()?.fixture).toBe("current");
    expect(h.querySelectorAll("canvas").length).toBe(1);
  });

  for (const failure of ["creation", "append"] as const) {
    it(`does not publish a stale ${failure} error over a newer loading`, async () => {
      ensureDocument();
      const h = host();
      const c = new ComparisonController(h, () => {});
      const release = deferred<void>();
      const create = document.createElement;
      const append = h.appendChild;
      let old = Promise.resolve();
      try {
        if (failure === "creation") document.createElement = (() => { throw new Error("setup failure"); }) as typeof create;
        else h.appendChild = (() => { throw new Error("setup failure"); }) as typeof append;
        old = c.show(sessionAdapter("failed"), fixture, true);
      } finally {
        document.createElement = create;
        h.appendChild = append;
      }
      const current = c.show(sessionAdapter("current", { mounting: release }),
        { ...fixture, id: "current", description: "current" }, true);
      await old;
      expect(c.status.state).toBe("loading");
      expect(c.status.message).toMatch(/current/);
      release.resolve();
      await current;
      expect(c.status.state).toBe("ready");
    });
  }

  it("invalidates an older pending mount when a newer container cannot be created", async () => {
    ensureDocument();
    const h = host();
    const c = new ComparisonController(h, () => {});
    const mounted = deferred<void>();
    const release = deferred<void>();
    const old = c.show(sessionAdapter("old", { mounting: release, mounted }), fixture, true);
    await mounted.promise;
    const create = document.createElement;
    try {
      document.createElement = (() => { throw new Error("creation failure"); }) as typeof create;
      await c.show(sessionAdapter("failed"), fixture, true);
    } finally { document.createElement = create; }
    release.resolve();
    await old;
    expect(c.status.state).toBe("error");
    expect(c.status.message).toMatch(/creation failure/);
    expect(c.snapshot()).toBeNull();
    expect(h.querySelectorAll("canvas").length).toBe(0);
  });
});
