import { afterEach, describe, expect, it } from "vitest";
import { render } from "svelte/server";
import PeriodicTable from "./PeriodicTable.svelte";
import { i18n } from "../i18n.svelte";
import codexExportJson from "../../../../../crates/kerotakis-codex/tests/golden/codex-export.json?raw";
import type { ElementExperimentIndexEntry } from "../elements";
import type { ShelfItem } from "../session.svelte";

/**
 * The periodic table, RENDERED, in German.
 *
 * The table lists what each element can be used for, and every one of those
 * lines came out English however the shell was set. The cause was a category
 * error rather than a missing translation: the row showed the codex entry's
 * `summary`, which is engine prose travelling with the content in a
 * `summary_de` sibling, and pushed it through `t()`, which only ever looks in
 * the shell's dictionary. A key that is not there comes back as itself, so
 * the German build printed the English sentence and nothing anywhere said
 * anything was wrong.
 *
 * These tests read the real exported codex rather than a fixture, because the
 * bug was about the shape of the shipped data and a hand-written entry would
 * have had whatever shape the test wanted.
 */

const codex = JSON.parse(codexExportJson) as {
  reactions: (ElementExperimentIndexEntry & { summary_de?: string | null })[];
};

/** Na and Cl: on the lab shelf, in the lab table, and used by the codex. */
const shelf: ShelfItem[] = [
  { key: "water", name: "water", formula: "H2O", phase: "liquid" },
  { key: "NaCl", name: "table salt", formula: "NaCl", phase: "solid" },
];

const SAMPLE = "crystallising-salt-from-brine";

function body(): string {
  return render(PeriodicTable, {
    props: {
      shelf,
      register: "lv2",
      element: "Na",
      experiments: codex.reactions,
      onadd: () => {},
      onexperiment: () => {},
      onclose: () => {},
    },
  }).body;
}

/** The rendered text, with the markup and Svelte's anchors taken out. */
function text(html: string): string {
  return html.replace(/<!--[\s\S]*?-->/g, "").replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim();
}

/** Codex prose sets its own spacing; the reader's eye and `text()` above
 * both collapse it, so compare on the same footing. */
function norm(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

afterEach(() => i18n.setLocale("en"));

describe("the periodic table's German", () => {
  it("has a sample experiment whose German really is different from its English", () => {
    const entry = codex.reactions.find((candidate) => candidate.id === SAMPLE);
    expect(entry, `${SAMPLE} is gone from the codex; pick another Na/Cl entry`).toBeDefined();
    expect(entry!.summary_de).toBeTruthy();
    expect(entry!.summary_de).not.toBe(entry!.summary);
  });

  it("names every listed experiment in German, not in English", () => {
    i18n.setLocale("de");
    const rendered = text(body());
    // The title the catalogue shows for the same entry, and the one this
    // table was printing instead.
    expect(rendered).toContain("Salz aus Sole kristallisieren");
    expect(rendered).not.toContain("crystallising salt from brine");
  });

  it("shows the experiment's own prose from its German sibling, not its English field", () => {
    const entry = codex.reactions.find((candidate) => candidate.id === SAMPLE)!;
    i18n.setLocale("de");
    const rendered = text(body());
    expect(rendered).toContain(norm(entry.summary_de!));
    expect(rendered).not.toContain(norm(entry.summary!));
  });

  it("still reads English when the shell is English", () => {
    const entry = codex.reactions.find((candidate) => candidate.id === SAMPLE)!;
    const rendered = text(body());
    expect(rendered).toContain("crystallising salt from brine");
    expect(rendered).toContain(norm(entry.summary!));
  });

  it("falls back to English rather than to nothing when a German sibling is missing", () => {
    const untranslated: ElementExperimentIndexEntry[] = [{
      id: "crystallising-salt-from-brine",
      summary: "an English line nobody has translated",
      setup: { script: "add v1 water 1000mL\nadd v1 NaCl 117g" },
    }];
    i18n.setLocale("de");
    const rendered = text(render(PeriodicTable, {
      props: {
        shelf,
        register: "lv2",
        element: "Na",
        experiments: untranslated,
        onadd: () => {},
        onexperiment: () => {},
        onclose: () => {},
      },
    }).body);
    // The title still localises — it is a dictionary phrase, not engine
    // prose — and the untranslated line degrades to English on its own
    // rather than leaving a blank row.
    expect(rendered).toContain("Salz aus Sole kristallisieren");
    expect(rendered).toContain("an English line nobody has translated");
  });
});

describe("the periodic table's reagents", () => {
  it("offers each shelf reagent as a button carrying the key the bench needs", () => {
    // The command the press turns into is `add v1 <key> …`, so the key is
    // the part that has to survive the round trip; the name beside it is
    // display only and is translated.
    const rendered = body();
    expect(rendered).toContain('data-key="NaCl"');
    // Unterminated on purpose, as every other rendered-markup assertion in
    // this directory is: Svelte appends its scoping class to any element the
    // component styles, so the attribute is `class="add svelte-…"`.
    expect(rendered).toContain('class="add');
  });

  it("names the reagent in German while keeping its key and formula", () => {
    i18n.setLocale("de");
    const rendered = body();
    expect(rendered).toContain('data-key="NaCl"');
    expect(text(rendered)).toContain("NaCl");
  });
});
