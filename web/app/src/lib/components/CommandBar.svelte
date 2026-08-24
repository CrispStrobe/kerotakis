<script lang="ts">
  let { onsubmit, busy }: { onsubmit: (line: string) => void; busy: boolean } = $props();

  let line = $state("");

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!line.trim()) return;
    onsubmit(line);
    line = "";
  }
</script>

<form class="bar" onsubmit={submit}>
  <span class="prompt" aria-hidden="true">kero&gt;</span>
  <input
    type="text"
    bind:value={line}
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
