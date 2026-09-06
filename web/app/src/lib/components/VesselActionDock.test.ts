import { describe, expect, it } from "vitest";
import type { ComponentProps } from "svelte";
import { render } from "svelte/server";
import VesselActionDock from "./VesselActionDock.svelte";

/**
 * The dock's four change buttons.
 *
 * Owner, from the German deploy: they read "gieße", "↻ rühren einstellen…",
 * "↑ erwärmen einstellen…", "❄ abkühlen einstellen…". Three of them spent a
 * whole second line of type on a caption that says what the ellipsis
 * already says, and the fourth was the only one with no icon at all.
 */
function dock(props: Partial<ComponentProps<typeof VesselActionDock>> = {}): string {
  return render(VesselActionDock, {
    props: {
      vessel: 0,
      label: "beaker",
      boundary: "open",
      busy: false,
      onaction: () => {},
      onconfigure: () => {},
      onpour: () => {},
      ondetails: () => {},
      onmore: () => {},
      ...props,
    },
  }).body;
}

describe("the vessel dock's buttons", () => {
  it("says a form opens with an ellipsis, not with a second line of type", () => {
    const rendered = dock();
    expect(rendered).toContain("stir…");
    expect(rendered).toContain("heat…");
    expect(rendered).toContain("cool…");
    expect(rendered).not.toContain("set…");
  });

  it("runs the immediate actions without promising a form", () => {
    const rendered = dock();
    expect(rendered).toContain(">look<");
    expect(rendered).not.toContain("look…");
  });

  it("gives pour an icon, like every other button in the row", () => {
    const rendered = dock();
    const pour = rendered.slice(rendered.indexOf('class="pour'));
    expect(pour).toContain('<span class="icon');
    expect(pour).toContain("⤵");
  });
});
