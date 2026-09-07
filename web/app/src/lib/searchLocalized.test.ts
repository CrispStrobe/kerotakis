/** The filter must match the language on screen, not the one underneath.
 *
 * The catalogue rendered German and filtered English, so a reader typing
 * "Säure" got "nothing matches" while the German word was visible in the
 * list. Nothing failed: an empty result set looks like a legitimate answer.
 * That is why this is a test and not a comment — the failure mode is
 * silence, and silence is what the other gates were reading as success.
 */
import { describe, expect, it } from "vitest";
import type { CodexEntry } from "./codex";
import { experimentMatches } from "./catalogSearch";

/** A stub standing in for the shell dictionary. */
const DE: Record<string, string> = {
  "acid base titration": "Säure-Base-Titration",
  "strong acids": "starke Säuren",
  "A strong acid neutralises a strong base.":
    "Eine starke Säure neutralisiert eine starke Base.",
};
const de = (text: string) => DE[text] ?? text;
const en = (text: string) => text;

const ENTRY = {
  id: "acid-base-titration",
  progress: "starter",
  equation: "HCl + NaOH -> NaCl + H2O",
  summary: "A strong acid neutralises a strong base.",
  concepts: ["strong-acids"],
  setup: { script: "add v1 water 100mL" },
} as CodexEntry;

describe("experimentMatches", () => {
  it("finds an entry by the German title the reader can see", () => {
    expect(experimentMatches(ENTRY, "Säure", de)).toBe(true);
  });

  it("finds an entry by German prose in its summary", () => {
    expect(experimentMatches(ENTRY, "neutralisiert", de)).toBe(true);
  });

  it("finds an entry by a German concept name", () => {
    expect(experimentMatches(ENTRY, "starke", de)).toBe(true);
  });

  it("still finds it by the English term while the UI is German", () => {
    // Reagents and reactions are often learned in English; a German UI
    // should not stop "titration" or a formula from matching.
    expect(experimentMatches(ENTRY, "titration", de)).toBe(true);
    expect(experimentMatches(ENTRY, "NaOH", de)).toBe(true);
  });

  it("matches nothing that is genuinely absent", () => {
    expect(experimentMatches(ENTRY, "chromatographie", de)).toBe(false);
  });

  it("is case- and whitespace-insensitive", () => {
    expect(experimentMatches(ENTRY, "  SÄURE  ", de)).toBe(true);
  });

  it("keeps every entry when the box is empty", () => {
    expect(experimentMatches(ENTRY, "", de)).toBe(true);
    expect(experimentMatches(ENTRY, "   ", de)).toBe(true);
  });

  it("behaves as before in English", () => {
    expect(experimentMatches(ENTRY, "acid", en)).toBe(true);
    expect(experimentMatches(ENTRY, "Säure", en)).toBe(false);
  });

  it("survives an entry with no summary, equation or concepts", () => {
    const bare = { id: "bare-entry", progress: "starter", setup: { script: "" } } as CodexEntry;
    expect(experimentMatches(bare, "bare", de)).toBe(true);
    expect(experimentMatches(bare, "Säure", de)).toBe(false);
  });
});
