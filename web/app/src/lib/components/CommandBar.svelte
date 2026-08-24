<script lang="ts">
  let {
    onsubmit,
    busy,
    onvalidate,
  }: {
    onsubmit: (line: string) => void;
    busy: boolean;
    onvalidate?: (line: string) => Promise<{ ok: boolean; error?: string }>;
  } = $props();

  let line = $state("");
  let history: string[] = [];
  let cursor = $state(-1);
  let draft = "";
  /** null = nothing to say; string = the grammar's complaint. */
  let problem = $state<string | null>(null);
  let debounce: ReturnType<typeof setTimeout> | undefined;

  // Live validation (GUI-005): ask the engine's parser, debounced, and
  // only ever complain — silence while typing something valid.
  $effect(() => {
    const current = line;
    clearTimeout(debounce);
    if (!current.trim() || !onvalidate) {
      problem = null;
      return;
    }
    debounce = setTimeout(() => {
      void onvalidate(current).then((r) => {
        if (line === current) problem = r.ok ? null : (r.error ?? "not a command");
      });
    }, 300);
  });

  function submit(event: SubmitEvent) {
    event.preventDefault();
    const trimmed = line.trim();
    if (!trimmed) return;
    if (history.at(-1) !== trimmed) history.push(trimmed);
    cursor = -1;
    onsubmit(trimmed);
    line = "";
    problem = null;
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "ArrowUp") {
      if (history.length === 0) return;
      e.preventDefault();
      if (cursor === -1) {
        draft = line;
        cursor = history.length - 1;
      } else if (cursor > 0) {
        cursor -= 1;
      }
      line = history[cursor] ?? "";
    } else if (e.key === "ArrowDown") {
      if (cursor === -1) return;
      e.preventDefault();
      if (cursor < history.length - 1) {
        cursor += 1;
        line = history[cursor] ?? "";
      } else {
        cursor = -1;
        line = draft;
      }
    }
  }
</script>

<div class="wrap">
  {#if problem}
    <p class="problem" role="status">{problem}</p>
  {/if}
  <form class="bar" class:invalid={problem !== null} onsubmit={submit}>
    <span class="prompt" aria-hidden="true">kero&gt;</span>
    <input
      type="text"
      bind:value={line}
      {onkeydown}
      placeholder="add v1 water 100mL"
      aria-label="command"
      aria-invalid={problem !== null}
      autocomplete="off"
      autocapitalize="off"
      spellcheck="false"
      disabled={busy}
    />
  </form>
</div>

<style>
  .wrap {
    background: var(--panel);
  }
  .problem {
    margin: 0;
    padding: 0.25rem 1rem 0;
    font-size: 0.75rem;
    color: var(--warn);
  }
  .bar {
    display: flex;
    align-items: center;
    border-top: 1px solid var(--edge);
  }
  .bar.invalid {
    border-top-color: var(--warn);
  }
  .prompt {
    color: var(--hot);
    padding: 0 0.4rem 0 1rem;
  }
  input {
    flex: 1;
    background: none;
    border: 0;
    color: var(--ink);
    font: inherit;
    padding: 0.8rem 1rem 0.8rem 0;
    outline: none;
    min-height: 44px;
  }
</style>
