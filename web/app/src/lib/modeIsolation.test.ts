import { describe, expect, it } from "vitest";
import { decodeAppSave, emptyAppSave, encodeAppSave, type AppSave, type JsonValue } from "./appSave";
import { AtomicAppSaveStorage, type AppSaveCodec, type KeyValueStorage } from "./appSaveStorage";
import * as repositoryApi from "./appSaveRepository";
import {
  AppSaveModeStorage,
  MODE_APPARATUS_KEY,
  MODE_GUIDES_KEY,
  MODE_LAYOUT_KEY,
  bootstrapAppSave,
  cloneStoryBenchToSandbox,
  readSharedSetting,
  saveSharedSetting,
} from "./appSaveRepository";

type Mode = "story" | "sandbox";
type History = { commands: string[]; position: number };

const codec: AppSaveCodec<AppSave> = {
  encode(value) {
    const encoded = encodeAppSave(value);
    if (!encoded.ok) throw new Error(encoded.error);
    return encoded.value;
  },
  decode(raw) {
    const decoded = decodeAppSave(raw);
    if (!decoded.ok) throw new Error(decoded.error);
    return decoded.value;
  },
};

class CutStorage implements KeyValueStorage {
  values = new Map<string, string>();
  mutations = 0;
  reads = 0;
  failMutationAt: number | null = null;
  failReadAt: number | null = null;
  getItem(key: string) {
    this.reads += 1;
    if (this.reads === this.failReadAt) throw new Error(`read cut ${this.reads}`);
    return this.values.get(key) ?? null;
  }
  setItem(key: string, value: string) {
    this.mutations += 1;
    if (this.mutations === this.failMutationAt) throw new Error(`mutation cut ${this.mutations}`);
    this.values.set(key, value);
  }
  removeItem(key: string) {
    this.mutations += 1;
    if (this.mutations === this.failMutationAt) throw new Error(`mutation cut ${this.mutations}`);
    this.values.delete(key);
  }
}

function history(save: AppSave, mode: Mode): History {
  return save[mode].session as unknown as History;
}

function replaceHistory(save: AppSave, mode: Mode, next: History): AppSave {
  return {
    ...save,
    [mode]: { ...save[mode], session: next as unknown as JsonValue },
  };
}

function canonical(value: unknown): string {
  const sorted = (input: unknown): unknown => {
    if (Array.isArray(input)) return input.map(sorted);
    if (input !== null && typeof input === "object") {
      return Object.fromEntries(Object.entries(input).sort(([a], [b]) => a.localeCompare(b)).map(([key, item]) => [key, sorted(item)]));
    }
    return input;
  };
  return JSON.stringify(sorted(value));
}

function seeded(): AppSave {
  return {
    ...emptyAppSave(),
    profile: { name: "Shared laboratory" },
    settings: { locale: "de", theme: "contrast" },
    story: { version: 1, session: { commands: ["add v1 water 100mL", "add v1 NaCl 1g"], position: 2 } },
    sandbox: { version: 1, session: { commands: ["add v1 water 100mL", "add v1 AgNO3 1g"], position: 2 } },
  };
}

function loaded(repository: AtomicAppSaveStorage<AppSave>): AppSave {
  const result = repository.load();
  if (result.status !== "loaded") throw new Error(`wanted loaded, got ${result.status}`);
  return result.value;
}

describe("WORLD-002 envelope isolation", () => {
  it("keeps divergent undo, redo, reset and reopen histories independent", () => {
    const storage = new CutStorage();
    const repository = new AtomicAppSaveStorage(storage, codec);
    repository.save(seeded());
    const sandboxBytes = canonical(loaded(repository).sandbox);
    const sharedBytes = canonical({ profile: loaded(repository).profile, settings: loaded(repository).settings });

    let save = replaceHistory(loaded(repository), "story", { ...history(loaded(repository), "story"), position: 1 });
    repository.save(save); // undo
    expect(history(loaded(repository), "story").position).toBe(1);
    save = replaceHistory(loaded(repository), "story", { ...history(loaded(repository), "story"), position: 2 });
    repository.save(save); // redo
    save = replaceHistory(loaded(repository), "story", { commands: [], position: 0 });
    repository.save(save); // reset and reopen

    const reopened = new AtomicAppSaveStorage(storage, codec);
    expect(history(loaded(reopened), "story")).toEqual({ commands: [], position: 0 });
    expect(canonical(loaded(reopened).sandbox)).toBe(sandboxBytes);
    expect(canonical({ profile: loaded(reopened).profile, settings: loaded(reopened).settings })).toBe(sharedBytes);

    const storyBytes = canonical(loaded(reopened).story);
    repository.save(replaceHistory(loaded(reopened), "sandbox", { commands: [], position: 0 }));
    expect(canonical(loaded(reopened).story)).toBe(storyBytes);
  });

  it("survives twenty alternating writes without stale-envelope cross-mutation", () => {
    const storage = new CutStorage();
    const repository = new AtomicAppSaveStorage(storage, codec);
    repository.save(seeded());
    for (let index = 0; index < 20; index += 1) {
      const mode: Mode = index % 2 === 0 ? "story" : "sandbox";
      const other: Mode = mode === "story" ? "sandbox" : "story";
      const before = loaded(repository);
      const otherBytes = canonical(before[other]);
      const sharedBytes = canonical({ profile: before.profile, settings: before.settings });
      const current = history(before, mode);
      const next = replaceHistory(before, mode, {
        commands: [...current.commands, `${mode}-${index}`],
        position: current.position + 1,
      });
      expect(repository.save(next)).toEqual({ status: "saved" });
      const after = loaded(repository);
      expect(canonical(after[other])).toBe(otherBytes);
      expect(canonical({ profile: after.profile, settings: after.settings })).toBe(sharedBytes);
    }
  });

  it("keeps the untouched namespace whole across every mutation cut", () => {
    const operations = ["write-staging", "write-last-known-good", "write-current", "clear-staging"];
    for (const [index, operation] of operations.entries()) {
      const storage = new CutStorage();
      const repository = new AtomicAppSaveStorage(storage, codec);
      storage.values.set(repository.key("current"), codec.encode(seeded()));
      const before = loaded(repository);
      storage.reads = 0;
      storage.failMutationAt = index + 1;
      expect(repository.save(replaceHistory(before, "story", { commands: ["new"], position: 1 })))
        .toMatchObject({ status: "unavailable", operation });
      storage.failMutationAt = null;
      const recovered = loaded(repository);
      expect(canonical(recovered.sandbox)).toBe(canonical(before.sandbox));
      expect(canonical({ profile: recovered.profile, settings: recovered.settings }))
        .toBe(canonical({ profile: before.profile, settings: before.settings }));
      expect([canonical(before.story), canonical({ version: 1, session: { commands: ["new"], position: 1 } })])
        .toContain(canonical(recovered.story));
    }
  });

  it("keeps the untouched namespace whole across every save-time read cut", () => {
    const operations = ["read-staging", "read-current", "read-last-known-good", "read-current"];
    for (const [index, operation] of operations.entries()) {
      const storage = new CutStorage();
      const repository = new AtomicAppSaveStorage(storage, codec);
      storage.values.set(repository.key("current"), codec.encode(seeded()));
      const before = loaded(repository);
      storage.reads = 0;
      storage.failReadAt = index + 1;
      expect(repository.save(replaceHistory(before, "sandbox", { commands: ["new"], position: 1 })))
        .toMatchObject({ status: "unavailable", operation });
      storage.failReadAt = null;
      storage.reads = 0;
      const recovered = loaded(repository);
      expect(canonical(recovered.story)).toBe(canonical(before.story));
      expect(canonical({ profile: recovered.profile, settings: recovered.settings }))
        .toBe(canonical({ profile: before.profile, settings: before.settings }));
      expect([canonical(before.sandbox), canonical({ version: 1, session: { commands: ["new"], position: 1 } })])
        .toContain(canonical(recovered.sandbox));
    }
  });

  it("shares settings while leaving both histories byte-stable", () => {
    const storage = new CutStorage();
    const repository = new AtomicAppSaveStorage(storage, codec);
    repository.save(seeded());
    const before = loaded(repository);
    repository.save({ ...before, settings: { ...before.settings, locale: "en" } });
    const reopened = loaded(new AtomicAppSaveStorage(storage, codec));
    expect(reopened.settings.locale).toBe("en");
    expect(canonical(reopened.story)).toBe(canonical(before.story));
    expect(canonical(reopened.sandbox)).toBe(canonical(before.sandbox));
  });

  it("clones Story to Sandbox one way, exactly, then diverges independently", () => {
    const storage = new CutStorage();
    const bootstrapped = bootstrapAppSave(storage);
    if (bootstrapped.status !== "ready") throw new Error(`bootstrap ${bootstrapped.status}`);
    const { repository } = bootstrapped;
    const story = new AppSaveModeStorage(repository, "story");
    const sandbox = new AppSaveModeStorage(repository, "sandbox");
    story.setItem("kero.session.v1", "story-bench");
    story.setItem(MODE_LAYOUT_KEY, "story-layout");
    // Apparatus is deliberately absent in Story: an exact clone must delete
    // the stale destination value rather than retain a hybrid bench.
    sandbox.setItem("kero.session.v1", "old-sandbox");
    sandbox.setItem(MODE_LAYOUT_KEY, "old-layout");
    sandbox.setItem(MODE_APPARATUS_KEY, "stale-apparatus");
    sandbox.setItem(MODE_GUIDES_KEY, "shown");
    sandbox.setItem("kero.missions.done.v1", '["sandbox-progress"]');
    expect(saveSharedSetting(repository, "theme", "contrast")).toEqual({ status: "saved" });

    expect(cloneStoryBenchToSandbox(repository)).toEqual({ status: "saved" });
    expect(sandbox.getItem("kero.session.v1")).toBe("story-bench");
    expect(sandbox.getItem(MODE_LAYOUT_KEY)).toBe("story-layout");
    expect(sandbox.getItem(MODE_APPARATUS_KEY)).toBeNull();
    expect(sandbox.getItem(MODE_GUIDES_KEY)).toBe("shown");
    expect(sandbox.getItem("kero.missions.done.v1")).toBe('["sandbox-progress"]');
    expect(story.getItem("kero.session.v1")).toBe("story-bench");
    expect(readSharedSetting(repository, "theme")).toBe("contrast");

    sandbox.setItem("kero.session.v1", "sandbox-after-clone");
    expect(story.getItem("kero.session.v1")).toBe("story-bench");
    expect(sandbox.getItem("kero.session.v1")).toBe("sandbox-after-clone");
    expect(repositoryApi).not.toHaveProperty("cloneSandboxBenchToStory");
  });
});
