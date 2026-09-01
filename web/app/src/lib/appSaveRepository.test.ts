import { describe, expect, it } from "vitest";
import { emptyAppSave } from "./appSave";
import {
  AppSaveModeStorage,
  MODE_APPARATUS_KEY,
  MODE_GUIDES_KEY,
  MODE_LAYOUT_KEY,
  bootstrapAppSave,
  cloneStoryBenchToSandbox,
  createAppSaveRepository,
  readSharedProfile,
  readSharedSetting,
  saveSharedProfile,
  saveSharedSetting,
} from "./appSaveRepository";

class MemoryStorage {
  values = new Map<string, string>();
  failReads = false;
  failWrites = false;
  reads = 0;
  failReadAt: number | null = null;
  getItem(key: string) {
    this.reads += 1;
    if (this.failReads || this.reads === this.failReadAt) throw new Error("read blocked");
    return this.values.get(key) ?? null;
  }
  setItem(key: string, value: string) {
    if (this.failWrites) throw new Error("write blocked");
    this.values.set(key, value);
  }
  removeItem(key: string) {
    if (this.failWrites) throw new Error("write blocked");
    this.values.delete(key);
  }
}

function ready(storage = new MemoryStorage()) {
  const bootstrapped = bootstrapAppSave(storage);
  if (bootstrapped.status !== "ready") throw new Error(`bootstrap ${bootstrapped.status}`);
  return { storage, ...bootstrapped };
}

describe("AppSave repository integration", () => {
  it("presents isolated synchronous key-value stores over one envelope", () => {
    const { repository } = ready();
    const story = new AppSaveModeStorage(repository, "story");
    const sandbox = new AppSaveModeStorage(repository, "sandbox");
    story.setItem("kero.session.v1", "story-session");
    story.setItem("kero.codex.done.v1", '["story-discovery"]');
    sandbox.setItem("kero.session.v1", "sandbox-session");
    sandbox.setItem("kero.codex.done.v1", '["sandbox-discovery"]');
    expect(story.getItem("kero.session.v1")).toBe("story-session");
    expect(sandbox.getItem("kero.session.v1")).toBe("sandbox-session");
    expect(story.getItem("kero.codex.done.v1")).toBe('["story-discovery"]');
    expect(sandbox.getItem("kero.codex.done.v1")).toBe('["sandbox-discovery"]');
    story.removeItem("kero.session.v1");
    expect(story.getItem("kero.session.v1")).toBeNull();
    expect(sandbox.getItem("kero.session.v1")).toBe("sandbox-session");
  });

  it("keeps layout, apparatus, progress, stock, and briefing mode-scoped", () => {
    const { repository } = ready();
    const story = new AppSaveModeStorage(repository, "story");
    const sandbox = new AppSaveModeStorage(repository, "sandbox");
    for (const key of [MODE_LAYOUT_KEY, MODE_APPARATUS_KEY, "kero.missions.done.v1", "kero.story-stock.v1", "kero.case.contaminated-sample.briefed.v1"]) {
      story.setItem(key, `story:${key}`);
      sandbox.setItem(key, `sandbox:${key}`);
      expect(story.getItem(key)).toBe(`story:${key}`);
      expect(sandbox.getItem(key)).toBe(`sandbox:${key}`);
    }
  });

  it("stores shared profile and settings without changing either mode", () => {
    const { repository } = ready();
    const story = new AppSaveModeStorage(repository, "story");
    const sandbox = new AppSaveModeStorage(repository, "sandbox");
    story.setItem("kero.session.v1", "story");
    sandbox.setItem("kero.session.v1", "sandbox");
    expect(saveSharedProfile(repository, { version: 1, name: "Ada", createdAt: "2026-09-01" })).toEqual({ status: "saved" });
    expect(saveSharedSetting(repository, "theme", "contrast")).toEqual({ status: "saved" });
    expect(readSharedProfile(repository)).toEqual({ version: 1, name: "Ada", createdAt: "2026-09-01" });
    expect(readSharedSetting(repository, "theme")).toBe("contrast");
    expect(story.getItem("kero.session.v1")).toBe("story");
    expect(sandbox.getItem("kero.session.v1")).toBe("sandbox");
  });

  it("copies every supported legacy namespace without deleting source bytes", () => {
    const storage = new MemoryStorage();
    const legacy = {
      "kero.session.v1": "unscoped-session",
      "kero.codex.done.v1": '["unscoped"]',
      "kero.mode.story.kero.session.v1": "story-session",
      "kero.mode.sandbox.kero.session.v1": "scoped-sandbox-session",
      "kero.mode.story.kero.missions.done.v1": '["mission"]',
      [`${MODE_LAYOUT_KEY}.story`]: "story-layout",
      [MODE_LAYOUT_KEY]: "old-sandbox-layout",
      [`${MODE_APPARATUS_KEY}.sandbox`]: "sandbox-apparatus",
      [`${MODE_GUIDES_KEY}.story`]: "shown",
      "kerotakis.lab-profile.v1": JSON.stringify({ version: 1, name: "Ada", createdAt: "2026-09-01" }),
      "kerotakis.theme": "dark",
      "kerotakis.locale": "de",
    };
    for (const [key, value] of Object.entries(legacy)) storage.values.set(key, value);
    const result = bootstrapAppSave(storage);
    expect(result.status).toBe("ready");
    if (result.status !== "ready") return;
    expect(result.source).toBe("migrated");
    const story = new AppSaveModeStorage(result.repository, "story");
    const sandbox = new AppSaveModeStorage(result.repository, "sandbox");
    expect(story.getItem("kero.session.v1")).toBe("story-session");
    expect(sandbox.getItem("kero.session.v1")).toBe("scoped-sandbox-session");
    expect(sandbox.getItem("kero.codex.done.v1")).toBe('["unscoped"]');
    expect(story.getItem("kero.missions.done.v1")).toBe('["mission"]');
    expect(story.getItem(MODE_LAYOUT_KEY)).toBe("story-layout");
    expect(sandbox.getItem(MODE_LAYOUT_KEY)).toBe("old-sandbox-layout");
    expect(sandbox.getItem(MODE_APPARATUS_KEY)).toBe("sandbox-apparatus");
    expect(story.getItem(MODE_GUIDES_KEY)).toBe("shown");
    expect(readSharedSetting(result.repository, "theme")).toBe("dark");
    expect(readSharedSetting(result.repository, "locale")).toBe("de");
    expect(readSharedProfile(result.repository)?.name).toBe("Ada");
    for (const [key, value] of Object.entries(legacy)) expect(storage.values.get(key)).toBe(value);
  });

  it("is idempotent and never lets changed legacy bytes replace AppSave", () => {
    const storage = new MemoryStorage();
    storage.values.set("kero.session.v1", "first");
    const first = bootstrapAppSave(storage);
    expect(first.status).toBe("ready");
    storage.values.set("kero.session.v1", "changed");
    const second = bootstrapAppSave(storage);
    expect(second.status).toBe("ready");
    if (second.status !== "ready") return;
    expect(second.source).toBe("existing");
    expect(new AppSaveModeStorage(second.repository, "sandbox").getItem("kero.session.v1")).toBe("first");
  });

  it("preserves corrupt forensic slots and refuses all mutation", () => {
    const storage = new MemoryStorage();
    const repository = createAppSaveRepository(storage);
    storage.values.set(repository.key("current"), "{corrupt");
    const before = new Map(storage.values);
    const result = bootstrapAppSave(storage);
    expect(result.status).toBe("corrupt");
    expect(storage.values).toEqual(before);
    const adapter = new AppSaveModeStorage(repository, "story");
    expect(adapter.getItem("kero.session.v1")).toBeNull();
    expect(() => adapter.setItem("kero.session.v1", "new")).toThrow("AppSave is corrupt");
    expect(storage.values).toEqual(before);
  });

  it("reads recovered LKG without repairing on load", () => {
    const storage = new MemoryStorage();
    const repository = createAppSaveRepository(storage);
    const save = emptyAppSave();
    const initialized = repository.save(save);
    expect(initialized.status).toBe("saved");
    const currentKey = repository.key("current");
    const lkgKey = repository.key("last-known-good");
    storage.values.set(lkgKey, storage.values.get(currentKey)!);
    storage.values.set(currentKey, "corrupt evidence");
    const before = new Map(storage.values);
    const result = bootstrapAppSave(storage);
    expect(result.status).toBe("ready");
    expect(result.status === "ready" && result.source).toBe("recovered");
    expect(storage.values).toEqual(before);
    const recoveredStory = new AppSaveModeStorage(repository, "story");
    expect(() => recoveredStory.setItem("kero.session.v1", "new")).toThrow("recovery evidence is read-only");
    expect(saveSharedSetting(repository, "theme", "dark")).toEqual({ status: "recovery-read-only" });
    expect(cloneStoryBenchToSandbox(repository)).toEqual({ status: "recovery-read-only" });
    expect(storage.values).toEqual(before);
  });

  it("refuses to bootstrap when reads or writes are unavailable", () => {
    const readBlocked = new MemoryStorage();
    readBlocked.failReads = true;
    expect(bootstrapAppSave(readBlocked).status).toBe("unavailable");
    const writeBlocked = new MemoryStorage();
    writeBlocked.failWrites = true;
    expect(bootstrapAppSave(writeBlocked).status).toBe("unavailable");
    expect(writeBlocked.values.size).toBe(0);
  });

  it("writes no AppSave when any individual legacy probe fails", () => {
    // Three repository-slot probes precede 28 one-shot legacy probes.
    for (let failReadAt = 4; failReadAt <= 31; failReadAt += 1) {
      const storage = new MemoryStorage();
      storage.failReadAt = failReadAt;
      expect(bootstrapAppSave(storage).status, `read ${failReadAt}`).toBe("unavailable");
      expect([...storage.values.keys()].filter((key) => key.startsWith("kero.app-save.v1"))).toEqual([]);
    }
  });

  it("treats an unknown internal namespace payload as read-only evidence", () => {
    const storage = new MemoryStorage();
    const repository = createAppSaveRepository(storage);
    const save = emptyAppSave();
    save.story.session = { unexpected: true };
    expect(repository.save(save)).toEqual({ status: "saved" });
    const before = new Map(storage.values);
    const errors: string[] = [];
    const story = new AppSaveModeStorage(repository, "story", (message) => errors.push(message));
    expect(story.getItem("kero.session.v1")).toBeNull();
    expect(() => story.setItem("kero.session.v1", "new")).toThrow("mode session payload is invalid");
    expect(errors).toEqual(["AppSave mode session payload is invalid"]);
    expect(storage.values).toEqual(before);
  });

  it("clones only the Story bench whitelist into Sandbox", () => {
    const { repository } = ready();
    const story = new AppSaveModeStorage(repository, "story");
    const sandbox = new AppSaveModeStorage(repository, "sandbox");
    story.setItem("kero.session.v1", "story-session");
    story.setItem(MODE_LAYOUT_KEY, "story-layout");
    story.setItem(MODE_APPARATUS_KEY, "story-apparatus");
    story.setItem("kero.missions.done.v1", '["story-mission"]');
    sandbox.setItem("kero.session.v1", "sandbox-session");
    sandbox.setItem(MODE_LAYOUT_KEY, "sandbox-layout");
    sandbox.setItem(MODE_APPARATUS_KEY, "sandbox-apparatus");
    sandbox.setItem("kero.missions.done.v1", '["sandbox-mission"]');
    sandbox.setItem("kero.story-stock.v1", '{"water":3}');
    expect(cloneStoryBenchToSandbox(repository)).toEqual({ status: "saved" });
    expect(sandbox.getItem("kero.session.v1")).toBe("story-session");
    expect(sandbox.getItem(MODE_LAYOUT_KEY)).toBe("story-layout");
    expect(sandbox.getItem(MODE_APPARATUS_KEY)).toBe("story-apparatus");
    expect(sandbox.getItem("kero.missions.done.v1")).toBe('["sandbox-mission"]');
    expect(sandbox.getItem("kero.story-stock.v1")).toBe('{"water":3}');
    expect(story.getItem("kero.missions.done.v1")).toBe('["story-mission"]');
  });

  it("an exact clone removes absent Story bench fields but keeps preferences", () => {
    const { repository } = ready();
    const sandbox = new AppSaveModeStorage(repository, "sandbox");
    sandbox.setItem("kero.session.v1", "old");
    sandbox.setItem(MODE_LAYOUT_KEY, "old-layout");
    sandbox.setItem(MODE_APPARATUS_KEY, "old-apparatus");
    sandbox.setItem(MODE_GUIDES_KEY, "shown");
    expect(cloneStoryBenchToSandbox(repository)).toEqual({ status: "saved" });
    expect(sandbox.getItem("kero.session.v1")).toBeNull();
    expect(sandbox.getItem(MODE_LAYOUT_KEY)).toBeNull();
    expect(sandbox.getItem(MODE_APPARATUS_KEY)).toBeNull();
    expect(sandbox.getItem(MODE_GUIDES_KEY)).toBe("shown");
  });
});
