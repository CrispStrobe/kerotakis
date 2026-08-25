/**
 * TauriHost: the EngineHost over the shell's `engine_request` command
 * (web/app/src-tauri). Same envelope, same result_json strings as the
 * worker — the UI cannot tell the transports apart (PROTOCOL.md).
 *
 * Uses the global injected by `withGlobalTauri` rather than the npm API
 * package: no extra dependency on the licence surface for one function.
 */

import {
  EngineError,
  type EngineHost,
  type ParticleCensus,
  type Scene,
  type ScriptResult,
  type StepResult,
} from "./EngineHost";

type TauriGlobal = {
  core: { invoke: (cmd: string, args: Record<string, unknown>) => Promise<unknown> };
};

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI__" in window;
}

export class TauriHost implements EngineHost {
  private get tauri(): TauriGlobal {
    return (window as unknown as { __TAURI__: TauriGlobal }).__TAURI__;
  }

  private async req(cmd: string, fields: Record<string, unknown> = {}): Promise<string> {
    try {
      return (await this.tauri.core.invoke("engine_request", {
        req: { cmd, ...fields },
      })) as string;
    } catch (e) {
      throw new EngineError(typeof e === "string" ? e : String(e), "engine");
    }
  }

  async hello(): Promise<{ protocol: number; can_solve?: boolean }> {
    return JSON.parse(await this.req("hello"));
  }
  async step(operatorJson: string): Promise<StepResult> {
    return JSON.parse(await this.req("step", { operator_json: operatorJson }));
  }
  async runScript(script: string): Promise<ScriptResult> {
    return JSON.parse(await this.req("run_script", { script }));
  }
  async parse(line: string): Promise<{ ok: boolean; operator?: unknown; error?: string }> {
    return JSON.parse(await this.req("parse", { line }));
  }
  async grammar(): Promise<{ verb: string; example: string; options?: string[] }[]> {
    return JSON.parse(await this.req("grammar"));
  }
  async relations(): Promise<{ name: string; equation: string; args: string }[]> {
    return JSON.parse(await this.req("relations"));
  }
  async calc(name: string, args: string[]) {
    return JSON.parse(await this.req("calc", { name, args }));
  }
  async loadPack(bytes: Uint8Array) {
    let bin = "";
    for (const b of bytes) bin += String.fromCharCode(b);
    return JSON.parse(await this.req("load_pack", { bytes_b64: btoa(bin) }));
  }
  async snapshot(): Promise<string> {
    const doc = JSON.parse(await this.req("snapshot")) as { snapshot: string };
    return doc.snapshot;
  }
  async restore(snapshot: string): Promise<void> {
    await this.req("restore", { snapshot });
  }
  async setRegister(level: string): Promise<void> {
    await this.req("set_register", { level });
  }
  async scene(): Promise<Scene> {
    return JSON.parse(await this.req("scene"));
  }
  async state(): Promise<unknown> {
    return JSON.parse(await this.req("state"));
  }
  async species(): Promise<unknown[]> {
    return JSON.parse(await this.req("species"));
  }
  async inspect(vessel: number): Promise<{ rendered: string[] }> {
    return JSON.parse(await this.req("inspect", { vessel }));
  }
  async particles(vessel: number): Promise<{ census?: ParticleCensus; rendered: string[] }> {
    return JSON.parse(await this.req("particles", { vessel }));
  }
  async reset(): Promise<void> {
    await this.req("reset");
  }
  dispose(): void {
    // The engine lives with the app process; nothing to tear down.
  }
}
