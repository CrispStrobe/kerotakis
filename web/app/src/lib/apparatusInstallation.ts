export type ApparatusInstallation = {
  tool: string;
  target: number;
  values: Record<string, number | string>;
};

export interface InstallationStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

export function loadApparatusInstallation(
  storage: Pick<InstallationStorage, "getItem"> | null,
  key: string,
): ApparatusInstallation | null {
  if (!storage) return null;
  try {
    const value: unknown = JSON.parse(storage.getItem(key) ?? "null");
    if (!value || typeof value !== "object" || Array.isArray(value)) return null;
    const candidate = value as Record<string, unknown>;
    if (typeof candidate.tool !== "string" || typeof candidate.target !== "number" || !Number.isInteger(candidate.target)) return null;
    const values = candidate.values;
    if (!values || typeof values !== "object" || Array.isArray(values)) return null;
    const validValues = Object.fromEntries(
      Object.entries(values).filter(([, item]) => typeof item === "string" || (typeof item === "number" && Number.isFinite(item))),
    ) as Record<string, number | string>;
    return { tool: candidate.tool, target: candidate.target as number, values: validValues };
  } catch {
    return null;
  }
}

export function saveApparatusInstallation(
  storage: Pick<InstallationStorage, "setItem" | "removeItem"> | null,
  key: string,
  installation: ApparatusInstallation | null,
): void {
  if (!storage) return;
  try {
    if (installation) storage.setItem(key, JSON.stringify(installation));
    else storage.removeItem(key);
  } catch {
    // The installation remains usable for this visit when storage is unavailable.
  }
}
