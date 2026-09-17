/** The keys the PLAYER derives are the keys the CATALOGUE was written for.
 *
 * Lesson prose is keyed by a label the `.lab` carries, and two parsers
 * have to agree about which lines a label owns: `tools/lesson_prose.py`,
 * which builds the payload and generates `lessons/prose/en.toml`, and
 * `parseLesson` here, which is what the screen actually asks with. If they
 * ever disagree the app asks for a key nothing ships, and every check on
 * either side stays green while the reader meets English — #505's failure
 * shape, one layer along.
 *
 * So this walks the 113 real lessons with the real player parser and holds
 * its key set against the generated catalogue. It is deliberately not a
 * fixture: a fixture would be a third thing to keep in step.
 *
 * `en.toml` is read with a small regex rather than a TOML library because
 * the app ships no TOML parser and should not grow one for a test. The
 * file is GENERATED in exactly this shape, and `tools/lesson-prose-lint.py
 * --check` fails if it drifts from the lessons, so the narrow read is safe.
 */
import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { parseLesson } from "./lesson";

const LESSONS = join(dirname(fileURLToPath(import.meta.url)), "../../../../lessons");
const files = readdirSync(LESSONS).filter((f) => f.endsWith(".lab"));

/** `["stem"]` headers and `label = "…"` rows, flattened to dotted keys. */
function catalogueKeys(toml: string): Set<string> {
  const keys = new Set<string>();
  let stem = "";
  for (const raw of toml.split("\n")) {
    const line = raw.trim();
    if (line.startsWith("#") || !line) continue;
    const table = /^\["?([^"\]]+)"?\]$/.exec(line);
    if (table) {
      stem = table[1]!;
      continue;
    }
    const row = /^([A-Za-z0-9_.-]+)\s*=/.exec(line);
    if (row && stem) keys.add(`${stem}.${row[1]!}`);
  }
  return keys;
}

const playerKeys = new Set<string>(
  files.flatMap((file) => {
    const stem = file.replace(/\.lab$/, "");
    return parseLesson(stem, readFileSync(join(LESSONS, file), "utf8")).steps.flatMap(
      (step) => (step.kind === "note" && step.key ? [step.key] : []),
    );
  }),
);

describe("lesson prose keys", () => {
  it("walks the shipped lessons, so the comparison is not vacuous", () => {
    expect(files.length).toBeGreaterThan(100);
  });

  it("derives exactly the keys the generated English catalogue carries", () => {
    const authored = catalogueKeys(readFileSync(join(LESSONS, "prose/en.toml"), "utf8"));
    expect([...playerKeys].sort()).toEqual([...authored].sort());
  });

  it("finds a German row for every key the player will ask for", () => {
    // German is the complete language; a hole in it is a bug rather than
    // the ordinary half-translated state other languages may be in.
    const german = catalogueKeys(readFileSync(join(LESSONS, "prose/de.toml"), "utf8"));
    const missing = [...playerKeys].filter((key) => !german.has(key)).sort();
    expect(missing).toEqual([]);
  });

  it("ships no German row that no lesson asks for", () => {
    const german = catalogueKeys(readFileSync(join(LESSONS, "prose/de.toml"), "utf8"));
    const orphans = [...german].filter((key) => !playerKeys.has(key)).sort();
    expect(orphans).toEqual([]);
  });
});
