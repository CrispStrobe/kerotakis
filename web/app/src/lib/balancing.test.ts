/**
 * The marker is the part of GUI-095 that can be wrong without looking wrong,
 * so it is pinned case by case — and above all on the verdict the roadmap
 * calls the actual lesson: an answer that balances at a multiple of the
 * simplest ratio is *not* an error, and a marker that reports it as one has
 * taught the wrong thing while appearing to work.
 *
 * The reports below are the shape `balance` returns, written out by hand so
 * the marker is tested against the wire contract rather than against the
 * solver — and the composition matrices are the ones the engine's own tests
 * pin (`stoich.rs::the_reported_matrix_annihilates_the_reported_answer`).
 */
import { describe, expect, it } from "vitest";
import { hasGermanTranslation } from "./i18n.svelte";
import type { BalanceReport } from "./host/EngineHost";
import {
  balancingSources,
  blankEquation,
  markBalance,
  markMessage,
  nextSource,
  writeEquation,
} from "./balancing";

/** Mg + O₂ → MgO — the unique case, and the smallest one worth teaching. */
const magnesium: BalanceReport = {
  ok: true,
  species: ["Mg", "O2", "MgO"],
  reactants: 2,
  elements: ["Mg", "O", "charge"],
  matrix: [
    [1, 0, -1],
    [0, 2, -1],
    [0, 0, -0],
  ],
  coefficients: [2, 1, 2],
  basis: [],
  reversible: false,
};

/** Ag⁺ + Cl⁻ → AgCl — atoms alone would pass a charge error. */
const silverChloride: BalanceReport = {
  ok: true,
  species: ["Ag⁺", "Cl⁻", "AgCl"],
  reactants: 2,
  elements: ["Ag", "Cl", "charge"],
  matrix: [
    [1, 0, -1],
    [0, 1, -1],
    [1, -1, 0],
  ],
  coefficients: [1, 1, 1],
  basis: [],
  reversible: false,
};

/** C + O₂ → CO + CO₂ — two independent reactions under one arrow. */
const carbonOxidation: BalanceReport = {
  ok: true,
  species: ["C", "O2", "CO", "CO2"],
  reactants: 2,
  elements: ["C", "O", "charge"],
  matrix: [
    [1, 0, -1, -1],
    [0, 2, -1, -2],
    [0, 0, -0, -0],
  ],
  // 3 C + 2 O₂ → 2 CO + CO₂ — one member of the family, all-positive.
  coefficients: [3, 2, 2, 1],
  basis: [
    [2, 1, 2, 0],
    [1, 1, 0, 1],
  ],
  reversible: false,
};

describe("marking a balancing answer", () => {
  it("accepts the smallest whole-number ratio", () => {
    expect(markBalance(magnesium, [2, 1, 2])).toEqual({
      verdict: "correct", misses: [], family: false,
    });
  });

  it("calls a correct multiple correct, and says what to divide by", () => {
    // The lesson GUI-095 exists for. 4 Mg + 2 O₂ → 4 MgO conserves every
    // atom; it is simply twice the answer, and a marker that says "wrong"
    // has taught a learner that a balanced equation is unbalanced.
    expect(markBalance(magnesium, [4, 2, 4])).toEqual({
      verdict: "multiple", factor: 2, misses: [], family: false,
    });
    expect(markBalance(magnesium, [6, 3, 6]).factor).toBe(3);
    expect(markBalance(magnesium, [20, 10, 20]).factor).toBe(10);
  });

  it("rejects an unbalanced answer and names what is out", () => {
    const mark = markBalance(magnesium, [1, 1, 1]);
    expect(mark.verdict).toBe("unbalanced");
    // One O₂ on the left against one O on the right: one oxygen surplus.
    expect(mark.misses).toEqual([{ element: "O", amount: 1 }]);
  });

  it("catches a charge error that conserves every atom", () => {
    // Ag⁺ + Cl⁻ → AgCl is right because the charges cancel. Drop one of
    // them and the atoms still balance — which is the student error the
    // charge row exists to catch.
    const wrongCharge: BalanceReport = {
      ...silverChloride,
      species: ["Ag⁺", "Cl", "AgCl"],
      matrix: [[1, 0, -1], [0, 1, -1], [1, 0, 0]],
    };
    const mark = markBalance(wrongCharge, [1, 1, 1]);
    expect(mark.verdict).toBe("unbalanced");
    expect(mark.misses).toEqual([{ element: "charge", amount: 1 }]);
  });

  it("reports the worst miss first", () => {
    const mark = markBalance(magnesium, [1, 5, 3]);
    expect(mark.verdict).toBe("unbalanced");
    expect(mark.misses.map((miss) => miss.element)).toEqual(["O", "Mg"]);
  });

  it("refuses to mark an answer that is not yet one", () => {
    for (const answer of [[2, 1], [2, 0, 2], [2, -1, 2], [2, 0.5, 2], [Number.NaN, 1, 2]]) {
      expect(markBalance(magnesium, answer).verdict).toBe("incomplete");
    }
  });

  it("marks any primitive member of an under-determined family correct", () => {
    // The advanced round: there is no single right answer, so the mark is
    // "does this balance", not "does this match".
    expect(markBalance(carbonOxidation, [3, 2, 2, 1])).toEqual({
      verdict: "correct", misses: [], family: true,
    });
    // A different member of the same family: one part 2 C + O₂ → 2 CO to
    // two parts C + O₂ → CO₂.
    expect(markBalance(carbonOxidation, [4, 3, 2, 2]).verdict).toBe("correct");
    expect(markBalance(carbonOxidation, [2, 1, 2, 0]).verdict).toBe("incomplete");
  });

  it("still calls a multiple a multiple inside a family", () => {
    const mark = markBalance(carbonOxidation, [6, 4, 4, 2]);
    expect(mark).toEqual({ verdict: "multiple", factor: 2, misses: [], family: true });
  });

  it("rejects an answer outside the family's null space", () => {
    expect(markBalance(carbonOxidation, [3, 2, 1, 1]).verdict).toBe("unbalanced");
  });
});

describe("the question a round asks", () => {
  it("strips every coefficient, so the blank never leaks the answer", () => {
    expect(blankEquation(magnesium)).toBe("Mg + O2 → MgO");
    expect(blankEquation(carbonOxidation)).toBe("C + O2 → CO + CO2");
  });

  it("writes an answer back the way it is written on paper", () => {
    expect(writeEquation(magnesium, [2, 1, 2])).toBe("2 Mg + O2 → 2 MgO");
    expect(writeEquation(silverChloride, [1, 1, 1])).toBe("Ag⁺ + Cl⁻ → AgCl");
  });

  it("keeps a reversible arrow reversible", () => {
    expect(writeEquation({ ...magnesium, reversible: true }, [2, 1, 2]))
      .toBe("2 Mg + O2 ⇌ 2 MgO");
  });
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
      id: "session:0", equation: "Zn + CuSO₄ → ZnSO₄ + Cu", origin: "bench",
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
    expect(markMessage(markBalance(magnesium, [2, 1, 2])).key)
      .toBe("balanced, in the smallest whole numbers");
    expect(markMessage(markBalance(magnesium, [4, 2, 4])))
      .toEqual({
        key: "balanced — but every coefficient divides by {factor}, so this is not the smallest whole-number ratio",
        vars: { factor: "2" },
      });
    expect(markMessage(markBalance(carbonOxidation, [3, 2, 2, 1])).key)
      .toBe("balanced — and this skeleton has more than one answer, so yours is one of a family");
  });

  it("says which side is heavy, with a key per side rather than a sign", () => {
    expect(markMessage(markBalance(magnesium, [1, 1, 1])))
      .toEqual({ key: "{amount} too much {element} on the left as written", vars: { amount: "1", element: "O" } });
    expect(markMessage(markBalance(magnesium, [1, 1, 2])))
      .toEqual({ key: "{amount} too much {element} on the right as written", vars: { amount: "1", element: "Mg" } });
  });

  /**
   * These keys never appear as a `t("…")` literal — the component
   * translates `markMessage().key`, which is a variable — so the scan in
   * `i18n.test.ts` cannot see them. Without this, a German bench would mark
   * answers in English and nothing would say so.
   */
  it("has German for every sentence a mark can produce", () => {
    const marks = [
      markBalance(magnesium, [2, 1, 2]),
      markBalance(magnesium, [4, 2, 4]),
      markBalance(magnesium, [1, 1, 1]),
      markBalance(magnesium, [1, 1, 2]),
      markBalance(magnesium, [0, 0, 0]),
      markBalance(carbonOxidation, [3, 2, 2, 1]),
      { verdict: "unbalanced" as const, misses: [], family: false },
    ];
    const missing = marks
      .map((mark) => markMessage(mark).key)
      .filter((key) => !hasGermanTranslation(key));
    expect(missing).toEqual([]);
  });
});
