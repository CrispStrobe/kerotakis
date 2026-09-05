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
