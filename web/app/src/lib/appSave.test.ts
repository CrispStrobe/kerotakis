import { describe, expect, it } from "vitest";
import {
  APP_SAVE_MAX_BYTES,
  APP_SAVE_MAX_DEPTH,
  decodeAppSave,
  emptyAppSave,
  encodeAppSave,
  migrateLegacySession,
  type AppSave,
} from "./appSave";

function populatedSave(): AppSave {
  return {
    version: 1,
    profile: { name: "Ada", flags: [true, null] },
    settings: { theme: "dark", accessibility: { motion: "reduced" } },
    story: { version: 1, session: { log: ["story"], position: 1 } },
    sandbox: { version: 1, session: { log: ["sandbox"], position: 1 } },
  };
}

describe("AppSave envelope", () => {
  it("round-trips independent namespaces and shared fields", () => {
    const encoded = encodeAppSave(populatedSave());
    expect(encoded.ok).toBe(true);
    if (!encoded.ok) return;
    expect(decodeAppSave(encoded.value)).toEqual({ ok: true, value: populatedSave() });
  });

  it("encodes deterministically by recursively sorting object keys", () => {
    const first = populatedSave();
    const second: AppSave = {
      sandbox: { version: 1, session: { position: 1, log: ["sandbox"] } },
      story: { version: 1, session: { position: 1, log: ["story"] } },
      settings: { accessibility: { motion: "reduced" }, theme: "dark" },
      profile: { flags: [true, null], name: "Ada" },
      version: 1,
    };
    expect(encodeAppSave(first)).toEqual(encodeAppSave(second));
  });

  it.each([
    ["corrupt JSON", "{oops", "invalid-json"],
    ["a future version", JSON.stringify({ ...emptyAppSave(), version: 2 }), "unsupported-version"],
    ["a future namespace", JSON.stringify({ ...emptyAppSave(), story: { version: 2, session: null } }), "unsupported-version"],
    ["a missing namespace", JSON.stringify({ ...emptyAppSave(), story: undefined }), "invalid-shape"],
    ["an extra envelope field", JSON.stringify({ ...emptyAppSave(), surprise: true }), "invalid-shape"],
    ["an unversioned namespace", JSON.stringify({ ...emptyAppSave(), story: { session: null } }), "invalid-shape"],
    ["an extra namespace field", JSON.stringify({ ...emptyAppSave(), story: { version: 1, session: null, progress: {} } }), "invalid-shape"],
  ])("fails closed for %s", (_label, raw, error) => {
    expect(decodeAppSave(raw)).toEqual({ ok: false, error });
  });

  it("rejects oversized UTF-8 input on decode and encode", () => {
    expect(decodeAppSave(` ${"x".repeat(APP_SAVE_MAX_BYTES)}`)).toEqual({ ok: false, error: "too-large" });
    const save = emptyAppSave();
    save.sandbox.session = "é".repeat(APP_SAVE_MAX_BYTES / 2);
    expect(encodeAppSave(save)).toEqual({ ok: false, error: "too-large" });
  });

  it("rejects values JSON would silently coerce and cyclic inputs", () => {
    const invalid = emptyAppSave() as unknown as Record<string, unknown>;
    invalid.profile = { omitted: undefined };
    expect(encodeAppSave(invalid)).toEqual({ ok: false, error: "invalid-shape" });

    const cyclic: Record<string, unknown> = {};
    cyclic.self = cyclic;
    invalid.profile = cyclic;
    expect(encodeAppSave(invalid)).toEqual({ ok: false, error: "invalid-shape" });
  });

  it("rejects adversarial depth without overflowing the validator", () => {
    let nested: unknown = null;
    for (let index = 0; index < APP_SAVE_MAX_DEPTH + 10; index += 1) nested = [nested];
    const save = emptyAppSave();
    save.sandbox.session = nested as never;
    expect(encodeAppSave(save)).toEqual({ ok: false, error: "complexity-limit" });
    expect(decodeAppSave(JSON.stringify(save))).toEqual({ ok: false, error: "complexity-limit" });
    expect(migrateLegacySession(JSON.stringify(nested))).toEqual({ ok: false, error: "complexity-limit" });
  });

  it("bounds the total JSON node count", () => {
    const save = emptyAppSave();
    save.sandbox.session = Array.from({ length: 50_001 }, () => null);
    expect(encodeAppSave(save)).toEqual({ ok: false, error: "complexity-limit" });
  });

  it("is byte-stable across encode, decode, and encode", () => {
    const first = encodeAppSave(populatedSave());
    expect(first.ok).toBe(true);
    if (!first.ok) return;
    const decoded = decodeAppSave(first.value);
    expect(decoded.ok).toBe(true);
    if (!decoded.ok) return;
    expect(encodeAppSave(decoded.value)).toEqual(first);
  });

  it("migrates a legacy session only into Sandbox and leaves the base untouched", () => {
    const base = populatedSave();
    const before = structuredClone(base);
    const legacy = JSON.stringify({ log: ["add v1 water 100mL"], position: 1, register: "lv1" });
    const result = migrateLegacySession(legacy, base);
    expect(result).toEqual({
      ok: true,
      value: {
        ...before,
        sandbox: { version: 1, session: JSON.parse(legacy) },
      },
    });
    expect(base).toEqual(before);
    expect(result.ok && result.value.story).toEqual(before.story);
  });

  it("fails closed on corrupt legacy data and never returns a partial save", () => {
    expect(migrateLegacySession("{not json")).toEqual({ ok: false, error: "invalid-json" });
    expect(migrateLegacySession("{}", { ...emptyAppSave(), version: 2 } as never)).toEqual({
      ok: false,
      error: "unsupported-version",
    });
    const oversizedBase = emptyAppSave();
    oversizedBase.story.session = "x".repeat(APP_SAVE_MAX_BYTES);
    expect(migrateLegacySession("{}", oversizedBase)).toEqual({ ok: false, error: "too-large" });
  });
});
