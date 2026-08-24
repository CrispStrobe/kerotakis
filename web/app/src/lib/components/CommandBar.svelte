<script lang="ts">
  let { onsubmit, busy }: { onsubmit: (line: string) => void; busy: boolean } = $props();

  let line = $state("");
  let history: string[] = [];
  let cursor = $state(-1);
  let draft = "";

  function submit(event: SubmitEvent) {
    event.preventDefault();
    const trimmed = line.trim();
    if (!trimmed) return;
    if (history.at(-1) !== trimmed) history.push(trimmed);
    cursor = -1;
    onsubmit(trimmed);
    line = "";
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

<form class="bar" onsubmit={submit}>
  <span class="prompt" aria-hidden="true">kero&gt;</span>
  <input
    type="text"
    bind:value={line}
    {onkeydown}
    placeholder="add v1 water 100mL"
    aria-label="command"
    autocomplete="off"
    autocapitalize="off"
    spellcheck="false"
    disabled={busy}
  />
</form>

<style>
  .bar {
    display: flex;
    align-items: center;
    border-top: 1px solid var(--edge);
    background: var(--panel);
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
