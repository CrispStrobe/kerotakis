/**
 * Who owns "the cupboard is open" and "what was measured last".
 *
 * Both pieces are needed by two components that are not related: the strip
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
  #hydrated = false;

  /** Reads the stored order the first time anyone asks; cheap afterwards. */
  hydrate(): void {
    if (this.#hydrated) return;
    this.#hydrated = true;
    this.recent = loadRecentInstruments(browserStorage(), RECENT_INSTRUMENTS_KEY);
  }

  used(token: string, known: readonly string[]): void {
    this.hydrate();
    this.recent = rememberInstrument(this.recent, token, known);
    saveRecentInstruments(browserStorage(), RECENT_INSTRUMENTS_KEY, this.recent);
  }
}

export const instrumentSurface = new InstrumentSurface();
export { DEFAULT_RECENT_INSTRUMENTS };
