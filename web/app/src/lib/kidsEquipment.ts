import type { InfoRow } from "./infoPanel";

export type KidsEquipment = {
  id: string;
  engineVerb: string;
  action: "apparatus" | "transfer" | "instrument";
  title: string;
  blurb: string;
  boundary: string;
  icon: string;
  parts: string[];
  instrument?: string;
};

/** Familiar classroom skins over commands the engine already owns. */
export const KIDS_EQUIPMENT: KidsEquipment[] = [
  {
    id: "balloon-kit",
    engineVerb: "regulate",
    action: "apparatus",
    title: "balloon or gas bag",
    blurb: "collect gas with a flexible pressure boundary",
    boundary: "uses the pressure-controlled headspace; rubber stretch is not modeled",
    icon: "balloon",
    parts: ["balloon or gas bag", "sealed connection", "sample vessel"],
  },
  {
    id: "candle-kit",
    engineVerb: "bunsen",
    action: "apparatus",
    title: "candle and wick",
    blurb: "touch a flame to contents or deliver measured heat",
    boundary: "uses ignition and heat; wick, melt pool, soot, and flame shape are not modeled",
    icon: "candle",
    parts: ["candle wax", "wick", "ignition flame", "sample vessel"],
  },
  {
    id: "paper-chromatography-kit",
    engineVerb: "measure:chromatograph",
    action: "instrument",
    instrument: "chromatograph",
    title: "paper chromatography kit",
    blurb: "spot a sample and separate its components",
    boundary: "uses the computed chromatography operator; the tile and paper are classroom skins",
    icon: "chromatography-paper",
    parts: ["spotting tile", "paper strip", "developing chamber"],
  },
  {
    id: "filter-funnel-kit",
    engineVerb: "filter",
    action: "transfer",
    title: "filter funnel and receiver",
    blurb: "hold back solids and collect the liquid",
    boundary: "uses the existing two-vessel filter; paper pore geometry is not modeled",
    icon: "filter",
    parts: ["filter paper", "funnel", "receiver vessel"],
  },
  {
    id: "magnet-kit",
    engineVerb: "magnet",
    action: "transfer",
    title: "horseshoe magnet",
    blurb: "lift engine-classified magnetic solids into a receiver",
    boundary: "only registry-declared magnetic solids move; field strength is not modeled",
    icon: "magnet",
    parts: ["horseshoe magnet", "sample vessel", "receiver vessel"],
  },
];

/**
 * The two things a kit card no longer says out loud.
 *
 * A kit used to print four blocks on every card — title, purpose, a
 * three-part inventory and a full sentence of modelling caveat — in a
 * half-width column of the cabinet. Five of those is a wall of text where
 * a learner is trying to find a tool. The parts list and the caveat are
 * both true and both worth keeping; they are just answers to "tell me
 * more", not to "which one is the magnet". So they move behind the (i),
 * in the same panel the reagent shelf opens.
 *
 * `translate` is passed in rather than imported so this stays a pure
 * function of the data — the row labels are literals here, which is what
 * lets the translation scan see them.
 */
export function kitInfoRows(item: KidsEquipment, translate: (key: string) => string): InfoRow[] {
  return [
    { term: translate("parts"), detail: item.parts.map((part) => translate(part)).join(" · ") },
    {
      term: translate("what the model computes"),
      detail: translate(item.boundary),
      // The boundary is a sentence, and a sentence set opposite its label
      // starts in a different place on every card.
      block: true,
    },
  ];
}
