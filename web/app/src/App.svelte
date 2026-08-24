<script lang="ts">
  import { onMount } from "svelte";
  import { Session } from "./lib/session.svelte";
  import { WorkerHost, resolvePayloadBase } from "./lib/host/WorkerHost";
  import Bench from "./lib/components/Bench.svelte";
  import Feed from "./lib/components/Feed.svelte";
  import CommandBar from "./lib/components/CommandBar.svelte";
  import RegisterDial from "./lib/components/RegisterDial.svelte";
  import Shelf from "./lib/components/Shelf.svelte";
  import Inspector from "./lib/components/Inspector.svelte";
  import Timeline from "./lib/components/Timeline.svelte";
  import LessonBar from "./lib/components/LessonBar.svelte";

  const session = new Session(WorkerHost.create());
  let lessons = $state<{ file: string; name: string }[]>([]);

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

  function saveLab() {
    const blob = new Blob([session.exportLab()], { type: "text/plain" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = "session.lab";
    a.click();
    URL.revokeObjectURL(a.href);
  }

  function onkeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === "z" && !e.shiftKey) {
      e.preventDefault();
      void session.undo();
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
      {#each lessons as l (l.file)}
        <option value={l.file}>{l.name}</option>
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

<main>
  <nav class="shelf-pane">
    <Shelf
      items={session.shelf}
      register={session.register}
      target={session.selected}
      onadd={(line) => void session.submit(line)}
    />
  </nav>
  <Bench
    scene={session.scene}
    register={session.register}
    selected={session.selected}
    onselect={(id) => void session.inspect(id)}
  />
  <aside>
    {#if session.inspector}
      <Inspector
        vessel={session.inspector.vessel}
        lines={session.inspector.lines}
        onparticles={() => void session.particles()}
        onclose={() => session.closeInspector()}
      />
    {/if}
    <Feed entries={session.feed} />
  </aside>
</main>

<CommandBar onsubmit={(line) => void session.submit(line)} busy={session.busy} />

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
  @media (max-width: 900px) {
    main {
      flex-direction: column;
    }
    .shelf-pane {
      width: auto;
      max-height: 26vh;
      border-right: 0;
      border-bottom: 1px solid var(--edge);
    }
    aside {
      width: auto;
      border-left: 0;
      border-top: 1px solid var(--edge);
      max-height: 38vh;
    }
  }
</style>
