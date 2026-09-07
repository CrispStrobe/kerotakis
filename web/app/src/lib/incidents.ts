import type { EngineEvent, Effect } from "./magnitudes";
import { t } from "./i18n.svelte";

export function incidentEffects(effects: Record<number, Effect[]>, now = Date.now()): Effect[] {
  const live = Object.values(effects).flat().filter((effect) =>
    (effect.kind === "spill" || effect.kind === "break")
      && now - effect.at < (effect.durationMs ?? 5000),
  );
  // A broken charged vessel emits both events atomically. The shards already
  // include a pool, so coalesce the companion spill instead of double-painting.
  return live.filter((effect) => effect.kind !== "spill" || !live.some((candidate) =>
    candidate.kind === "break"
      && candidate.source === effect.source
      && Math.abs(candidate.at - effect.at) < 250,
  ));
}

/** Durable, animation-independent evidence for the exported lab notebook. */
export function incidentNotebookEvidence(event: EngineEvent): string | null {
  const tag = String(event.event ?? "");
  // `discarded` is here for the same reason the other three are: matter
  // left a vessel and is now somewhere else on the bench. A disposal is
  // the one of the four the reader MEANT to do, which is exactly why the
  // notebook has to record it — an experiment that ends "and then we
  // threw it away" is missing the line that says how much of what.
  if (tag === "discarded") {
    return t("Evidence: vessel v{vessel} was emptied into the waste — {grams} g ({moles} mol), still held and still weighed by the bench.", {
      vessel: Number(event.vessel ?? 0) + 1,
      grams: Number(event.grams_total ?? 0).toFixed(3),
      moles: Number(event.moles_total ?? 0).toFixed(3),
    });
  }
  if (tag !== "spill_created" && tag !== "container_broken" && tag !== "spill_recovered") return null;
  const destination = event.destination && typeof event.destination === "object"
    ? event.destination as Record<string, unknown>
    : {};
  const surface = String(destination.surface ?? "bench");
  const location = String(destination.zone ?? destination.tray ?? "unknown");
  if (tag === "container_broken") {
    return t("Evidence: vessel v{vessel} broke; contents routed to {destination}.", {
      vessel: Number(event.vessel ?? 0) + 1,
      destination: `${surface} ${location}`,
    });
  }
  if (tag === "spill_recovered") {
    const percent = (Math.max(0, Math.min(1, Number(event.fraction ?? 0))) * 100).toFixed(1);
    return t("Evidence: {percent}% of {destination} was recovered into vessel v{vessel}.", {
      percent,
      destination: `${surface} ${location}`,
      vessel: Number(event.to ?? 0) + 1,
    });
  }
  const percent = (Math.max(0, Math.min(1, Number(event.fraction ?? 0))) * 100).toFixed(1);
  return t("Evidence: {percent}% of vessel v{vessel} entered {destination}.", {
    percent,
    vessel: Number(event.source ?? event.from ?? 0) + 1,
    destination: `${surface} ${location}`,
  });
}
