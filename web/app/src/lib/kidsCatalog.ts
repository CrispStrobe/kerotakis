import { normalizeCatalogText } from "./catalogSearch";

export type KidsStatus = "computed" | "partial" | "boundary" | "declined" | "unreachable";
export type KidsSafety = "home" | "school";

export interface KidsExperiment {
  id: string;
  title: string;
  phenomenon: string;
  title_de?: string;
  phenomenon_de?: string;
  status: KidsStatus;
  topics: string[];
  ingredients: string[];
  apparatus: string[];
  lesson?: string;
  quest?: string;
  /** Reviewed exact cross-references. These are identifiers, never search terms. */
  capabilities?: string[];
  codex?: string[];
  safety: KidsSafety;
  boundary?: string;
  boundary_de?: string;
}

export function parseKidsCatalog(raw: unknown): KidsExperiment[] {
  if (!raw || typeof raw !== "object") return [];
  const document = raw as { schema?: unknown; experiments?: unknown };
  if (document.schema !== 1) return [];
  const rows = document.experiments;
  if (!Array.isArray(rows)) return [];
  return rows.filter((row): row is KidsExperiment => {
    if (!row || typeof row !== "object") return false;
    const value = row as Partial<KidsExperiment>;
    return /^K\d{2}$/.test(value.id ?? "")
      && typeof value.title === "string"
      && typeof value.phenomenon === "string"
      && typeof value.title_de === "string"
      && typeof value.phenomenon_de === "string"
      && (!value.boundary || typeof value.boundary_de === "string")
      && ["computed", "partial", "boundary", "declined", "unreachable"].includes(value.status ?? "")
      && (value.safety === "home" || value.safety === "school")
      && [value.topics, value.ingredients, value.apparatus]
        .every((list) => Array.isArray(list) && list.every((item) => typeof item === "string"))
      && [value.capabilities, value.codex].every((list) => list === undefined || (
        Array.isArray(list) && list.length > 0 && list.every((item) => typeof item === "string" && item.length > 0)
      ));
  });
}

export interface KidsConnections {
  capabilities: string[];
  codex: string[];
  lessonCompleted: boolean;
  codexCompleted: string[];
}

/** Resolve only exported identifiers. Broken references disappear safely. */
export function kidsConnections(
  entry: KidsExperiment,
  capabilityIds: ReadonlySet<string>,
  codexIds: ReadonlySet<string>,
  completedMissions: ReadonlySet<string>,
  completedExperiments: ReadonlySet<string>,
): KidsConnections {
  const capabilities = (entry.capabilities ?? []).filter((id) => capabilityIds.has(id));
  const codex = (entry.codex ?? []).filter((id) => codexIds.has(id));
  const lessonId = entry.lesson?.replace(/\.lab$/, "") ?? null;
  return {
    capabilities,
    codex,
    lessonCompleted: lessonId !== null && completedMissions.has(lessonId),
    codexCompleted: codex.filter((id) => completedExperiments.has(id)),
  };
}

export function kidsText(item: KidsExperiment, field: "title" | "phenomenon" | "boundary", locale: string): string {
  if (locale === "de") {
    const localized = field === "title" ? item.title_de : field === "phenomenon" ? item.phenomenon_de : item.boundary_de;
    return localized ?? item[field] ?? "";
  }
  return item[field] ?? "";
}

export function kidsExperimentMatches(item: KidsExperiment, query: string, locale = "en"): boolean {
  const needle = normalizeCatalogText(query.trim().replaceAll("_", " ").replaceAll("-", " "));
  if (!needle) return true;
  return [item.id, kidsText(item, "title", locale), kidsText(item, "phenomenon", locale), kidsText(item, "boundary", locale),
    item.title, item.phenomenon, item.boundary ?? "", ...item.topics, ...item.ingredients, ...item.apparatus,
    ...(item.capabilities ?? []), ...(item.codex ?? [])]
    .some((value) => normalizeCatalogText(value.replaceAll("_", " ").replaceAll("-", " ")).includes(needle));
}
