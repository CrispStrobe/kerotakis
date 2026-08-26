export type LabMode = "story" | "sandbox";

export interface KeyValueStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

export interface LabProfile {
  version: 1;
  name: string;
  createdAt: string;
}

export const MODE_KEY = "kerotakis.mode.v1";
export const PROFILE_KEY = "kerotakis.lab-profile.v1";
export const HOME_SEEN_KEY = "kerotakis.home-seen.v1";
export const PENDING_MISSION_KEY = "kerotakis.pending-mission.v1";

const LEGACY_SESSION_KEYS = ["kero.session.v1", "kero.codex.done.v1"];
const SANDBOX_MIGRATION_KEY = "kero.mode.sandbox.migrated-v1";

export function readLabMode(storage: KeyValueStorage | null): LabMode {
  try {
    return storage?.getItem(MODE_KEY) === "story" ? "story" : "sandbox";
  } catch {
    return "sandbox";
  }
}

export function writeLabMode(storage: KeyValueStorage | null, mode: LabMode): void {
  try {
    storage?.setItem(MODE_KEY, mode);
  } catch {
    // The live shell can still enter the chosen mode for this visit.
  }
}

/** Separate every Session key by game mode. The engine still owns all
 * chemistry; this only decides which deterministic log is restored. */
export class ModeStorage implements KeyValueStorage {
  constructor(
    private backing: KeyValueStorage,
    private mode: LabMode,
  ) {
    // Existing pre-mode installs were sandbox labs. Copy once, never move:
    // migration cannot destroy a user's only save.
    if (mode === "sandbox" && this.safeGet(SANDBOX_MIGRATION_KEY) !== "yes") {
      for (const key of LEGACY_SESSION_KEYS) {
        const legacy = this.safeGet(key);
        if (legacy !== null && this.safeGet(this.scoped(key)) === null) {
          try {
            this.backing.setItem(this.scoped(key), legacy);
          } catch {
            // A storage-blocked browser simply starts without migration.
          }
        }
      }
      try {
        this.backing.setItem(SANDBOX_MIGRATION_KEY, "yes");
      } catch {
        // Session persistence is unavailable too; the live visit still works.
      }
    }
  }

  private scoped(key: string): string {
    return `kero.mode.${this.mode}.${key}`;
  }

  private safeGet(key: string): string | null {
    try {
      return this.backing.getItem(key);
    } catch {
      return null;
    }
  }

  getItem(key: string): string | null {
    return this.safeGet(this.scoped(key));
  }

  setItem(key: string, value: string): void {
    this.backing.setItem(this.scoped(key), value);
  }

  removeItem(key: string): void {
    this.backing.removeItem(this.scoped(key));
  }
}

export function loadLabProfile(
  storage: KeyValueStorage | null,
  now = () => new Date().toISOString(),
): LabProfile {
  try {
    const raw = storage?.getItem(PROFILE_KEY);
    if (raw) {
      const value = JSON.parse(raw) as Partial<LabProfile>;
      if (value.version === 1 && typeof value.name === "string" && value.name.trim()) {
        return { version: 1, name: value.name.trim().slice(0, 48), createdAt: String(value.createdAt ?? now()) };
      }
    }
  } catch {
    // A corrupt profile becomes a fresh identity; mode saves are untouched.
  }
  const profile: LabProfile = { version: 1, name: "My Chemistry Lab", createdAt: now() };
  saveLabProfile(storage, profile);
  return profile;
}

export function saveLabProfile(storage: KeyValueStorage | null, profile: LabProfile): void {
  try {
    storage?.setItem(PROFILE_KEY, JSON.stringify({ ...profile, name: profile.name.trim().slice(0, 48) || "My Chemistry Lab" }));
  } catch {
    // Identity remains usable in memory when persistence is unavailable.
  }
}
