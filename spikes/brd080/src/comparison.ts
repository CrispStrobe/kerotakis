import { AdapterError, boundedViewport, validateFixture, type CandidateAdapter, type ViewerFixture, type ViewerSession, type ViewerSnapshot } from "./adapter";

export type ComparisonStatus =
  | { state: "idle"; message: string }
  | { state: "loading"; message: string }
  | { state: "ready"; message: string }
  | { state: "unsupported" | "error"; message: string };

export class ComparisonController {
  #session: ViewerSession | null = null;
  #generation = 0;
  status: ComparisonStatus = { state: "idle", message: "Choose a candidate and fixture." };

  constructor(private readonly host: HTMLElement, private readonly announce: (status: ComparisonStatus) => void) {}

  async show(adapter: CandidateAdapter, fixture: ViewerFixture, reducedMotion: boolean): Promise<void> {
    const generation = ++this.#generation;
    await this.#disposeSession();
    this.host.replaceChildren();
    try {
      validateFixture(fixture);
      if (!adapter.supports(fixture.kind)) throw new AdapterError("unsupported-fixture", `${adapter.label} does not support ${fixture.kind} fixtures in this spike.`);
      this.#set({ state: "loading", message: `Loading ${fixture.description} in ${adapter.label}…` });
      const session = await adapter.mount(this.host, fixture, { labelsVisible: false, reducedMotion });
      if (generation !== this.#generation) { await session.dispose(); return; }
      this.#session = session;
      const box = this.host.getBoundingClientRect();
      const viewport = boundedViewport(box.width, box.height, globalThis.devicePixelRatio);
      await session.resize(viewport.width, viewport.height, viewport.dpr);
      this.#set({ state: "ready", message: `${fixture.description} is ready in ${adapter.label}. The table below remains the accessible source.` });
    } catch (error) {
      if (generation !== this.#generation) return;
      const unsupported = error instanceof AdapterError && error.code === "unsupported-fixture";
      this.#set({ state: unsupported ? "unsupported" : "error", message: error instanceof Error ? error.message : "The renderer failed without an error message." });
    }
  }

  async setLabels(visible: boolean) { await this.#session?.setLabels(visible); }
  async select(atomIds: readonly number[]) { await this.#session?.select(atomIds); }
  snapshot(): ViewerSnapshot | null { return this.#session?.snapshot() ?? null; }
  async resize(width: number, height: number, dpr: number) {
    const bounded = boundedViewport(width, height, dpr);
    await this.#session?.resize(bounded.width, bounded.height, bounded.dpr);
  }
  async dispose() { ++this.#generation; await this.#disposeSession(); this.host.replaceChildren(); }

  async #disposeSession() {
    const previous = this.#session;
    this.#session = null;
    if (previous) await previous.dispose();
  }
  #set(status: ComparisonStatus) { this.status = status; this.announce(status); }
}
