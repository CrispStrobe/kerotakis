import { normalizeCatalogText } from "./catalogSearch";

export type KidsStatus = "computed" | "partial" | "boundary" | "declined" | "unreachable";
export type KidsSafety = "home" | "school";

export interface KidsExperiment {
  id: string;
  title: string;
  phenomenon: string;
  status: KidsStatus;
  topics: string[];
  ingredients: string[];
  apparatus: string[];
  lesson?: string;
  quest?: string;
  safety: KidsSafety;
  boundary?: string;
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
      && ["computed", "partial", "boundary", "declined", "unreachable"].includes(value.status ?? "")
      && (value.safety === "home" || value.safety === "school")
      && [value.topics, value.ingredients, value.apparatus]
        .every((list) => Array.isArray(list) && list.every((item) => typeof item === "string"));
  });
}

export function kidsExperimentMatches(item: KidsExperiment, query: string): boolean {
  const needle = normalizeCatalogText(query.trim());
  if (!needle) return true;
  return [item.id, item.title, item.phenomenon, item.boundary ?? "", ...item.topics, ...item.ingredients, ...item.apparatus]
    .some((value) => normalizeCatalogText(value.replaceAll("_", " ")).includes(needle));
}
