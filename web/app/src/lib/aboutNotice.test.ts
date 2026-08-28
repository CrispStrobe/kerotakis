/** The credits screen says what NOTICE says.
 *
 * NOTICE is the legal source of truth for what this app embeds. The About
 * dialog is the only place a reader will ever see that list, and a
 * hand-kept copy of it would drift — silently, because nobody diffs a
 * credits screen against a licence file.
 *
 * So the dialog reads a generated file, and this fails when regenerating
 * would change it. Adding a component to NOTICE and forgetting the dialog
 * is a red test rather than a quiet omission.
 */
import { describe, expect, it } from "vitest";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { NOTICE_SECTIONS } from "./about";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../../../..");

describe("the About dialog's third-party list", () => {
  it("is current with NOTICE", () => {
    // The generator exits non-zero when the committed file is stale, and
    // prints the command that fixes it.
    expect(() =>
      execFileSync("node", [join(ROOT, "tools/about-notice.mjs")], { cwd: ROOT }),
    ).not.toThrow();
  });

  it("carries every component NOTICE lists", () => {
    // Counted from NOTICE directly rather than from the generated file:
    // a generator that silently dropped a section would otherwise agree
    // with itself.
    const notice = readFileSync(join(ROOT, "NOTICE"), "utf8");
    const bullets = notice.split("\n").filter((l) => l.startsWith("- ")).length;
    const shown = NOTICE_SECTIONS.reduce((n, s) => n + s.entries.length, 0);
    expect(shown).toBe(bullets);
  });

  it("names the things a reader would look for", () => {
    const all = NOTICE_SECTIONS.flatMap((s) => s.entries).join("\n");
    // IPhreeqc is the one that does the chemistry, and its licence is the
    // unusual one — a USGS notice rather than an SPDX identifier.
    expect(all).toContain("IPhreeqc");
    expect(all).toContain("USGS User Rights Notice");
  });

  it("does not invent licences the file never claims", () => {
    // An earlier generator split each bullet into {name, licence} and got
    // "Apache-2.0" wrong, called serde a licence, and turned two prose
    // paragraphs into components. Entries are verbatim now; this pins that.
    const notice = readFileSync(join(ROOT, "NOTICE"), "utf8").replace(/\s+/g, " ");
    for (const entry of NOTICE_SECTIONS.flatMap((s) => s.entries)) {
      expect(notice).toContain(entry.replace(/\s+/g, " "));
    }
  });
});
