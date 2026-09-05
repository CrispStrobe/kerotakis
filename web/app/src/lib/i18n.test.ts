import { readdirSync, readFileSync } from "node:fs";
import { extname, join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { hasGermanTranslation, i18n, t } from "./i18n.svelte";

function sourceFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return sourceFiles(path);
    if (entry.name.endsWith(".test.ts")) return [];
    return [".svelte", ".ts"].includes(extname(entry.name)) ? [path] : [];
  });
}

describe("i18n", () => {
  afterEach(() => {
    i18n.locale = "en";
  });

  it("translates interface and engine-domain vocabulary into German", () => {
    i18n.locale = "de";
    expect(t("save notes")).toBe("Notizen speichern");
    expect(t("sodium chloride")).toBe("Natriumchlorid");
    expect(t("supply cabinet")).toBe("Materialschrank");
    expect(t("high contrast")).toBe("Hoher Kontrast");
    expect(t("place vessel here")).toBe("Gefäß hier abstellen");
    expect(t("Your saves stay separate.")).toBe("Deine Spielstände bleiben getrennt.");
    expect(t("after one mission")).toBe("nach einer Mission");
    expect(t("mission kit")).toBe("Missionsset");
    expect(t("place on bench")).toBe("auf den Labortisch stellen");
    expect(t("stockroom replenished")).toBe("Materiallager aufgefüllt");
    expect(t("one use left")).toBe("noch 1");
    expect(t("The contaminated sample")).toBe("Die verunreinigte Probe");
    expect(t("open the case file")).toBe("Fallakte öffnen");
    expect(t("investigate")).toBe("untersuchen");
    expect(t("inspect")).toBe("prüfen");
    expect(t("assessed by the solver")).toBe("durch die Simulation bewertet");
    expect(t("solver-assessed outcome")).toBe("durch Simulation bewertetes Ziel");
    expect(t("Observable silver chloride formed")).toBe("Sichtbares Silberchlorid gebildet");
  });

  it("interpolates translated messages", () => {
    i18n.locale = "de";
    expect(t("timeline: step {position} of {total}", { position: 2, total: 5 })).toBe(
      "Zeitleiste: Schritt 2 von 5",
    );
    expect(t("vessel v{vessel} moved to {zone}", { vessel: 2, zone: t("analyse") })).toBe(
      "Gefäß v2 nach Analysieren verschoben",
    );
    expect(t("after {count} missions", { count: 3 })).toBe("nach 3 Missionen");
    // Shortened from "7 Entnahmen übrig": 18 characters did not fit the
    // 4.2rem stock column and rendered as "7 Entnahmen ü…" on a tablet.
    expect(t("{count} uses left", { count: 7 })).toBe("noch 7");
    expect(t("{done} of {total} evidence checks", { done: 1, total: 2 })).toBe("1 von 2 Nachweisprüfungen");
  });

  it("uses the source message as the English catalog", () => {
    i18n.locale = "en";
    expect(t("save notes")).toBe("save notes");
  });

  it("has German entries for every literal UI translation call", () => {
    const missing = new Set<string>();
    for (const path of sourceFiles(join(import.meta.dirname))) {
      // Comments are not call sites. A `t("…")` written in a doc comment
      // to explain what this very test scans for made it fail, reporting
      // an ellipsis as an untranslated string — so strip comments first,
      // or the next person documenting i18n trips the same wire.
      const source = readFileSync(path, "utf8")
        .replace(/\/\*[\s\S]*?\*\//g, "")
        .replace(/(^|[^:])\/\/.*$/gm, "$1");
      for (const match of source.matchAll(/\bt\("([^"]+)"/g)) {
        if (!hasGermanTranslation(match[1]!)) missing.add(match[1]!);
      }
    }
    expect([...missing].sort()).toEqual([]);
  });

  it("does not bypass translation for static accessible copy", () => {
    const untranslated: string[] = [];
    for (const path of sourceFiles(join(import.meta.dirname))) {
      if (!path.endsWith(".svelte")) continue;
      const source = readFileSync(path, "utf8");
      for (const match of source.matchAll(/\b(aria-label|placeholder|title)="([^"]*[A-Za-zÄÖÜäöüß][^"]*)"/g)) {
        untranslated.push(`${path}:${match[1]}=${match[2]}`);
      }
    }
    expect(untranslated).toEqual([]);
  });

  /**
   * The label tables that reach `t()` as a VARIABLE.
   *
   * The literal scan above walks straight past `t(field.label)`,
   * `t(meta.label)` and `t(vessel.boundary)`, so every one of these tables
   * could — and did — ship a string the dictionary had never heard of. The
   * German build then rendered the English source, or worse the engine's
   * raw wire tag: "pressure_controlled" reached a badge, and "flame power",
   * "air collar" and "exposure" reached the apparatus panel.
   *
   * Each case reads the table out of its own source rather than importing
   * it, because most of them live inside a component and are not exported;
   * each also asserts the table was actually found, so a rename turns this
   * into a failure rather than into a vacuous pass.
   */
  describe("label tables that reach t() as a variable", () => {
    const read = (relative: string): string =>
      readFileSync(join(import.meta.dirname, relative), "utf8");

    /** Authored prose fields. `equation` is deliberately excluded: a formula
     *  is the same in every language and must not be sent to a translator. */
    const authored = (source: string): string[] =>
      [...source.matchAll(/\b(?:label|title|blurb):\s*"((?:[^"\\]|\\.)*)"/g)].map((m) => m[1]!);

    const untranslated = (strings: string[]): string[] =>
      [...new Set(strings.filter((s) => s && !hasGermanTranslation(s)))].sort();

    it("every apparatus spec string has German — these reach t() as t(f.label)", () => {
      const strings = authored(read("apparatus.ts"));
      expect(strings.length).toBeGreaterThan(40);
      expect(untranslated(strings)).toEqual([]);
    });

    it("every reading label has German — the inset renders t(meta.label)", () => {
      const strings = authored(read("components/ReadingInset.svelte"));
      expect(strings.length).toBe(4);
      expect(untranslated(strings)).toEqual([]);
    });

    it("every curated reaction blurb has German — the picker renders t(detail.blurb)", () => {
      const strings = authored(read("components/ReactPicker.svelte"));
      expect(strings.length).toBe(5);
      expect(untranslated(strings)).toEqual([]);
    });

    it("every deployed-apparatus tool name has German", () => {
      const table = /const toolNames[^=]*=\s*\{([^}]*)\}/.exec(read("components/DeployedApparatus.svelte"));
      expect(table).not.toBeNull();
      const names = [...table![1]!.matchAll(/"([^"]+)"/g)].map((m) => m[1]!);
      expect(names.length).toBeGreaterThan(10);
      expect(untranslated(names)).toEqual([]);
    });

    it("the vessel badge shows words, never the engine's boundary tag", () => {
      const source = read("components/Vessel.svelte");
      const table = /const BOUNDARY_LABELS[^=]*=\s*\{([^}]*)\}/.exec(source);
      expect(table).not.toBeNull();
      const labels = [...table![1]!.matchAll(/:\s*"([^"]+)"/g)].map((m) => m[1]!);
      expect(labels.length).toBe(3);
      expect(untranslated(labels)).toEqual([]);
      // The wire tag itself must never be handed to the dictionary: t() falls
      // back to its key, so `t(vessel.boundary)` renders "pressure_controlled".
      expect(source).not.toContain("t(vessel.boundary)");
    });

    it("every capability support level and age band has German", () => {
      const source = read("components/CapabilityExplorer.svelte");
      const bands = /const AGE_LABELS[^=]*=\s*\{([^}]*)\}/.exec(source);
      expect(bands).not.toBeNull();
      const labels = [...bands![1]!.matchAll(/:\s*"([^"]+)"/g)].map((m) => m[1]!);
      expect(labels.length).toBe(4);
      expect(untranslated(labels)).toEqual([]);
      expect(untranslated(["computed", "curated", "qualitative", "boundary", "missing"])).toEqual([]);
    });
  });

  it("has German display names for every registry material", () => {
    const registry = JSON.parse(readFileSync(
      join(import.meta.dirname, "../../../../data/registry/registry-source-v1.json"),
      "utf8",
    ));
    const names = new Set<string>();
    const collect = (value: unknown): void => {
      if (Array.isArray(value)) {
        value.forEach(collect);
      } else if (value && typeof value === "object") {
        const record = value as Record<string, unknown>;
        if (typeof record.name === "string") names.add(record.name);
        Object.values(record).forEach(collect);
      }
    };
    collect(registry);
    expect([...names].filter((name) => !hasGermanTranslation(name)).sort()).toEqual([]);
  });
});
