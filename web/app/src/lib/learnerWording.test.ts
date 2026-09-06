/**
 * No learner is addressed by age (GUI-470).
 *
 * The German build shipped a shelf called "EXPERIMENTIERKÄSTEN FÜR KINDER",
 * catalogue cards stamped "ab 8 Jahren", and a capability filter offering
 * "9–12 Jahre". Every one of those is the same mistake: this bench addresses
 * every person who opens it. An adult who wants to blow up a balloon with
 * vinegar is not doing a children's experiment, and a fourteen-year-old
 * reading "ab 8 Jahren" on a card reads a verdict on themselves rather than
 * a description of the work.
 *
 * The content IS graded — it has to be — but it is graded by LEVEL: "first
 * steps", "going further", "in depth". Those are claims about the chemistry,
 * which is what a level is for; an age band is a claim about the reader.
 *
 * So this walks what a learner can actually be shown — every translated
 * value, every source key a component asks `t()` for — and fails on the
 * vocabulary. It is deliberately a wording test and not a lint: the
 * underlying identifiers (`age_band`, `ageMin`, `KIDS_EQUIPMENT`,
 * `levelForAge`) are fine and stay, because the corpus really is banded by
 * school placement. What must never happen is that banding reaching the
 * interface as words.
 */
import { readdirSync, readFileSync } from "node:fs";
import { extname, join } from "node:path";
import { describe, expect, it } from "vitest";

const LOCALES = join(import.meta.dirname, "../locales");

const bundle = (file: string) =>
  JSON.parse(readFileSync(join(LOCALES, file), "utf8")) as {
    terms?: Record<string, string>;
    messages?: Record<string, string>;
  };

/**
 * Word-boundaried, case-insensitive, and both languages at once — a German
 * file carries English keys, so both halves of every entry are scanned.
 *
 * `Jahr` alone is NOT here: "Jedes Jahr müssen Menschen ins Krankenhaus"
 * is a hazard warning about how often something happens, which is a fact
 * about the world rather than a label on the reader. The plural forms are,
 * because "9–12 Jahre" and "ab 8 Jahren" are how an age band is written.
 */
const FORBIDDEN =
  /\b(kind|kinder|kindern|kindes|kinderlabor|kinderversuch|kinderexperiment|kids?|child|childs|children|children's|alter|altersgruppe|altersgruppen|altersband|jahre|jahren|jahrgang|ages?|aged|years)\b/i;

/**
 * Words that are the chemistry, not the audience.
 *
 * `ageing` of a precipitate and the `alter` of an operation are real terms;
 * none is in the bundles today, and this list is where one goes if it ever
 * is — with the reason, so the exemption is arguable rather than inherited.
 */
const ALLOWED = new Set<string>([]);

const offences = (text: string): boolean => !ALLOWED.has(text) && FORBIDDEN.test(text);

function sourceFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return sourceFiles(path);
    if (entry.name.endsWith(".test.ts")) return [];
    return [".svelte", ".ts"].includes(extname(entry.name)) ? [path] : [];
  });
}

describe("the interface addresses a learner by level, never by age", () => {
  it.each(readdirSync(LOCALES).filter((file) => file.endsWith(".json")))(
    "%s has no age or children's wording, in key or in translation",
    (file) => {
      const found: string[] = [];
      const data = bundle(file);
      for (const section of ["terms", "messages"] as const) {
        for (const [key, value] of Object.entries(data[section] ?? {})) {
          if (offences(key)) found.push(`${section} key: ${key}`);
          if (typeof value === "string" && offences(value)) found.push(`${section} value: ${key} → ${value}`);
        }
      }
      expect(found).toEqual([]);
    },
  );

  it("no component asks t() for a string that names an age or a child", () => {
    const found: string[] = [];
    for (const path of sourceFiles(join(import.meta.dirname))) {
      const source = readFileSync(path, "utf8")
        .replace(/\/\*[\s\S]*?\*\//g, "")
        .replace(/(^|[^:])\/\/.*$/gm, "$1");
      for (const match of source.matchAll(/\bt\("([^"]+)"/g)) {
        if (offences(match[1]!)) found.push(`${path}: ${match[1]}`);
      }
    }
    expect(found).toEqual([]);
  });

  it("the level names it uses instead say nothing about a reader's age", () => {
    const de = bundle("de.json");
    const german = (key: string) => de.messages?.[key] ?? de.terms?.[key];
    for (const level of ["first steps", "going further", "in depth"]) {
      const word = german(level);
      expect(word, `${level} has no German`).toBeTruthy();
      expect(offences(word!), `${level} → ${word}`).toBe(false);
    }
    expect(german("all levels")).toBe("Alle Stufen");
  });
});
