import { AdapterError, boundedViewport, validateFixture, type CandidateAdapter, type ViewerFixture, type ViewerSession, type ViewerSnapshot } from "./adapter";

export type ComparisonStatus =
  | { state: "idle"; message: string }
  | { state: "loading"; message: string }
  | { state: "ready"; message: string }
  | { state: "unsupported" | "error"; message: string };

type ActiveRecord = {
  container: HTMLElement | null;
  session: ViewerSession | null;
  generation: number;
};

export class ComparisonController {
  #active: ActiveRecord | null = null;
  #generation = 0;
  status: ComparisonStatus = { state: "idle", message: "Choose a candidate and fixture." };

  constructor(private readonly host: HTMLElement, private readonly announce: (status: ComparisonStatus) => void) {}

  async #cleanup(record: ActiveRecord): Promise<void> {
    const session = record.session;
    const container = record.container;
    record.session = null;
    record.container = null;
    if (container) {
      try { container.remove(); } catch {}
    }
    if (session) {
      try { await session.dispose(); } catch {}
    }
  }

  async show(adapter: CandidateAdapter, fixture: ViewerFixture, reducedMotion: boolean): Promise<void> {
    const generation = ++this.#generation;
    const record: ActiveRecord = { container: null, session: null, generation };

    const prevActive = this.#active;
    this.#active = record;

    let container: HTMLElement | null = null;
    try {
      container = document.createElement("div");
      container.dataset.viewerGeneration = String(generation);
      container.style.width = "100%";
      container.style.height = "100%";
      record.container = container;
    } catch (e) {
      if (this.#active === record) this.#active = null;
      this.#set({ state: "error", message: e instanceof Error ? e.message : "Failed to create viewer container." });
      await this.#cleanup(record);
      if (prevActive) await this.#cleanup(prevActive);
      return;
    }

    try {
      this.host.appendChild(container);
    } catch (e) {
      if (this.#active === record) this.#active = null;
      this.#set({ state: "error", message: e instanceof Error ? e.message : "Failed to attach viewer container." });
      await this.#cleanup(record);
      if (prevActive) await this.#cleanup(prevActive);
      return;
    }

    if (prevActive) {
      await this.#cleanup(prevActive);
    }
    if (this.#active !== record) {
      await this.#cleanup(record);
      return;
    }

    try {
      validateFixture(fixture);
      if (!adapter.supports(fixture.kind)) throw new AdapterError("unsupported-fixture", `${adapter.label} does not support ${fixture.kind} fixtures in this spike.`);
      this.#set({ state: "loading", message: `Loading ${fixture.description} in ${adapter.label}…` });
      if (this.#active !== record) {
        await this.#cleanup(record);
        return;
      }
      const session = await adapter.mount(container, fixture, { labelsVisible: false, reducedMotion });
      if (this.#active !== record) {
        const stale: ActiveRecord = { container, session, generation };
        await this.#cleanup(stale);
        return;
      }
      record.session = session;
      const box = container.getBoundingClientRect();
      const viewport = boundedViewport(box.width, box.height, (globalThis as any).devicePixelRatio);
      await session.resize(viewport.width, viewport.height, viewport.dpr);
      if (this.#active !== record) return;
      if (record.session !== session) return;
      this.#set({ state: "ready", message: `${fixture.description} is ready in ${adapter.label}. The table below remains the accessible source.` });
    } catch (error) {
      if (this.#active !== record) {
        await this.#cleanup(record);
        return;
      }
      this.#active = null;
      const unsupported = error instanceof AdapterError && error.code === "unsupported-fixture";
      this.#set({ state: unsupported ? "unsupported" : "error", message: error instanceof Error ? error.message : "The renderer failed without an error message." });
      await this.#cleanup(record);
    }
  }

  async setLabels(visible: boolean) { await this.#active?.session?.setLabels(visible); }
  async select(atomIds: readonly number[]) { await this.#active?.session?.select(atomIds); }
  snapshot(): ViewerSnapshot | null { return this.#active?.session?.snapshot() ?? null; }
  async resize(width: number, height: number, dpr: number) {
    const bounded = boundedViewport(width, height, dpr);
    await this.#active?.session?.resize(bounded.width, bounded.height, bounded.dpr);
  }
  async dispose() {
    ++this.#generation;
    const toClean = this.#active;
    this.#active = null;
    if (toClean) await this.#cleanup(toClean);
  }

  #set(status: ComparisonStatus) { this.status = status; this.announce(status); }
}
