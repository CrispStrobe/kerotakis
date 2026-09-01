/** Atomic, schema-neutral persistence for WORLD-001's single AppSave envelope. */
export interface KeyValueStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

export interface AppSaveCodec<T> {
  encode(value: T): string;
  /** Return a validated value or throw. */
  decode(serialized: string): T;
}

type Slot = "current" | "last-known-good" | "staging";
export type AppSaveEvidence = Readonly<Partial<Record<Slot, string>>>;
export type AppSaveLoadResult<T> =
  | { status: "loaded"; value: T; source: Slot; evidence: AppSaveEvidence }
  | { status: "empty"; evidence: AppSaveEvidence }
  | { status: "corrupt"; evidence: AppSaveEvidence }
  | { status: "unavailable"; operation: "read"; error: unknown; evidence: AppSaveEvidence };
export type AppSaveSaveResult =
  | { status: "saved" }
  | { status: "invalid"; operation: "encode" | "validate"; error: unknown }
  | { status: "unavailable"; operation: "write-staging" | "read-staging" | "write-last-known-good" | "read-last-known-good" | "write-current" | "read-current" | "clear-staging"; error: unknown };

export class AtomicAppSaveStorage<T> {
  constructor(
    private readonly storage: KeyValueStorage,
    private readonly codec: AppSaveCodec<T>,
    private readonly prefix = "kero.app-save.v1",
  ) {}

  key(slot: Slot): string { return `${this.prefix}.${slot}`; }

  private evidence():
    | { status: "available"; evidence: AppSaveEvidence }
    | { status: "unavailable"; error: unknown; evidence: AppSaveEvidence } {
    const evidence: Partial<Record<Slot, string>> = {};
    try {
      for (const slot of ["current", "last-known-good", "staging"] as const) {
        const raw = this.storage.getItem(this.key(slot));
        if (raw !== null) evidence[slot] = raw;
      }
      return { status: "available", evidence };
    } catch (error) {
      return { status: "unavailable", error, evidence };
    }
  }

  private decode(raw: string | undefined): T | null {
    if (raw === undefined) return null;
    try { return this.codec.decode(raw); } catch { return null; }
  }

  /** Read without repairing or deleting corrupt forensic evidence. */
  load(): AppSaveLoadResult<T> {
    const snapshot = this.evidence();
    if (snapshot.status === "unavailable") {
      return { status: "unavailable", operation: "read", error: snapshot.error, evidence: snapshot.evidence };
    }
    const { evidence } = snapshot;
    for (const slot of ["current", "last-known-good", "staging"] as const) {
      const value = this.decode(evidence[slot]);
      if (value !== null) return { status: "loaded", value, source: slot, evidence };
    }
    return Object.keys(evidence).length === 0 ? { status: "empty", evidence } : { status: "corrupt", evidence };
  }

  /** Validate, stage, retain the old valid current as LKG, then promote. */
  save(value: T): AppSaveSaveResult {
    let encoded: string;
    try { encoded = this.codec.encode(value); }
    catch (error) { return { status: "invalid", operation: "encode", error }; }
    try { this.codec.decode(encoded); }
    catch (error) { return { status: "invalid", operation: "validate", error }; }

    try { this.storage.setItem(this.key("staging"), encoded); }
    catch (error) { return { status: "unavailable", operation: "write-staging", error }; }
    let staged: string | null;
    try {
      staged = this.storage.getItem(this.key("staging"));
      if (staged === null) throw new Error("staging write could not be read back");
      this.codec.decode(staged);
    } catch (error) { return { status: "unavailable", operation: "read-staging", error }; }

    let oldCurrent: string | null;
    try { oldCurrent = this.storage.getItem(this.key("current")); }
    catch (error) { return { status: "unavailable", operation: "read-current", error }; }
    if (oldCurrent !== null && this.decode(oldCurrent) !== null) {
      try { this.storage.setItem(this.key("last-known-good"), oldCurrent); }
      catch (error) { return { status: "unavailable", operation: "write-last-known-good", error }; }
      try {
        const lkg = this.storage.getItem(this.key("last-known-good"));
        if (lkg === null) throw new Error("LKG write could not be read back");
        this.codec.decode(lkg);
      } catch (error) { return { status: "unavailable", operation: "read-last-known-good", error }; }
    }

    try { this.storage.setItem(this.key("current"), staged); }
    catch (error) { return { status: "unavailable", operation: "write-current", error }; }
    try {
      const promoted = this.storage.getItem(this.key("current"));
      if (promoted === null) throw new Error("promotion could not be read back");
      this.codec.decode(promoted);
    } catch (error) { return { status: "unavailable", operation: "read-current", error }; }
    try { this.storage.removeItem(this.key("staging")); }
    catch (error) { return { status: "unavailable", operation: "clear-staging", error }; }
    return { status: "saved" };
  }
}
