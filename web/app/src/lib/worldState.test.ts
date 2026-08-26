import { describe, expect, it } from "vitest";
import { ModeStorage, loadLabProfile, readLabMode, saveLabProfile, writeLabMode } from "./worldState";

class MemoryStorage {
  values = new Map<string, string>();
  getItem(key: string) { return this.values.get(key) ?? null; }
  setItem(key: string, value: string) { this.values.set(key, value); }
  removeItem(key: string) { this.values.delete(key); }
}

describe("world and mode persistence", () => {
  it("keeps Story and Sandbox session keys independent", () => {
    const root = new MemoryStorage();
    const story = new ModeStorage(root, "story");
    const sandbox = new ModeStorage(root, "sandbox");
    story.setItem("kero.session.v1", "story-save");
    sandbox.setItem("kero.session.v1", "sandbox-save");
    expect(story.getItem("kero.session.v1")).toBe("story-save");
    expect(sandbox.getItem("kero.session.v1")).toBe("sandbox-save");
    sandbox.removeItem("kero.session.v1");
    expect(story.getItem("kero.session.v1")).toBe("story-save");
  });

  it("copies a legacy bench into Sandbox without deleting it", () => {
    const root = new MemoryStorage();
    root.setItem("kero.session.v1", "legacy");
    expect(new ModeStorage(root, "sandbox").getItem("kero.session.v1")).toBe("legacy");
    expect(root.getItem("kero.session.v1")).toBe("legacy");
    expect(new ModeStorage(root, "story").getItem("kero.session.v1")).toBeNull();
    new ModeStorage(root, "sandbox").removeItem("kero.session.v1");
    expect(new ModeStorage(root, "sandbox").getItem("kero.session.v1")).toBeNull();
    expect(root.getItem("kero.session.v1")).toBe("legacy");
  });

  it("persists mode and a bounded lab identity", () => {
    const root = new MemoryStorage();
    expect(readLabMode(root)).toBe("sandbox");
    writeLabMode(root, "story");
    expect(readLabMode(root)).toBe("story");
    const profile = loadLabProfile(root, () => "2026-08-26T00:00:00.000Z");
    expect(profile.name).toBe("My Chemistry Lab");
    saveLabProfile(root, { ...profile, name: `  ${"x".repeat(80)}  ` });
    expect(loadLabProfile(root).name).toHaveLength(48);
  });
});
