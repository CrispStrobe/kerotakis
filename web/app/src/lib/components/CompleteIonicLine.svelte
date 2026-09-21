<script lang="ts">
  /**
   * The complete ionic equation, with the spectators struck through
   * (GUI-092).
   *
   * The net line says what happened. This says *why* it is what it is: the
   * ions that took no part stand on both sides, drawn with a line through
   * them so the cancellation is something a learner watches happen rather
   * than something they are told about.
   *
   * **Every decision here was made in the engine.** Which ions, how many of
   * each, and which ones cancel all arrive solved and verified in
   * `CompleteIonic`; a term carries `spectator: true` and this draws the
   * line. The component never works out a coefficient and never assembles
   * an equation — where the engine could not solve one it emits none, the
   * strip shows the net line alone, and filling that gap from the shell
   * would be exactly the fiction the engine refused to write.
   */
  import { writtenTerm, type CompleteIonic } from "../ionic";

  let {
    complete,
    label,
  }: {
    complete: CompleteIonic;
    label: string;
  } = $props();
</script>

<span class="complete">
  <span class="complete-tag">{label}</span>
  {#each complete.reactants as term, i (`r${i}-${term.species}`)}
    {#if i > 0}<span class="op">+</span>{/if}{#if term.spectator}<s
        class="struck">{writtenTerm(term)}</s>{:else}<span
        class="term">{writtenTerm(term)}</span>{/if}
  {/each}
  <span class="arrow">→</span>
  {#each complete.products as term, i (`p${i}-${term.species}`)}
    {#if i > 0}<span class="op">+</span>{/if}{#if term.spectator}<s
        class="struck">{writtenTerm(term)}</s>{:else}<span
        class="term">{writtenTerm(term)}</span>{/if}
  {/each}
</span>

<style>
  /* A third row under the molecular and net lines, and quieter than both:
     it is the same reaction said at greater length, not another one. The
     strip's own tokens, so it sits in the row rather than beside it —
     and kept HERE rather than in the host's stylesheet, because the
     strike-through and its spacing belong to whoever draws them. */
  .complete {
    display: block;
    margin-top: 0.15rem;
    font-size: 0.85em;
    color: var(--ink-muted);
  }
  .complete-tag {
    margin-right: 0.45rem;
    font-size: 0.78em;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    opacity: 0.75;
  }
  .op,
  .arrow {
    margin: 0 0.3rem;
  }
  /* The strike is the whole point of the line, so it is drawn in the ink
     the rest of the equation is written in rather than in a muted grey:
     a cancelled term is still a term the reader has to read before they
     can watch it go. The opacity does the "spent" part; a low-contrast
     colour would do the "unreadable" part instead. */
  .struck {
    opacity: 0.6;
    text-decoration-thickness: 0.09em;
  }
</style>
