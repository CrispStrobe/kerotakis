import { afterEach, describe, expect, it, vi } from "vitest";
import { APPARATUS } from "./apparatus";
import { INSTRUMENTS, instrumentVerb } from "./instruments";
import { KIDS_EQUIPMENT } from "./kidsEquipment";
import { TRANSFER_TOOLS } from "./transferTools";
import { hasGermanTranslation, i18n } from "./i18n.svelte";
import {
  EQUIPMENT_CATALOGUE,
  EQUIPMENT_GROUPS,
  GROUP_LABELS,
  accessId,
  deployedLabel,
  equipmentById,
  equipmentInfoRows,
  equipmentIn,
  gatedIds,
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
    expect(equipmentIn("measure").length).toBe(INSTRUMENTS.length);
    expect(equipmentIn("sets").length).toBe(KIDS_EQUIPMENT.length);
  });

  it("makes a kit a skin over a tool that is already here, not a second listing", () => {
    for (const entry of EQUIPMENT_CATALOGUE.filter((item) => item.group === "sets")) {
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
    // The wall's own count used to be over a list whose length changed when
    // `react` arrived, so the denominator moved mid-session.
    expect(gatedIds(true).length).toBe(EQUIPMENT_CATALOGUE.length - KIDS_EQUIPMENT.length);
    expect(gatedIds(false)).not.toContain("react");
    expect(gatedIds(false).length).toBe(gatedIds(true).length - 1);
    for (const id of gatedIds(true)) expect(id.startsWith("measure:") || !id.endsWith("-kit")).toBe(true);
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

describe("the catalogue's own vocabulary", () => {
  afterEach(() => {
    i18n.locale = "en";
  });

  it("has German for every name, purpose, boundary and shelf heading", () => {
    // These reach `t()` through a variable, so the source scan in
    // i18n.test.ts cannot see them: an English sentence would sit inside a
    // German cupboard and fail nothing at all.
    const missing = new Set<string>();
    for (const label of Object.values(GROUP_LABELS)) {
      if (!hasGermanTranslation(label)) missing.add(label);
    }
    for (const entry of EQUIPMENT_CATALOGUE) {
      for (const text of [entry.name, entry.blurb, entry.boundary, ...(entry.parts ?? [])]) {
        if (!hasGermanTranslation(text)) missing.add(text);
      }
    }
    expect([...missing].sort()).toEqual([]);
  });
});
