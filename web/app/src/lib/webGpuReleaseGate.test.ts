import { describe, expect, it } from "vitest";
import { GPU_RELEASE_GZIP_DELTA_LIMIT_BYTES, GPU_RELEASE_HOSTS, evaluateWebGpuReleaseGate, nearestRank, type MeasuredReleaseRowInput, type ReleaseHost } from "./webGpuReleaseGate";

const samples = (count: number, value: number): number[] => Array.from({ length: count }, () => value);
const run = (cpu = samples(600, 9), raf = samples(600, 16.7)) => ({ warmupCpuEncodeSubmitMs: samples(60, 20), measuredCpuEncodeSubmitMs: cpu, measuredRafIntervalMs: raf });
const measuredRow = (host: ReleaseHost, overrides: Partial<MeasuredReleaseRowInput> = {}): MeasuredReleaseRowInput => ({
  status: "measured", host,
  baselineColdStartupMs: samples(10, 1_000), candidateColdStartupMs: samples(10, 1_050),
  baselineGzipBytes: 100_000, candidateGzipBytes: 100_000 + GPU_RELEASE_GZIP_DELTA_LIMIT_BYTES,
  runs: [run(), run(), run()], ...overrides,
});
const matrix = (web = measuredRow("web")) => ({ schemaVersion: 1, rows: GPU_RELEASE_HOSTS.map((host) => host === "web" ? web : measuredRow(host)) });

describe("WebGPU release gate", () => {
  it("uses nearest-rank metrics and passes exact boundaries for all five hosts", () => {
    expect(nearestRank([1, 2, 3, 4], 0.5)).toBe(2);
    expect(nearestRank([1, 2, 3, 4], 0.95)).toBe(4);
    const report = evaluateWebGpuReleaseGate(matrix());
    expect(report).toMatchObject({ passed: true, complete: true, errors: [] });
    expect(report.rows).toHaveLength(5);
    expect(report.rows[0]).toMatchObject({ status: "evaluated", passed: true, metrics: {
      cpuEncodeSubmitMs: { p50: 9, p95: 9, max: 9 }, cpuRunP95Ms: [9, 9, 9],
      worstCpuRunP95Ms: 9, coldStartupLimitMs: 1_050,
    } });
  });

  it("fails a bad individual CPU run even when pooled p95 hides it", () => {
    const hiddenBadRun = [...samples(540, 1), ...samples(60, 12)];
    const row = measuredRow("web", { runs: [run(hiddenBadRun), run(samples(600, 1)), run(samples(600, 1))] });
    const result = evaluateWebGpuReleaseGate(matrix(row)).rows[0]!;
    if (result.status !== "evaluated") throw new Error("expected evaluated row");
    expect(result.metrics.cpuEncodeSubmitMs.p95).toBe(1);
    expect(result.metrics.cpuRunP95Ms).toEqual([12, 1, 1]);
    expect(result.passed).toBe(false);
  });

  it("fails startup and gzip while rAF remains advisory", () => {
    const row = measuredRow("web", {
      candidateColdStartupMs: samples(10, 1_051), candidateGzipBytes: 165_537,
      runs: [run(samples(600, 9), [...samples(500, 16), ...samples(100, 30)]), run(), run()],
    });
    const result = evaluateWebGpuReleaseGate(matrix(row)).rows[0]!;
    if (result.status !== "evaluated") throw new Error("expected evaluated row");
    expect(result.failures).toHaveLength(2);
    expect(result.advisories).toEqual(["rAF p95 exceeds 25 ms", "rAF samples above 25 ms observed"]);
    expect(result.metrics.rafOver25MsRatio).toBeCloseTo(1 / 18);
  });

  it("uses +5% when larger than +50 ms", () => {
    const row = measuredRow("web", { baselineColdStartupMs: samples(10, 2_000), candidateColdStartupMs: samples(10, 2_100) });
    expect(evaluateWebGpuReleaseGate(matrix(row)).rows[0]).toMatchObject({ status: "evaluated", passed: true, metrics: { coldStartupLimitMs: 2_100 } });
  });

  it.each([
    ["missing run", () => measuredRow("web", { runs: measuredRow("web").runs.slice(0, 2) })],
    ["undersampled warmup", () => { const row = measuredRow("web"); row.runs[0]!.warmupCpuEncodeSubmitMs.pop(); return row; }],
    ["undersampled measurement", () => { const row = measuredRow("web"); row.runs[0]!.measuredCpuEncodeSubmitMs.pop(); return row; }],
    ["nonfinite CPU", () => { const row = measuredRow("web"); row.runs[0]!.measuredCpuEncodeSubmitMs[0] = Number.NaN; return row; }],
    ["missing rAF", () => { const row = measuredRow("web"); row.runs[0]!.measuredRafIntervalMs = []; return row; }],
    ["9/10 baseline startup samples", () => measuredRow("web", { baselineColdStartupMs: samples(9, 1_000) })],
    ["9/10 candidate startup samples", () => measuredRow("web", { candidateColdStartupMs: samples(9, 1_000) })],
  ])("rejects %s evidence", (_label, makeRow) => {
    const report = evaluateWebGpuReleaseGate(matrix(makeRow()));
    expect(report).toMatchObject({ passed: false, complete: false });
    expect(report.rows[0]?.status).toBe("invalid");
  });

  it("rejects missing, duplicate and unsupported host rows", () => {
    const missing = matrix(); missing.rows.pop();
    expect(evaluateWebGpuReleaseGate(missing).errors).toContain("missing required host row: windows");
    const duplicate = matrix(); duplicate.rows.push(measuredRow("web"));
    expect(evaluateWebGpuReleaseGate(duplicate).errors).toContain("duplicate host row: web");
    const unsupported = matrix() as { schemaVersion: number; rows: unknown[] };
    unsupported.rows.push({ ...measuredRow("web"), host: "linux" });
    expect(evaluateWebGpuReleaseGate(unsupported).errors).toContain("unsupported host row: linux");
  });

  it("records unavailable device rows without claiming pass", () => {
    const input = matrix(); input.rows[2] = { status: "unavailable", host: "ios", reason: "no WebGPU adapter" } as never;
    const report = evaluateWebGpuReleaseGate(JSON.parse(JSON.stringify(input)));
    expect(report).toMatchObject({ passed: false, complete: false, errors: [] });
    expect(report.rows[2]).toEqual({ status: "unavailable", host: "ios", reason: "no WebGPU adapter", passed: null });
  });

  it("rejects malformed top-level JSON and malformed unavailable rows", () => {
    expect(evaluateWebGpuReleaseGate(null).errors).toContain("input must be an object");
    expect(evaluateWebGpuReleaseGate({ schemaVersion: 2, rows: [] }).passed).toBe(false);
    const input = matrix(); input.rows[2] = { status: "unavailable", host: "ios" } as never;
    expect(evaluateWebGpuReleaseGate(input).rows[2]?.status).toBe("invalid");
  });
});
