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

export class WorkerHost implements EngineHost {
  private channel: RequestChannel;

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
    const base =
      options.engineBase ??
      (import.meta.env?.VITE_ENGINE_BASE as string | undefined) ??
      "./engine/";
    const resolved =
      typeof document !== "undefined" ? new URL(base, document.baseURI).href : base;
    void this.channel.request("init", { engine_base: resolved });
  }

  static create(options: WorkerHostOptions = {}): WorkerHost {
    const worker = new Worker(new URL("./engineWorker.ts", import.meta.url), {
      type: "module",
      name: "kerotakis-engine",
    });
    return new WorkerHost(worker, options);
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
