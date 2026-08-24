<script lang="ts">
  import { onMount } from "svelte";
  import { Session } from "./lib/session.svelte";
  import { WorkerHost } from "./lib/host/WorkerHost";
  import Bench from "./lib/components/Bench.svelte";
  import Feed from "./lib/components/Feed.svelte";
  import CommandBar from "./lib/components/CommandBar.svelte";
  import RegisterDial from "./lib/components/RegisterDial.svelte";

  const session = new Session(WorkerHost.create());
  onMount(() => void session.connect());
</script>

<header>
  <h1>Kerotakis <small>a chemistry bench that computes</small></h1>
  <RegisterDial value={session.register} onchange={(lv) => void session.setRegister(lv)} />
  <span class="status" class:live={session.canSolve}>
    {session.engineReady ? (session.canSolve ? "live" : "shipped results") : "starting…"}
  </span>
</header>

<main>
  <Bench scene={session.scene} register={session.register} />
  <aside>
    <Feed entries={session.feed} />
  </aside>
</main>

<CommandBar onsubmit={(line) => void session.submit(line)} busy={session.busy} />

<style>
  header {
    display: flex;
    align-items: center;
    gap: 1rem;
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
  .status {
    margin-left: auto;
    font-size: 0.8rem;
    color: var(--warn);
  }
  .status.live {
    color: var(--good);
  }
  main {
    display: flex;
    flex: 1;
    min-height: 0;
  }
  aside {
    width: min(24rem, 40vw);
    border-left: 1px solid var(--edge);
    background: var(--panel);
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  aside > :global(.feed) {
    flex: 1;
  }
  @media (max-width: 700px) {
    main {
      flex-direction: column;
    }
    aside {
      width: auto;
      border-left: 0;
      border-top: 1px solid var(--edge);
      max-height: 38vh;
    }
  }
</style>
