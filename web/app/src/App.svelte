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
  import { defaultAmount } from "./lib/amounts";
  import { notebookMarkdown } from "./lib/notebook";
  import HelpDialog from "./lib/components/HelpDialog.svelte";

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
    // Lessons ship beside the engine payload; their absence is quiet —
    // the sandbox is complete without them.
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
  /** The burette: clamped over the selected vessel when out (GUI-033). */
  let buretteOut = $state(false);
  /** The transfer tool: filter/decant/drain share click-source-then-
   * target; decant carries its fraction. */
  let transfer = $state<{ verb: "filter" | "decant" | "drain"; fraction: number; from: number | null } | null>(null);
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
      verb === "decant"
        ? `decant v${from + 1} v${id + 1} ${fraction}`
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
      if (helpOpen) helpOpen = false;
      else session.closeInspector();
    }
  }
</script>

<svelte:window {onkeydown} />

<header>
  <h1>Kerotakis <small>a chemistry bench that computes</small></h1>
  <RegisterDial value={session.register} onchange={(lv) => void session.setRegister(lv)} />
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
    burette
  </button>
  {#each ["filter", "decant", "drain"] as verb (verb)}
    <button
      class="tool"
      class:active-tool={transfer?.verb === verb}
      onclick={() =>
        (transfer =
          transfer?.verb === verb
            ? null
            : { verb: verb as "filter" | "decant" | "drain", fraction: 0.5, from: null })}
      title={`${verb}: pick the source vessel, then the target`}
    >
      {verb}
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
  <span class="status" class:live={session.canSolve}>
    {session.engineReady ? (session.canSolve ? "live" : "shipped results") : "starting…"}
  </span>
  <a class="console-link" href="../">console</a>
</header>

{#if transfer}
  <div class="transfer-banner" role="status">
    <strong>{transfer.verb}</strong>
    {#if transfer.verb === "decant"}
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
    onnext={() => void session.lessonNext()}
    onexit={() => session.exitLesson()}
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
      onnewvessel={(kind) => void session.submit(kind === "beaker" ? "new" : `new ${kind}`)}
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

{#if helpOpen}
  <HelpDialog onclose={() => (helpOpen = false)} />
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
  .transfer-banner {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
    padding: 0.4rem 1rem;
    border-bottom: 1px solid var(--warn);
    background: var(--panel);
    font-size: 0.85rem;
  }
  .transfer-banner button {
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
  .transfer-banner .cancel {
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
  @media (max-width: 900px) {
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
