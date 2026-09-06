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
  type BalanceAnswerReply,
  type BalanceExerciseReply,
  type BalanceMarkReply,
  type CatalogRequest,
  type CatalogResponse,
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
  async parse(line: string): Promise<{ ok: boolean; operator?: unknown; error?: string; canonical?: string }> {
    return JSON.parse(await this.req("parse", { line }));
  }
  async grammar(): Promise<{ verb: string; example: string; typed?: string | null; options?: string[] }[]> {
    return JSON.parse(await this.req("grammar"));
  }
  async catalog(request: CatalogRequest): Promise<CatalogResponse> {
    return JSON.parse(await this.req("catalog", { request }));
  }

  async relations(): Promise<{ name: string; equation: string; args: string }[]> {
    return JSON.parse(await this.req("relations"));
  }
  async calc(name: string, args: string[]) {
    return JSON.parse(await this.req("calc", { name, args }));
  }
  async balanceExercise(equation: string): Promise<BalanceExerciseReply> {
    return JSON.parse(await this.req("balance_exercise", { equation }));
  }
  async balanceMark(equation: string, answer: number[]): Promise<BalanceMarkReply> {
    return JSON.parse(await this.req("balance_mark", { equation, answer: JSON.stringify(answer) }));
  }
  async balanceReveal(equation: string): Promise<BalanceAnswerReply> {
    return JSON.parse(await this.req("balance_reveal", { equation }));
  }
  async questStart(specJson: string): Promise<void> {
    await this.req("quest_start", { spec_json: specJson });
  }
  async questStop(): Promise<void> {
    await this.req("quest_stop");
  }
  async questAnswer(alias: string, guess: string) {
    return JSON.parse(await this.req("quest_answer", { alias, guess }));
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
  async setLocale(code: string): Promise<void> {
    await this.req("set_locale", { code });
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
  async elementCoverage(): Promise<unknown> {
    return JSON.parse(await this.req("element_coverage"));
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
