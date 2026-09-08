import type { CodexAssertion, CodexAssertionSample } from "./codex";
import type { Scene } from "./host/EngineHost";

export interface ComparisonSnapshot { step: "initial" | "final" | `after:${number}`; scene: Scene | null }
export interface ComparisonCell { sample: CodexAssertionSample; vessel: string | null; value: number | null; unit: string }
export interface ComparisonRow { kind: CodexAssertion["kind"]; cells: ComparisonCell[]; holds: boolean | null }

const vesselNumber = (vessel?: string) => vessel?.match(/^v(\d+)$/)?.[1];
export function metricValue(scene: Scene | null, sample: CodexAssertionSample, vesselNumbers: readonly number[] = [], vesselOffset = 0): { value: number | null; unit: string } {
  const all = scene?.vessels ?? [];
  const number = vesselNumber(sample.vessel);
  const vessels = number === undefined
    ? all.filter((v) => vesselNumbers.length === 0 || vesselNumbers.includes(v.id + 1))
    : all.filter((v) => v.id === Number(number) - 1 + vesselOffset);
  const first = vessels[0];
  if (number !== undefined && first === undefined) return { value: null, unit: "" };
  if (sample.metric === "mass_g") return { value: vessels.reduce((n, v) => n + v.mass_g, 0), unit: "g" };
  if (sample.metric === "elapsed_s") return { value: vessels.reduce((n, v) => n + v.elapsed_s, 0), unit: "s" };
  if (sample.metric === "temperature_c") return { value: first ? first.temperature_k - 273.15 : null, unit: "°C" };
  if (sample.metric === "pressure_kpa") return { value: first ? first.pressure_pa / 1_000 : null, unit: "kPa" };
  if (sample.metric === "ph") return { value: first?.badges.find((b) => b.key === "ph")?.value ?? null, unit: "pH" };
  // Scene v1 contains neither replay event payloads nor a complete species
  // inventory. Keep both unavailable instead of substituting an unsafe proxy.
  if (sample.metric.startsWith("event:")) return { value: null, unit: "" };
  if (sample.metric.startsWith("moles:")) return { value: null, unit: "mol" };
  return { value: null, unit: "" };
}
function relationHolds(assertion: CodexAssertion, values: number[]): boolean | null {
  if (values.length < 2) return null;
  const tolerance = assertion.tolerance ?? 1e-9;
  const kind = assertion.kind;
  if (kind === "increasing") return values.slice(1).every((value, i) => value > values[i]! + tolerance);
  if (kind === "decreasing") return values.slice(1).every((value, i) => value < values[i]! - tolerance);
  if (kind === "ratio") {
    if (values.length !== 2 || assertion.ratio === undefined) return null;
    const expected = assertion.ratio * values[0]!;
    const allowed = tolerance + (assertion.relative_tolerance ?? 0) * Math.max(Math.abs(expected), Math.abs(values[1]!));
    return Math.abs(values[1]! - expected) <= allowed;
  }
  return values.slice(1).every((value) => {
    const first = values[0]!;
    const allowed = tolerance + (assertion.relative_tolerance ?? 0) * Math.max(Math.abs(first), Math.abs(value));
    return Math.abs(value - first) <= allowed;
  });
}
export function comparisonRows(assertions: readonly CodexAssertion[], snapshots: readonly ComparisonSnapshot[], vesselNumbers: readonly number[] = [], vesselOffset = 0): ComparisonRow[] {
  return assertions.map((assertion) => {
    const cells = assertion.samples.map((sample) => ({
      sample,
      vessel: sample.vessel ? `v${Number(vesselNumber(sample.vessel)) + vesselOffset}` : null,
      ...metricValue(snapshots.find((s) => s.step === sample.step)?.scene ?? null, sample, vesselNumbers, vesselOffset),
    }));
    const values = cells.map((cell) => cell.value);
    return { kind: assertion.kind, cells, holds: values.some((v) => v === null) ? null : relationHolds(assertion, values as number[]) };
  });
}
