<script lang="ts">
  import type { ShelfItem } from "../session.svelte";
  import KitStrip from "./KitStrip.svelte";

  let {
    name,
    next,
    busy,
    deviation = 0,
    kit = [],
    register = "lv2",
    target = 0,
    onnext,
    onreturn,
    onexit,
    onadd,
  }: {
    name: string;
    next: string | null;
    busy: boolean;
    /** Free commands run since the lesson's last own step. */
    deviation?: number;
    kit?: ShelfItem[];
    register?: string;
    target?: number;
    onnext: () => void;
    /** Rewind the deviation (an undo, not an erasure). */
    onreturn?: () => void;
    onexit: () => void;
    onadd?: (line: string) => void;
  } = $props();
</script>

<div class="lesson" role="region" aria-label={`lesson ${name}`}>
  <div class="controls">
    <span class="name">{name}</span>
    {#if next}
      <code>{next}</code>
      <button class="next" onclick={onnext} disabled={busy}>do it</button>
    {/if}
    {#if deviation > 0}
      <span class="deviation">
        off the script by {deviation} {deviation === 1 ? "step" : "steps"} — exploring is allowed
      </span>
      <button onclick={onreturn} disabled={busy}>return to the script</button>
    {/if}
    <button class="leave" onclick={onexit}>leave lesson</button>
  </div>
  {#if kit.length > 0 && onadd}
    <KitStrip items={kit} {register} {target} {onadd} />
  {/if}
</div>

<style>
  .lesson {
    display: flex;
    flex-direction: column;
    padding: 0.45rem 1rem;
    border-bottom: 1px solid var(--edge);
    background: var(--panel);
    font-size: 0.85rem;
    gap: 0.25rem;
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 0.7rem;
    flex-wrap: wrap;
  }
  .name {
    color: var(--warn);
  }
  code {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    padding: 0.15rem 0.5rem;
  }
  button {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.25rem 0.7rem;
    cursor: pointer;
    min-height: 34px;
  }
  .next {
    border-color: var(--hot);
  }
  .deviation {
    color: var(--dim);
    font-size: 0.78rem;
  }
  .leave {
    margin-left: auto;
  }
</style>
