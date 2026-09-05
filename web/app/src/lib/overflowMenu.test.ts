import { describe, expect, it } from "vitest";
import { dismissesUtilityDrawer, type DrawerNode } from "./overflowMenu";

const node = (tag: string, keepsDrawer = false): DrawerNode => ({ tag, keepsDrawer });

describe("the utilities drawer dismissing itself", () => {
  it("closes when a tool is chosen", () => {
    expect(dismissesUtilityDrawer([node("BUTTON")])).toBe(true);
  });

  it("closes when the click landed on something inside the button", () => {
    // The tools render an icon and a label inside the button; the click
    // target is usually the span, which is why per-handler closing kept
    // being written and kept being forgotten.
    expect(dismissesUtilityDrawer([node("SPAN"), node("BUTTON"), node("DIV")])).toBe(true);
  });

  it("closes when a link out of the drawer is followed", () => {
    expect(dismissesUtilityDrawer([node("A")])).toBe(true);
  });

  it("stays open for a click on the panel itself", () => {
    expect(dismissesUtilityDrawer([node("DIV"), node("STRONG")])).toBe(false);
    expect(dismissesUtilityDrawer([])).toBe(false);
  });

  it("stays open for the settings inside it, and for what they contain", () => {
    // Appearance, language and the time scrubber are adjustments, not
    // destinations: closing the menu under the reader's finger would be the
    // opposite bug.
    expect(dismissesUtilityDrawer([node("BUTTON"), node("DIV", true)])).toBe(false);
    expect(dismissesUtilityDrawer([node("SPAN"), node("BUTTON"), node("DIV", true)])).toBe(false);
    expect(dismissesUtilityDrawer([node("SELECT", true)])).toBe(false);
  });

  it("is not case-sensitive about tag names", () => {
    expect(dismissesUtilityDrawer([node("button")])).toBe(true);
  });
});
