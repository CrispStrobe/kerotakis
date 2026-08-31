<script lang="ts">
  import { onMount } from "svelte";
  import { AdapterError, type CandidateAdapter, type SemanticAtom, type ViewerFixture } from "./adapter";
  import { ComparisonController, type ComparisonStatus } from "./comparison";

  type FixtureSeed = Omit<ViewerFixture, "text"> & { url: URL };
  const atoms = (rows: Array<[string, number, number, number, string?]>): SemanticAtom[] => rows.map(([element, x, y, z, label], id) => ({ id, element, x, y, z, label }));
  const fixtures: readonly FixtureSeed[] = [
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
  async function materialize(seed: FixtureSeed): Promise<ViewerFixture> {
    const response = await fetch(seed.url);
    if (!response.ok) throw new AdapterError("invalid-fixture", `Could not load fixture ${seed.id} (${response.status}).`);
    return { ...seed, text: await response.text() };
  }

  let host: HTMLElement;
  let controller: ComparisonController | null = null;
  let candidate = "3dmol";
  let fixtureId = "water";
  let labelsVisible = false;
  let reducedMotion = false;
  let currentFixture: ViewerFixture | null = null;
  let selectedAtomIds: number[] = [];
  let status: ComparisonStatus = { state: "idle", message: "Preparing comparison…" };
  let loadGeneration = 0;

  async function show() {
    if (!controller) return;
    const generation = ++loadGeneration;
    status = { state: "loading", message: "Loading local fixture and renderer…" };
    try {
      const fixture = await materialize(fixtures.find(({ id }) => id === fixtureId)!);
      if (generation !== loadGeneration) return;
      currentFixture = fixture;
      selectedAtomIds = [];
      labelsVisible = false;
      const adapter = await candidateLoads[candidate]();
      if (generation !== loadGeneration) return;
      await controller.show(adapter, fixture, reducedMotion);
    } catch (error) {
      if (generation !== loadGeneration) return;
      status = { state: "error", message: error instanceof Error ? error.message : "The comparison could not be loaded." };
    }
  }

  onMount(() => {
    controller = new ComparisonController(host, (next) => { status = next; });
    reducedMotion = matchMedia("(prefers-reduced-motion: reduce)").matches;
    const observer = new ResizeObserver(([entry]) => controller?.resize(entry.contentRect.width, entry.contentRect.height, devicePixelRatio));
    observer.observe(host);
    const bridge = { snapshot: () => controller?.snapshot() ?? null, resize: (width: number, height: number, dpr: number) => controller?.resize(width, height, dpr), select: (ids: readonly number[]) => controller?.select(ids), setLabels: (visible: boolean) => controller?.setLabels(visible) };
    (globalThis as typeof globalThis & { __brd080?: typeof bridge }).__brd080 = bridge;
    void show();
    return () => { ++loadGeneration; observer.disconnect(); delete (globalThis as typeof globalThis & { __brd080?: typeof bridge }).__brd080; void controller?.dispose(); controller = null; };
  });
</script>

<header><p class="eyebrow">Disposable decision spike · BRD-080</p><h1>Molecular viewer comparison</h1><p>This route compares presentation libraries. The visual canvas is neither authoritative chemistry nor the only way to inspect a fixture.</p></header>
<section class="controls" aria-label="Comparison controls">
  <fieldset><legend>Candidate</legend><div class="choice-row">{#each Object.keys(candidateLoads) as id}<label><input type="radio" name="candidate" value={id} bind:group={candidate} onchange={show}> {id === "3dmol" ? "3Dmol.js" : "Mol*"}</label>{/each}</div></fieldset>
  <fieldset><legend>Fixture</legend><div class="choice-row">{#each fixtures as fixture}<label><input type="radio" name="fixture" value={fixture.id} bind:group={fixtureId} onchange={show}> {fixture.kind}</label>{/each}</div></fieldset>
  <div class="toggles"><label><input id="labels" type="checkbox" bind:checked={labelsVisible} onchange={() => controller?.setLabels(labelsVisible)}> Visual labels</label><label><input id="reduce-motion" type="checkbox" bind:checked={reducedMotion} onchange={show}> Reduce motion</label></div>
</section>
<p id="status" class="status" role="status" aria-live="polite" data-state={status.state}>{status.message}</p>
<section class="visual-card" aria-labelledby="visual-title"><h2 id="visual-title">Optional visual rendering</h2><div id="viewer" aria-hidden="true" bind:this={host}></div><p class="boundary">If WebGL or a format is unavailable, the explicit status above and semantic view below remain available.</p></section>
<section id="semantic-view" class="semantic-card" aria-labelledby="semantic-title" tabindex="-1"><h2 id="semantic-title">Semantic fixture view</h2>
  {#if currentFixture}
    <p id="description">{currentFixture.description}</p><dl id="facts"><div><dt>Kind</dt><dd>{currentFixture.kind}</dd></div><div><dt>Format</dt><dd>{currentFixture.format.toUpperCase()}</dd></div><div><dt>Atoms</dt><dd>{currentFixture.atoms.length}</dd></div><div><dt>Frames</dt><dd>{currentFixture.frameCount ?? 1}</dd></div></dl>
    <table><caption>Atoms supplied by the fixture</caption><thead><tr><th scope="col">Select</th><th scope="col">ID</th><th scope="col">Label</th><th scope="col">Element</th><th scope="col">x</th><th scope="col">y</th><th scope="col">z</th></tr></thead><tbody id="atoms">{#each currentFixture.atoms as atom}<tr><td><input type="checkbox" data-atom={atom.id} value={atom.id} bind:group={selectedAtomIds} onchange={() => controller?.select(selectedAtomIds)} aria-label={`Select atom ${atom.id}: ${atom.label ?? atom.element}`}></td><th scope="row">{atom.id}</th><td>{atom.label ?? "—"}</td><td>{atom.element}</td><td>{atom.x}</td><td>{atom.y}</td><td>{atom.z}</td></tr>{/each}</tbody></table>
  {:else}<p id="description">Loading the local fixture…</p>{/if}
</section>
