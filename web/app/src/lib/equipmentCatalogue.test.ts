import { afterEach, describe, expect, it, vi } from "vitest";
import { APPARATUS } from "./apparatus";
import { INSTRUMENTS, instrumentVerb } from "./instruments";
import { KIDS_EQUIPMENT } from "./kidsEquipment";
import { TRANSFER_TOOLS } from "./transferTools";
import { hasGermanTranslation, i18n } from "./i18n.svelte";
import {
  EQUIPMENT_CATALOGUE,
  EQUIPMENT_GROUPS,
  GATED_IDS,
  GROUP_BLURBS,
  GROUP_LABELS,
  SHELF_ENTRIES,
  accessId,
  catalogIdForApparatus,
  cupboardTally,
  deployedLabel,
  equipmentById,
  equipmentInfoRows,
  equipmentIn,
  setSkinOf,
  runEquipment,
  type EquipmentEntry,
} from "./equipmentCatalogue";

const NO_DEPLOYMENT = {
  apparatusOut: null,
  buretteOut: false,
  transferVerb: null,
  mixActive: false,
} as const;

describe("the merged equipment catalogue", () => {
  it("lists every instrument, apparatus, transfer verb, special and kit exactly once", () => {
    // The whole point of the merge: three surfaces listed overlapping
    // subsets, and the calorimeter and the Geiger counter were reachable in
    // exactly one of them. A thing named twice here would be the old bug
    // with a new home.
    const expected = [
      ...INSTRUMENTS.map((item) => instrumentVerb(item.token)),
      ...APPARATUS.map((item) => item.verb),
      ...TRANSFER_TOOLS.map((item) => item.verb),
      "burette",
      "mix",
      "transport",
      "react",
      ...KIDS_EQUIPMENT.map((item) => item.id),
    ];
    expect([...EQUIPMENT_CATALOGUE.map((entry) => entry.id)].sort()).toEqual([...expected].sort());
    expect(new Set(EQUIPMENT_CATALOGUE.map((entry) => entry.id)).size).toBe(EQUIPMENT_CATALOGUE.length);
  });

  it("resolves authored instrument tokens through the one equipment model", () => {
    expect(catalogIdForApparatus("ph")).toBe("measure:ph");
    expect(catalogIdForApparatus("thermometer")).toBe("measure:thermometer");
    expect(catalogIdForApparatus("balance")).toBe("measure:balance");
    expect(catalogIdForApparatus("stir")).toBe("stir");
    expect(catalogIdForApparatus("beaker")).toBe("beaker");
  });

  it("gives every entry a shelf and something to do", () => {
    for (const entry of EQUIPMENT_CATALOGUE) {
      expect(EQUIPMENT_GROUPS).toContain(entry.group);
      expect(["measure", "install", "transfer", "mix", "burette"]).toContain(entry.action.kind);
      expect(entry.name.length).toBeGreaterThan(0);
      expect(entry.blurb.length).toBeGreaterThan(0);
      // The (i) is the reason the cupboard can be dense: an item says what
      // it is in three words and what it models behind a disclosure.
      expect(entry.boundary.length).toBeGreaterThan(20);
    }
  });

  it("fills every shelf, so no group heading is ever an empty promise", () => {
    for (const group of EQUIPMENT_GROUPS) expect(equipmentIn(group).length).toBeGreaterThan(0);
    // `equipmentIn` answers over the whole catalogue, kits included, and a
    // kit stands on the shelf of the tool it names — so the twelve
    // instruments are the twelve SLOTS on the measure shelf, beside which
    // the paper-chromatography set is a name for one of them.
    expect(SHELF_ENTRIES.filter((entry) => entry.group === "measure").length).toBe(INSTRUMENTS.length);
  });

  it("stands on five shelves, each of which says what it holds", () => {
    // GUI-103: `antreiben` was the heading that did not answer "what do I
    // want to do", and the kits were a shelf for a naming rather than for a
    // kind of tool. Five headings, each with a sentence — a two-word label
    // does not by itself tell a learner that the mortar is on this one.
    expect(EQUIPMENT_GROUPS.length).toBe(5);
    expect([...EQUIPMENT_GROUPS]).not.toContain("drive");
    for (const group of EQUIPMENT_GROUPS) {
      expect(GROUP_LABELS[group].length).toBeGreaterThan(0);
      expect(GROUP_BLURBS[group].length).toBeGreaterThan(20);
    }
    // The verbs the owner moved off `antreiben`, at their new addresses.
    for (const id of ["stir", "centrifuge", "grind"]) expect(equipmentById(id)?.group).toBe("prepare");
    for (const id of ["electrolyse", "cell", "transport"]) expect(equipmentById(id)?.group).toBe("contain");
  });

  it("gives a kit the shelf of the tool it names, rather than a shelf of its own", () => {
    // A kit is drawn INTO a tool's slot, never beside it — but an entry
    // with no shelf could not answer "where is this", so it borrows one.
    for (const kit of KIDS_EQUIPMENT) {
      const entry = equipmentById(kit.id) as EquipmentEntry;
      expect(entry.group).toBe(equipmentById(kit.engineVerb)?.group);
    }
    expect(SHELF_ENTRIES.length).toBe(EQUIPMENT_CATALOGUE.length - KIDS_EQUIPMENT.length);
    expect(SHELF_ENTRIES.every((entry) => entry.aliasOf === undefined)).toBe(true);
  });

  it("makes a kit a skin over a tool that is already here, not a second listing", () => {
    const kits = EQUIPMENT_CATALOGUE.filter((item) => item.aliasOf !== undefined);
    expect(kits.length).toBe(KIDS_EQUIPMENT.length);
    for (const entry of kits) {
      expect(entry.aliasOf).toBeTruthy();
      // Availability is READ from the tool the kit stands for. A candle kit
      // that could be earned separately from the flame it opens would be a
      // second progression table.
      expect(accessId(entry)).toBe(entry.aliasOf);
      expect(EQUIPMENT_CATALOGUE.some((item) => item.id === entry.aliasOf)).toBe(true);
    }
    expect(equipmentById("candle-kit")?.action).toEqual({ kind: "install", verb: "bunsen", preset: { source: "candle" } });
    expect(equipmentById("paper-chromatography-kit")?.action).toEqual({ kind: "measure", token: "chromatograph" });
    expect(equipmentById("filter-funnel-kit")?.action).toEqual({ kind: "transfer", verb: "filter" });
  });

  it("tallies the tools the engine gates, and not the skins over them", () => {
    expect(GATED_IDS.length).toBe(EQUIPMENT_CATALOGUE.length - KIDS_EQUIPMENT.length);
    for (const id of GATED_IDS) expect(id.startsWith("measure:") || !id.endsWith("-kit")).toBe(true);
  });

  it("counts a denominator that cannot move mid-session", () => {
    // The defect: the list the wall counted dropped `react` while the
    // session had no curated reaction to offer, so the fraction read
    // "31/33" and then "32/34" one command later. A denominator that grows
    // when the numerator does is not a measure of anything. `react` is a
    // tool this learner can EVER have, so it is always in the bottom half.
    expect(GATED_IDS).toContain("react");
    const everything = cupboardTally(() => true);
    expect(everything.total).toBe(GATED_IDS.length);
    expect(everything.available).toBe(GATED_IDS.length);
    // "34/34" is a fact, not progress: Sandbox prints no fraction at all.
    expect(everything.show).toBe(false);
    const withoutReact = cupboardTally((id) => id !== "react");
    expect(withoutReact.total).toBe(everything.total);
    expect(withoutReact.available).toBe(everything.available - 1);
    expect(withoutReact.show).toBe(true);
    expect(cupboardTally(() => false)).toEqual({ available: 0, total: GATED_IDS.length, show: true });
  });

  it("routes each action to the handler that already existed for it", () => {
    const handlers = {
      onmeasure: vi.fn(),
      onapparatus: vi.fn(),
      ontransfer: vi.fn(),
      onmix: vi.fn(),
      onburette: vi.fn(),
    };
    const run = (id: string) => runEquipment(equipmentById(id) as EquipmentEntry, 1, handlers);
    run("measure:thermometer");
    expect(handlers.onmeasure).toHaveBeenCalledWith("measure v2 thermometer");
    run("measure:chromatograph");
    expect(handlers.onmeasure).toHaveBeenCalledWith("chromatograph v2");
    run("candle-kit");
    expect(handlers.onapparatus).toHaveBeenCalledWith("bunsen", { source: "candle" });
    run("filter");
    expect(handlers.ontransfer).toHaveBeenCalledWith("filter");
    run("mix");
    expect(handlers.onmix).toHaveBeenCalledTimes(1);
    run("burette");
    expect(handlers.onburette).toHaveBeenCalledTimes(1);
  });

  it("badges only what is actually on the bench, and never a measurement", () => {
    const thermometer = equipmentById("measure:thermometer") as EquipmentEntry;
    // A reading happens and is over; "on bench" would be a lie with a
    // shelf life of one command.
    expect(deployedLabel(thermometer, { ...NO_DEPLOYMENT, apparatusOut: "measure:thermometer" })).toBeNull();
    expect(deployedLabel(equipmentById("stir") as EquipmentEntry, { ...NO_DEPLOYMENT, apparatusOut: "stir" })).toBe("on bench");
    expect(deployedLabel(equipmentById("stir") as EquipmentEntry, { ...NO_DEPLOYMENT, apparatusOut: "heat" })).toBeNull();
    expect(deployedLabel(equipmentById("burette") as EquipmentEntry, { ...NO_DEPLOYMENT, buretteOut: true })).toBe("on bench");
    expect(deployedLabel(equipmentById("filter") as EquipmentEntry, { ...NO_DEPLOYMENT, transferVerb: "filter" })).toBe("select source");
    expect(deployedLabel(equipmentById("mix") as EquipmentEntry, { ...NO_DEPLOYMENT, mixActive: true })).toBe("select sources");
    // A kit is the tool it skins, so the candle kit lights up when the
    // flame panel is out — one deployment, not two cards disagreeing.
    expect(deployedLabel(equipmentById("candle-kit") as EquipmentEntry, { ...NO_DEPLOYMENT, apparatusOut: "bunsen" })).toBe("on bench");
  });

  it("puts the parts and the boundary behind the (i), and nothing else", () => {
    const identity = (key: string) => key;
    const kit = equipmentById("magnet-kit") as EquipmentEntry;
    expect(equipmentInfoRows(kit, identity).map((row) => row.term)).toEqual(["parts", "what the model computes"]);
    const tool = equipmentById("measure:calorimeter") as EquipmentEntry;
    const rows = equipmentInfoRows(tool, identity);
    expect(rows.map((row) => row.term)).toEqual(["what the model computes"]);
    // A sentence is set under its label, not opposite it.
    expect(rows[0]?.block).toBe(true);
    expect(rows[0]?.detail).toBe(tool.boundary);
  });
});

describe("the activity kits, now that the chip is gone (GUI-111)", () => {
  it("draws one slot per tool, under the tool's own name", () => {
    // The chip renamed five of these and nothing else. What matters after
    // its removal is what mattered before: the candle is on the wall once.
    expect(new Set(SHELF_ENTRIES.map((entry) => entry.id)).size).toBe(SHELF_ENTRIES.length);
    expect(SHELF_ENTRIES.filter((entry) => accessId(entry) === "bunsen").length).toBe(1);
    expect(SHELF_ENTRIES.filter((entry) => accessId(entry) === "regulate").length).toBe(1);
    expect(equipmentById("bunsen")?.name).toBe("candle / Bunsen flame");
    for (const entry of SHELF_ENTRIES) expect(entry.aliasOf).toBeUndefined();
  });

  it("still knows which kit names which tool, because two things still need it", () => {
    // `setSkinOf` outlived the chip: it feeds the cupboard's search
    // haystack and the (i) panel. Delete it and the kit vocabulary becomes
    // unreachable, which is the one thing the owner ruled out.
    expect(setSkinOf("bunsen")?.id).toBe("candle-kit");
    expect(setSkinOf("measure:chromatograph")?.name).toBe("paper chromatography kit");
    expect(setSkinOf("measure:geiger")).toBeUndefined();
    expect(KIDS_EQUIPMENT.every((kit) => setSkinOf(kit.engineVerb)?.id === kit.id)).toBe(true);
  });

  it("names the kit and lists its parts in the tool's own (i) panel", () => {
    // Where the kit's name, parts and inventory went. Nothing a kit said
    // is gone; it is said on the tool it was always a skin over.
    const rows = equipmentInfoRows(equipmentById("bunsen") as EquipmentEntry, (key) => key);
    expect(rows[0]).toEqual({ term: "also called", detail: "candle and wick" });
    expect(rows[1]?.term).toBe("parts");
    expect(rows[1]?.detail).toContain("wick");
    expect(rows.at(-1)?.term).toBe("what the model computes");
  });

  it("adds no kit row to a tool no kit names", () => {
    const rows = equipmentInfoRows(equipmentById("measure:geiger") as EquipmentEntry, (key) => key);
    expect(rows.map((row) => row.term)).not.toContain("also called");
    expect(rows.map((row) => row.term)).not.toContain("parts");
  });

  it("keeps the candle reachable as a flame, which is what it always was", () => {
    // The one feature the chip carried beyond a label: `candle-kit` had
    // `preset: { source: "candle" }`, and the engine caps a candle 100 °C
    // below a laboratory burner. The capability is the flame panel's own
    // `source` field, so removing the shortcut removed no chemistry.
    const burner = APPARATUS.find((item) => item.verb === "bunsen");
    const source = burner?.fields.find((field) => field.name === "source");
    expect(source?.options?.map((option) => option.value)).toContain("candle");
    expect(KIDS_EQUIPMENT.find((kit) => kit.id === "candle-kit")?.preset)
      .toEqual({ source: "candle" });
  });
});
