import {
  AdapterError,
  boundedViewport,
  validateFixture,
  type CandidateAdapter,
  type ViewerFixture,
  type ViewerSession,
  type ViewerSnapshot,
} from "./adapter";

const MAX_SOURCE_CHARACTERS = 8 * 1024 * 1024;
const SUPPORTED_FORMATS = new Map([
  ["molecule", new Set(["mol", "mol2", "sdf", "xyz"])],
  ["crystal", new Set(["cif", "mmcif", "mcif"])],
  ["protein", new Set(["pdb", "mmcif", "cif"])],
  ["orbital", new Set(["cube", "cub"])],
  ["trajectory", new Set(["xyz"])],
]);

const DISABLED_EXTENSIONS = [
  "mp4-export", "backgrounds", "mvs", "g3d", "dnatco-ntcs", "geo-export",
  "model-export", "pdbe-structure-quality-report", "sb-ncbr-partial-charges",
  "wwpdb-chemical-component-dictionary", "zenodo-import", "kinemage",
  "debug-helpers", "assembly-symmetry", "rcsb-validation-report",
  "anvil-membrane-orientation", "ma-quality-assessment", "tunnels",
];

type MolstarViewer = {
  plugin: any;
  loadFiles(files: File[]): Promise<void>;
  handleResize(): void;
  structureInteractivity(options: any): void;
  dispose(): void;
};

type ViewerConstructor = {
  create(host: HTMLElement, options: Record<string, unknown>): Promise<MolstarViewer>;
};

export type MolstarLoader = () => Promise<{ Viewer: ViewerConstructor }>;

async function defaultLoader(): Promise<{ Viewer: ViewerConstructor }> {
  // Deliberately kept behind the browser/SSR guard. No Mol* code executes during SSR.
  const [module] = await Promise.all([
    import("molstar/lib/apps/viewer/app"),
    import("molstar/lib/mol-plugin-ui/skin/light.scss"),
  ]);
  return module as { Viewer: ViewerConstructor };
}

function normalizedFormat(fixture: ViewerFixture): string {
  return fixture.format.trim().toLowerCase().replace(/^\./, "");
}

function ensureSupported(fixture: ViewerFixture): string {
  const format = normalizedFormat(fixture);
  if (!SUPPORTED_FORMATS.get(fixture.kind)?.has(format)) {
    throw new AdapterError(
      "unsupported-fixture",
      `Mol* spike does not support local ${fixture.kind} fixtures in ${fixture.format} format.`,
    );
  }
  // A DCD/XTC-like coordinates file without topology must never be presented as supported.
  if (fixture.kind === "trajectory" && format !== "xyz") {
    throw new AdapterError("unsupported-fixture", "Mol* trajectories require a paired topology; this spike accepts local multi-model XYZ only.");
  }
  return format;
}

function filenameFor(fixture: ViewerFixture, format: string): string {
  const safeId = fixture.id.replace(/[^a-zA-Z0-9_-]/g, "_").slice(0, 80) || "fixture";
  return `${safeId}.${format}`;
}

async function setMolstarLabels(viewer: MolstarViewer, refs: string[], visible: boolean): Promise<void> {
  if (visible && refs.length === 0) {
    const components = viewer.plugin?.managers?.structure?.hierarchy?.current?.structures
      ?.flatMap((structure: any) => structure.components ?? []) ?? [];
    for (const component of components) {
      const selector = await viewer.plugin.builders.structure.representation.addRepresentation(component.cell, {
        type: "label",
        typeParams: { level: "element" },
      });
      if (selector?.ref) refs.push(selector.ref);
    }
  }
  for (const ref of refs) viewer.plugin?.state?.data?.updateCellState?.(ref, { isHidden: !visible });
}

export function createMolstarAdapter(loadMolstar: MolstarLoader = defaultLoader): CandidateAdapter {
  return {
    id: "molstar",
    label: "Mol* 5.11.0 (spike only)",
    supports(kind) {
      return SUPPORTED_FORMATS.has(kind);
    },
    async mount(host, fixture, options): Promise<ViewerSession> {
      validateFixture(fixture);
      const format = ensureSupported(fixture);
      if (fixture.text.length > MAX_SOURCE_CHARACTERS) {
        throw new AdapterError("resource-limit", "Fixture source exceeds the 8 MiB spike input bound.");
      }
      if (typeof window === "undefined" || typeof document === "undefined" || typeof File === "undefined") {
        throw new AdapterError("renderer-unavailable", "Mol* is browser-only and cannot mount during SSR.");
      }
      if (!(host instanceof HTMLElement) || !host.isConnected) {
        throw new AdapterError("renderer-unavailable", "Mol* requires a connected HTML host.");
      }

      const root = document.createElement("div");
      root.dataset.viewerCandidate = "molstar";
      root.style.cssText = "position:relative;overflow:hidden;width:1px;height:1px;max-width:1280px;max-height:960px";
      host.append(root);

      let viewer: MolstarViewer | undefined;
      let disposed = false;
      let labelsVisible = false;
      let selectedAtomIds: number[] = [];
      let viewport = boundedViewport(1, 1, 1);
      const labelRefs: string[] = [];

      try {
        const { Viewer } = await loadMolstar();
        viewer = await Viewer.create(root, {
          disabledExtensions: DISABLED_EXTENSIONS,
          layoutIsExpanded: false,
          layoutShowControls: false,
          layoutShowRemoteState: false,
          layoutShowSequence: false,
          layoutShowLog: false,
          layoutShowLeftPanel: false,
          viewportShowExpand: false,
          viewportShowToggleFullscreen: false,
          viewportShowAnimation: false,
          viewportShowTrajectoryControls: false,
          viewportShowScreenshotControls: false,
          volumeStreamingDisabled: true,
          pluginStateServer: "",
          volumeStreamingServer: "",
          resolutionMode: "scaled",
          pixelScale: 1,
          pickScale: 0.25,
          disableAntialiasing: true,
          powerPreference: "low-power",
        });
        const localFile = new File([fixture.text], filenameFor(fixture, format), { type: "text/plain" });
        await viewer.loadFiles([localFile]);
        labelsVisible = options.labelsVisible;
        await setMolstarLabels(viewer, labelRefs, labelsVisible);
        if (options.reducedMotion) await viewer.plugin?.managers?.animation?.stop?.();
      } catch (error) {
        try { viewer?.dispose(); } finally { root.remove(); }
        if (error instanceof AdapterError) throw error;
        throw new AdapterError("renderer-failed", `Mol* failed to load local fixture ${fixture.id}.`, { cause: error });
      }

      const requireLive = (): MolstarViewer => {
        if (disposed || !viewer) throw new AdapterError("renderer-failed", "Mol* session is disposed.");
        return viewer;
      };

      return {
        async setLabels(visible) {
          await setMolstarLabels(requireLive(), labelRefs, visible);
          labelsVisible = visible;
        },
        select(atomIds) {
          const unique = [...new Set(atomIds)];
          if (unique.some((id) => !Number.isSafeInteger(id) || !fixture.atoms.some((atom) => atom.id === id))) {
            throw new AdapterError("invalid-fixture", "Selection contains an atom outside the local fixture.");
          }
          const active = requireLive();
          active.structureInteractivity({ action: "select" });
          if (unique.length) {
            active.structureInteractivity({
              action: "select",
              expression: (Q: any) => Q.struct.generator.atomGroups({
                // Semantic fixture ids are zero-based input-record indices.
                "atom-test": Q.core.set.has([Q.set(...unique), Q.acp("sourceIndex")]),
              }),
            });
          }
          selectedAtomIds = unique.sort((a, b) => a - b);
        },
        resize(width, height, dpr) {
          viewport = boundedViewport(width, height, dpr);
          root.style.width = `${viewport.width}px`;
          root.style.height = `${viewport.height}px`;
          requireLive().handleResize();
        },
        snapshot(): ViewerSnapshot {
          return {
            candidate: "molstar", fixture: fixture.id, selectedAtomIds: [...selectedAtomIds],
            labelsVisible, ...viewport, status: disposed ? "disposed" : "ready",
          };
        },
        dispose() {
          if (disposed) return;
          disposed = true;
          try { viewer?.dispose(); } finally { viewer = undefined; root.remove(); }
        },
      };
    },
  };
}

export const molstarAdapter = createMolstarAdapter();
