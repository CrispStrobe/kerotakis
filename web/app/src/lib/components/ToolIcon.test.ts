import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { EQUIPMENT_CATALOGUE } from "../equipmentCatalogue";
import { INSTRUMENTS } from "../instruments";

/**
 * GUI-109 — the drawings, and the one way they can fail silently.
 *
 * `ToolIcon.svelte` renders nothing at all when it is handed a name it has
 * no path for: `{#if d}`. That is the right runtime behaviour — a missing
 * drawing must not throw — and it is exactly why it needs checking here,
 * because the failure is an empty box on a shelf and nothing else. A
 * component test could not see it either: every one of them renders
 * through `svelte/server`, which would produce the same empty markup and
 * assert happily against it.
 *
 * So this reads the paths out of the component and holds them against the
 * catalogue that names them.
 */
const SOURCE = readFileSync(join(import.meta.dirname, "ToolIcon.svelte"), "utf8");

/** The keys of the `PATHS` record, read from the component itself. */
function drawn(): Set<string> {
  const start = SOURCE.indexOf("const PATHS");
  const end = SOURCE.indexOf("};", start);
  expect(start, "ToolIcon.svelte no longer declares PATHS").toBeGreaterThan(-1);
  const block = SOURCE.slice(start, end);
  return new Set(
    [...block.matchAll(/^\s*"?([A-Za-z0-9-]+)"?:\s*"([^"]*)"/gm)]
      .map((match) => match[1])
      .filter((name): name is string => name !== undefined),
  );
}

/** Every `d` string, in declaration order. */
function paths(): [string, string][] {
  const start = SOURCE.indexOf("const PATHS");
  const end = SOURCE.indexOf("};", start);
  return [...SOURCE.slice(start, end).matchAll(/^\s*"?([A-Za-z0-9-]+)"?:\s*"([^"]*)"/gm)]
    .map((match) => [match[1] ?? "", match[2] ?? ""] as [string, string]);
}

describe("the tool portraits", () => {
  it("has a path for every icon the catalogue asks for", () => {
    const available = drawn();
    const missing = EQUIPMENT_CATALOGUE
      .filter((entry) => entry.render.kind === "icon")
      .map((entry) => (entry.render as { kind: "icon"; name: string }).name)
      .filter((name) => !available.has(name));
    // A name with no path is an empty box on the shelf, and nothing else
    // anywhere says so.
    expect([...new Set(missing)], "icons named by the catalogue but never drawn").toEqual([]);
  });

  it("draws every instrument that claims one", () => {
    const available = drawn();
    const missing = INSTRUMENTS
      .filter((item) => item.icon !== undefined)
      .filter((item) => !available.has(item.icon as string))
      .map((item) => item.token);
    expect(missing).toEqual([]);
  });

  it("leaves exactly the two readings a drawing would not help", () => {
    // The honest answer to "draw the Geiger counter". `pH` and `Bq` are
    // the quantity's and the unit's own notation, translated into no
    // language the app ships, and neither a probe nor a grey box reads as
    // its instrument at 24 px. A letter is honest about being a label; a
    // shape that means nothing is not. Pinned so that adding an icon to
    // one of them is a decision somebody makes on purpose.
    const lettered = INSTRUMENTS.filter((item) => item.icon === undefined);
    expect(lettered.map((item) => item.token)).toEqual(["ph", "geiger"]);
    expect(lettered.map((item) => item.glyph)).toEqual(["pH", "Bq"]);
  });

  it("carries no text inside a drawing", () => {
    // Text in an icon is text nobody translates. The drawings are `d`
    // attributes on one `<path>`, so this is really a check that nobody
    // adds a `<text>` or a `<tspan>` to the template later.
    expect(SOURCE).not.toMatch(/<text|<tspan|<foreignObject/);
  });

  it("takes its colour from the theme rather than from a hex", () => {
    // Both themes, for free, and the reason these are inline SVG rather
    // than an icon font or a sprite the PWA payload could silently lose.
    expect(SOURCE).toContain("stroke: currentColor");
    const style = SOURCE.slice(SOURCE.indexOf("<style>"));
    expect(style, "a hard-coded colour will be wrong in one of the two themes")
      .not.toMatch(/#[0-9a-fA-F]{3,8}\b|rgb\(|hsl\(/);
  });

  it("keeps every drawing inside the 18x18 box it is scaled from", () => {
    // A path that runs outside the viewBox is clipped, and the clip is
    // invisible at 24 px — it just looks like a badly drawn icon.
    const strays: string[] = [];
    for (const [name, d] of paths()) {
      const numbers = [...d.matchAll(/-?\d+(?:\.\d+)?/g)].map((match) => Number(match[0]));
      // Arc flags and radii live in the same stream as coordinates, so this
      // is a generous bound: it catches a decimal point in the wrong place,
      // not a half-pixel overhang.
      if (numbers.some((value) => value < -1 || value > 19)) strays.push(name);
    }
    expect(strays).toEqual([]);
  });
});
