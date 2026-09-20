/**
 * GUI-112 — what may follow this word, derived rather than listed.
 *
 * The owner wants a prompt bar with autocomplete: *"suggestions from
 * available commands, parts, chemicals, etc, relative to context insofar
 * as possible"*. The widget was never the hard part — `CommandBar.svelte`
 * has existed since GUI-005 and `session.parse(line)` already validates a
 * line without running it. The hard part is the completion MODEL, and the
 * temptation is an array in this file naming the verbs and a second one
 * naming the reagents. `reagentRoles.ts` argues at length why that is the
 * wrong shape and it applies here word for word: a hand list goes stale
 * within a month, silently, because nothing tells it that the engine grew
 * a verb or the data pack grew a species.
 *
 * So nothing here names a verb, a vessel or a chemical. Every suggestion
 * comes from something the engine already ships:
 *
 *   - **The verbs** are `kerotakis_core::script::VERBS`, reaching the app
 *     through `Lab::grammar()` — the same inventory the protocol
 *     conformance suite checks `affordances.json` against, so a verb the
 *     bar offers is a verb the parser has. Each row carries a canonical
 *     English `example` and (I18N) a `typed` line spelled as a learner of
 *     this session's language would write it. The FIRST WORD of `typed`
 *     is that language's word for the verb, produced by the engine's own
 *     alias layer, so a German reader is offered *erhitze* without this
 *     file knowing any German.
 *
 *   - **The vessels** are the scene's own `vessels[].id`, which is what
 *     `vN` counts.
 *
 *   - **The chemicals** are `session.shelf`, the engine registry as the
 *     shelf already has it — key, formula and translatable name.
 *
 * And the CONTEXT — what belongs at *this* position of *this* verb — is
 * the example line itself, read as a positional schema. `VERBS` says
 * `add` is `add v1 water 100mL`: position 1 is a vessel because the
 * example has a vessel there, and position 2 is a chemical because the
 * example has a shelf species there. That is the one idea in this file.
 * It costs no new engine surface, it cannot drift from the grammar it is
 * read out of, and a verb added tomorrow arrives with its own schema.
 *
 * Where the example says something else — a unit, a number, a joining
 * word like `until` or `stages` — this slice offers nothing rather than
 * guessing. See the roadmap entry for what that leaves for a second one.
 */

/** Which inventory a suggestion came from; the popup groups by it. */
export type CompletionKind = "verb" | "vessel" | "reagent";

export interface Completion {
  kind: CompletionKind;
  /** What replaces the word being typed. Always something the parser takes. */
  insert: string;
  /** What the reader sees, in their language. */
  label: string;
  /** The second line: an example, a vessel's kind, a formula. May be empty. */
  hint: string;
}

export interface GrammarRow {
  /** The canonical English verb — the parser's own token. */
  verb: string;
  /** The canonical English example line; this file reads it as a schema. */
  example: string;
  /** The same line as a learner of this language would type it, or null. */
  typed?: string | null;
}

export interface CompletionSources {
  grammar: GrammarRow[];
  vessels: { id: number; label: string }[];
  shelf: { key: string; name: string; formula?: string }[];
  /** The app's `t()`. Engine words are English keys with catalogue rows. */
  translate: (text: string) => string;
}

export interface CompletionResult {
  /** Index of the first character the suggestion replaces. */
  start: number;
  /** Index one past the last character it replaces. */
  end: number;
  options: Completion[];
}

/** The most a popup offers at once: a list nobody scrolls is a list nobody reads. */
export const COMPLETION_LIMIT = 8;

const EMPTY: CompletionResult = { start: 0, end: 0, options: [] };

const words = (line: string): string[] => line.split(/\s+/).filter((w) => w.length > 0);

/** This language's word for a verb: the first word of its own example. */
export function verbWord(row: GrammarRow): string {
  return words(row.typed ?? "")[0] ?? row.verb;
}

/**
 * The word the caret is in, and where it sits in the line.
 *
 * Whitespace-separated, because that is how `parse_command` splits a line.
 * A caret immediately after a space starts a NEW word — the reader has
 * finished one argument and is about to write the next, which is exactly
 * when a suggestion is most useful and when an empty query should offer
 * everything that fits rather than nothing.
 */
export function wordAt(line: string, caret: number): { text: string; start: number; end: number; index: number } {
  const position = Math.max(0, Math.min(caret, line.length));
  let start = position;
  while (start > 0 && !/\s/.test(line[start - 1]!)) start -= 1;
  let end = position;
  while (end < line.length && !/\s/.test(line[end]!)) end += 1;
  return {
    text: line.slice(start, end),
    start,
    end,
    index: words(line.slice(0, start)).length,
  };
}

const fold = (text: string): string => text.trim().toLowerCase();

/** Prefix match on either spelling. An empty query matches everything. */
const matches = (query: string, ...candidates: string[]): boolean => {
  const needle = fold(query);
  return needle === "" || candidates.some((candidate) => fold(candidate).startsWith(needle));
};

/**
 * The grammar row the line's first word names — in either language.
 *
 * The alias layer lets a German reader type `erhitze` and an English one
 * type `heat`, and `parse_command` accepts both, so the bar has to
 * recognise both before it can say what follows.
 */
export function rowForVerb(grammar: GrammarRow[], word: string): GrammarRow | undefined {
  const needle = fold(word);
  return grammar.find((row) => fold(row.verb) === needle || fold(verbWord(row)) === needle);
}

function verbOptions(grammar: GrammarRow[], query: string): Completion[] {
  const options: Completion[] = [];
  for (const row of grammar) {
    const localised = verbWord(row);
    const hitsLocalised = matches(query, localised);
    const hitsCanonical = matches(query, row.verb);
    if (!hitsLocalised && !hitsCanonical) continue;
    // Typing English keeps English. A reader who has written `hea` is
    // writing `heat`, and completing them into `erhitze` would be the bar
    // rewriting their sentence rather than finishing it.
    const insert = hitsLocalised ? localised : row.verb;
    options.push({
      kind: "verb",
      insert,
      label: insert,
      // The whole line, which is what turns a verb into something a
      // reader can actually write: `heat` alone says nothing about `40kJ`.
      hint: row.typed ?? row.example,
    });
  }
  return options;
}

function vesselOptions(sources: CompletionSources, query: string): Completion[] {
  return sources.vessels
    .map((vessel) => ({
      kind: "vessel" as const,
      insert: `v${vessel.id + 1}`,
      label: `v${vessel.id + 1}`,
      hint: sources.translate(vessel.label),
    }))
    .filter((option) => matches(query, option.insert));
}

function reagentOptions(sources: CompletionSources, query: string): Completion[] {
  return sources.shelf
    .map((item) => ({
      kind: "reagent" as const,
      // The KEY is inserted, because that is the word the grammar is
      // guaranteed to take; the reader's own language is what they SEE
      // and what they can search on. A species alias may or may not exist
      // in every language, and a suggestion that does not parse is worse
      // than no suggestion.
      insert: item.key,
      label: sources.translate(item.name),
      hint: item.formula ?? "",
    }))
    .filter((option) => matches(query, option.insert, option.label));
}

/**
 * What the example line expects at this position.
 *
 * `null` where the slot is a number, a unit or a joining word — this
 * slice says nothing rather than guessing at an amount.
 */
export function slotAt(sources: CompletionSources, row: GrammarRow, index: number): CompletionKind | null {
  const slot = words(row.example)[index];
  if (slot === undefined) return null;
  if (/^v\d+$/i.test(slot)) return "vessel";
  if (sources.shelf.some((item) => fold(item.key) === fold(slot))) return "reagent";
  return null;
}

/** Everything the bar may offer for the word the caret is in. */
export function completionsFor(
  line: string,
  caret: number,
  sources: CompletionSources,
): CompletionResult {
  const word = wordAt(line, caret);
  if (word.index === 0) {
    return { start: word.start, end: word.end, options: verbOptions(sources.grammar, word.text).slice(0, COMPLETION_LIMIT) };
  }
  const row = rowForVerb(sources.grammar, words(line)[0] ?? "");
  if (!row) return EMPTY;
  const slot = slotAt(sources, row, word.index);
  if (slot === null) return EMPTY;
  const options = slot === "vessel" ? vesselOptions(sources, word.text) : reagentOptions(sources, word.text);
  return { start: word.start, end: word.end, options: options.slice(0, COMPLETION_LIMIT) };
}

/** The line with one suggestion taken, and where the caret lands after it. */
export function applyCompletion(
  line: string,
  result: CompletionResult,
  option: Completion,
): { line: string; caret: number } {
  const head = `${line.slice(0, result.start)}${option.insert}`;
  const tail = line.slice(result.end);
  // A trailing space, because every one of these is followed by something
  // — a vessel by an amount, a verb by its first argument. Not doubled
  // where the line already has one.
  const spaced = tail.startsWith(" ") ? tail : ` ${tail}`;
  return { line: `${head}${spaced}`, caret: head.length + 1 };
}
