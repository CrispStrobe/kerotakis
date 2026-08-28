/**
 * GUI-022: the feed as a Markdown lab notebook. Everything a learner made
 * leaves the app (ROADMAP-GUI.md interaction principle 6): commands as
 * code, observations as text, hazards as called-out quotes, charts as
 * their data tables.
 */

import { seriesPoints } from "./chart";
import type { FeedEntry } from "./session.svelte";
import { t } from "./i18n.svelte";
import { engineText } from "./engineText";

export function notebookMarkdown(
  entries: FeedEntry[],
  meta: { title?: string; date?: string; register?: string } = {},
): string {
  const out: string[] = [];
  out.push(`# ${meta.title ?? t("Kerotakis lab notebook")}`);
  const line2: string[] = [];
  if (meta.date) line2.push(meta.date);
  if (meta.register) line2.push(`${t("register")} ${meta.register}`);
  if (line2.length > 0) out.push("", line2.join(" · "));
  out.push("");

  for (const entry of entries) {
    switch (entry.kind) {
      case "command":
        out.push("```", `kero> ${entry.text}`, "```");
        break;
      case "line":
        out.push(engineText(entry.text), "");
        break;
      case "note":
        out.push(`*${engineText(entry.text)}*`, "");
        break;
      case "user-note":
        out.push(`> **${t("my note")}${entry.createdAt ? ` · ${entry.createdAt}` : ""}**`, `> ${entry.text}`, "");
        break;
      case "hazard":
        out.push(
          `> **${t(entry.severity || "hazard")}** — ${entry.hazardText && entry.realWorld
            ? `${engineText(entry.hazardText)} — ${engineText(entry.realWorld)}`
            : engineText(entry.text)}`,
          "",
        );
        break;
      case "refusal":
      case "error":
        out.push(`> ${engineText(entry.text)}`, "");
        break;
      case "chart": {
        if (!entry.chart) break;
        const c = entry.chart;
        out.push(`### ${engineText(c.title)}`, "");
        const axis = (a: { label: string; unit?: string }) =>
          a.unit ? `${engineText(a.label)} (${engineText(a.unit)})` : engineText(a.label);
        for (const s of c.series) {
          out.push(
            `| ${axis(c.x)} | ${axis(c.y)} |`,
            "|---:|---:|",
            ...seriesPoints(s).map(([x, y]) => `| ${x} | ${y} |`),
            "",
          );
        }
        if (c.provenance) out.push(`*${engineText(c.provenance)}*`, "");
        break;
      }
    }
  }
  return out.join("\n").replace(/\n{3,}/g, "\n\n") + "\n";
}
