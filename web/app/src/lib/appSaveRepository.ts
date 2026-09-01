import {
  decodeAppSave,
  emptyAppSave,
  encodeAppSave,
  type AppSave,
  type JsonObject,
  type JsonValue,
} from "./appSave";
import {
  AtomicAppSaveStorage,
  type AppSaveLoadResult,
  type AppSaveSaveResult,
  type KeyValueStorage,
} from "./appSaveStorage";
import type { LabMode } from "./worldState";

export const MODE_SESSION_STORAGE_VERSION = 1 as const;

interface ModeSessionStorage {
  storageVersion: typeof MODE_SESSION_STORAGE_VERSION;
  values: Record<string, string>;
}

export type AppSaveRepository = AtomicAppSaveStorage<AppSave>;

export type AppSaveBootstrapResult =
  | { status: "ready"; repository: AppSaveRepository; save: AppSave; source: "existing" | "recovered" | "migrated" | "empty" }
  | { status: "corrupt" | "unavailable"; repository: AppSaveRepository };

export type AppSaveMutationResult =
  | { status: "saved" }
  | { status: "unavailable" | "corrupt" | "recovery-read-only" | "invalid-session"; detail?: AppSaveSaveResult };

const SESSION_KEYS = [
  "kero.session.v1",
  "kero.codex.done.v1",
  "kero.missions.done.v1",
  "kero.story-stock.v1",
  "kero.case.contaminated-sample.briefed.v1",
] as const;

export const MODE_LAYOUT_KEY = "kerotakis.bench.layout.v1";
export const MODE_APPARATUS_KEY = "kero.apparatus-installation.v1";
export const MODE_GUIDES_KEY = "kero.bench-guides.v1";
export const MODE_ROOM_KEY = "kero.room.v1";
export const MODE_CABINET_PANEL_KEY = "kerotakis.panel.cabinet-collapsed.v1";
export const MODE_JOURNAL_PANEL_KEY = "kerotakis.panel.journal-collapsed.v1";

const MODE_UI_KEYS = [
  MODE_LAYOUT_KEY,
  MODE_APPARATUS_KEY,
  MODE_GUIDES_KEY,
  MODE_ROOM_KEY,
  MODE_CABINET_PANEL_KEY,
  MODE_JOURNAL_PANEL_KEY,
] as const;

const CLONE_KEYS = ["kero.session.v1", MODE_LAYOUT_KEY, MODE_APPARATUS_KEY] as const;

function unwrap<T>(result: { ok: true; value: T } | { ok: false; error: string }): T {
  if (!result.ok) throw new Error(result.error);
  return result.value;
}

export function createAppSaveRepository(storage: KeyValueStorage): AppSaveRepository {
  return new AtomicAppSaveStorage(storage, {
    encode: (save) => unwrap(encodeAppSave(save)),
    decode: (raw) => unwrap(decodeAppSave(raw)),
  });
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function readModeValues(save: AppSave, mode: LabMode): Record<string, string> | null {
  const session = save[mode].session;
  if (session === null) return {};
  if (
    !isRecord(session)
    || Object.keys(session).sort().join(",") !== "storageVersion,values"
    || session.storageVersion !== MODE_SESSION_STORAGE_VERSION
    || !isRecord(session.values)
    || !Object.values(session.values).every((value) => typeof value === "string")
  ) return null;
  return { ...(session.values as Record<string, string>) };
}

function writeModeValues(save: AppSave, mode: LabMode, values: Record<string, string>): AppSave {
  return {
    ...save,
    [mode]: {
      ...save[mode],
      session: { storageVersion: MODE_SESSION_STORAGE_VERSION, values: { ...values } },
    },
  };
}

function loadedSave(repository: AppSaveRepository): AppSaveLoadResult<AppSave> {
  return repository.load();
}

/** Existing Session and helpers keep their synchronous Web Storage contract;
 * each mutation is promoted as one complete AppSave envelope. */
export class AppSaveModeStorage {
  constructor(
    private readonly repository: AppSaveRepository,
    private readonly mode: LabMode,
    private readonly onError?: (message: string) => void,
  ) {}

  getItem(key: string): string | null {
    const loaded = loadedSave(this.repository);
    if (loaded.status !== "loaded") return null;
    return readModeValues(loaded.value, this.mode)?.[key] ?? null;
  }

  setItem(key: string, value: string): void {
    this.update((values) => ({ ...values, [key]: value }));
  }

  removeItem(key: string): void {
    this.update((values) => {
      const next = { ...values };
      delete next[key];
      return next;
    });
  }

  private update(change: (values: Record<string, string>) => Record<string, string>): void {
    const loaded = loadedSave(this.repository);
    if (loaded.status !== "loaded") this.fail(`AppSave is ${loaded.status}`);
    if (loaded.source !== "current") this.fail("AppSave recovery evidence is read-only");
    const values = readModeValues(loaded.value, this.mode);
    if (values === null) this.fail("AppSave mode session payload is invalid");
    const result = this.repository.save(writeModeValues(loaded.value, this.mode, change(values)));
    if (result.status !== "saved") this.fail(`AppSave mutation failed: ${result.status}`);
  }

  private fail(message: string): never {
    this.onError?.(message);
    throw new Error(message);
  }
}

function safeLegacyRead(storage: KeyValueStorage, key: string): string | null | undefined {
  try {
    return storage.getItem(key);
  } catch {
    return undefined;
  }
}

function validProfile(raw: string | null): JsonObject | null {
  if (raw === null) return null;
  try {
    const value: unknown = JSON.parse(raw);
    if (!isRecord(value) || value.version !== 1 || typeof value.name !== "string" || typeof value.createdAt !== "string") return null;
    return { version: 1, name: value.name, createdAt: value.createdAt };
  } catch {
    return null;
  }
}

/** Copy legacy bytes into the first global envelope. Legacy keys are evidence:
 * this function never removes or marks them, and never runs over an existing save. */
export function bootstrapAppSave(storage: KeyValueStorage): AppSaveBootstrapResult {
  const repository = createAppSaveRepository(storage);
  const existing = repository.load();
  if (existing.status === "loaded") {
    return {
      status: "ready",
      repository,
      save: existing.value,
      source: existing.source === "current" ? "existing" : "recovered",
    };
  }
  if (existing.status === "corrupt" || existing.status === "unavailable") {
    return { status: existing.status, repository };
  }

  let save = emptyAppSave();
  const values: Record<LabMode, Record<string, string>> = { story: {}, sandbox: {} };
  let foundLegacy = false;

  // Pre-mode installs were Sandbox. Scoped bytes take precedence below.
  for (const key of ["kero.session.v1", "kero.codex.done.v1"] as const) {
    const value = safeLegacyRead(storage, key);
    if (value === undefined) return { status: "unavailable", repository };
    if (value !== null) { values.sandbox[key] = value; foundLegacy = true; }
  }
  for (const mode of ["story", "sandbox"] as const) {
    for (const key of SESSION_KEYS) {
      const source = `kero.mode.${mode}.${key}`;
      const value = safeLegacyRead(storage, source);
      if (value === undefined) return { status: "unavailable", repository };
      if (value !== null) { values[mode][key] = value; foundLegacy = true; }
    }
    for (const key of MODE_UI_KEYS) {
      const value = safeLegacyRead(storage, `${key}.${mode}`);
      if (value === undefined) return { status: "unavailable", repository };
      if (value !== null) { values[mode][key] = value; foundLegacy = true; }
    }
  }
  if (!(MODE_LAYOUT_KEY in values.sandbox)) {
    const value = safeLegacyRead(storage, MODE_LAYOUT_KEY);
    if (value === undefined) return { status: "unavailable", repository };
    if (value !== null) { values.sandbox[MODE_LAYOUT_KEY] = value; foundLegacy = true; }
  }

  const profileRaw = safeLegacyRead(storage, "kerotakis.lab-profile.v1");
  const theme = safeLegacyRead(storage, "kerotakis.theme");
  const locale = safeLegacyRead(storage, "kerotakis.locale");
  if (profileRaw === undefined || theme === undefined || locale === undefined) return { status: "unavailable", repository };
  const profile = validProfile(profileRaw);
  if (profile) { save.profile = profile; foundLegacy = true; }
  if (theme === "light" || theme === "dark" || theme === "contrast") { save.settings.theme = theme; foundLegacy = true; }
  if (locale === "en" || locale === "de") { save.settings.locale = locale; foundLegacy = true; }

  save = writeModeValues(writeModeValues(save, "story", values.story), "sandbox", values.sandbox);
  const saved = repository.save(save);
  if (saved.status !== "saved") return { status: "unavailable", repository };
  return { status: "ready", repository, save, source: foundLegacy ? "migrated" : "empty" };
}

function mutateShared(
  repository: AppSaveRepository,
  change: (save: AppSave) => AppSave,
): AppSaveMutationResult {
  const loaded = repository.load();
  if (loaded.status === "corrupt" || loaded.status === "unavailable") return { status: loaded.status };
  if (loaded.status !== "loaded") return { status: "unavailable" };
  if (loaded.source !== "current") return { status: "recovery-read-only" };
  const result = repository.save(change(loaded.value));
  return result.status === "saved" ? result : { status: "unavailable", detail: result };
}

export function readSharedProfile(repository: AppSaveRepository): JsonObject | null {
  const loaded = repository.load();
  return loaded.status === "loaded" ? { ...loaded.value.profile } : null;
}

export function saveSharedProfile(repository: AppSaveRepository, profile: JsonObject): AppSaveMutationResult {
  return mutateShared(repository, (save) => ({ ...save, profile: { ...profile } }));
}

export function readSharedSetting(repository: AppSaveRepository, key: string): JsonValue | undefined {
  const loaded = repository.load();
  return loaded.status === "loaded" ? loaded.value.settings[key] : undefined;
}

export function saveSharedSetting(repository: AppSaveRepository, key: string, value: JsonValue): AppSaveMutationResult {
  return mutateShared(repository, (save) => ({ ...save, settings: { ...save.settings, [key]: value } }));
}

/** Story can seed a free Sandbox bench. Progress and inventory never cross,
 * and no reverse API exists that could inject Sandbox state into Story. */
export function cloneStoryBenchToSandbox(repository: AppSaveRepository): AppSaveMutationResult {
  const loaded = repository.load();
  if (loaded.status === "corrupt" || loaded.status === "unavailable") return { status: loaded.status };
  if (loaded.status !== "loaded") return { status: "unavailable" };
  if (loaded.source !== "current") return { status: "recovery-read-only" };
  const story = readModeValues(loaded.value, "story");
  const sandbox = readModeValues(loaded.value, "sandbox");
  if (story === null || sandbox === null) return { status: "invalid-session" };
  const next = { ...sandbox };
  for (const key of CLONE_KEYS) {
    if (key in story) next[key] = story[key]!;
    else delete next[key];
  }
  const result = repository.save(writeModeValues(loaded.value, "sandbox", next));
  return result.status === "saved" ? result : { status: "unavailable", detail: result };
}
