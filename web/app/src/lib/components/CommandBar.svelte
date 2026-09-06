<script lang="ts">
  import { i18n, t } from "../i18n.svelte";
  let {
    onsubmit,
    busy,
    onvalidate,
    examples = [],
  }: {
    onsubmit: (line: string) => void;
    busy: boolean;
    onvalidate?: (line: string) => Promise<{ ok: boolean; error?: string }>;
    /** One example line per verb, already in the learner's language
     * (I18N): the engine's own inventory, so the bar can never offer a
     * verb the parser does not have. */
    examples?: string[];
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
      placeholder={t("add v1 water 100mL")}
      aria-label={t("command")}
      aria-invalid={problem !== null}
      autocomplete="off"
      autocapitalize="off"
      spellcheck="false"
      list={examples.length > 0 ? "kero-verbs" : undefined}
      disabled={busy}
    />
    <!-- The grammar, offered rather than remembered. In German these are
         German lines, because the engine composed them from the same
         alias tables its parser reads — a suggestion here is always a
         line the bench will take. -->
    {#if examples.length > 0}
      <datalist id="kero-verbs">
        {#each examples as example}
          <option value={example}></option>
        {/each}
      </datalist>
    {/if}
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
