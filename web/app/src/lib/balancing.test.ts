/**
 * What is left of GUI-095 in the client: choosing the questions, and saying
 * what a mark means.
 *
 * The marker itself used to be tested here, case by case, against
 * hand-written `BalanceReport`s. It is gone from the client — the
 * composition matrix it marked against was the answer one null space away,
 * and it crossed the wire to reach this file. Marking now lives in
 * `kerotakis_core::stoich::mark`, and those cases live with it
 * (`stoich.rs`, and `balance_exercise.rs` for the pool-wide property).
 *
 * What remains here is what the client still decides: which strings are
 * worth offering as questions, which one to ask next, and which complete
 * sentence a verdict earns. `markMessage` takes the engine's mark as it
 * arrives on the wire, so the marks below are written in that shape.
 */
import { describe, expect, it } from "vitest";
import { hasGermanTranslation } from "./i18n.svelte";
import type { BalanceMark } from "./host/EngineHost";
import { balancingCandidates, balancingSources, markMessage, nextSource, originReason, skeletonOf } from "./balancing";

/** A mark as `balanceMark` delivers it. */
const mark = (
  verdict: BalanceMark["verdict"],
  extra: Partial<Omit<BalanceMark, "ok" | "verdict">> = {},
): BalanceMark => ({
  ok: true,
  verdict,
  misses: [],
  factor: 0,
  family: false,
  ...extra,
});

describe("where the questions come from", () => {
  const entries = [
    { id: "salt-dissolves", equation: "NaCl(s) → Na⁺(aq) + Cl⁻(aq)" },
    { id: "buffer", equation: "CH₃COOH / CH₃COO⁻ buffer" },
    { id: "prose", equation: "a colour change" },
    { id: "no-equation", equation: null },
    { id: "silver", equation: "AgNO₃(aq) + NaCl(aq) → AgCl(s)↓ + NaNO₃(aq)" },
  ];

  it("offers only strings that could be equations, and lets the engine judge the rest", () => {
    expect(balancingSources(entries).map((source) => source.id))
      .toEqual(["salt-dissolves", "silver"]);
  });

  it("puts the bench's own reactions first", () => {
    const sources = balancingSources(entries, ["Zn + CuSO₄ → ZnSO₄ + Cu"]);
    expect(sources[0]).toEqual({
      id: "session:0", equation: "Zn + CuSO₄ → ZnSO₄ + Cu", origin: "bench", solved: false,
    });
    expect(sources).toHaveLength(3);
  });

  it("does not ask the same equation twice under two ids", () => {
    const sources = balancingSources(
      [{ id: "codex-copy", equation: "Zn + CuSO₄ → ZnSO₄ + Cu" }],
      ["Zn + CuSO₄ → ZnSO₄ + Cu"],
    );
    expect(sources).toHaveLength(1);
    expect(sources[0]?.origin).toBe("bench");
  });

  it("never runs out — the point of generating rather than authoring", () => {
    const sources = balancingSources(entries);
    expect(nextSource(sources, [])?.id).toBe("salt-dissolves");
    expect(nextSource(sources, ["salt-dissolves"])?.id).toBe("silver");
    expect(nextSource(sources, ["salt-dissolves", "silver"])?.id).toBe("salt-dissolves");
    expect(nextSource([], [])).toBeNull();
  });
});

describe("what a mark says", () => {
  it("picks one complete sentence per case rather than composing one", () => {
    expect(markMessage(mark("correct")).key)
      .toBe("balanced, in the smallest whole numbers");
    expect(markMessage(mark("multiple", { factor: 2 })))
      .toEqual({
        key: "balanced — but every coefficient divides by {factor}, so this is not the smallest whole-number ratio",
        vars: { factor: "2" },
      });
    expect(markMessage(mark("correct", { family: true })).key)
      .toBe("balanced — and this skeleton has more than one answer, so yours is one of a family");
  });

  it("says which side is heavy, with a key per side rather than a sign", () => {
    expect(markMessage(mark("unbalanced", { misses: [{ element: "O", amount: 1 }] })))
      .toEqual({ key: "{amount} too much {element} on the left as written", vars: { amount: "1", element: "O" } });
    expect(markMessage(mark("unbalanced", { misses: [{ element: "Mg", amount: -1 }] })))
      .toEqual({ key: "{amount} too much {element} on the right as written", vars: { amount: "1", element: "Mg" } });
  });

  it("names the worst miss, since the engine sends them worst first", () => {
    expect(markMessage(mark("unbalanced", {
      misses: [{ element: "O", amount: -4 }, { element: "H", amount: 1 }],
    })).vars.element).toBe("O");
  });

  /**
   * These keys never appear as a `t("…")` literal — the component
   * translates `markMessage().key`, which is a variable — so the scan in
   * `i18n.test.ts` cannot see them. Without this, a German bench would mark
   * answers in English and nothing would say so.
   */
  it("has German for every sentence a mark can produce", () => {
    const marks: BalanceMark[] = [
      mark("correct"),
      mark("correct", { family: true }),
      mark("multiple", { factor: 2 }),
      mark("unbalanced", { misses: [{ element: "O", amount: 1 }] }),
      mark("unbalanced", { misses: [{ element: "Mg", amount: -1 }] }),
      mark("unbalanced"),
      mark("incomplete"),
    ];
    const missing = marks
      .map((entry) => markMessage(entry).key)
      .filter((key) => !hasGermanTranslation(key));
    expect(missing).toEqual([]);
  });
});

describe("which question, and why that one", () => {
  /**
   * The drill used to look like a shuffle because nothing said why it was
   * asking. It never was random — it walked a list — but the list had one
   * order and no reason attached, so "next" and "arbitrary" were
   * indistinguishable from the outside. These tests pin the order and the
   * sentence beside it, which together are the whole answer.
   */
  const entries = [
    { id: "silver", equation: "AgNO₃ + NaCl → AgCl↓ + NaNO₃" },
    { id: "rusting", equation: "4 Fe + 3 O₂ → 2 Fe₂O₃" },
    { id: "chalk", equation: "CaCO₃ + 2 HCl → CaCl₂ + H₂O + CO₂" },
  ];
  const kit: Record<string, string[]> = {
    silver: ["water", "AgNO3", "NaCl"],
    rusting: ["water", "Fe"],
    chalk: ["CaCO3", "HCl"],
  };
  const entryKit = (id: string) => kit[id] ?? [];

  it("asks the bench, then the mission, then the catalogue", () => {
    const sources = balancingCandidates({
      entries,
      benchEquations: ["Zn + CuSO₄ → ZnSO₄ + Cu"],
      missionKit: ["water", "AgNO3", "NaCl", "phenolphthalein"],
      entryKit,
    });
    expect(sources.map((source) => [source.id, source.origin])).toEqual([
      ["session:0", "bench"],
      ["silver", "mission"],
      ["rusting", "codex"],
      ["chalk", "codex"],
    ]);
  });

  it("only calls an entry the mission's when the mission put out every reagent", () => {
    // `rusting` needs Fe, which this mission never dispenses — a partial
    // overlap is not the mission's chemistry, it is a coincidence of water.
    const sources = balancingCandidates({
      entries,
      missionKit: ["water", "AgNO3", "NaCl"],
      entryKit,
    });
    expect(sources.filter((source) => source.origin === "mission").map((s) => s.id)).toEqual(["silver"]);
  });

  it("has no mission tier at all when nothing is running", () => {
    const sources = balancingCandidates({ entries, entryKit });
    expect(sources.every((source) => source.origin === "codex")).toBe(true);
  });

  it("puts equations from experiments the learner has run ahead of ones they have not", () => {
    const sources = balancingCandidates({ entries, met: new Set(["chalk"]) });
    expect(sources.map((source) => source.id)).toEqual(["chalk", "silver", "rusting"]);
  });

  it("sinks a solved equation inside its tier rather than dropping it", () => {
    // Practice may be repeated; it just must not be offered ahead of
    // something the learner has never balanced.
    const sources = balancingCandidates({ entries, solved: new Set(["silver"]) });
    expect(sources.map((source) => [source.id, source.solved])).toEqual([
      ["rusting", false],
      ["chalk", false],
      ["silver", true],
    ]);
  });

  it("names a reason for every origin, and has German for all three", () => {
    expect((["bench", "mission", "codex"] as const).map(originReason)).toEqual([
      "from your bench",
      "from your mission",
      "next in the catalogue",
    ]);
    const missing = (["bench", "mission", "codex"] as const)
      .map(originReason)
      .filter((key) => !hasGermanTranslation(key));
    expect(missing).toEqual([]);
  });

  it("labels a candidate without handing over its answer", () => {
    // The picker names bench equations, and a balanced equation beside the
    // skeleton it is about to ask would end the exercise before it starts.
    expect(skeletonOf("4 Mg + 2 O₂ → 4 MgO")).toBe("Mg + O₂ → MgO");
    expect(skeletonOf("2 H₂ + O₂ ⇌ 2 H₂O")).toBe("H₂ + O₂ → H₂O");
    // A coefficient of one is written by being absent, and stays absent.
    expect(skeletonOf("AgNO₃ + NaCl → AgCl↓ + NaNO₃")).toBe("AgNO₃ + NaCl → AgCl↓ + NaNO₃");
    // Nothing that is not a leading coefficient is touched: the 2 in H₂O
    // is part of the formula, not an answer.
    expect(skeletonOf("2 H₂O₂ → 2 H₂O + O₂")).toBe("H₂O₂ → H₂O + O₂");
  });
});
