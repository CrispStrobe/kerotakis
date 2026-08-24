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
      onadd={(line) => {
        void session.submit(line);
        pane = "bench";
      }}
    />
  </nav>
  <div class="bench-pane">
    <Bench
      scene={session.scene}
      register={session.register}
      selected={session.selected}
      onselect={(id) => {
        void session.inspect(id);
        pane = "notes";
      }}
      pristine={session.commandLog.length === 0 && !session.lesson}
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
        onparticles={() => void session.particles()}
        onclose={() => session.closeInspector()}
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
    min-width: 0;
    min-height: 0;
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
