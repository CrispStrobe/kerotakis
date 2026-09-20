<script lang="ts">
  import { i18n, t } from "../i18n.svelte";
  import {
    applyCompletion,
    completionsFor,
    type Completion,
    type CompletionSources,
  } from "../completions";
  let {
    onsubmit,
    busy,
    onvalidate,
    completionSources,
    onclose,
  }: {
    onsubmit: (line: string) => void;
    busy: boolean;
    onvalidate?: (line: string) => Promise<{ ok: boolean; error?: string }>;
    /**
     * GUI-112. Everything the bar may suggest, in the shapes the engine
     * ships them: the grammar's own verb inventory with its example
     * lines, the scene's vessels, and the shelf. Absent (an older host,
     * or a caller that has none) and the bar is exactly what it was — the
     * popup simply never opens.
     */
    completionSources?: CompletionSources;
    /** Absent where the console is not dismissible (the desktop shell). */
    onclose?: () => void;
  } = $props();

  let line = $state("");
  let caret = $state(0);
  let input = $state<HTMLInputElement | null>(null);
  let history: string[] = [];
  let cursor = $state(-1);
  let draft = "";
  /** null = nothing to say; string = the grammar's complaint. */
  let problem = $state<string | null>(null);
  let debounce: ReturnType<typeof setTimeout> | undefined;

  /* ── The completion popup (GUI-112) ────────────────────────────────
   *
   * An ARIA combobox, not a list of divs: this repo audits keyboard
   * paths and 44 px touch targets, and a popup that a screen reader
   * cannot announce is a feature only some readers get. The input owns
   * `aria-expanded`/`aria-controls`/`aria-activedescendant`; the list is
   * a `listbox` of `option`s; arrows move, Escape closes, Enter takes.
   */
  /**
   * A mark and a word per inventory, so "v2" and "NaCl" are not two
   * unexplained strings in one list. The mark is decorative; the word is
   * for a screen reader, which announces an option's whole text and
   * would otherwise read the two rows identically.
   */
  const KIND_MARKS = { verb: "\u25b8", vessel: "\u25bd", reagent: "\u25cf" } as const;
  const KIND_LABELS = { verb: "command", vessel: "vessel", reagent: "chemical" } as const;
  /**
   * Why the popup never fights the typist.
   *
   * It starts dismissed, so the bar is a bar until the reader touches it:
   * a list of every verb the moment the console opens is a wall of text
   * nobody asked for, over the journal it sits on. Focus or a keystroke
   * opens it; blur, Escape and running a line close it; and Escape stays
   * meant until the next keystroke changes the word being completed.
   */
  let dismissed = $state(true);
  let active = $state(0);
  const listId = "kero-completions";
  const suggestions = $derived(
    completionSources && !busy
      ? completionsFor(line, caret, completionSources)
      : { start: 0, end: 0, options: [] },
  );
  const open = $derived(!dismissed && suggestions.options.length > 0);
  const activeIndex = $derived(Math.min(active, Math.max(0, suggestions.options.length - 1)));
  const optionId = (index: number) => `${listId}-${index}`;

  function take(option: Completion) {
    const next = applyCompletion(line, suggestions, option);
    line = next.line;
    caret = next.caret;
    dismissed = false;
    active = 0;
    // The caret has to be MOVED, not just recorded: the reader carries on
    // typing the next argument, and a caret left at the end of the line
    // would complete the wrong word on a line they went back to edit.
    const target = input;
    if (target) {
      queueMicrotask(() => {
        target.focus();
        target.setSelectionRange(next.caret, next.caret);
      });
    }
  }

  /** Wherever the caret may have moved — a click, an arrow, a selection. */
  function syncCaret(event: Event) {
    const field = event.currentTarget as HTMLInputElement;
    caret = field.selectionStart ?? field.value.length;
  }

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
        if (line === current) problem = r.ok ? null : (r.error ?? t("not a command"));
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
    caret = 0;
    // Closed, not reopened: a popup that springs back over the feed the
    // instant a command runs would cover the answer the reader just
    // asked for. Their next keystroke brings it back.
    dismissed = true;
    active = 0;
    problem = null;
  }

  // Voice input (GUI-028): progressive enhancement over the same
  // grammar. The transcript lands IN THE INPUT for the speaker to read,
  // correct, and submit — never executed straight from the microphone;
  // live parse validation judges it like anything typed. No support, no
  // button.
  type Recognition = {
    lang: string;
    interimResults: boolean;
    maxAlternatives: number;
    onresult: ((e: { results: { [i: number]: { [j: number]: { transcript: string } } } }) => void) | null;
    onend: (() => void) | null;
    onerror: ((e: { error?: string }) => void) | null;
    start(): void;
    stop(): void;
  };
  const RecognitionCtor =
    typeof window !== "undefined"
      ? ((window as { SpeechRecognition?: new () => Recognition; webkitSpeechRecognition?: new () => Recognition })
          .SpeechRecognition ??
        (window as { webkitSpeechRecognition?: new () => Recognition }).webkitSpeechRecognition ??
        null)
      : null;
  let listening = $state(false);
  let recognizer: Recognition | null = null;

  function toggleVoice() {
    if (listening) {
      recognizer?.stop();
      return;
    }
    if (!RecognitionCtor) return;
    recognizer = new RecognitionCtor();
    recognizer.lang = i18n.locale === "de" ? "de-DE" : "en-US";
    recognizer.interimResults = false;
    recognizer.maxAlternatives = 1;
    recognizer.onresult = (e) => {
      const heard = e.results[0]?.[0]?.transcript ?? "";
      // Spoken chemistry arrives in prose case; the grammar is lowercase.
      line = heard.trim().toLowerCase();
    };
    recognizer.onend = () => (listening = false);
    recognizer.onerror = () => (listening = false);
    listening = true;
    recognizer.start();
  }

  function onkeydown(e: KeyboardEvent) {
    // The popup takes the arrows only while it is open, so the history
    // recall this bar has always had is untouched the rest of the time.
    if (open) {
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        const count = suggestions.options.length;
        active = (activeIndex + (e.key === "ArrowDown" ? 1 : count - 1)) % count;
        return;
      }
      if (e.key === "Escape") {
        e.preventDefault();
        dismissed = true;
        return;
      }
      if (e.key === "Enter" || e.key === "Tab") {
        const option = suggestions.options[activeIndex];
        if (option) {
          // Enter takes the suggestion rather than running the line:
          // a reader looking at a highlighted row means that row, and a
          // command that ran instead would be a command they did not
          // finish writing.
          e.preventDefault();
          take(option);
          return;
        }
      }
    }
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
  <!-- Above the bar, because on a phone the bar sits at the foot of the
       screen and a list hanging below it would be under the thumb or off
       the bottom of the viewport. -->
  {#if open}
    <ul class="completions" id={listId} role="listbox" aria-label={t("suggestions")}>
      {#each suggestions.options as option, index (option.kind + option.insert)}
        <!-- The row IS the option, with no button inside it. An
             `option` may not contain interactive content, and it does not
             need to: keyboard focus stays in the input and
             `aria-activedescendant` moves — which is the whole point of
             the combobox pattern, and why the list is not a tab stop.
             `onpointerdown` rather than `onclick`, because the input's
             blur fires first and would close the popup out from under the
             finger pressing it. -->
        <li
          id={optionId(index)}
          role="option"
          aria-selected={index === activeIndex}
          class:active={index === activeIndex}
          onpointerdown={(e) => {
            e.preventDefault();
            take(option);
          }}
        >
          <span class="kind" aria-hidden="true">{KIND_MARKS[option.kind]}</span>
          <span class="what">
            <span class="label">{option.label}</span>
            <span class="sr-only">{t(KIND_LABELS[option.kind])}</span>
          </span>
          {#if option.hint}<span class="hint">{option.hint}</span>{/if}
        </li>
      {/each}
    </ul>
  {/if}
  {#if problem}
    <p class="problem" role="status">{problem}</p>
  {/if}
  <form class="bar" class:invalid={problem !== null} onsubmit={submit}>
    <span class="prompt" aria-hidden="true">kero&gt;</span>
    <input
      bind:this={input}
      type="text"
      bind:value={line}
      {onkeydown}
      oninput={(e) => {
        dismissed = false;
        active = 0;
        syncCaret(e);
      }}
      onfocus={(e) => {
        dismissed = false;
        syncCaret(e);
      }}
      onclick={syncCaret}
      onkeyup={syncCaret}
      onblur={() => (dismissed = true)}
      placeholder={t("add v1 water 100mL")}
      aria-label={t("command")}
      aria-invalid={problem !== null}
      autocomplete="off"
      autocapitalize="off"
      spellcheck="false"
      role="combobox"
      aria-expanded={open}
      aria-controls={listId}
      aria-autocomplete="list"
      aria-activedescendant={open ? optionId(activeIndex) : undefined}
      disabled={busy}
    />
    {#if RecognitionCtor}
      <button
        type="button"
        class="mic"
        class:listening
        onclick={toggleVoice}
        title={t("speak a command — it lands here to read and correct before you run it")}
        aria-label={listening ? t("stop listening") : t("speak a command")}
        aria-pressed={listening}
      >
        <svg viewBox="0 0 18 18" aria-hidden="true">
          <rect x="6.5" y="2" width="5" height="8" rx="2.5" />
          <path d="M 4 9 Q 4 13 9 13 Q 14 13 14 9 M 9 13 V 16 M 6.5 16 H 11.5" />
        </svg>
      </button>
    {/if}
    {#if onclose}
      <button type="button" class="icon-close" onclick={onclose} aria-label={t("hide the command line")} title={t("hide the command line")}>×</button>
    {/if}
  </form>
</div>

<style>
  .wrap {
    margin: 0 0.75rem 0.75rem;
    border: 1px solid var(--edge);
    border-radius: 14px;
    background: var(--surface);
    box-shadow: 0 6px 22px var(--shadow);
    overflow: hidden;
  }
  /* Above the bar and scrolling inside itself: on a phone the bar is at
     the foot of the screen, so a list hanging below it would be under the
     thumb or off the viewport entirely. */
  .completions {
    max-height: 15rem;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
    border-bottom: 1px solid var(--edge);
  }
  .completions li + li {
    border-top: 1px solid color-mix(in srgb, var(--edge) 55%, transparent);
  }
  .completions li {
    display: flex;
    min-height: 44px;
    align-items: center;
    gap: 0.5rem;
    padding: 0.35rem 1rem;
    color: var(--ink);
    font: 0.8rem/1.3 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    cursor: pointer;
  }
  .completions li.active,
  .completions li:hover {
    background: color-mix(in srgb, var(--action) 14%, transparent);
  }
  /* The highlight is a background AND a bar: colour alone is not a
     carrier, and this list is read at a glance. */
  .completions li.active {
    box-shadow: inset 3px 0 0 var(--action);
  }
  .completions .kind {
    flex: none;
    width: 1rem;
    color: var(--action);
    text-align: center;
  }
  .completions .what {
    min-width: 0;
    flex: none;
  }
  .completions .label {
    font-weight: 700;
  }
  .completions .hint {
    min-width: 0;
    overflow: hidden;
    flex: 1;
    color: var(--dim);
    font-size: 0.72rem;
    text-align: right;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
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
    border-top: 0;
  }
  .bar.invalid {
    border-top-color: var(--warn);
  }
  .prompt {
    color: var(--action);
    padding: 0 0.4rem 0 1rem;
    font: 700 0.78rem/1 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  }
  input {
    flex: 1;
    background: none;
    border: 0;
    color: var(--ink);
    font: 0.84rem/1.4 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    padding: 0.8rem 1rem 0.8rem 0;
    outline: none;
    min-height: 44px;
  }
  .mic {
    background: none;
    border: 0;
    color: var(--dim);
    cursor: pointer;
    padding: 0 1rem;
    min-height: 44px;
  }
  .mic svg {
    width: 18px;
    height: 18px;
  }
  .mic svg rect,
  .mic svg path {
    fill: none;
    stroke: currentColor;
    stroke-width: 1.3;
    stroke-linecap: round;
  }
  .mic:hover {
    color: var(--ink);
  }
  /* The console is opt-in, so it carries the one way back out — the same
     "×" every panel uses (app.css owns its look). Only the room around it
     belongs to this bar, so the 44px target lives on the padding. */
  .icon-close {
    margin-right: 0.7rem;
  }
  .mic.listening {
    color: var(--hot);
    animation: mic-pulse 1.2s ease-in-out infinite;
  }
  @keyframes mic-pulse {
    50% {
      opacity: 0.45;
    }
  }
  @media (max-width: 980px) {
    .wrap {
      margin-inline: 0.5rem;
      margin-bottom: 0.5rem;
    }
  }
</style>
