import { describe, expect, it } from "vitest";
import {
  CONSOLE_KEY,
  CONTAMINATED_SAMPLE_BRIEFED_KEY,
  ModeStorage,
  loadLabProfile,
  readConsoleVisible,
  readLabMode,
  saveLabProfile,
  writeConsoleVisible,
  writeLabMode,
} from "./worldState";

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
    story.setItem(CONTAMINATED_SAMPLE_BRIEFED_KEY, "yes");
    expect(story.getItem("kero.session.v1")).toBe("story-save");
    expect(sandbox.getItem("kero.session.v1")).toBe("sandbox-save");
    expect(story.getItem(CONTAMINATED_SAMPLE_BRIEFED_KEY)).toBe("yes");
    expect(sandbox.getItem(CONTAMINATED_SAMPLE_BRIEFED_KEY)).toBeNull();
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

  /** The shell opens on the bench, in the laboratory last stood in. There
   * is no first-run question to answer, so an empty store has to be a
   * complete answer on its own — Sandbox, and no world map. */
  it("answers Sandbox for a reader who has never chosen a laboratory", () => {
    expect(readLabMode(new MemoryStorage())).toBe("sandbox");
    expect(readLabMode(null)).toBe("sandbox");
    const returning = new MemoryStorage();
    writeLabMode(returning, "story");
    expect(readLabMode(returning)).toBe("story");
  });

  it("remembers the command line across visits, and hides it by default", () => {
    const root = new MemoryStorage();
    expect(readConsoleVisible(root)).toBe(false);
    writeConsoleVisible(root, true);
    expect(root.getItem(CONSOLE_KEY)).toBe("shown");
    expect(readConsoleVisible(root)).toBe(true);
    writeConsoleVisible(root, false);
    expect(readConsoleVisible(root)).toBe(false);
    // A stored value from some other version is not a reason to open it.
    root.setItem(CONSOLE_KEY, "yes");
    expect(readConsoleVisible(root)).toBe(false);
  });

  it("keeps the console preference outside both laboratories", () => {
    // It is a fact about how this reader drives the bench, not about a
    // save: switching mode must not silently close the console.
    const root = new MemoryStorage();
    writeConsoleVisible(root, true);
    expect(new ModeStorage(root, "story").getItem(CONSOLE_KEY)).toBeNull();
    expect(new ModeStorage(root, "sandbox").getItem(CONSOLE_KEY)).toBeNull();
    expect(readConsoleVisible(root)).toBe(true);
  });

  it("survives storage that throws on every access", () => {
    const blocked = {
      getItem() { throw new Error("blocked"); },
      setItem() { throw new Error("blocked"); },
      removeItem() { throw new Error("blocked"); },
    };
    expect(readConsoleVisible(blocked)).toBe(false);
    expect(() => writeConsoleVisible(blocked, true)).not.toThrow();
    expect(readLabMode(blocked)).toBe("sandbox");
  });
});
