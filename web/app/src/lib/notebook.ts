/**
 * GUI-022: the feed as a Markdown lab notebook. Everything a learner made
 * leaves the app (ROADMAP-GUI.md interaction principle 6): commands as
 * code, observations as text, hazards as called-out quotes, charts as
 * their data tables.
 */

import { seriesPoints } from "./chart";
import type { FeedEntry } from "./session.svelte";

export function notebookMarkdown(
  entries: FeedEntry[],
  meta: { title?: string; date?: string; register?: string } = {},
): string {
  const out: string[] = [];
  out.push(`# ${meta.title ?? "Kerotakis lab notebook"}`);
  const line2: string[] = [];
  if (meta.date) line2.push(meta.date);
  if (meta.register) line2.push(`register ${meta.register}`);
  if (line2.length > 0) out.push("", line2.join(" · "));
  out.push("");

  for (const entry of entries) {
    switch (entry.kind) {
      case "command":
        out.push("```", `kero> ${entry.text}`, "```");
        break;
      case "line":
        out.push(entry.text, "");
        break;
      case "note":
        out.push(`*${entry.text}*`, "");
        break;
      case "hazard":
        out.push(`> **${entry.severity || "hazard"}** — ${entry.text}`, "");
        break;
      case "refusal":
      case "error":
        out.push(`> ${entry.text}`, "");
        break;
      case "chart": {
        if (!entry.chart) break;
        const c = entry.chart;
        out.push(`### ${c.title}`, "");
        const axis = (a: { label: string; unit?: string }) =>
          a.unit ? `${a.label} (${a.unit})` : a.label;
        for (const s of c.series) {
          out.push(
            `| ${axis(c.x)} | ${axis(c.y)} |`,
            "|---:|---:|",
            ...seriesPoints(s).map(([x, y]) => `| ${x} | ${y} |`),
            "",
          );
        }
        if (c.provenance) out.push(`*${c.provenance}*`, "");
        break;
      }
    }
  }
  return out.join("\n").replace(/\n{3,}/g, "\n\n") + "\n";
}
