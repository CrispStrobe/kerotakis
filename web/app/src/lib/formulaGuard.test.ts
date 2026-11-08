import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

/**
 * A formula reaches the reader typeset or not at all. `formula.ts` is the
 * one formatter; this fails when a component interpolates a `.formula`
 * field into markup without it (`{item.formula}`, `{s.formula}`), which is
 * how `H2O` sat on the shelf beside an equation strip that wrote `H₂O`.
 * Attribute values (`data-formula={…}`) and script code are not markup the
 * reader sees, so only text interpolations outside `<script>`/`<style>`
 * are checked.
 */
const ROOTS = [join(import.meta.dirname, "components"), join(import.meta.dirname, "..")];

function svelteFiles(): string[] {
  const out: string[] = [];
  for (const root of ROOTS) {
    for (const name of readdirSync(root)) {
      if (name.endsWith(".svelte")) out.push(join(root, name));
    }
  }
  return out;
}

function markup(source: string): string {
  return source.replace(/<script[\s\S]*?<\/script>/g, "").replace(/<style[\s\S]*?<\/style>/g, "");
}

describe("formula display guard", () => {
  it("finds the components it guards", () => {
    expect(svelteFiles().some((f) => f.endsWith("Shelf.svelte"))).toBe(true);
  });

  it("never renders a .formula field raw", () => {
    const offenders: string[] = [];
    for (const file of svelteFiles()) {
      const text = markup(readFileSync(file, "utf8"));
      for (const m of text.matchAll(/(=)?\{([^{}]*\.formula)\s*\}/g)) {
        if (m[1]) continue; // attribute value, not visible text
        offenders.push(`${file.split("/").slice(-2).join("/")}: {${m[2]}}`);
      }
    }
    expect(offenders, "wrap with typesetFormula/typesetFormulaTokens from lib/formula").toEqual([]);
  });
});
