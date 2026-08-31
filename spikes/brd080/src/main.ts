import "./style.css";
import { AdapterError, type CandidateAdapter, type SemanticAtom, type ViewerFixture } from "./adapter";
import { ComparisonController } from "./comparison";

type FixtureSeed = Omit<ViewerFixture, "text"> & { url: URL };

const atoms = (rows: Array<[string, number, number, number, string?]>): SemanticAtom[] => rows.map(([element, x, y, z, label], id) => ({ id, element, x, y, z, label }));
const fixtures: FixtureSeed[] = [
  { id: "water", kind: "molecule", format: "sdf", url: new URL("../fixtures/water.sdf", import.meta.url), description: "Project-authored water format probe", atoms: atoms([["O",0,0,0,"O"],["H",.9572,0,0,"H1"],["H",-.239,.927,0,"H2"]]), bonds: [{from:0,to:1,order:1},{from:0,to:2,order:1}] },
  { id: "nacl", kind: "crystal", format: "cif", url: new URL("../fixtures/nacl.cif", import.meta.url), description: "Project-authored sodium chloride unit-cell format probe", atoms: atoms([["Na",0,0,0,"Na1"],["Cl",.5,.5,.5,"Cl1"]]), bonds: [], unitCell: [5.6402,5.6402,5.6402,90,90,90] },
  { id: "peptide", kind: "protein", format: "pdb", url: new URL("../fixtures/peptide.pdb", import.meta.url), description: "Project-authored glycine-fragment format probe", atoms: atoms([["N",0,0,0,"N"],["C",1.45,0,0,"CA"],["C",2,1.41,0,"C"],["O",1.3,2.39,0,"O"]]), bonds: [] },
  { id: "orbital", kind: "orbital", format: "cube", url: new URL("../fixtures/orbital.cube", import.meta.url), description: "Synthetic volume-grid format probe; not a scientific orbital", atoms: atoms([["H",0,0,0,"H"]]), bonds: [], gridPointCount: 8 },
  { id: "trajectory", kind: "trajectory", format: "xyz", url: new URL("../fixtures/trajectory.xyz", import.meta.url), description: "Synthetic two-frame hydrogen trajectory format probe", atoms: atoms([["H",0,0,0,"H1"],["H",.7,0,0,"H2"]]), bonds: [{from:0,to:1,order:1}], frameCount: 2 },
];

const candidateLoads: Record<string, () => Promise<CandidateAdapter>> = {
  "3dmol": async () => candidateExport(await import("./3dmol"), "3dmol"),
  molstar: async () => candidateExport(await import("./molstar"), "molstar"),
};

function candidateExport(module: Record<string, unknown>, id: string): CandidateAdapter {
  const value = module.adapter ?? module.default;
  if (!value || typeof value !== "object" || !("mount" in value)) throw new AdapterError("renderer-unavailable", `${id} adapter is not implemented in this spike build.`);
  return value as CandidateAdapter;
}

function escape(value: string) { return value.replace(/[&<>"']/g, (character) => ({ "&":"&amp;", "<":"&lt;", ">":"&gt;", '"':"&quot;", "'":"&#39;" })[character]!); }

const root = document.querySelector<HTMLElement>("#app")!;
root.innerHTML = `
  <header><p class="eyebrow">Disposable decision spike · BRD-080</p><h1>Molecular viewer comparison</h1><p>This route compares presentation libraries. The visual canvas is neither authoritative chemistry nor the only way to inspect a fixture.</p></header>
  <section class="controls" aria-label="Comparison controls">
    <fieldset><legend>Candidate</legend><div class="choice-row">${Object.keys(candidateLoads).map((id, index) => `<label><input type="radio" name="candidate" value="${id}" ${index === 0 ? "checked" : ""}> ${id === "3dmol" ? "3Dmol.js" : "Mol*"}</label>`).join("")}</div></fieldset>
    <fieldset><legend>Fixture</legend><div class="choice-row">${fixtures.map((fixture, index) => `<label><input type="radio" name="fixture" value="${fixture.id}" ${index === 0 ? "checked" : ""}> ${escape(fixture.kind)}</label>`).join("")}</div></fieldset>
    <div class="toggles"><label><input id="labels" type="checkbox"> Visual labels</label><label><input id="reduce-motion" type="checkbox"> Reduce motion</label></div>
  </section>
  <p id="status" class="status" role="status" aria-live="polite">Preparing comparison…</p>
  <section class="visual-card" aria-labelledby="visual-title"><h2 id="visual-title">Optional visual rendering</h2><div id="viewer" aria-hidden="true"></div><p class="boundary">If WebGL or a format is unavailable, the explicit status above and semantic view below remain available.</p></section>
  <section id="semantic-view" class="semantic-card" aria-labelledby="semantic-title" tabindex="-1"><h2 id="semantic-title">Semantic fixture view</h2><p id="description"></p><dl id="facts"></dl><table><caption>Atoms supplied by the fixture</caption><thead><tr><th scope="col">Select</th><th scope="col">ID</th><th scope="col">Label</th><th scope="col">Element</th><th scope="col">x</th><th scope="col">y</th><th scope="col">z</th></tr></thead><tbody id="atoms"></tbody></table></section>`;

const host = document.querySelector<HTMLElement>("#viewer")!;
const status = document.querySelector<HTMLElement>("#status")!;
const controller = new ComparisonController(host, (next) => { status.dataset.state = next.state; status.textContent = next.message; });
(globalThis as typeof globalThis & { __brd080?: { snapshot: () => ReturnType<ComparisonController["snapshot"]>; resize: (width: number, height: number, dpr: number) => Promise<void>; select: (ids: readonly number[]) => Promise<void>; setLabels: (visible: boolean) => Promise<void> } }).__brd080 = {
  snapshot: () => controller.snapshot(),
  resize: (width, height, dpr) => controller.resize(width, height, dpr),
  select: (ids) => controller.select(ids),
  setLabels: (visible) => controller.setLabels(visible),
};
let currentFixture: ViewerFixture | null = null;
let loadGeneration = 0;

function checked(name: string): string { return document.querySelector<HTMLInputElement>(`input[name="${name}"]:checked`)!.value; }
function reducedMotion(): boolean { return document.querySelector<HTMLInputElement>("#reduce-motion")!.checked; }

async function materialize(seed: FixtureSeed): Promise<ViewerFixture> {
  const response = await fetch(seed.url);
  if (!response.ok) throw new AdapterError("invalid-fixture", `Could not load fixture ${seed.id} (${response.status}).`);
  return { ...seed, text: await response.text() };
}

function renderSemantics(fixture: ViewerFixture) {
  document.querySelector<HTMLElement>("#description")!.textContent = fixture.description;
  document.querySelector<HTMLElement>("#facts")!.innerHTML = `<div><dt>Kind</dt><dd>${escape(fixture.kind)}</dd></div><div><dt>Format</dt><dd>${escape(fixture.format.toUpperCase())}</dd></div><div><dt>Atoms</dt><dd>${fixture.atoms.length}</dd></div><div><dt>Frames</dt><dd>${fixture.frameCount ?? 1}</dd></div>`;
  document.querySelector<HTMLElement>("#atoms")!.innerHTML = fixture.atoms.map((atom) => `<tr><td><input type="checkbox" data-atom="${atom.id}" aria-label="Select atom ${atom.id}: ${escape(atom.label ?? atom.element)}"></td><th scope="row">${atom.id}</th><td>${escape(atom.label ?? "—")}</td><td>${escape(atom.element)}</td><td>${atom.x}</td><td>${atom.y}</td><td>${atom.z}</td></tr>`).join("");
}

async function show() {
  const generation = ++loadGeneration;
  status.dataset.state = "loading";
  status.textContent = "Loading local fixture and renderer…";
  try {
    const fixture = await materialize(fixtures.find(({ id }) => id === checked("fixture"))!);
    if (generation !== loadGeneration) return;
    currentFixture = fixture;
    renderSemantics(fixture);
    const adapter = await candidateLoads[checked("candidate")]();
    if (generation !== loadGeneration) return;
    await controller.show(adapter, fixture, reducedMotion());
  } catch (error) {
    status.dataset.state = "error";
    status.textContent = error instanceof Error ? error.message : "The comparison could not be loaded.";
  }
}

document.querySelectorAll<HTMLInputElement>('input[name="candidate"], input[name="fixture"]').forEach((input) => input.addEventListener("change", show));
document.querySelector<HTMLInputElement>("#labels")!.addEventListener("change", (event) => controller.setLabels((event.currentTarget as HTMLInputElement).checked));
document.querySelector<HTMLInputElement>("#reduce-motion")!.checked = matchMedia("(prefers-reduced-motion: reduce)").matches;
document.querySelector<HTMLInputElement>("#reduce-motion")!.addEventListener("change", show);
document.querySelector<HTMLElement>("#atoms")!.addEventListener("change", () => {
  if (!currentFixture) return;
  const ids = [...document.querySelectorAll<HTMLInputElement>("[data-atom]:checked")].map((input) => Number(input.dataset.atom));
  controller.select(ids);
});
const observer = new ResizeObserver(([entry]) => controller.resize(entry.contentRect.width, entry.contentRect.height, devicePixelRatio));
observer.observe(host);
addEventListener("pagehide", () => { observer.disconnect(); void controller.dispose(); }, { once: true });
void show();
