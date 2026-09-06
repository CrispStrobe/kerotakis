/**
 * The two-vessel verbs, as data rather than as an array inside a component.
 *
 * These six lived in `EquipmentCabinet.svelte`, which made them the only
 * equipment in the app with no module and therefore no unit test: nothing
 * could assert that the wall offered them, or that each one named a verb
 * `twoVesselLine` can actually compile. Lifting them out is what lets the
 * merged catalogue in `equipmentCatalogue.ts` count them.
 */
import type { TwoVesselAction } from "./directActions";

export interface TransferTool {
  verb: TwoVesselAction;
  title: string;
  blurb: string;
}

export const TRANSFER_TOOLS: TransferTool[] = [
  { verb: "filter", title: "filter", blurb: "separate solids from liquid" },
  { verb: "decant", title: "decant", blurb: "pour off a chosen fraction" },
  { verb: "drain", title: "drain", blurb: "move the lower liquid layer" },
  { verb: "magnet", title: "magnet", blurb: "lift out magnetic solids" },
  { verb: "cell", title: "voltmeter", blurb: "connect two half-cells" },
  { verb: "distil", title: "still", blurb: "separate by volatility" },
];
