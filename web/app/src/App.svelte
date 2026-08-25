<script lang="ts">
  import { onMount } from "svelte";
  import { Session } from "./lib/session.svelte";
  import { WorkerHost, resolvePayloadBase } from "./lib/host/WorkerHost";
  import { TauriHost, isTauri } from "./lib/host/TauriHost";
  import Bench from "./lib/components/Bench.svelte";
  import Feed from "./lib/components/Feed.svelte";
  import CommandBar from "./lib/components/CommandBar.svelte";
  import RegisterDial from "./lib/components/RegisterDial.svelte";
  import Shelf from "./lib/components/Shelf.svelte";
  import Inspector from "./lib/components/Inspector.svelte";
  import Timeline from "./lib/components/Timeline.svelte";
  import LessonBar from "./lib/components/LessonBar.svelte";
  import Burette from "./lib/components/Burette.svelte";
  import ReactPicker from "./lib/components/ReactPicker.svelte";
  import TransportBuilder from "./lib/components/TransportBuilder.svelte";
  import ApparatusForm from "./lib/components/ApparatusForm.svelte";
  import { APPARATUS } from "./lib/apparatus";
  import { defaultAmount } from "./lib/amounts";
  import { notebookMarkdown } from "./lib/notebook";
  import HelpDialog from "./lib/components/HelpDialog.svelte";
  import PeriodicTable from "./lib/components/PeriodicTable.svelte";
  import ExperimentCatalog from "./lib/components/ExperimentCatalog.svelte";
  import ReadingInset from "./lib/components/ReadingInset.svelte";
  import Toolbox from "./lib/components/Toolbox.svelte";
  import ConceptMap from "./lib/components/ConceptMap.svelte";
  import ToolIcon from "./lib/components/ToolIcon.svelte";
  import { parseCodexIndex, type CodexEntry } from "./lib/codex";
  import { pwa } from "./lib/pwa.svelte";

  // In the Tauri shell the engine is native and in-process; on the web it
  // lives in the module worker. The session cannot tell the difference.
  const session = new Session(isTauri() ? new TauriHost() : WorkerHost.create());
  let lessons = $state<{ file: string; name: string; blurb?: string; topic?: string }[]>([]);
  const lessonTopics = $derived(
    [...new Set(lessons.map((l) => l.topic ?? "more"))].map((topic) => ({
      topic,
      entries: lessons.filter((l) => (l.topic ?? "more") === topic),
    })),
  );

  // CI self-test hook (?selftest=1): report readiness to the harness once
  // the scene has arrived — a worker-driven app cannot be probed by
  // dumping the DOM at a fixed instant, so it phones home instead.
  const selftest =
    typeof location !== "undefined" &&
    new URLSearchParams(location.search).has("selftest");
  let selftestReported = false;
  $effect(() => {
    if (!selftest || selftestReported) return;
    const ready = session.engineReady && session.scene !== null;
    const failed = session.feed.find((f) => f.kind === "error");
    if (!ready && !failed) return;
    selftestReported = true;
    void fetch("/selftest", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        ready,
        can_solve: session.canSolve,
        vessels: session.scene?.vessels.length ?? 0,
        error: failed?.text ?? null,
      }),
    });
  });

  onMount(() => {
    void session.connect();
    // Offline-first and installable: the bench registers the payload-root
    // service worker itself rather than inheriting one from a visit to the
    // console page, which is the only reason /app/ ever worked offline.
    pwa.register();
    // Lessons ship beside the engine payload; their absence is quiet —
    // the sandbox is complete without them.
    // The codex export ships beside the payload once `kero codex export`
    // lands; until then the catalog button simply stays hidden.
    void fetch(new URL("codex/index.json", resolvePayloadBase()).href)
      .then((r) => (r.ok ? r.json() : null))
      .then((raw) => (codexEntries = parseCodexIndex(raw)))
      .catch(() => {});
    void fetch(new URL("lessons/index.json", resolvePayloadBase()).href)
      .then((r) => (r.ok ? r.json() : []))
      .then((list) => (lessons = list))
      .catch(() => {});
  });

  async function startLesson(file: string) {
    if (!file) return;
    const res = await fetch(new URL(`lessons/${file}`, resolvePayloadBase()).href);
    if (res.ok) session.startLesson(file.replace(/\.lab$/, ""), await res.text());
  }

  let helpOpen = $state(false);
  let tableOpen = $state(false);
  let toolboxOpen = $state(false);
  let mapOpen = $state(false);
  /** An entry handed from the map straight to the experiment page. */
  let catalogInitial = $state<CodexEntry | null>(null);
  let catalogOpen = $state(false);
  /** A tapped badge, magnified (the visual bar's reading inset). */
  let inset = $state<{ vessel: number; reading: { key: string; value: number; confidence: string } } | null>(null);
  let codexEntries = $state<CodexEntry[]>([]);
  /** The burette: clamped over the selected vessel when out (GUI-033). */
  let buretteOut = $state(false);
  /** Which parameter-form apparatus is out, by verb (GUI-033). */
  let apparatusOut = $state<string | null>(null);
  const apparatusSpec = $derived(APPARATUS.find((s) => s.verb === apparatusOut) ?? null);
  /** The transfer tool: filter/decant/drain share click-source-then-
   * target; decant carries its fraction. */
  type TwoVesselVerb = "filter" | "decant" | "drain" | "cell" | "distil";
  let transfer = $state<{ verb: TwoVesselVerb; fraction: number; from: number | null } | null>(null);
  const TWO_VESSEL_TOOLS: { verb: TwoVesselVerb; label: string }[] = [
    { verb: "filter", label: "filter" },
    { verb: "decant", label: "decant" },
    { verb: "drain", label: "drain" },
    { verb: "cell", label: "voltmeter" },
    { verb: "distil", label: "still" },
  ];
  function vesselTapped(id: number) {
    if (!transfer) {
      void session.inspect(id);
      pane = "notes";
      return;
    }
    if (transfer.from === null) {
      transfer = { ...transfer, from: id };
      return;
    }
    if (transfer.from === id) return; // same vessel: keep waiting
    const { verb, fraction, from } = transfer;
    const line =
      verb === "decant" || verb === "distil"
        ? `${verb} v${from + 1} v${id + 1} ${fraction}`
        : `${verb} v${from + 1} v${id + 1}`;
    transfer = null;
    void session.submit(line);
  }
  let titrating = $state(false);
  async function startTitration(line: string) {
    titrating = true;
    try {
      await session.submit(line);
    } finally {
      titrating = false;
    }
  }
  /** Narrow screens show one pane at a time; wide screens show all three. */
  let pane = $state<"bench" | "shelf" | "notes">("bench");
  /** Narrow screens collapse the toolbar; wide screens ignore this. */
  let toolsOpen = $state(false);

  function download(name: string, text: string, type = "text/plain") {
    const blob = new Blob([text], { type });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = name;
    a.click();
    URL.revokeObjectURL(a.href);
  }

  const saveLab = () => download("session.lab", session.exportLab());
  let labFileInput: HTMLInputElement | undefined = $state();
  async function openLabFile(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file) return;
    await session.importLab(file.name, await file.text());
  }
  const saveNotes = () =>
    download(
      "notebook.md",
      notebookMarkdown(session.feed, {
        date: new Date().toISOString().slice(0, 10),
        register: session.register,
      }),
      "text/markdown",
    );

  function onkeydown(e: KeyboardEvent) {
    const mod = e.ctrlKey || e.metaKey;
    const typing =
      e.target instanceof HTMLElement && ["INPUT", "TEXTAREA"].includes(e.target.tagName);
    if (mod && e.key.toLowerCase() === "z") {
      e.preventDefault();
      void (e.shiftKey ? session.redo() : session.undo());
    } else if (mod && e.key.toLowerCase() === "k") {
      e.preventDefault();
      document.querySelector<HTMLInputElement>(".bar input")?.focus();
    } else if (e.key === "?" && !typing) {
      helpOpen = true;
    } else if (e.key === "Escape") {
      if (inset) inset = null;
      else if (mapOpen) mapOpen = false;
      else if (toolboxOpen) toolboxOpen = false;
      else if (helpOpen) helpOpen = false;
      else session.closeInspector();
    }
  }
</script>

<svelte:window {onkeydown} />

<header>
  <h1>Kerotakis <small>a chemistry bench that computes</small></h1>
  <RegisterDial value={session.register} onchange={(lv) => void session.setRegister(lv)} />
  <!-- On a phone this toolbar is fifteen buttons deep and would push the
       bench off the screen entirely, so narrow layouts collapse it behind
       the disclosure below. `display: contents` keeps the wide layout
       byte-identical: the buttons stay direct flex children of <header>. -->
  <button
    class="tool tools-toggle"
    aria-expanded={toolsOpen}
    onclick={() => (toolsOpen = !toolsOpen)}
  >
    tools {toolsOpen ? "\u25b4" : "\u25be"}
  </button>
  <div class="tools" class:open={toolsOpen}>
  <button
    class="tool"
    onclick={() => void session.undo()}
    disabled={session.commandLog.length === 0 || session.busy}
  >
    undo
  </button>
  <button class="tool" onclick={saveLab} disabled={session.commandLog.length === 0}>
    save .lab
  </button>
  <button class="tool" onclick={saveNotes} disabled={session.feed.length === 0}>
    save notes
  </button>
  <button
    class="tool"
    onclick={() => window.print()}
    disabled={session.feed.length === 0}
    title="print the notebook — or save it as PDF from the print dialog"
  >
    print
  </button>
  <button class="tool" onclick={() => labFileInput?.click()} disabled={session.busy}>
    open .lab
  </button>
  <input
    bind:this={labFileInput}
    type="file"
    accept=".lab,text/plain"
    onchange={openLabFile}
    style="display:none"
    aria-hidden="true"
    tabindex="-1"
  />
  <button
    class="tool"
    onclick={() => void session.clear()}
    disabled={session.busy || session.commandLog.length === 0}
  >
    clear
  </button>
  <button
    class="tool"
    onclick={() => void session.submit("wait 30s")}
    disabled={session.busy}
    title="let 30 seconds of bench time pass"
  >
    wait 30 s
  </button>
  <button
    class="tool"
    class:active-tool={buretteOut}
    onclick={() => (buretteOut = !buretteOut)}
    title="clamp the burette over the selected vessel"
  >
    <ToolIcon name="burette" />burette
  </button>
  <select
    class="tool"
    aria-label="more apparatus"
    value={apparatusOut ?? ""}
    onchange={(e) => {
      apparatusOut = e.currentTarget.value || null;
      e.currentTarget.value = apparatusOut ?? "";
    }}
  >
    <option value="">apparatus…</option>
    {#each APPARATUS as s (s.verb)}
      <option value={s.verb}>{s.title}</option>
    {/each}
    {#if session.reactOptions.length > 0}
      <option value="react">curated reaction</option>
    {/if}
    <option value="transport">column train</option>
  </select>
  {#each TWO_VESSEL_TOOLS as tool (tool.verb)}
    <button
      class="tool"
      class:active-tool={transfer?.verb === tool.verb}
      onclick={() =>
        (transfer =
          transfer?.verb === tool.verb
            ? null
            : { verb: tool.verb, fraction: 0.5, from: null })}
      title={`${tool.verb}: pick the source vessel, then the target`}
    >
      <ToolIcon name={tool.verb} />{tool.label}
    </button>
  {/each}
  <Timeline
    position={session.position}
    total={session.commandLog.length}
    busy={session.busy}
    onjump={(to) => void session.jumpTo(to)}
  />
  {#if lessons.length > 0 && !session.lesson}
    <select
      class="tool"
      aria-label="start a lesson"
      onchange={(e) => {
        void startLesson(e.currentTarget.value);
        e.currentTarget.value = "";
      }}
    >
      <option value="">lessons…</option>
      {#each lessonTopics as group (group.topic)}
        <optgroup label={group.topic}>
          {#each group.entries as l (l.file)}
            <option value={l.file} title={l.blurb}>{l.name}</option>
          {/each}
        </optgroup>
      {/each}
    </select>
  {/if}
  </div>
  <span
    class="status"
    class:live={session.canSolve}
    title={session.engineIdentity ? `engine ${session.engineIdentity}` : undefined}
  >
    {session.engineReady ? (session.canSolve ? "live" : "shipped results") : "starting…"}
  </span>
  <div class="tools" class:open={toolsOpen}>
  <button class="tool" onclick={() => (tableOpen = true)} title="the periodic table, wired to the shelf">
    elements
  </button>
  <button class="tool" onclick={() => (toolboxOpen = true)} title="named relations: compute with provenance">
    toolbox
  </button>
  {#if codexEntries.length > 0}
    <button class="tool" onclick={() => (catalogOpen = true)} title="codex experiments: predict, run, check">
      experiments
    </button>
    <button class="tool" onclick={() => (mapOpen = true)} title="the concept map: what you have met, what is ready">
      map
    </button>
  {/if}
  </div>
  {#if pwa.installable}
    <button
      class="tool"
      onclick={() => void pwa.install()}
      title="install the bench — it runs offline, engine and all"
    >
      install
    </button>
  {/if}
  <!-- The console page is the other half of the web payload; a packaged
       app bundles only the bench, so the link would lead out of it. -->
  {#if !isTauri()}
    <a class="console-link" href="../">console</a>
  {/if}
</header>

{#if pwa.updateReady}
  <div class="update-banner" role="status">
    A newer bench is downloaded and ready.
    <button onclick={() => void pwa.applyUpdate()}>reload into it</button>
    <button class="cancel" onclick={() => (pwa.updateReady = false)}>later</button>
  </div>
{/if}

{#if transfer}
  <div class="transfer-banner" role="status">
    <strong>{transfer.verb}</strong>
    {#if transfer.verb === "decant" || transfer.verb === "distil"}
      — pour
      {#each [0.25, 0.5, 0.75, 1.0] as f (f)}
        <button class:on={transfer.fraction === f} onclick={() => (transfer = { ...transfer!, fraction: f })}>
          {f * 100}%
        </button>
      {/each}
    {/if}
    {transfer.from === null
      ? " · tap the source vessel"
      : ` · from v${transfer.from + 1} — now tap the target`}
    <button class="cancel" onclick={() => (transfer = null)}>cancel</button>
  </div>
{/if}

{#if session.lesson}
  <LessonBar
    name={session.lesson.lesson.name}
    next={session.lessonNextCommand}
    busy={session.busy}
    deviation={session.lessonDeviation}
    kit={session.shelf.filter(s => session.lesson!.kit.includes(s.key))}
    register={session.register}
    target={session.selected}
    onnext={() => void session.lessonNext()}
    onreturn={() => void session.lessonReturn()}
    onexit={() => session.exitLesson()}
    onadd={(line) => void session.submit(line)}
  />
{/if}

<main data-pane={pane}>
  <nav class="shelf-pane">
    <Shelf
      items={session.shelf}
      register={session.register}
      target={session.selected}
      kit={session.lesson?.kit ?? null}
      onadd={(line) => {
        void session.submit(line);
        pane = "bench";
      }}
    />
  </nav>
  <div class="bench-pane">
    {#if apparatusOut === "react"}
      <ReactPicker
        vessel={session.selected}
        options={session.reactOptions}
        busy={session.busy}
        onrun={(line) => void session.submit(line)}
        onclose={() => (apparatusOut = null)}
      />
    {:else if apparatusOut === "transport"}
      <TransportBuilder
        vessels={session.scene?.vessels ?? []}
        busy={session.busy}
        onrun={(line) => void session.submit(line)}
        onclose={() => (apparatusOut = null)}
      />
    {/if}
    {#if apparatusSpec}
      {#key apparatusSpec.verb}
        <ApparatusForm
          spec={apparatusSpec}
          vessel={session.selected}
          shelf={session.shelf}
          busy={session.busy}
          onrun={(line) => void session.submit(line)}
          onclose={() => (apparatusOut = null)}
        />
      {/key}
    {/if}
    {#if buretteOut}
      <Burette
        vessel={session.selected}
        shelf={session.shelf}
        busy={session.busy}
        running={titrating}
        onstart={(line) => void startTitration(line)}
        onclose={() => (buretteOut = false)}
      />
    {/if}
    {#if session.register !== "lv1" && session.lastEquation}
      <p class="equation" aria-label="latest reaction equation">
        {session.lastEquation}
      </p>
    {/if}
    <Bench
      scene={session.scene}
      register={session.register}
      selected={session.selected}
      onselect={(id) => vesselTapped(id)}
      pristine={session.commandLog.length === 0 && !session.lesson}
      effects={session.vesselEffects}
      titrationPlayback={session.titrationPlayback}
      onnewvessel={(kind) => void session.submit(kind === "beaker" ? "new" : `new ${kind}`)}
      onbadge={(vessel, reading) => (inset = { vessel, reading })}
      fluidLookup={(key) => {
        const item = session.shelf.find((s) => s.key === key);
        return {
          key,
          srgb: item?.srgb ?? item?.solution_srgb ?? [140, 160, 200],
          density: item?.density ?? 1,
        };
      }}
      ondropspecies={(id, p) =>
        void session.submit(
          `add v${id + 1} ${p.key} ${defaultAmount(session.register, p.phase)}`,
        )}
    />
  </div>
  <aside>
    {#if session.inspector}
      <Inspector
        vessel={session.inspector.vessel}
        lines={session.inspector.lines}
        particles={session.inspector.particles}
        boundary={session.scene?.vessels.find((v) => v.id === session.inspector?.vessel)
          ?.boundary ?? "open"}
        busy={session.busy}
        onparticles={() => void session.particles()}
        onclose={() => session.closeInspector()}
        onaction={(line) => void session.submit(line)}
      />
    {/if}
    <Feed entries={session.feed} />
  </aside>
</main>

<CommandBar
  onsubmit={(line) => void session.submit(line)}
  busy={session.busy}
  onvalidate={(line) => session.parse(line)}
/>

<nav class="tabs" aria-label="panes">
  {#each [["bench", "bench"], ["shelf", "shelf"], ["notes", "notes"]] as [key, label] (key)}
    <button
      aria-pressed={pane === key}
      class:active={pane === key}
      onclick={() => (pane = key as typeof pane)}
    >
      {label}
    </button>
  {/each}
</nav>

{#if inset}
  <ReadingInset vessel={inset.vessel} reading={inset.reading} onclose={() => (inset = null)} />
{/if}

{#if helpOpen}
  <HelpDialog onclose={() => (helpOpen = false)} />
{/if}

{#if toolboxOpen}
  <Toolbox {session} onclose={() => (toolboxOpen = false)} />
{/if}

{#if catalogOpen}
  <ExperimentCatalog
    entries={codexEntries}
    {session}
    initial={catalogInitial}
    onclose={() => {
      catalogOpen = false;
      catalogInitial = null;
    }}
  />
{/if}

{#if mapOpen}
  <ConceptMap
    entries={codexEntries}
    {session}
    onopenentry={(e) => {
      mapOpen = false;
      catalogInitial = e;
      catalogOpen = true;
    }}
    onclose={() => (mapOpen = false)}
  />
{/if}

{#if tableOpen}
  <PeriodicTable
    shelf={session.shelf}
    register={session.register}
    onadd={(item) => {
      tableOpen = false;
      void session.submit(
        `add v${session.selected + 1} ${item.key} ${defaultAmount(session.register, item.phase)}`,
      );
    }}
    onclose={() => (tableOpen = false)}
  />
{/if}

<style>
  header {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    flex-wrap: wrap;
    padding: 0.7rem 1rem;
    border-bottom: 1px solid var(--edge);
  }
  h1 {
    font-size: 1rem;
    margin: 0;
    font-weight: 600;
  }
  h1 small {
    color: var(--dim);
    font-weight: 400;
    margin-left: 0.5rem;
  }
  .tool {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.3rem 0.7rem;
    cursor: pointer;
    min-height: 36px;
  }
  .tool:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .active-tool {
    border-color: var(--hot);
  }
  .transfer-banner,
  .update-banner {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
    padding: 0.4rem 1rem;
    border-bottom: 1px solid var(--warn);
    background: var(--panel);
    font-size: 0.85rem;
  }
  .transfer-banner button,
  .update-banner button {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    font: inherit;
    font-size: 0.76rem;
    padding: 0.15rem 0.55rem;
    cursor: pointer;
    min-height: 30px;
  }
  .transfer-banner button.on {
    border-color: var(--hot);
  }
  .transfer-banner .cancel,
  .update-banner .cancel {
    margin-left: auto;
    color: var(--dim);
  }
  .status {
    margin-left: auto;
    font-size: 0.8rem;
    color: var(--warn);
  }
  .status.live {
    color: var(--good);
  }
  .console-link {
    color: var(--cool);
    font-size: 0.8rem;
    text-decoration: none;
  }
  .console-link:hover {
    text-decoration: underline;
  }
  main {
    display: flex;
    flex: 1;
    min-height: 0;
  }
  .shelf-pane {
    width: min(17rem, 26vw);
    border-right: 1px solid var(--edge);
    background: var(--panel);
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .shelf-pane > :global(.shelf) {
    flex: 1;
  }
  aside {
    width: min(24rem, 34vw);
    border-left: 1px solid var(--edge);
    background: var(--panel);
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  aside > :global(.feed) {
    flex: 1;
  }
  .bench-pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }
  .equation {
    margin: 0;
    padding: 0.45rem 1rem;
    border-bottom: 1px solid var(--edge);
    font-size: 0.95rem;
    text-align: center;
    color: var(--ink);
    background: var(--panel);
    overflow-x: auto;
    white-space: nowrap;
  }
  .bench-pane > :global(.bench) {
    flex: 1;
  }
  /* The tab bar exists only on narrow screens. */
  .tabs {
    display: none;
  }
  /* Wide: the toolbar is not a box at all. `display: contents` dissolves
     the wrapper so its buttons stay direct flex children of <header>, and
     the layout is exactly what it was before the wrapper existed. */
  .tools {
    display: contents;
  }
  .tools-toggle {
    display: none;
  }
  @media (max-width: 900px) {
    /* Narrow: fifteen buttons wrap to nine rows and push the bench off
       the screen. Collapse them; the title, the register dial and the
       engine status are what a phone keeps. */
    header {
      padding: 0.5rem 0.75rem;
      gap: 0.5rem;
    }
    header h1 small {
      display: none;
    }
    .tools-toggle {
      display: inline-block;
    }
    .tools {
      display: none;
      flex-basis: 100%;
      flex-wrap: wrap;
      gap: 0.4rem;
    }
    .tools.open {
      display: flex;
    }

    /* One pane at a time, chosen by the tab bar. */
    main[data-pane="bench"] .shelf-pane,
    main[data-pane="bench"] aside,
    main[data-pane="shelf"] .bench-pane,
    main[data-pane="shelf"] aside,
    main[data-pane="notes"] .bench-pane,
    main[data-pane="notes"] .shelf-pane {
      display: none;
    }
    .shelf-pane,
    aside {
      width: auto;
      flex: 1;
      border-left: 0;
      border-right: 0;
    }
    .tabs {
      display: flex;
      border-top: 1px solid var(--edge);
      background: var(--panel);
    }
    .tabs button {
      flex: 1;
      background: none;
      border: 0;
      color: var(--dim);
      font: inherit;
      font-size: 0.85rem;
      padding: 0.55rem;
      min-height: 44px;
      cursor: pointer;
    }
    .tabs button.active {
      color: var(--ink);
      box-shadow: inset 0 2px 0 var(--hot);
    }
  }
</style>
