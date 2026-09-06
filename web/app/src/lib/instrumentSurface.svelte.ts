/**
 * Who owns "the cupboard is open", "what was measured last", and "does the
 * cupboard use the activity sets' names".
 *
 * The first two are needed by two components that are not related: the strip
 * lives inside `Inspector.svelte` (the journal pane) and the cupboard is a
 * modal the app mounts at the top level. Threading a prop between them would
 * mean editing every component on the path for state neither of them owns,
 * so the state lives here and both read it — the same shape `i18n` and `pwa`
 * already use.
 *
 * The recents are loaded once, on first read, rather than in a constructor:
 * this module is imported during SSR-free tests and by the vitest suite,
 * where `localStorage` may not exist at import time.
 */
import { SETS_VIEW_KEY, loadSetsView, saveSetsView } from "./equipmentCatalogue";
import {
  DEFAULT_RECENT_INSTRUMENTS,
  RECENT_INSTRUMENTS_KEY,
  loadRecentInstruments,
  rememberInstrument,
  saveRecentInstruments,
} from "./instrumentRecents";

const browserStorage = (): Storage | null => {
  try {
    return typeof localStorage === "undefined" ? null : localStorage;
  } catch {
    // Safari in private mode throws on the property itself.
    return null;
  }
};

class InstrumentSurface {
  /** The one cupboard, open or not. */
  open = $state(false);
  /** Most recent first. Empty until `hydrate()` has run at least once. */
  recent = $state<string[]>([]);
  /**
   * Whether the cupboard names its tools after the activity sets (GUI-103).
   *
   * It lives here rather than in the cupboard because the cupboard is
   * mounted and unmounted on every open, and a view preference that forgot
   * itself between two openings would be a switch the learner has to flip
   * again every time they come back to it.
   */
  sets = $state(false);
  #hydrated = false;

  /** Reads what was stored the first time anyone asks; cheap afterwards. */
  hydrate(): void {
    if (this.#hydrated) return;
    this.#hydrated = true;
    this.recent = loadRecentInstruments(browserStorage(), RECENT_INSTRUMENTS_KEY);
    this.sets = loadSetsView(browserStorage(), SETS_VIEW_KEY);
  }

  showSets(on: boolean): void {
    this.hydrate();
    this.sets = on;
    saveSetsView(browserStorage(), SETS_VIEW_KEY, on);
  }

  used(token: string, known: readonly string[]): void {
    this.hydrate();
    this.recent = rememberInstrument(this.recent, token, known);
    saveRecentInstruments(browserStorage(), RECENT_INSTRUMENTS_KEY, this.recent);
  }
}

export const instrumentSurface = new InstrumentSurface();
export { DEFAULT_RECENT_INSTRUMENTS };
