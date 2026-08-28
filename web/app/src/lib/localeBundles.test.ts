/** A language file declares which language it is, and what to call it.
 *
 * `src/locales/*.json` are keyed into the app by the `@@locale` field
 * INSIDE each file, not by its filename, and the language picker shows
 * `@@name` falling back to the bare code. So the two ways of adding a
 * language wrongly are both silent:
 *
 *   - copy `_template.json` and forget to change `@@locale` from "xx",
 *     and the bundle registers as "xx". Selecting French does nothing,
 *     because no bundle claims "fr".
 *   - leave `@@name` empty and the picker offers "fr" rather than
 *     "Français" — a language chooser that a reader cannot read.
 *
 * Neither throws. This walks the directory rather than the app's own
 * import, because the failure being checked for is a file that the app
 * loaded under the wrong name — asking the app would ask the thing that
 * is already confused.
 */
import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const DIR = join(dirname(fileURLToPath(import.meta.url)), "../locales");
const files = readdirSync(DIR).filter((f) => f.endsWith(".json") && !f.startsWith("_"));

describe("locale bundles", () => {
  it("finds at least the German one, so the walk is not vacuous", () => {
    // A test that silently checks nothing is the failure mode this whole
    // translation kept hitting; assert the input exists.
    expect(files).toContain("de.json");
  });

  it.each(files)("%s declares a locale matching its filename", (file) => {
    const bundle = JSON.parse(readFileSync(join(DIR, file), "utf8"));
    expect(bundle["@@locale"]).toBe(file.replace(/\.json$/, ""));
  });

  it.each(files)("%s names itself in its own language", (file) => {
    const bundle = JSON.parse(readFileSync(join(DIR, file), "utf8"));
    expect(bundle["@@name"] ?? "").not.toBe("");
  });

  it("the template is excluded, and would fail these checks if it were not", () => {
    const template = JSON.parse(readFileSync(join(DIR, "_template.json"), "utf8"));
    expect(files).not.toContain("_template.json");
    // Pinned rather than assumed: the template is deliberately unfilled,
    // which is exactly what a careless copy of it inherits.
    expect(template["@@locale"]).toBe("xx");
    expect(template["@@name"]).toBe("");
  });
});
