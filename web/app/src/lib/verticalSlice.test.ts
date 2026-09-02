/**
 * WORLD-008 — the vertical slice, client half.
 *
 * The engine half lives in `crates/kerotakis-codex/tests/vertical_slice.rs`:
 * three concurrent leads, a sealed unknown, two treatment traces, one
 * derived unlock. What only the client can answer is what happens to a
 * learner's SAVE across the slice — that Story and Sandbox stay apart, and
 * that an old install migrates to exactly one thing, byte for byte.
 */
import { describe, expect, it } from "vitest";
import { encodeAppSave, decodeAppSave, emptyAppSave, migrateLegacySession } from "./appSave";
import { caseAwardedTools, contaminatedSampleComplete } from "./storyChapter";

/** One shipped install from before app saves existed. Pinned literally: a
 * golden is only a golden if it is the same bytes every time. */
const LEGACY_SESSION = JSON.stringify({
  log: ["add v1 water 100mL", "add v1 NaCl 0.01mol", "add v1 AgNO3 0.01mol"],
  position: 3,
  register: "lv2",
});

describe("save migration is deterministic (WORLD-008)", () => {
  it("migrates the same legacy install to the same bytes every time", () => {
    const once = migrateLegacySession(LEGACY_SESSION);
    const twice = migrateLegacySession(LEGACY_SESSION);
    expect(once.ok && twice.ok).toBe(true);
    if (!once.ok || !twice.ok) return;
    // Byte equality, not deep equality: a migration that reorders keys
    // produces a different save file for the same input, and "the same
    // install migrated twice" must be indistinguishable.
    const a = encodeAppSave(once.value);
    const b = encodeAppSave(twice.value);
    expect(a.ok && b.ok).toBe(true);
    expect(a).toEqual(b);
  });

  it("puts a legacy bench in Sandbox and leaves Story empty", () => {
    const result = migrateLegacySession(LEGACY_SESSION);
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.sandbox.session).toEqual(JSON.parse(LEGACY_SESSION));
    // The slice's whole premise: a returning learner's old experiments are
    // a sandbox, and their Story starts unspent.
    expect(result.value.story).toEqual(emptyAppSave().story);
  });

  it("round-trips the migrated save without drift", () => {
    const result = migrateLegacySession(LEGACY_SESSION);
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    const encoded = encodeAppSave(result.value);
    expect(encoded.ok).toBe(true);
    if (!encoded.ok) return;
    const decoded = decodeAppSave(encoded.value);
    expect(decoded.ok).toBe(true);
    if (!decoded.ok) return;
    expect(encodeAppSave(decoded.value)).toEqual(encoded);
  });

  it("fails closed rather than migrating half an install", () => {
    expect(migrateLegacySession("{not json").ok).toBe(false);
    // And a failure leaves nothing behind to be mistaken for a save.
    const failed = migrateLegacySession("{not json");
    expect("value" in failed).toBe(false);
  });
});

describe("the case's permanent unlock, across the slice", () => {
  const CORE = ["silver-and-salt", "first-warmth", "one-thing-at-a-time"];

  it("is not granted until every core lead is secured", () => {
    expect(caseAwardedTools(new Set())).toEqual([]);
    expect(caseAwardedTools(new Set(CORE.slice(0, 2)))).toEqual([]);
    expect(contaminatedSampleComplete(new Set(CORE))).toBe(true);
    expect(caseAwardedTools(new Set(CORE))).toEqual(["measure:uvvis"]);
  });

  it("is derived, so replaying the case cannot grant it twice", () => {
    // The property that makes the unlock safe under a retried commit: it is
    // a function of the leads, not a ledger that accumulates.
    const completed = new Set([...CORE, "never-mix"]);
    expect(caseAwardedTools(completed)).toEqual(caseAwardedTools(completed));
    expect(caseAwardedTools(completed)).toHaveLength(1);
  });
});
