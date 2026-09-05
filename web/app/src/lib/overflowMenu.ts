/**
 * When the "•••" utilities drawer gets out of the way.
 *
 * The drawer is an overflow menu: it exists because the toolbar cannot hold
 * every tool at once. Choosing something from it therefore *finishes* with
 * it — but each item's own handler had to remember to close it, three of
 * them did, and the rest left the panel standing over the very thing they
 * had just opened (the balancing drill, the concept map, the periodic
 * table). Menus do not work that way anywhere else, so nobody looked for a
 * second gesture to dismiss it.
 *
 * One delegated decision replaces twelve remembered ones. The rule has an
 * exception worth naming: some of the drawer's contents are *settings*, not
 * destinations — the appearance radio group, the language picker, the time
 * scrubber. Adjusting one of those and having the panel vanish would be the
 * opposite bug, so they opt out with `data-keeps-drawer` and the opt-out is
 * inherited by everything inside them.
 *
 * Expressed over a plain ancestor chain rather than over DOM nodes: the
 * component owns the walk, this owns the rule, and the rule is testable
 * without a browser.
 */

/** One ancestor of the clicked element, on the way up to the drawer. */
export type DrawerNode = {
  /** The element's tag name; case is not significant. */
  tag: string;
  /** Whether it carries `data-keeps-drawer`. */
  keepsDrawer: boolean;
};

/** Tags that count as "choosing an item" rather than as clicking the panel. */
const ACTIONS = new Set(["BUTTON", "A", "SELECT", "OPTION"]);

/**
 * Does this click dismiss the drawer?
 *
 * `path` is the clicked element and its ancestors, nearest first, stopping
 * at the drawer itself (which is not included). An empty path — a click on
 * the drawer's own padding — dismisses nothing.
 */
export function dismissesUtilityDrawer(path: readonly DrawerNode[]): boolean {
  if (path.some((node) => node.keepsDrawer)) return false;
  return path.some((node) => ACTIONS.has(node.tag.toUpperCase()));
}
