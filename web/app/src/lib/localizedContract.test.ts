/**
 * WORLD-007 — the localized content contract.
 *
 * `i18n.test.ts` already scans source for literal `t("…")` call sites, which
 * catches interface copy. It cannot catch CONTENT: a mission's objective, a
 * criterion's label, a case lead, an award's name. Those reach `t()` as
 * variables — `t(criterion.label)` — so the literal scan walks straight past
 * them, and a new mission can ship English-only while every gate stays green.
 *
 * That is the gap this file closes. It walks the content tables themselves,
 * and it checks the shape of a translation as well as its existence: a German
 * string that drops a placeholder renders a sentence with a hole in it, which
 * no coverage count would notice.
 */
import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { allCriteria, outcomeMissionContract } from "./outcomeMission";
import { CASE_AWARDS, contaminatedSampleLeads } from "./storyChapter";

const de = JSON.parse(
  readFileSync(join(import.meta.dirname, "../locales/de.json"), "utf8"),
) as { terms: Record<string, string>; messages: Record<string, string> };

const german = (key: string): string | undefined => de.messages[key] ?? de.terms[key];

/** `{name}` placeholders, in the order they appear. */
const placeholders = (s: string): string[] => [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]!);

const MISSIONS = ["silver-and-salt", "first-warmth", "one-thing-at-a-time", "never-mix"];

describe("mission content is localized, not just interface copy", () => {
  it("every mission's objective, brief and hint has German", () => {
    const missing: string[] = [];
    for (const id of MISSIONS) {
      const contract = outcomeMissionContract(id)!;
      for (const [field, text] of [
        ["objective", contract.objective],
        ["brief", contract.brief],
        ["hint", contract.hint],
      ] as const) {
        if (german(text) === undefined) missing.push(`${id}.${field}`);
      }
    }
    expect(missing).toEqual([]);
  });

  it("every criterion label has German — these reach t() as variables", () => {
    const missing: string[] = [];
    for (const id of MISSIONS) {
      for (const criterion of allCriteria(outcomeMissionContract(id)!)) {
        if (german(criterion.label) === undefined) missing.push(`${id}:${criterion.id}`);
      }
    }
    expect(missing).toEqual([]);
  });

  it("every route label has German, including the ones only one mission shows", () => {
    const missing: string[] = [];
    for (const id of MISSIONS) {
      for (const route of outcomeMissionContract(id)!.routes) {
        if (german(route.label) === undefined) missing.push(`${id}:${route.id}`);
      }
    }
    expect(missing).toEqual([]);
  });

  it("every case lead and every case award has German", () => {
    const missions = MISSIONS.map((id) => ({ file: `${id}.lab`, name: id }));
    const missing: string[] = [];
    for (const lead of contaminatedSampleLeads(missions, new Set())) {
      if (german(lead.objective) === undefined) missing.push(`lead:${lead.id}.objective`);
      if (german(lead.evidence) === undefined) missing.push(`lead:${lead.id}.evidence`);
    }
    for (const award of Object.values(CASE_AWARDS)) {
      if (german(award.title) === undefined) missing.push(`award:${award.verb}.title`);
      if (german(award.description) === undefined) missing.push(`award:${award.verb}.description`);
    }
    expect(missing).toEqual([]);
  });
});

describe("interpolation parity", () => {
  it("no translation drops, adds, or renames a placeholder", () => {
    // A German string missing `{count}` renders a sentence with a hole in
    // it. Coverage counting cannot see that; only comparing shapes can.
    const broken: string[] = [];
    let checked = 0;
    for (const [source, translated] of Object.entries(de.messages)) {
      if (translated === "") continue;
      const wanted = [...new Set(placeholders(source))].sort();
      const got = [...new Set(placeholders(translated))].sort();
      if (wanted.length > 0) checked += 1;
      if (wanted.join(",") !== got.join(",")) {
        broken.push(`"${source}" wants [${wanted}], German has [${got}]`);
      }
    }
    expect(broken).toEqual([]);
    // A gate that silently stops finding anything to check is not a gate.
    expect(checked).toBeGreaterThan(100);
  });

  it("catches a dropped placeholder, so the check is not vacuous", () => {
    // The gate above passes today; prove it would fail if it stopped being
    // true, rather than trusting an empty list.
    const source = "after {count} missions";
    const sabotaged = "nach Missionen";
    expect(placeholders(source)).toEqual(["count"]);
    expect(placeholders(sabotaged)).toEqual([]);
  });
});

describe("the template stays a complete brief for a new translator", () => {
  it("offers every key German has, so nothing is invisible to translate", () => {
    const template = JSON.parse(
      readFileSync(join(import.meta.dirname, "../locales/_template.json"), "utf8"),
    ) as { terms: Record<string, string>; messages: Record<string, string> };
    for (const section of ["terms", "messages"] as const) {
      const missing = Object.keys(de[section]).filter((k) => !(k in template[section]));
      expect({ section, missing }).toEqual({ section, missing: [] });
    }
  });
});

describe("quest prose: an acknowledged debt, ratcheted so it cannot grow", () => {
  /**
   * Every `lv1`/`lv2`/`lv3` string in `quests/*.toml` reaches a learner
   * through `t()`, so German for it means an entry in `de.json`. Today there
   * are none: quest content is entirely untranslated, which is a real gap
   * and a content job — 861 strings needing a chemistry-literate translator,
   * not an afternoon of engineering.
   *
   * Baselining it is the honest response. The number below is the debt as
   * measured, and it may only ever go DOWN: a new quest that ships English
   * prose fails this test, so the gap stops growing today while the existing
   * strings are translated at content pace. When it reaches zero, delete the
   * allowance and assert the gate directly.
   */
  const UNTRANSLATED_BASELINE = 861;

  const questProse = (): string[] => {
    const dir = join(import.meta.dirname, "../../../../quests");
    const strings = new Set<string>();
    for (const file of readdirSync(dir).filter((f) => f.endsWith(".toml"))) {
      for (const line of readFileSync(join(dir, file), "utf8").split("\n")) {
        const match = /^\s*lv[123]\s*=\s*"(.*)"\s*$/.exec(line);
        if (match) strings.add(match[1]!);
      }
    }
    return [...strings];
  };

  it("finds the quest corpus, so the measurement is not vacuous", () => {
    expect(questProse().length).toBeGreaterThan(500);
  });

  it("does not add untranslated quest prose", () => {
    const untranslated = questProse().filter((s) => german(s) === undefined);
    expect(
      untranslated.length,
      `quest prose without German rose to ${untranslated.length}. New quest ` +
        `content must ship German, or lower the baseline if you translated some.`,
    ).toBeLessThanOrEqual(UNTRANSLATED_BASELINE);
  });

  it("keeps the baseline honest — it must be lowered when the debt is paid", () => {
    // A baseline far above the real number would silently stop gating. Pin
    // it within sight of the truth.
    const untranslated = questProse().filter((s) => german(s) === undefined);
    expect(UNTRANSLATED_BASELINE - untranslated.length).toBeLessThanOrEqual(20);
  });
});
