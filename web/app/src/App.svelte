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
  import EquipmentCabinet from "./lib/components/EquipmentCabinet.svelte";
  import LocaleSwitcher from "./lib/components/LocaleSwitcher.svelte";
  import VesselActionDock from "./lib/components/VesselActionDock.svelte";
  import MissionControl from "./lib/components/MissionControl.svelte";
  import { t } from "./lib/i18n.svelte";
  import { parseCodexIndex, type CodexEntry } from "./lib/codex";
  import { pwa } from "./lib/pwa.svelte";
  import { twoVesselLine, type TwoVesselAction } from "./lib/directActions";

  // In the Tauri shell the engine is native and in-process; on the web it
  // lives in the module worker. The session cannot tell the difference.
  const session = new Session(isTauri() ? new TauriHost() : WorkerHost.create());
  type Theme = "light" | "dark" | "contrast";
  let theme = $state<Theme>("light");
  $effect(() => {
    if (typeof document !== "undefined") document.documentElement.dataset.theme = theme;
  });

  function setTheme(next: Theme) {
    theme = next;
    try {
      localStorage.setItem("kerotakis.theme", next);
    } catch {
      // The selected theme still works when persistence is unavailable.
    }
  }

  let lessons = $state<{ file: string; name: string; blurb?: string; topic?: string }[]>([]);

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
    try {
      const savedTheme = localStorage.getItem("kerotakis.theme");
      if (savedTheme === "light" || savedTheme === "dark" || savedTheme === "contrast") {
        theme = savedTheme;
      }
    } catch {
      // Bright mode is the intentional first-run default.
    }
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
    if (res.ok) {
      session.startLesson(file.replace(/\.lab$/, ""), await res.text());
      missionOpen = false;
    }
  }

  let helpOpen = $state(false);
  let missionOpen = $state(false);
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
  let transfer = $state<{ verb: TwoVesselAction; fraction: number; from: number | null } | null>(null);
  function vesselTapped(id: number) {
    if (!transfer) {
      void session.inspect(id);
      return;
    }
    if (transfer.from === null) {
      transfer = { ...transfer, from: id };
      return;
    }
    if (transfer.from === id) return; // same vessel: keep waiting
    const { verb, fraction, from } = transfer;
    const line = twoVesselLine(verb, from, id, fraction);
    if (!line) return;
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
  /** File/research actions live in one utility drawer instead of competing
   * with the bench in the permanent top rail. */
  let toolsOpen = $state(false);
  /** The supply room keeps chemicals and reusable equipment distinct. */
  let cabinetTab = $state<"reagents" | "equipment">("reagents");
  const selectedVessel = $derived(
    session.scene?.vessels.find((v) => v.id === session.selected) ?? null,
  );

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
      else if (missionOpen) missionOpen = false;
      else if (mapOpen) mapOpen = false;
      else if (toolboxOpen) toolboxOpen = false;
      else if (helpOpen) helpOpen = false;
      else if (toolsOpen) toolsOpen = false;
      else session.closeInspector();
    }
  }
</script>

<svelte:window {onkeydown} />

<header class="topbar">
  <div class="brand">
    <span class="brand-mark" aria-hidden="true">
      <svg viewBox="0 0 40 40">
        <path d="M14 5h12M17 5v10L8 31c-1 2 1 4 3 4h18c2 0 4-2 3-4l-9-16V5" />
        <path class="brand-liquid" d="M11 28c5-3 12 3 18-1l3 6H8z" />
        <circle cx="16" cy="24" r="1.7" />
        <circle cx="23" cy="20" r="1.3" />
      </svg>
    </span>
    <span class="brand-copy">
      <strong>Kerotakis</strong>
      <small>{t("a chemistry bench that computes")}</small>
    </span>
  </div>

  <button
    class="mode-pill"
    class:mission-active={session.lesson !== null}
    aria-label={t("open Mission Control")}
    onclick={() => (missionOpen = true)}
  >
    <span class="mode-signal" aria-hidden="true">{session.lesson ? "◆" : "∞"}</span>
    <span class="mode-copy"><small>{session.lesson ? t("guided mission") : t("lab mode")}</small><strong>{session.lesson ? t(session.lesson.lesson.name) : t("Sandbox")}</strong></span>
    <span class="mode-arrow" aria-hidden="true">⌄</span>
  </button>

  <div class="top-controls">
    <RegisterDial value={session.register} onchange={(lv) => void session.setRegister(lv)} />
    <span
      class="status"
      class:live={session.canSolve}
      title={session.engineIdentity ? t("engine {identity}", { identity: session.engineIdentity }) : undefined}
    >
      <span class="status-dot" aria-hidden="true"></span>
      {session.engineReady ? (session.canSolve ? t("live") : t("shipped results")) : t("starting…")}
    </span>
    <LocaleSwitcher />
    <button
      class="utility-toggle"
      aria-expanded={toolsOpen}
      aria-label={t("open utilities")}
      onclick={() => (toolsOpen = !toolsOpen)}
    >
      <span aria-hidden="true">•••</span>
      <span class="utility-label">{t("utilities")}</span>
    </button>
  </div>
</header>

{#if toolsOpen}
  <section class="utility-drawer" aria-label={t("utilities")}>
    <div class="utility-group">
      <strong>{t("time and history")}</strong>
      <button class="tool" onclick={() => void session.undo()} disabled={session.commandLog.length === 0 || session.busy}>{t("undo")}</button>
      <button class="tool" onclick={() => void session.submit("wait 30s")} disabled={session.busy} title={t("let 30 seconds of bench time pass")}>{t("wait 30 s")}</button>
      <button class="tool danger-tool" onclick={() => void session.clear()} disabled={session.busy || session.commandLog.length === 0}>{t("clear")}</button>
      <Timeline position={session.position} total={session.commandLog.length} busy={session.busy} onjump={(to) => void session.jumpTo(to)} />
    </div>
    <div class="utility-group">
      <strong>{t("files and notebook")}</strong>
      <button class="tool" onclick={saveLab} disabled={session.commandLog.length === 0}>{t("save .lab")}</button>
      <button class="tool" onclick={() => labFileInput?.click()} disabled={session.busy}>{t("open .lab")}</button>
      <button class="tool" onclick={saveNotes} disabled={session.feed.length === 0}>{t("save notes")}</button>
      <button class="tool" onclick={() => window.print()} disabled={session.feed.length === 0} title={t("print the notebook — or save it as PDF from the print dialog")}>{t("print")}</button>
      <input bind:this={labFileInput} type="file" accept=".lab,text/plain" onchange={openLabFile} style="display:none" aria-hidden="true" tabindex="-1" />
    </div>
    <div class="utility-group">
      <strong>{t("explore and study")}</strong>
      <button class="tool" onclick={() => (tableOpen = true)}>{t("elements")}</button>
      <button class="tool" onclick={() => (toolboxOpen = true)}>{t("toolbox")}</button>
      {#if codexEntries.length > 0}
        <button class="tool" onclick={() => (catalogOpen = true)}>{t("experiments")}</button>
        <button class="tool" onclick={() => (mapOpen = true)}>{t("map")}</button>
      {/if}
      <button class="tool mission-tool" onclick={() => { toolsOpen = false; missionOpen = true; }}>{t("Mission Control")}</button>
      {#if pwa.installable}<button class="tool" onclick={() => void pwa.install()}>{t("install")}</button>{/if}
      {#if !isTauri()}<a class="console-link" href="../">{t("console")}</a>{/if}
      <div class="utility-locale"><span>{t("Language")}</span><LocaleSwitcher /></div>
      <div class="theme-choice" role="radiogroup" aria-label={t("appearance")}>
        <span>{t("appearance")}</span>
        {#each [["light", "light"], ["dark", "dark"], ["contrast", "high contrast"]] as [value, label] (value)}
          <button class="theme-button" role="radio" aria-checked={theme === value} class:active={theme === value} onclick={() => setTheme(value as Theme)}>{t(label)}</button>
        {/each}
      </div>
    </div>
  </section>
{/if}

{#if pwa.updateReady}
  <div class="update-banner" role="status">
    {t("A newer bench is downloaded and ready.")}
    <button onclick={() => void pwa.applyUpdate()}>{t("reload into it")}</button>
    <button class="cancel" onclick={() => (pwa.updateReady = false)}>{t("later")}</button>
  </div>
{/if}

{#if transfer}
  <div class="transfer-banner" role="status">
    <strong>{t(transfer.verb)}</strong>
    {#if transfer.verb === "decant" || transfer.verb === "distil"}
      — {t("pour")}
      {#each [0.25, 0.5, 0.75, 1.0] as f (f)}
        <button class:on={transfer.fraction === f} onclick={() => (transfer = { ...transfer!, fraction: f })}>
          {f * 100}%
        </button>
      {/each}
    {/if}
    {transfer.from === null
      ? ` · ${t("tap the source vessel")}`
      : ` · ${t("from v{vessel} — now tap the target", { vessel: transfer.from + 1 })}`}
    <button class="cancel" onclick={() => (transfer = null)}>{t("cancel")}</button>
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
    cursor={session.lesson.cursor}
    total={session.lesson.lesson.steps.length}
    onnext={() => void session.lessonNext()}
    onreturn={() => void session.lessonReturn()}
    onexit={() => session.exitLesson()}
    onadd={(line) => void session.submit(line)}
  />
{/if}

<main data-pane={pane}>
  <nav class="shelf-pane">
    <div class="pane-heading">
      <span class="pane-icon" aria-hidden="true">▦</span>
      <span><strong>{t("supply cabinet")}</strong><small>{t("choose what goes on the bench")}</small></span>
    </div>
    <div class="cabinet-tabs" role="tablist" aria-label={t("supply cabinet")}>
      <button role="tab" aria-selected={cabinetTab === "reagents"} class:active={cabinetTab === "reagents"} onclick={() => (cabinetTab = "reagents")}>{t("reagents")}</button>
      <button role="tab" aria-selected={cabinetTab === "equipment"} class:active={cabinetTab === "equipment"} onclick={() => (cabinetTab = "equipment")}>{t("equipment")}</button>
    </div>
    {#if cabinetTab === "reagents"}
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
    {:else}
      <EquipmentCabinet
        target={session.selected}
        targetLabel={selectedVessel?.label ?? "beaker"}
        {buretteOut}
        {apparatusOut}
        transferVerb={transfer?.verb ?? null}
        reactAvailable={session.reactOptions.length > 0}
        onburette={() => {
          buretteOut = !buretteOut;
          pane = "bench";
        }}
        onapparatus={(verb) => {
          apparatusOut = apparatusOut === verb ? null : verb;
          pane = "bench";
        }}
        ontransfer={(verb) => {
          transfer = transfer?.verb === verb ? null : { verb, fraction: 0.5, from: null };
          pane = "bench";
        }}
      />
    {/if}
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
      <p class="equation" aria-label={t("latest reaction equation")}>
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
      transferFrom={transfer?.from ?? null}
      ondropspecies={(id, p) =>
        void session.submit(
          `add v${id + 1} ${p.key} ${defaultAmount(session.register, p.phase)}`,
        )}
    />
    {#if selectedVessel}
      <VesselActionDock
        vessel={selectedVessel.id}
        label={selectedVessel.label}
        boundary={selectedVessel.boundary}
        busy={session.busy}
        onaction={(line) => void session.submit(line)}
        onpour={() => (transfer = { verb: "decant", fraction: 0.5, from: selectedVessel!.id })}
        ondetails={() => {
          void session.inspect(selectedVessel!.id);
          pane = "notes";
        }}
        onmore={() => {
          cabinetTab = "equipment";
          pane = "shelf";
        }}
      />
    {/if}
  </div>
  <aside>
    <div class="pane-heading journal-heading">
      <span class="pane-icon" aria-hidden="true">≡</span>
      <span><strong>{t("lab journal")}</strong><small>{t("observations and evidence")}</small></span>
      <span class="entry-count" title={t("notebook entries")}>{session.feed.length}</span>
    </div>
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

<nav class="tabs" aria-label={t("panes")}>
  {#each [["bench", "workspace"], ["shelf", "cabinet"], ["notes", "journal"]] as [key, label] (key)}
    <button
      aria-pressed={pane === key}
      class:active={pane === key}
      onclick={() => (pane = key as typeof pane)}
    >
      {t(label)}
    </button>
  {/each}
</nav>

{#if inset}
  <ReadingInset vessel={inset.vessel} reading={inset.reading} onclose={() => (inset = null)} />
{/if}

{#if missionOpen}
  <MissionControl
    missions={lessons}
    experiments={codexEntries}
    active={session.lesson?.lesson.name ?? null}
    cursor={session.lesson?.cursor ?? 0}
    total={session.lesson?.lesson.steps.length ?? 0}
    onstart={(file) => void startLesson(file)}
    onsandbox={() => {
      if (session.lesson) session.exitLesson();
      missionOpen = false;
    }}
    onexperiments={() => {
      missionOpen = false;
      catalogOpen = true;
    }}
    onmap={() => {
      missionOpen = false;
      mapOpen = true;
    }}
    onclose={() => (missionOpen = false)}
  />
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
  /* GUI-070/071: bench-first shell. Kept after the legacy declarations so
     the migration remains reviewable while every child component retains its
     semantic token aliases. */
  .topbar {
    min-height: 68px;
    display: flex;
    flex-wrap: nowrap;
    align-items: center;
    gap: 1rem;
    padding: 0.65rem 1rem;
    border-bottom: 1px solid color-mix(in srgb, var(--edge) 82%, transparent);
    background: color-mix(in srgb, var(--surface) 94%, transparent);
    box-shadow: 0 8px 26px var(--shadow);
    z-index: 20;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    min-width: 12rem;
  }
  .brand-mark {
    width: 42px;
    height: 42px;
    display: grid;
    place-items: center;
    flex: none;
    color: var(--primary);
    border-radius: 13px;
    background: color-mix(in srgb, var(--primary) 12%, var(--surface));
  }
  .brand-mark svg {
    width: 34px;
    height: 34px;
  }
  .brand-mark path,
  .brand-mark circle {
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .brand-mark .brand-liquid {
    fill: color-mix(in srgb, var(--instrument) 32%, transparent);
    stroke: var(--instrument);
  }
  .brand-copy {
    display: flex;
    flex-direction: column;
    line-height: 1.15;
  }
  .brand-copy strong {
    font-size: 1.08rem;
    letter-spacing: 0.01em;
  }
  .brand-copy small {
    margin-top: 0.22rem;
    color: var(--dim);
    font-size: 0.69rem;
  }
  .mode-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    min-height: 46px;
    padding: 0.35rem 0.45rem 0.35rem 0.55rem;
    border: 1px solid color-mix(in srgb, var(--discovery) 38%, var(--edge));
    border-radius: 999px;
    color: var(--discovery);
    background: color-mix(in srgb, var(--discovery) 9%, var(--surface));
    font-size: 0.75rem;
    font-weight: 700;
    cursor: pointer;
    text-align: left;
  }
  .mode-pill:hover {
    border-color: var(--discovery);
    transform: translateY(-1px);
    box-shadow: 0 7px 18px color-mix(in srgb, var(--discovery) 18%, transparent);
  }
  .mode-pill.mission-active {
    color: var(--surface);
    border-color: var(--discovery);
    background: linear-gradient(135deg, var(--discovery), color-mix(in srgb, var(--discovery) 65%, var(--primary)));
  }
  .mode-signal {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border-radius: 9px;
    background: color-mix(in srgb, currentColor 12%, transparent);
    font-size: 0.95rem;
  }
  .mode-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    line-height: 1.08;
  }
  .mode-copy small {
    opacity: 0.74;
    font-size: 0.56rem;
    font-weight: 750;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .mode-copy strong {
    max-width: 12rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.73rem;
  }
  .mode-arrow {
    margin-left: 0.15rem;
    opacity: 0.72;
  }
  .top-controls {
    margin-left: auto;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.65rem;
    min-width: 0;
  }
  .tool {
    background: var(--surface-raised);
    border-radius: var(--radius-sm);
    min-height: 40px;
    padding: 0.45rem 0.7rem;
  }
  .tool:hover:not(:disabled) {
    border-color: var(--primary);
    background: color-mix(in srgb, var(--primary) 8%, var(--surface-raised));
  }
  .danger-tool:hover:not(:disabled) {
    color: var(--danger);
    border-color: var(--danger);
  }
  .mission-tool {
    color: var(--discovery);
    border-color: color-mix(in srgb, var(--discovery) 35%, var(--edge));
  }
  .utility-toggle {
    min-height: 40px;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.4rem 0.7rem;
    border: 1px solid var(--edge);
    border-radius: var(--radius-sm);
    color: var(--ink);
    background: var(--surface-raised);
    cursor: pointer;
    font-weight: 650;
  }
  .utility-toggle:hover,
  .utility-toggle[aria-expanded="true"] {
    border-color: var(--primary);
    color: var(--primary);
  }
  .utility-drawer {
    position: fixed;
    top: calc(env(safe-area-inset-top) + 74px);
    right: calc(env(safe-area-inset-right) + 1rem);
    width: min(52rem, calc(100vw - 2rem));
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.75rem;
    padding: 0.85rem;
    z-index: 40;
    border: 1px solid var(--edge);
    border-radius: var(--radius-lg);
    background: color-mix(in srgb, var(--surface) 97%, transparent);
    box-shadow: 0 18px 55px var(--shadow-strong);
  }
  .utility-group {
    display: flex;
    align-content: flex-start;
    flex-wrap: wrap;
    gap: 0.4rem;
    padding: 0.75rem;
    border-radius: var(--radius-md);
    background: var(--surface-raised);
  }
  .utility-group > strong {
    flex-basis: 100%;
    margin-bottom: 0.15rem;
    color: var(--dim);
    font-size: 0.7rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .utility-group > :global(.timeline) {
    flex-basis: 100%;
  }
  .theme-choice {
    flex-basis: 100%;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.3rem;
    margin-top: 0.3rem;
    padding-top: 0.55rem;
    border-top: 1px solid var(--edge);
  }
  .utility-locale {
    flex-basis: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding-top: 0.5rem;
    color: var(--dim);
    font-size: 0.7rem;
  }
  .theme-choice > span {
    grid-column: 1 / -1;
    color: var(--dim);
    font-size: 0.68rem;
  }
  .theme-button {
    min-height: 34px;
    border: 1px solid var(--edge);
    border-radius: 8px;
    color: var(--dim);
    background: var(--surface);
    cursor: pointer;
    font-size: 0.7rem;
  }
  .theme-button.active {
    border-color: var(--primary);
    color: var(--primary);
    box-shadow: inset 0 -2px 0 var(--primary);
  }
  .status {
    margin-left: 0;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    white-space: nowrap;
  }
  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: currentColor;
    box-shadow: 0 0 0 3px color-mix(in srgb, currentColor 14%, transparent);
  }
  .console-link {
    align-self: center;
    color: var(--primary);
    padding: 0.4rem;
  }
  main {
    gap: 0.75rem;
    padding: 0.75rem;
  }
  .shelf-pane {
    width: min(15rem, 20vw);
    border: 1px solid var(--edge);
    border-radius: var(--radius-lg);
    background: var(--surface);
    overflow: hidden;
    box-shadow: 0 8px 28px var(--shadow);
  }
  aside {
    width: min(18rem, 23vw);
    border: 1px solid var(--edge);
    border-radius: var(--radius-lg);
    background: var(--surface);
    overflow: hidden;
    box-shadow: 0 8px 28px var(--shadow);
  }
  .bench-pane {
    overflow: hidden;
    border: 1px solid var(--edge);
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: 0 10px 32px var(--shadow);
  }
  .pane-heading {
    min-height: 62px;
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.7rem 0.85rem;
    border-bottom: 1px solid var(--edge);
  }
  .pane-heading > span:nth-child(2) {
    min-width: 0;
    display: flex;
    flex: 1;
    flex-direction: column;
    line-height: 1.2;
  }
  .pane-heading strong {
    font-size: 0.86rem;
  }
  .pane-heading small {
    margin-top: 0.2rem;
    color: var(--dim);
    font-size: 0.67rem;
  }
  .pane-icon {
    width: 34px;
    height: 34px;
    display: grid;
    place-items: center;
    flex: none;
    border-radius: 10px;
    color: var(--primary);
    background: color-mix(in srgb, var(--primary) 11%, var(--surface-raised));
    font-size: 1.15rem;
    font-weight: 800;
  }
  .journal-heading .pane-icon {
    color: var(--discovery);
    background: color-mix(in srgb, var(--discovery) 10%, var(--surface-raised));
  }
  .entry-count {
    min-width: 1.8rem;
    padding: 0.22rem 0.4rem;
    border-radius: 999px;
    color: var(--dim);
    background: var(--surface-raised);
    font-size: 0.7rem;
    text-align: center;
  }
  .cabinet-tabs {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.3rem;
    padding: 0.55rem 0.65rem 0;
  }
  .cabinet-tabs button {
    min-height: 38px;
    border: 0;
    border-radius: 10px;
    color: var(--dim);
    background: transparent;
    cursor: pointer;
    font-size: 0.78rem;
    font-weight: 650;
  }
  .cabinet-tabs button.active {
    color: var(--primary);
    background: color-mix(in srgb, var(--primary) 10%, var(--surface-raised));
  }
  .equation {
    background: var(--surface);
  }

  @media (max-width: 980px) {
    main[data-pane="bench"] .shelf-pane,
    main[data-pane="bench"] aside,
    main[data-pane="shelf"] .bench-pane,
    main[data-pane="shelf"] aside,
    main[data-pane="notes"] .bench-pane,
    main[data-pane="notes"] .shelf-pane {
      display: none;
    }
    main {
      padding: 0.5rem;
    }
    .shelf-pane,
    aside {
      width: auto;
      flex: 1;
      border: 1px solid var(--edge);
      border-radius: var(--radius-md);
    }
    .tabs {
      display: flex;
      border-top: 1px solid var(--edge);
      background: var(--surface);
    }
    .tabs button {
      flex: 1;
      min-height: 48px;
      border: 0;
      color: var(--dim);
      background: transparent;
      cursor: pointer;
      font-size: 0.82rem;
      font-weight: 650;
    }
    .tabs button.active {
      color: var(--primary);
      box-shadow: inset 0 3px 0 var(--primary);
      background: color-mix(in srgb, var(--primary) 7%, transparent);
    }
  }
  @media (max-width: 760px) {
    .topbar {
      min-height: 60px;
      gap: 0.45rem;
      padding: 0.45rem 0.55rem;
    }
    .brand {
      min-width: 0;
    }
    .brand-mark {
      width: 38px;
      height: 38px;
      border-radius: 11px;
    }
    .brand-copy small,
    .mode-pill,
    .utility-label {
      display: none;
    }
    .top-controls {
      gap: 0.35rem;
    }
    .status {
      display: none;
    }
    .top-controls > :global(.locale) {
      display: none;
    }
    .utility-toggle {
      width: 40px;
      justify-content: center;
      padding: 0;
    }
    .utility-drawer {
      top: calc(env(safe-area-inset-top) + 64px);
      left: calc(env(safe-area-inset-left) + 0.5rem);
      right: calc(env(safe-area-inset-right) + 0.5rem);
      width: auto;
      max-height: calc(100dvh - 8rem);
      grid-template-columns: 1fr;
      overflow-y: auto;
      border-radius: var(--radius-md);
    }
    .top-controls :global(.dial button) {
      padding-inline: 0.45rem;
    }
    .top-controls :global(.dial button:not(.active)) {
      display: none;
    }
  }
  @media (max-width: 430px) {
    .brand-copy {
      display: none;
    }
    .top-controls {
      margin-left: 0;
      flex: 1;
    }
    .top-controls :global(.locale) {
      margin-left: auto;
    }
  }
</style>
