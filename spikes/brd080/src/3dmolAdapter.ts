import {
  AdapterError,
  boundedViewport,
  validateFixture,
  type CandidateAdapter,
  type FixtureKind,
  type ViewerFixture,
  type ViewerSession,
  type ViewerSnapshot,
} from "./adapter";

const MAX_SOURCE_BYTES = 8 * 1024 * 1024;
const MAX_LABELS = 128;
const SUPPORTED_FORMATS: Readonly<Record<FixtureKind, readonly string[]>> = Object.freeze({
  molecule: ["sdf", "mol", "mol2", "xyz"],
  crystal: ["cif"],
  protein: ["pdb"],
  orbital: ["cube"],
  trajectory: ["xyz"],
});

type ThreeDMolModule = {
  createViewer(host: HTMLElement, options?: Record<string, unknown>): ThreeDMolViewer | null;
};

type ThreeDMolModel = {
  setStyle(selection: Record<string, unknown>, style: Record<string, unknown>): void;
  setClickable(selection: Record<string, unknown>, clickable: boolean, callback: (atom: { index?: number }) => void): void;
};

type ThreeDMolViewer = {
  addModel(data: string, format: string, options?: Record<string, unknown>): ThreeDMolModel;
  addModelsAsFrames(data: string, format: string, options?: Record<string, unknown>): ThreeDMolModel;
  addUnitCell(model: ThreeDMolModel, options?: Record<string, unknown>): unknown;
  addVolumetricData(data: string, format: string, options: Record<string, unknown>): unknown;
  addLabel(text: string, options: Record<string, unknown>): unknown;
  removeAllLabels(): unknown;
  render(): unknown;
  zoomTo(): unknown;
  resize(): unknown;
  stopAnimate(): unknown;
  clear(): unknown;
  getView(): unknown;
};

export type ThreeDMolLoader = () => Promise<ThreeDMolModule>;

const defaultLoader: ThreeDMolLoader = async () => {
  // Deliberately browser-only and lazy: importing 3Dmol eagerly mutates the
  // window global and breaks server rendering.
  const module = await import("3dmol") as unknown as ThreeDMolModule & { default?: ThreeDMolModule };
  return typeof module.createViewer === "function" ? module : module.default as ThreeDMolModule;
};

function sourceBytes(text: string): number {
  return new TextEncoder().encode(text).byteLength;
}

function ensureSupported(fixture: ViewerFixture): string {
  const format = fixture.format.toLowerCase().replace(/^\./, "");
  if (!SUPPORTED_FORMATS[fixture.kind].includes(format)) {
    throw new AdapterError("unsupported-fixture", `3Dmol does not support ${fixture.kind} fixtures in ${fixture.format} format.`);
  }
  if (/^(?:https?|data|blob):/i.test(fixture.text.trimStart())) {
    throw new AdapterError("invalid-fixture", "3Dmol fixtures must contain local source text, not a URL.");
  }
  if (sourceBytes(fixture.text) > MAX_SOURCE_BYTES) {
    throw new AdapterError("resource-limit", "Fixture source exceeds the 8 MiB spike limit.");
  }
  return format;
}

function baseStyle(kind: FixtureKind): Record<string, unknown> {
  if (kind === "protein") return { cartoon: { color: "spectrum" }, stick: { radius: 0.12 } };
  return { stick: { radius: 0.18 }, sphere: { scale: 0.28 } };
}

function fail(code: "renderer-unavailable" | "renderer-failed", message: string, cause?: unknown): AdapterError {
  return new AdapterError(code, message, cause === undefined ? undefined : { cause });
}

export function createThreeDMolAdapter(load: ThreeDMolLoader = defaultLoader): CandidateAdapter {
  return {
    id: "3dmol",
    label: "3Dmol.js 2.5.5",
    supports: (kind) => Object.hasOwn(SUPPORTED_FORMATS, kind),
    async mount(host, fixture, options): Promise<ViewerSession> {
      validateFixture(fixture);
      const format = ensureSupported(fixture);
      if (typeof window === "undefined" || typeof document === "undefined") {
        throw fail("renderer-unavailable", "3Dmol is only available after browser mount.");
      }
      if (!host || typeof host.appendChild !== "function") {
        throw new AdapterError("invalid-fixture", "A browser host element is required.");
      }

      const originalChildren = new Set(Array.from(host.children));
      const originalWidth = host.style.width;
      const originalHeight = host.style.height;
      let viewport = boundedViewport(host.clientWidth || 640, host.clientHeight || 480, window.devicePixelRatio || 1);
      const prepareBoundedRendererSize = () => {
        // 3Dmol 2.5.5 always multiplies its logical size by the global DPR and
        // exposes no pixel-ratio setter. Give it a proportionally smaller
        // logical size, then CSS-scale its correctly bounded backing canvas.
        // Mutating canvas.width after resize would desynchronise its GL viewport.
        const actualDpr = Number.isFinite(window.devicePixelRatio) ? Math.max(1, window.devicePixelRatio) : 1;
        const rendererScale = Math.min(1, viewport.dpr / actualDpr);
        host.style.width = `${viewport.width * rendererScale}px`;
        host.style.height = `${viewport.height * rendererScale}px`;
      };
      prepareBoundedRendererSize();
      let viewer: ThreeDMolViewer | null = null;
      let model: ThreeDMolModel | null = null;
      try {
        const module = await load();
        if (typeof module?.createViewer !== "function") throw fail("renderer-unavailable", "The 3Dmol viewer factory is unavailable.");
        viewer = module.createViewer(host, { backgroundColor: "white", antialias: true });
        if (!viewer) throw fail("renderer-unavailable", "WebGL could not create a 3Dmol viewer.");

        if (fixture.kind === "trajectory") model = viewer.addModelsAsFrames(fixture.text, format);
        else model = viewer.addModel(fixture.text, format);
        if (!model) throw fail("renderer-failed", "3Dmol did not create a model for the fixture.");

        model.setStyle({}, baseStyle(fixture.kind));
        if (fixture.kind === "crystal") viewer.addUnitCell(model, { box: { color: "#57606a" } });
        if (fixture.kind === "orbital") {
          viewer.addVolumetricData(fixture.text, "cube", { isoval: 0.05, color: "#2563eb", opacity: 0.72 });
          viewer.addVolumetricData(fixture.text, "cube", { isoval: -0.05, color: "#dc2626", opacity: 0.72 });
        }
        viewer.zoomTo();
        viewer.render();
      } catch (error) {
        try { viewer?.stopAnimate(); viewer?.clear(); } catch { /* best-effort failed mount cleanup */ }
        for (const child of Array.from(host.children)) if (!originalChildren.has(child)) child.remove();
        host.style.width = originalWidth;
        host.style.height = originalHeight;
        if (error instanceof AdapterError) throw error;
        throw fail("renderer-failed", "3Dmol failed to parse or render the local fixture.", error);
      }

      let selectedAtomIds: number[] = [];
      let labelsVisible = false;
      let status: ViewerSnapshot["status"] = "ready";
      const atomIdToIndex = new Map(fixture.atoms.map((atom, index) => [atom.id, index]));
      const indexToAtomId = new Map(fixture.atoms.map((atom, index) => [index, atom.id]));

      const requireReady = () => {
        if (status === "disposed") throw fail("renderer-failed", "The 3Dmol session is disposed.");
      };
      const drawLabels = (visible: boolean) => {
        viewer!.removeAllLabels();
        labelsVisible = visible;
        if (visible) {
          for (const atom of fixture.atoms.slice(0, MAX_LABELS)) {
            viewer!.addLabel((atom.label || `${atom.element} ${atom.id}`).slice(0, 80), {
              position: { x: atom.x, y: atom.y, z: atom.z },
              fontSize: 12,
              backgroundOpacity: 0.75,
              inFront: true,
            });
          }
        }
        viewer!.render();
      };
      const applySelection = (ids: readonly number[]) => {
        const unique = [...new Set(ids)];
        if (unique.some((id) => !atomIdToIndex.has(id))) {
          throw new AdapterError("invalid-fixture", "Selection contains an atom id absent from the fixture.");
        }
        selectedAtomIds = unique.sort((a, b) => a - b);
        model!.setStyle({}, baseStyle(fixture.kind));
        if (unique.length) {
          model!.setStyle({ index: unique.map((id) => atomIdToIndex.get(id)!) }, {
            stick: { color: "#f59e0b", radius: 0.28 }, sphere: { color: "#f59e0b", scale: 0.42 },
          });
        }
        viewer!.render();
      };

      model.setClickable({}, true, (atom) => {
        const id = atom.index === undefined ? undefined : indexToAtomId.get(atom.index);
        if (id !== undefined && status === "ready") applySelection([id]);
      });

      const session: ViewerSession = {
        setLabels(visible) {
          requireReady();
          drawLabels(Boolean(visible));
        },
        select(atomIds) {
          requireReady();
          applySelection(atomIds);
        },
        resize(width, height, dpr) {
          requireReady();
          viewport = boundedViewport(width, height, dpr);
          prepareBoundedRendererSize();
          viewer!.resize();
          host.style.width = `${viewport.width}px`;
          host.style.height = `${viewport.height}px`;
          for (const canvas of Array.from(host.querySelectorAll("canvas"))) {
            canvas.style.width = `${viewport.width}px`;
            canvas.style.height = `${viewport.height}px`;
          }
          viewer!.render();
        },
        snapshot() {
          return {
            candidate: "3dmol",
            fixture: fixture.id,
            selectedAtomIds: [...selectedAtomIds],
            labelsVisible,
            ...viewport,
            status,
          };
        },
        dispose() {
          if (status === "disposed") return;
          status = "disposed";
          selectedAtomIds = [];
          try { viewer!.stopAnimate(); viewer!.clear(); } finally {
            for (const child of Array.from(host.children)) if (!originalChildren.has(child)) child.remove();
            host.style.width = originalWidth;
            host.style.height = originalHeight;
            viewer = null;
            model = null;
          }
        },
      };

      session.resize(viewport.width, viewport.height, viewport.dpr);
      if (options.labelsVisible) session.setLabels(true);
      // Reduced motion is honored by never starting 3Dmol's animation loop.
      void options.reducedMotion;
      return session;
    },
  };
}

export const threeDMolAdapter = createThreeDMolAdapter();
