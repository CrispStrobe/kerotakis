/** A persistence-neutral, versioned save boundary. Storage promotion is owned by
 * the host; this module only validates and serializes complete envelopes. */

export const APP_SAVE_VERSION = 1 as const;
export const APP_SAVE_NAMESPACE_VERSION = 1 as const;
export const APP_SAVE_MAX_BYTES = 1024 * 1024;
export const APP_SAVE_MAX_DEPTH = 64;
export const APP_SAVE_MAX_NODES = 50_000;

export type JsonPrimitive = string | number | boolean | null;
export type JsonValue = JsonPrimitive | JsonValue[] | { [key: string]: JsonValue };
export type JsonObject = { [key: string]: JsonValue };

export interface AppSave {
  version: typeof APP_SAVE_VERSION;
  profile: JsonObject;
  settings: JsonObject;
  story: { version: typeof APP_SAVE_NAMESPACE_VERSION; session: JsonValue | null };
  sandbox: { version: typeof APP_SAVE_NAMESPACE_VERSION; session: JsonValue | null };
}

export type AppSaveError =
  | "too-large"
  | "invalid-json"
  | "invalid-shape"
  | "complexity-limit"
  | "unsupported-version";

export type AppSaveResult =
  | { ok: true; value: AppSave }
  | { ok: false; error: AppSaveError };

export type EncodedAppSaveResult =
  | { ok: true; value: string }
  | { ok: false; error: AppSaveError };

const encoder = new TextEncoder();

function byteLength(value: string): number {
  return encoder.encode(value).byteLength;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function hasExactKeys(value: Record<string, unknown>, keys: readonly string[]): boolean {
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  return actual.length === expected.length && actual.every((key, index) => key === expected[index]);
}

interface ValidationBudget {
  nodes: number;
  exceeded: boolean;
}

function isJsonValue(
  value: unknown,
  ancestors = new Set<object>(),
  budget: ValidationBudget = { nodes: 0, exceeded: false },
  depth = 0,
): value is JsonValue {
  budget.nodes += 1;
  if (depth > APP_SAVE_MAX_DEPTH || budget.nodes > APP_SAVE_MAX_NODES) {
    budget.exceeded = true;
    return false;
  }
  if (value === null || typeof value === "string" || typeof value === "boolean") return true;
  if (typeof value === "number") return Number.isFinite(value);
  if (typeof value !== "object") return false;
  if (ancestors.has(value)) return false;

  ancestors.add(value);
  const valid = Array.isArray(value)
    ? value.every((item) => isJsonValue(item, ancestors, budget, depth + 1))
    : isRecord(value) && Object.keys(value).every((key) => isJsonValue(value[key], ancestors, budget, depth + 1));
  ancestors.delete(value);
  return valid;
}

function isNamespace(value: unknown, budget: ValidationBudget): value is AppSave["story"] {
  return isRecord(value)
    && hasExactKeys(value, ["version", "session"])
    && value.version === APP_SAVE_NAMESPACE_VERSION
    && isJsonValue(value.session, new Set(), budget);
}

function validate(value: unknown): AppSaveResult {
  if (!isRecord(value) || !Object.hasOwn(value, "version")) {
    return { ok: false, error: "invalid-shape" };
  }
  if (value.version !== APP_SAVE_VERSION) {
    return { ok: false, error: "unsupported-version" };
  }
  for (const namespace of [value.story, value.sandbox]) {
    if (isRecord(namespace) && Object.hasOwn(namespace, "version") && namespace.version !== APP_SAVE_NAMESPACE_VERSION) {
      return { ok: false, error: "unsupported-version" };
    }
  }
  const budget: ValidationBudget = { nodes: 0, exceeded: false };
  const invalid = (
    !hasExactKeys(value, ["version", "profile", "settings", "story", "sandbox"])
    || !isRecord(value.profile)
    || !isJsonValue(value.profile, new Set(), budget)
    || !isRecord(value.settings)
    || !isJsonValue(value.settings, new Set(), budget)
    || !isNamespace(value.story, budget)
    || !isNamespace(value.sandbox, budget)
  );
  if (invalid) {
    if (budget.exceeded) return { ok: false, error: "complexity-limit" };
    return { ok: false, error: "invalid-shape" };
  }
  return { ok: true, value: value as unknown as AppSave };
}

/** Decode a complete save. No usable partial state is returned on failure. */
export function decodeAppSave(raw: string): AppSaveResult {
  if (byteLength(raw) > APP_SAVE_MAX_BYTES) return { ok: false, error: "too-large" };
  let value: unknown;
  try {
    value = JSON.parse(raw);
  } catch {
    return { ok: false, error: "invalid-json" };
  }
  return validate(value);
}

function canonicalize(value: JsonValue): JsonValue {
  if (Array.isArray(value)) return value.map(canonicalize);
  if (value !== null && typeof value === "object") {
    const result: JsonObject = Object.create(null) as JsonObject;
    for (const key of Object.keys(value).sort()) result[key] = canonicalize(value[key]!);
    return result;
  }
  return value;
}

/** Validate and deterministically encode a save without mutating the caller's object. */
export function encodeAppSave(save: unknown): EncodedAppSaveResult {
  const checked = validate(save);
  if (!checked.ok) return checked;
  const value = JSON.stringify(canonicalize(checked.value as unknown as JsonValue));
  if (byteLength(value) > APP_SAVE_MAX_BYTES) return { ok: false, error: "too-large" };
  return { ok: true, value };
}

export function emptyAppSave(): AppSave {
  return {
    version: APP_SAVE_VERSION,
    profile: {},
    settings: {},
    story: { version: APP_SAVE_NAMESPACE_VERSION, session: null },
    sandbox: { version: APP_SAVE_NAMESPACE_VERSION, session: null },
  };
}

/** Import the old `kero.session.v1` value into Sandbox. This pure operation
 * cannot remove or overwrite the legacy storage key; promotion happens later. */
export function migrateLegacySession(raw: string, base: AppSave = emptyAppSave()): AppSaveResult {
  if (byteLength(raw) > APP_SAVE_MAX_BYTES) return { ok: false, error: "too-large" };
  let session: unknown;
  try {
    session = JSON.parse(raw);
  } catch {
    return { ok: false, error: "invalid-json" };
  }
  const budget: ValidationBudget = { nodes: 0, exceeded: false };
  if (!isJsonValue(session, new Set(), budget)) {
    return { ok: false, error: budget.exceeded ? "complexity-limit" : "invalid-shape" };
  }

  const encodedBase = encodeAppSave(base);
  if (!encodedBase.ok) return encodedBase;
  const clonedBase = decodeAppSave(encodedBase.value);
  if (!clonedBase.ok) return clonedBase;
  const migrated: AppSave = {
    ...clonedBase.value,
    sandbox: { version: APP_SAVE_NAMESPACE_VERSION, session },
  };
  const encoded = encodeAppSave(migrated);
  return encoded.ok ? decodeAppSave(encoded.value) : encoded;
}
