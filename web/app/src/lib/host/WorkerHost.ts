/**
 * WorkerHost: the EngineHost over a module Web Worker (GUI-004).
 *
 * The worker entry (engineWorker.ts) owns the wasm bench; this side only
 * moves envelopes. Everything JSON-shaped stays JSON until the last moment
 * so the contract on the wire is exactly PROTOCOL.md's.
 */

import {
  RequestChannel,
  type EngineHost,
  type MessagePortLike,
  type Scene,
  type ScriptResult,
  type StepResult,
} from "./EngineHost";

export interface WorkerHostOptions {
  /** Base URL serving the wasm-bindgen output (kerotakis_wasm.js etc.). */
  engineBase?: string;
}

/**
 * The payload root: where the engine files live, and beside them the
 * lessons. Resolved against the page URL (see the constructor note).
 */
export function resolvePayloadBase(engineBase?: string): string {
  const base =
    engineBase ??
    (import.meta.env?.VITE_ENGINE_BASE as string | undefined) ??
    "./engine/";
  return typeof document !== "undefined" ? new URL(base, document.baseURI).href : base;
}

export class WorkerHost implements EngineHost {
  /** Package-private for the crash handlers installed by create(). */
  channel: RequestChannel;

  /**
   * `port` is injectable for tests; production passes the real Worker.
   * The first request initializes the engine inside the worker.
   *
   * The engine base is resolved against the PAGE URL here, not inside the
   * worker — the worker's own URL lives under the bundler's assets/ dir,
   * so resolving a relative base there points at the wrong place.
   */
  constructor(
    private port: MessagePortLike & { terminate?: () => void },
    options: WorkerHostOptions = {},
  ) {
    this.channel = new RequestChannel(port);
    void this.channel.request("init", {
      engine_base: resolvePayloadBase(options.engineBase),
    });
  }

  static create(options: WorkerHostOptions = {}): WorkerHost {
    const worker = new Worker(new URL("./engineWorker.ts", import.meta.url), {
      type: "module",
      name: "kerotakis-engine",
    });
    // A real Worker satisfies MessagePortLike at runtime (its MessageEvent
    // has `data`), but property invariance keeps TS from proving it —
    // Worker.onmessage's own read type wants the full MessageEvent.
    const host = new WorkerHost(
      worker as unknown as MessagePortLike & { terminate?: () => void },
      options,
    );
    // A crashed worker must fail loudly, not hang every pending promise:
    // abandon() rejects everything in flight with an honest message.
    worker.addEventListener("error", (e) => {
      host.channel.abandon(`the engine worker crashed: ${e.message || "unknown error"}`);
    });
    worker.addEventListener("messageerror", () => {
      host.channel.abandon("the engine worker sent an unreadable message");
    });
    return host;
  }

  async hello(): Promise<{ protocol: number; can_solve?: boolean }> {
    return JSON.parse(await this.channel.request("hello"));
  }

  async step(operatorJson: string): Promise<StepResult> {
    return JSON.parse(await this.channel.request("step", { operator_json: operatorJson }));
  }

  async runScript(script: string): Promise<ScriptResult> {
    return JSON.parse(await this.channel.request("run_script", { script }));
  }

  async parse(line: string): Promise<{ ok: boolean; operator?: unknown; error?: string }> {
    return JSON.parse(await this.channel.request("parse", { line }));
  }

  async grammar(): Promise<{ verb: string; example: string; options?: string[] }[]> {
    return JSON.parse(await this.channel.request("grammar"));
  }

  async relations(): Promise<{ name: string; equation: string; args: string }[]> {
    return JSON.parse(await this.channel.request("relations"));
  }

  async calc(name: string, args: string[]) {
    return JSON.parse(await this.channel.request("calc", { name, args }));
  }

  async snapshot(): Promise<string> {
    const doc = JSON.parse(await this.channel.request("snapshot")) as { snapshot: string };
    return doc.snapshot;
  }

  async restore(snapshot: string): Promise<void> {
    await this.channel.request("restore", { snapshot });
  }

  async setRegister(level: string): Promise<void> {
    await this.channel.request("set_register", { level });
  }

  async scene(): Promise<Scene> {
    return JSON.parse(await this.channel.request("scene"));
  }

  async state(): Promise<unknown> {
    return JSON.parse(await this.channel.request("state"));
  }

  async species(): Promise<unknown[]> {
    return JSON.parse(await this.channel.request("species"));
  }

  async inspect(vessel: number): Promise<{ rendered: string[] }> {
    return JSON.parse(await this.channel.request("inspect", { vessel }));
  }

  async particles(vessel: number): Promise<{ rendered: string[] }> {
    return JSON.parse(await this.channel.request("particles", { vessel }));
  }

  async reset(): Promise<void> {
    await this.channel.request("reset");
  }

  dispose(): void {
    this.channel.abandon("host disposed");
    this.port.terminate?.();
  }
}
