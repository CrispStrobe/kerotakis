import { describe, expect, it } from "vitest";
import { AtomicAppSaveStorage, type AppSaveCodec, type KeyValueStorage } from "./appSaveStorage";

type Save = { version: 1; shared: { locale: string }; story: { commands: string[] }; sandbox: { commands: string[] } };
const codec: AppSaveCodec<Save> = {
  encode: JSON.stringify,
  decode(raw) {
    const value = JSON.parse(raw) as Partial<Save>;
    if (value.version !== 1 || typeof value.shared?.locale !== "string" || !Array.isArray(value.story?.commands) || !Array.isArray(value.sandbox?.commands)) throw new Error("invalid AppSave fixture");
    return value as Save;
  },
};
const saved = (marker: string): Save => ({ version: 1, shared: { locale: "de" }, story: { commands: [`story-${marker}`] }, sandbox: { commands: [`sandbox-${marker}`] } });

class MemoryStorage implements KeyValueStorage {
  values = new Map<string, string>();
  mutations = 0;
  failAt: number | null = null;
  reads = 0;
  failReadAt: number | null = null;
  getItem(key: string) {
    this.reads += 1;
    if (this.reads === this.failReadAt) throw new Error(`read cut ${this.reads}`);
    return this.values.get(key) ?? null;
  }
  private mutate(action: () => void) { this.mutations += 1; if (this.mutations === this.failAt) throw new Error(`storage cut ${this.mutations}`); action(); }
  setItem(key: string, value: string) { this.mutate(() => this.values.set(key, value)); }
  removeItem(key: string) { this.mutate(() => this.values.delete(key)); }
}

describe("atomic AppSave storage", () => {
  it("validates encoded data before touching storage", () => {
    const storage = new MemoryStorage();
    const repository = new AtomicAppSaveStorage(storage, { encode: () => "not-json", decode: codec.decode });
    expect(repository.save(saved("new"))).toMatchObject({ status: "invalid", operation: "validate" });
    expect(storage.mutations).toBe(0);
  });

  it("promotes validated staging and retains previous current as LKG", () => {
    const storage = new MemoryStorage();
    const repository = new AtomicAppSaveStorage(storage, codec);
    expect(repository.save(saved("old"))).toEqual({ status: "saved" });
    expect(repository.save(saved("new"))).toEqual({ status: "saved" });
    expect(repository.load()).toMatchObject({ status: "loaded", value: saved("new"), source: "current" });
    expect(codec.decode(storage.getItem(repository.key("last-known-good"))!)).toEqual(saved("old"));
    expect(storage.getItem(repository.key("staging"))).toBeNull();
  });

  it("returns an old or new whole envelope at every mutating cut point", () => {
    const operations = ["write-staging", "write-last-known-good", "write-current", "clear-staging"];
    for (const [index, operation] of operations.entries()) {
      const storage = new MemoryStorage();
      const repository = new AtomicAppSaveStorage(storage, codec);
      storage.values.set(repository.key("current"), codec.encode(saved("old")));
      storage.failAt = index + 1;
      expect(repository.save(saved("new"))).toMatchObject({ status: "unavailable", operation });
      const recovered = repository.load();
      expect(recovered.status).toBe("loaded");
      if (recovered.status === "loaded") expect([saved("old"), saved("new")]).toContainEqual(recovered.value);
    }
  });

  it("returns an old or new whole envelope at every save-time read cut", () => {
    const operations = ["read-staging", "read-current", "read-last-known-good", "read-current"];
    for (const [index, operation] of operations.entries()) {
      const storage = new MemoryStorage();
      const repository = new AtomicAppSaveStorage(storage, codec);
      storage.values.set(repository.key("current"), codec.encode(saved("old")));
      storage.failReadAt = index + 1;
      expect(repository.save(saved("new"))).toMatchObject({ status: "unavailable", operation });
      storage.failReadAt = null;
      storage.reads = 0;
      const recovered = repository.load();
      expect(recovered.status).toBe("loaded");
      if (recovered.status === "loaded") expect([saved("old"), saved("new")]).toContainEqual(recovered.value);
    }
  });

  it("recovers an interrupted first write from validated staging", () => {
    const storage = new MemoryStorage();
    const repository = new AtomicAppSaveStorage(storage, codec);
    storage.failAt = 2;
    expect(repository.save(saved("first"))).toMatchObject({ status: "unavailable", operation: "write-current" });
    expect(repository.load()).toMatchObject({ status: "loaded", value: saved("first"), source: "staging" });
  });

  it("recovers corrupt current from LKG without deleting forensic evidence", () => {
    const storage = new MemoryStorage();
    const repository = new AtomicAppSaveStorage(storage, codec);
    storage.values.set(repository.key("current"), "{corrupt current");
    storage.values.set(repository.key("last-known-good"), codec.encode(saved("safe")));
    const result = repository.load();
    expect(result).toMatchObject({ status: "loaded", value: saved("safe"), source: "last-known-good" });
    expect(result.evidence.current).toBe("{corrupt current");
    expect(storage.getItem(repository.key("current"))).toBe("{corrupt current");
  });

  it("never overwrites a valid LKG with corrupt current during a new save", () => {
    const storage = new MemoryStorage();
    const repository = new AtomicAppSaveStorage(storage, codec);
    storage.values.set(repository.key("current"), "corrupt current");
    storage.values.set(repository.key("last-known-good"), codec.encode(saved("safe")));
    expect(repository.save(saved("new"))).toEqual({ status: "saved" });
    expect(codec.decode(storage.getItem(repository.key("last-known-good"))!)).toEqual(saved("safe"));
    expect(repository.load()).toMatchObject({ status: "loaded", value: saved("new"), source: "current" });
  });

  it("reports empty, wholly corrupt and unavailable storage distinctly", () => {
    const storage = new MemoryStorage();
    const repository = new AtomicAppSaveStorage(storage, codec);
    expect(repository.load()).toMatchObject({ status: "empty" });
    storage.values.set(repository.key("current"), "bad");
    expect(repository.load()).toMatchObject({ status: "corrupt", evidence: { current: "bad" } });
    storage.reads = 0;
    storage.failReadAt = 1;
    expect(repository.load()).toMatchObject({ status: "unavailable", operation: "read" });
  });

  it("updates one mode without changing the other mode or shared settings", () => {
    const storage = new MemoryStorage();
    const repository = new AtomicAppSaveStorage(storage, codec);
    const initial = saved("initial");
    repository.save(initial);
    const storyUpdate: Save = { ...initial, story: { commands: ["story-new"] } };
    repository.save(storyUpdate);
    let loaded = repository.load();
    if (loaded.status !== "loaded") throw new Error("save did not load");
    expect(loaded.value.shared).toEqual(initial.shared);
    expect(loaded.value.sandbox).toEqual(initial.sandbox);
    repository.save({ ...loaded.value, sandbox: { commands: ["sandbox-new"] } });
    loaded = repository.load();
    if (loaded.status !== "loaded") throw new Error("save did not load");
    expect(loaded.value.shared).toEqual(initial.shared);
    expect(loaded.value.story).toEqual(storyUpdate.story);
    expect(loaded.value.sandbox.commands).toEqual(["sandbox-new"]);
  });
});
