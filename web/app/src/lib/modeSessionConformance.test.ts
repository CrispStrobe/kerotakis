import { describe, expect, it } from "vitest";
import type { EngineHost, Scene, ScriptResult } from "./host/EngineHost";
import { AppSaveModeStorage, bootstrapAppSave, type AppSaveRepository } from "./appSaveRepository";
import { Session } from "./session.svelte";

class MemoryStorage {
  values = new Map<string, string>();
  getItem(key: string) { return this.values.get(key) ?? null; }
  setItem(key: string, value: string) { this.values.set(key, value); }
  removeItem(key: string) { this.values.delete(key); }
}

function repository(storage: MemoryStorage): AppSaveRepository {
  const result = bootstrapAppSave(storage);
  if (result.status !== "ready") throw new Error(`bootstrap ${result.status}`);
  return result.repository;
}

class DeterministicHost {
  commands: string[] = [];
  calls: string[] = [];
  rejectRestore = false;

  sceneValue(): Scene {
    return { scene: 1, vessels: [], world002_commands: [...this.commands] } as unknown as Scene;
  }

  asEngineHost(): EngineHost {
    const self = this;
    return {
      async hello() { return { protocol: 1, can_solve: true }; },
      async setLocale() {},
      async setRegister(level: string) { self.calls.push(`register:${level}`); },
      async grammar() { return []; },
      async relations() { return []; },
      async species() { return [{ key: "NaCl", name: "sodium chloride", formula: "NaCl", phase: "solid" }]; },
      async elementCoverage() {
        return { schema: 1, elements: Array.from({ length: 118 }, (_, index) => ({ symbol: `E${index}`, capability: "identity_only", examples: [], routes: [] })) };
      },
      async parse() { return { ok: true }; },
      async runScript(script: string): Promise<ScriptResult> {
        self.calls.push(`run:${script}`);
        const lines = script.split("\n").filter(Boolean);
        self.commands.push(...lines);
        return {
          steps: lines.map((line) => ({ operator: {}, events: [], rendered: [`accepted:${line}`] })),
          scene: self.sceneValue(),
        };
      },
      async scene() { return self.sceneValue(); },
      async snapshot() { return JSON.stringify(self.commands); },
      async restore(token: string) {
        self.calls.push("restore");
        if (self.rejectRestore) throw new Error("stale snapshot");
        self.commands = JSON.parse(token) as string[];
      },
      async catalog(request: { mode?: string; completed?: number }) {
        return {
          mode: (request.mode ?? "story") as "story" | "sandbox",
          completed: request.completed ?? 0,
          items: [],
          packs: [],
        };
      },
  async reset() { self.calls.push("reset"); self.commands = []; },
      async state() { return {}; },
      async inspect() { return { rendered: [] }; },
      async particles() { return { rendered: [] }; },
      async step() { return { events: [], rendered: [] }; },
      async calc() { return { ok: false, error: "unused" }; },
      async balance() { return { ok: false, error: "unused" }; },
      async questStart() {},
      async questStop() {},
      async questAnswer() { return { outputs: [] }; },
      async loadPack() { return { added: 0, skipped: 0, loaded_total: 0 }; },
      dispose() {},
    } as unknown as EngineHost;
  }
}

const marker = (scene: Scene | null): string[] => (
  (scene as unknown as { world002_commands?: string[] } | null)?.world002_commands ?? []
);

describe("WORLD-002 Session chemistry conformance", () => {
  it("keeps divergent logs, undo, reset and reopen isolated by mode", async () => {
    const root = new MemoryStorage();
    let appSave = repository(root);
    let storyStorage = new AppSaveModeStorage(appSave, "story");
    let sandboxStorage = new AppSaveModeStorage(appSave, "sandbox");
    const storyHost = new DeterministicHost();
    const sandboxHost = new DeterministicHost();
    const story = new Session(storyHost.asEngineHost(), storyStorage, "story");
    const sandbox = new Session(sandboxHost.asEngineHost(), sandboxStorage, "sandbox");

    await story.submit("add v1 water 100mL");
    await story.submit("add v1 NaCl 1g");
    await story.undo();
    await sandbox.submit("add v1 AgNO3 1g");
    expect(story.position).toBe(1);
    expect(sandbox.position).toBe(1);

    // A real reopen constructs a new repository and both adapters from the
    // same global envelope; it does not retain either adapter's cached view.
    appSave = repository(root);
    storyStorage = new AppSaveModeStorage(appSave, "story");
    sandboxStorage = new AppSaveModeStorage(appSave, "sandbox");
    const reopenedStoryHost = new DeterministicHost();
    const reopenedSandboxHost = new DeterministicHost();
    const reopenedStory = new Session(reopenedStoryHost.asEngineHost(), storyStorage, "story");
    const reopenedSandbox = new Session(reopenedSandboxHost.asEngineHost(), sandboxStorage, "sandbox");
    await reopenedStory.connect();
    await reopenedSandbox.connect();
    expect(reopenedStory.commandLog).toEqual(["add v1 water 100mL", "add v1 NaCl 1g"]);
    expect(reopenedStory.position).toBe(1);
    expect(marker(reopenedStory.scene)).toEqual(["add v1 water 100mL"]);
    expect(reopenedSandbox.commandLog).toEqual(["add v1 AgNO3 1g"]);
    expect(marker(reopenedSandbox.scene)).toEqual(["add v1 AgNO3 1g"]);

    await reopenedSandbox.clear();
    expect(reopenedSandbox.commandLog).toEqual([]);
    const storyAgain = new Session(new DeterministicHost().asEngineHost(), storyStorage, "story");
    await storyAgain.connect();
    expect(storyAgain.position).toBe(1);
    expect(storyAgain.commandLog).toHaveLength(2);
  });

  it("produces equal chemistry for equal initial worlds and operators", async () => {
    const storyHost = new DeterministicHost();
    const sandboxHost = new DeterministicHost();
    const story = new Session(storyHost.asEngineHost(), null, "story");
    const sandbox = new Session(sandboxHost.asEngineHost(), null, "sandbox");
    for (const line of ["add v1 water 100mL", "add v1 NaCl 1g", "measure v1 eyes"]) {
      expect(await story.submit(line)).toBe(true);
      expect(await sandbox.submit(line)).toBe(true);
    }
    expect(story.commandLog).toEqual(sandbox.commandLog);
    expect(marker(story.scene)).toEqual(marker(sandbox.scene));
    expect(story.feed.filter((entry) => entry.kind === "line").map((entry) => entry.text))
      .toEqual(sandbox.feed.filter((entry) => entry.kind === "line").map((entry) => entry.text));
  });

  it("falls back to deterministic replay when a persisted snapshot is stale", async () => {
    const root = new MemoryStorage();
    const storyStorage = new AppSaveModeStorage(repository(root), "story");
    storyStorage.setItem("kero.session.v1", JSON.stringify({
      log: ["add v1 water 100mL", "add v1 NaCl 1g"],
      position: 2,
      register: "lv1",
      snapshot: "from-an-older-engine",
    }));
    const host = new DeterministicHost();
    host.rejectRestore = true;
    const session = new Session(host.asEngineHost(), storyStorage, "story");
    await session.connect();
    expect(host.calls).toContain("restore");
    expect(host.calls).toContain("run:add v1 water 100mL\nadd v1 NaCl 1g");
    expect(marker(session.scene)).toEqual(["add v1 water 100mL", "add v1 NaCl 1g"]);
    expect(session.feed.some((entry) => entry.text.includes("replayed"))).toBe(true);
  });
});
