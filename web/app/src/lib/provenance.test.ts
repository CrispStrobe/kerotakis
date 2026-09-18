import { describe, expect, it } from "vitest";
import { buildProvenance, carryRouting, type CarriedRouting } from "./provenance";

/**
 * GUI-052. Every fixture below is the wire shape the engine actually emits
 * (`SolverRoute` in `kerotakis-core/src/solve.rs`, `Event` in `ops.rs`) -
 * not a shape invented for the test. A fixture that drifts from the engine
 * is a test that passes while the drawer is blank.
 */

/**
 * An aqueous step: PHREEQC answered, the honesty pass had nothing to add.
 *
 * The provenance rides on `solution_routed`, which is the event that
 * carries `vessel::Provenance` for the aqueous solver. This fixture used
 * to hang it on `precipitated`, which was the one shape here the engine
 * never emitted — `Event::Precipitated` has no provenance field and never
 * had one. The drawer's aqueous source existed only in this file until
 * `SolutionRouted` shipped, and a fixture that drifts from the engine is
 * exactly the test that passes while the drawer is blank.
 */
const aqueousStep = {
  events: [
    {
      event: "solution_routed",
      vessel: 0,
      provenance: {
        engine: "PHREEQC (IPhreeqc)",
        dataset: "minteq.v4.dat",
        model: "Debye-Huckel (WATEQ) ion association",
        dataset_sources: ["Allison et al. 1991", "Nordstrom et al. 1990"],
        routing: "an aqueous solution is characterised, so the speciation engine leads",
      },
    },
  ],
  routes: [
    {
      solver: "mixing",
      kind: "computed",
      chemistry: false,
      outcome: { succeeded: { event_count: 1 } },
      vessel: 0,
    },
    {
      solver: "phreeqc-aqueous",
      kind: "computed",
      chemistry: true,
      outcome: { succeeded: { event_count: 1 } },
      vessel: 0,
    },
    {
      solver: "honesty",
      kind: "qualitative",
      chemistry: false,
      outcome: "not_applicable",
      vessel: 0,
      reason: "honesty does not apply to this vessel state",
    },
  ],
};

/** A CEA burn: the thermal engine answered from its own dataset. */
const burnStep = {
  events: [
    {
      event: "thermal_equilibrium",
      vessel: 1,
      temperature: 2769.15,
      provenance: {
        engine: "CEA (Gordon-McBride)",
        dataset: "thermo.inp",
        model: "Gibbs free-energy minimisation at constant pressure",
        dataset_sources: ["McBride, Zehe & Gordon, NASA/TP-2002-211556"],
        routing: "a fuel and an oxidiser are both present, so the combustion engine leads",
      },
    },
  ],
  routes: [
    {
      solver: "combustion",
      kind: "computed",
      chemistry: true,
      outcome: { succeeded: { event_count: 3 } },
      vessel: 1,
    },
  ],
};

/** A step the bench declined: the honesty pass is the only speaker. */
const declinedStep = {
  events: [
    {
      event: "not_yet_modeled",
      vessel: 0,
      what: "no wired solver models a molten salt at this temperature",
      cause: "no-solver",
    },
  ],
  routes: [
    {
      solver: "reaction-families",
      kind: "curated",
      chemistry: true,
      outcome: "not_applicable",
      vessel: 0,
      reason: "esterification v2 declined at temperature: cold and uncatalysed",
    },
    {
      solver: "honesty",
      kind: "qualitative",
      chemistry: false,
      outcome: { succeeded: { event_count: 1 } },
      vessel: 0,
    },
  ],
};

/** A reading with a bound on it: the note rides the answer, not a refusal. */
const measuredStep = {
  events: [
    {
      event: "measured",
      vessel: 0,
      instrument: "conductivity",
      value: 12.4,
      unit: "mS/cm",
      note: "a sum of limiting molar conductivities: the infinite-dilution limit, so above about 0.1 mol/kgw this number rests on a fitted correction",
    },
  ],
  routes: [
    {
      solver: "phreeqc-aqueous",
      kind: "computed",
      chemistry: true,
      outcome: { succeeded: { event_count: 1 } },
      vessel: 0,
    },
  ],
};

describe("buildProvenance", () => {
  it("names the chemistry engine that answered an aqueous step, and its dataset", () => {
    const report = buildProvenance(aqueousStep);
    expect(report.headline).toEqual({
      solver: "phreeqc-aqueous",
      dataset: "minteq.v4.dat",
      declined: 1,
    });
    // The physical mixing pass answered too; the headline must not be it.
    expect(report.routes.map((route) => route.solver)).toEqual([
      "mixing",
      "phreeqc-aqueous",
      "honesty",
    ]);
    expect(report.empty).toBe(false);
  });

  it("keeps the routing in the order the stack asked, with one outcome each", () => {
    const report = buildProvenance(aqueousStep);
    expect(report.routes.map((route) => route.outcome)).toEqual([
      "answered",
      "answered",
      "declined",
    ]);
    expect(report.routes[2]?.reason).toBe("honesty does not apply to this vessel state");
    expect(report.routes[1]?.eventCount).toBe(1);
  });

  it("carries the dataset, model, routing and literature sources verbatim", () => {
    const report = buildProvenance(aqueousStep);
    expect(report.sources).toEqual([
      {
        engine: "PHREEQC (IPhreeqc)",
        dataset: "minteq.v4.dat",
        model: "Debye-Huckel (WATEQ) ion association",
        routing: "an aqueous solution is characterised, so the speciation engine leads",
        datasetSources: ["Allison et al. 1991", "Nordstrom et al. 1990"],
      },
    ]);
  });

  it("reads a CEA burn the same way, from a different engine and dataset", () => {
    const report = buildProvenance(burnStep);
    expect(report.headline).toEqual({
      solver: "combustion",
      dataset: "thermo.inp",
      declined: 0,
    });
    expect(report.sources[0]?.engine).toBe("CEA (Gordon-McBride)");
    expect(report.sources[0]?.model).toBe(
      "Gibbs free-energy minimisation at constant pressure",
    );
  });

  it("shows a decline with the solver's own sentence, and the honesty gap beside it", () => {
    const report = buildProvenance(declinedStep);
    expect(report.routes[0]).toEqual({
      solver: "reaction-families",
      kind: "curated",
      chemistry: true,
      outcome: "declined",
      eventCount: 0,
      reason: "esterification v2 declined at temperature: cold and uncatalysed",
      vessel: 0,
    });
    expect(report.gaps).toEqual([
      {
        what: "no wired solver models a molten salt at this temperature",
        cause: "no-solver",
      },
    ]);
  });

  it("carries a reading's validity note under the instrument that made it", () => {
    const report = buildProvenance(measuredStep);
    expect(report.validity).toEqual([
      {
        instrument: "conductivity",
        note: "a sum of limiting molar conductivities: the infinite-dilution limit, so above about 0.1 mol/kgw this number rests on a fitted correction",
      },
    ]);
    // A bound on an answer is not a gap in the model.
    expect(report.gaps).toEqual([]);
  });

  it("narrows to one vessel, keeping records that do not name one", () => {
    const step = {
      events: [
        { event: "not_yet_modeled", vessel: 1, what: "the other beaker", cause: "no-solver" },
        { event: "not_yet_modeled", what: "unattributed", cause: "no-solver" },
      ],
      routes: [
        { solver: "a", kind: "computed", chemistry: true, outcome: "not_applicable", vessel: 0 },
        { solver: "b", kind: "computed", chemistry: true, outcome: "not_applicable", vessel: 1 },
      ],
    };
    const report = buildProvenance(step, { vessel: 0 });
    expect(report.vessel).toBe(0);
    expect(report.routes.map((route) => route.solver)).toEqual(["a"]);
    expect(report.gaps.map((gap) => gap.what)).toEqual(["unattributed"]);
  });

  it("is empty, not invented, when the engine sent no routing at all", () => {
    const report = buildProvenance({ events: [], routes: undefined });
    expect(report.empty).toBe(true);
    expect(report.headline).toBeNull();
    expect(report.routes).toEqual([]);
  });

  it("drops a route whose shape it does not recognise rather than guessing", () => {
    const report = buildProvenance({
      routes: [
        { solver: "unknown-kind", kind: "vibes", chemistry: true, outcome: "not_applicable" },
        { kind: "computed", chemistry: true, outcome: "not_applicable" },
        { solver: "no-outcome", kind: "computed", chemistry: true },
        "not a route",
      ],
      events: [],
    });
    expect(report.routes).toEqual([]);
    expect(report.empty).toBe(true);
  });

  it("survives a step object it was never given", () => {
    expect(buildProvenance(null).empty).toBe(true);
    expect(buildProvenance(undefined).headline).toBeNull();
  });

  it("does not repeat one dataset because two events cited it", () => {
    const twice = {
      events: [aqueousStep.events[0], aqueousStep.events[0]],
      routes: [],
    };
    expect(buildProvenance(twice).sources).toHaveLength(1);
  });

  it("still reports a failed solver as failed, with what it said", () => {
    const report = buildProvenance({
      events: [],
      routes: [
        {
          solver: "phreeqc-aqueous",
          kind: "computed",
          chemistry: true,
          outcome: "failed",
          vessel: 0,
          reason: "phreeqc-aqueous could not solve this state: charge balance did not converge",
        },
      ],
    });
    expect(report.routes[0]?.outcome).toBe("failed");
    expect(report.headline).toBeNull();
    expect(report.empty).toBe(false);
  });

  /**
   * The aqueous routing reaches the SAME surface as the combustion one.
   *
   * This is the whole point of `Event::SolutionRouted`: the drawer reads
   * `event.provenance`, and until it existed the only event carrying one
   * was `thermal_equilibrium`. A learner asking where a beaker's pH came
   * from could not be told. Both fixtures below go through one code path
   * and land in `report.sources`, which is what the drawer prints.
   */
  it("reads the aqueous routing off solution_routed, as it reads the burn's", () => {
    const aqueous = buildProvenance(aqueousStep).sources[0];
    const burn = buildProvenance(burnStep).sources[0];
    expect(aqueous?.dataset).toBe("minteq.v4.dat");
    expect(aqueous?.routing).toBe(
      "an aqueous solution is characterised, so the speciation engine leads",
    );
    expect(burn?.dataset).toBe("thermo.inp");
    // Same fields populated on both, from one reader.
    expect(Object.keys(aqueous ?? {}).sort()).toEqual(Object.keys(burn ?? {}).sort());
  });

  /**
   * The routing arrives translated, because `localize_event` renders it
   * before the event leaves the engine. The shell prints it verbatim and
   * must not care which language it is in — a fixture in German proves
   * there is no English-shaped assumption in the reader.
   */
  it("prints the routing in whatever language the engine sent", () => {
    const report = buildProvenance({
      events: [
        {
          event: "solution_routed",
          vessel: 0,
          provenance: {
            engine: "PHREEQC (IPhreeqc, USGS)",
            dataset: "pitzer.dat",
            model: "Pitzer specific-ion-interaction",
            dataset_sources: [],
            routing:
              "die Ionenstärke liegt über dem Bereich, in dem der Standarddatensatz zuverlässig ist",
          },
        },
      ],
      routes: [],
    });
    expect(report.sources[0]?.routing).toBe(
      "die Ionenstärke liegt über dem Bereich, in dem der Standarddatensatz zuverlässig ist",
    );
    expect(report.sources[0]?.dataset).toBe("pitzer.dat");
  });

  /**
   * The routing STANDS between statements.
   *
   * `solution_routed` fires on change, so a step that solved and said
   * nothing about its routing has not lost one. A drawer that showed a
   * dataset on the one step that changed it and nothing on the nine after
   * would be reading the engine's quietness as an absence of provenance.
   */
  describe("carrying the routing between steps", () => {
    const quietStep = {
      events: [{ event: "solution_characterized", vessel: 0, ph: 7.1, ionic_strength: 0.12 }],
      routes: [
        {
          solver: "phreeqc-aqueous",
          kind: "computed",
          chemistry: true,
          outcome: { succeeded: { event_count: 1 } },
          vessel: 0,
        },
      ],
    };

    it("carries the last routing forward, and says that it did", () => {
      const carried = carryRouting({}, aqueousStep);
      expect(carried[0]?.dataset).toBe("minteq.v4.dat");

      const report = buildProvenance(quietStep, { vessel: 0, carried });
      expect(report.sources).toHaveLength(1);
      expect(report.sources[0]?.dataset).toBe("minteq.v4.dat");
      expect(report.sources[0]?.carried).toBe(true);
      // And it is enough to answer "who answered, on what data".
      expect(report.headline?.dataset).toBe("minteq.v4.dat");
    });

    it("prefers what THIS step said, and never marks that carried", () => {
      const stale: CarriedRouting = {
        0: {
          engine: "PHREEQC (IPhreeqc)",
          dataset: "wateq4f.dat",
          model: "old",
          routing: "the default inorganic aqueous dataset",
          datasetSources: [],
        },
      };
      const report = buildProvenance(aqueousStep, { vessel: 0, carried: stale });
      expect(report.sources.map((source) => source.dataset)).toEqual([
        "minteq.v4.dat",
        "wateq4f.dat",
      ]);
      expect(report.sources[0]?.carried).toBeUndefined();
      expect(report.sources[1]?.carried).toBe(true);
    });

    it("keeps one beaker's routing off the other's drawer", () => {
      const carried = carryRouting({}, aqueousStep);
      const other = buildProvenance(quietStep, { vessel: 1, carried });
      expect(other.sources).toEqual([]);
    });

    it("returns the map it was given when a step announced nothing", () => {
      const before = carryRouting({}, aqueousStep);
      expect(carryRouting(before, quietStep)).toBe(before);
    });

    it("drops a provenance that does not say which beaker it belongs to", () => {
      const unowned = {
        events: [{ event: "solution_routed", provenance: { engine: "e", dataset: "d.dat" } }],
        routes: [],
      };
      expect(carryRouting({}, unowned)).toEqual({});
    });
  });
});
