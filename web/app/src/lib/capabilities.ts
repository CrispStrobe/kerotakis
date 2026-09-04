export type CapabilitySupport = "computed" | "curated" | "qualitative" | "boundary" | "missing";

export interface CapabilityPrompt {
  id: string;
  question: string;
  age_band: string;
  topic: string;
  material_class: string;
  tags: string[];
  script: string[];
  owning_task: string;
  support: CapabilitySupport;
  reason_code: string;
  boundary?: string | null;
}

export function parseCapabilityIndex(raw: unknown): CapabilityPrompt[] {
  const doc = raw as { schema?: number; prompts?: unknown[] };
  if (doc?.schema !== 1 || !Array.isArray(doc.prompts)) return [];
  return (doc.prompts as CapabilityPrompt[]).filter((prompt) =>
    typeof prompt?.id === "string" &&
    typeof prompt?.question === "string" &&
    typeof prompt?.support === "string" &&
    Array.isArray(prompt?.script),
  );
}

export function capabilityMatches(prompt: CapabilityPrompt, query: string): boolean {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return true;
  return [prompt.id, prompt.question, prompt.topic, prompt.material_class, prompt.owning_task, ...prompt.tags]
    .some((value) => value.replaceAll("_", " ").toLocaleLowerCase().includes(needle));
}
