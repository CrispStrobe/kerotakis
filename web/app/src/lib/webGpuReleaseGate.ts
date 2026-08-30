/** GPU-5b deterministic release evidence evaluator. */

export const GPU_RELEASE_WARMUP_SAMPLES = 60;
export const GPU_RELEASE_MEASURED_SAMPLES = 600;
export const GPU_RELEASE_RUNS = 3;
export const GPU_RELEASE_STARTUP_RUNS = 10;
export const GPU_RELEASE_CPU_P95_LIMIT_MS = 9;
export const GPU_RELEASE_RAF_JANK_MS = 25;
export const GPU_RELEASE_GZIP_DELTA_LIMIT_BYTES = 64 * 1024;

export interface ReleaseRunInput {
  warmupCpuEncodeSubmitMs: number[];
  measuredCpuEncodeSubmitMs: number[];
  measuredRafIntervalMs: number[];
}

export type ReleaseHost = "web" | "android" | "ios" | "macos" | "windows";
export const GPU_RELEASE_HOSTS: readonly ReleaseHost[] = ["web", "android", "ios", "macos", "windows"];

export interface MeasuredReleaseRowInput {
  status: "measured";
  host: ReleaseHost;
  baselineColdStartupMs: number[];
  candidateColdStartupMs: number[];
  baselineGzipBytes: number;
  candidateGzipBytes: number;
  runs: ReleaseRunInput[];
}

export interface UnavailableReleaseRowInput {
  status: "unavailable";
  host: ReleaseHost;
  reason: string;
}

export interface ReleaseGateInput {
  schemaVersion: 1;
  rows: Array<MeasuredReleaseRowInput | UnavailableReleaseRowInput>;
}

export interface DistributionMetrics {
  p50: number;
  p95: number;
  max: number;
}

export type ReleaseRowResult =
  | { status: "invalid"; host: string; errors: string[] }
  | { status: "unavailable"; host: string; reason: string; passed: null }
  | {
      status: "evaluated";
      host: string;
      passed: boolean;
      failures: string[];
      advisories: string[];
      metrics: {
        cpuEncodeSubmitMs: DistributionMetrics;
        cpuRunP95Ms: number[];
        worstCpuRunP95Ms: number;
        rafIntervalMs: DistributionMetrics;
        rafOver25MsRatio: number;
        baselineColdStartupMs: DistributionMetrics;
        candidateColdStartupMs: DistributionMetrics;
        coldStartupLimitMs: number;
        gzipDeltaBytes: number;
      };
    };

export interface ReleaseGateReport {
  schemaVersion: 1;
  passed: boolean;
  complete: boolean;
  rows: ReleaseRowResult[];
  errors: string[];
}

function finiteNonnegative(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value) && value >= 0;
}

export function nearestRank(values: readonly number[], percentile: number): number {
  if (values.length === 0 || !values.every(Number.isFinite)) {
    throw new RangeError("nearest-rank requires finite samples");
  }
  if (!Number.isFinite(percentile) || percentile <= 0 || percentile > 1) {
    throw new RangeError("percentile must be within (0, 1]");
  }
  const ordered = [...values].sort((left, right) => left - right);
  return ordered[Math.ceil(percentile * ordered.length) - 1]!;
}

function distribution(values: readonly number[]): DistributionMetrics {
  return {
    p50: nearestRank(values, 0.5),
    p95: nearestRank(values, 0.95),
    max: nearestRank(values, 1),
  };
}

function validSamples(value: unknown, count: number): value is number[] {
  return Array.isArray(value) && value.length === count && value.every(finiteNonnegative);
}

function evaluateMeasured(row: MeasuredReleaseRowInput): ReleaseRowResult {
  const errors: string[] = [];
  if (!validSamples(row.baselineColdStartupMs, GPU_RELEASE_STARTUP_RUNS)) errors.push(`baselineColdStartupMs must contain ${GPU_RELEASE_STARTUP_RUNS} finite non-negative samples`);
  if (!validSamples(row.candidateColdStartupMs, GPU_RELEASE_STARTUP_RUNS)) errors.push(`candidateColdStartupMs must contain ${GPU_RELEASE_STARTUP_RUNS} finite non-negative samples`);
  if (!Number.isSafeInteger(row.baselineGzipBytes) || row.baselineGzipBytes < 0) errors.push("baselineGzipBytes must be a non-negative safe integer");
  if (!Number.isSafeInteger(row.candidateGzipBytes) || row.candidateGzipBytes < 0) errors.push("candidateGzipBytes must be a non-negative safe integer");
  if (!Array.isArray(row.runs) || row.runs.length !== GPU_RELEASE_RUNS) {
    errors.push(`runs must contain exactly ${GPU_RELEASE_RUNS} entries`);
  } else {
    row.runs.forEach((run, index) => {
      if (!run || typeof run !== "object") {
        errors.push(`run ${index + 1} is missing`);
        return;
      }
      if (!validSamples(run.warmupCpuEncodeSubmitMs, GPU_RELEASE_WARMUP_SAMPLES)) {
        errors.push(`run ${index + 1} warmupCpuEncodeSubmitMs must contain ${GPU_RELEASE_WARMUP_SAMPLES} finite non-negative samples`);
      }
      if (!validSamples(run.measuredCpuEncodeSubmitMs, GPU_RELEASE_MEASURED_SAMPLES)) {
        errors.push(`run ${index + 1} measuredCpuEncodeSubmitMs must contain ${GPU_RELEASE_MEASURED_SAMPLES} finite non-negative samples`);
      }
      if (!validSamples(run.measuredRafIntervalMs, GPU_RELEASE_MEASURED_SAMPLES)) {
        errors.push(`run ${index + 1} measuredRafIntervalMs must contain ${GPU_RELEASE_MEASURED_SAMPLES} finite non-negative samples`);
      }
    });
  }
  if (errors.length > 0) return { status: "invalid", host: row.host, errors };

  const cpu = row.runs.flatMap((run) => run.measuredCpuEncodeSubmitMs);
  const raf = row.runs.flatMap((run) => run.measuredRafIntervalMs);
  const cpuRunP95Ms = row.runs.map((run) => nearestRank(run.measuredCpuEncodeSubmitMs, 0.95));
  const cpuMetrics = distribution(cpu);
  const rafMetrics = distribution(raf);
  const baselineStartupMetrics = distribution(row.baselineColdStartupMs);
  const candidateStartupMetrics = distribution(row.candidateColdStartupMs);
  const coldStartupLimitMs = baselineStartupMetrics.p50
    + Math.max(50, baselineStartupMetrics.p50 * 0.05);
  const gzipDeltaBytes = row.candidateGzipBytes - row.baselineGzipBytes;
  const failures: string[] = [];
  if (cpuRunP95Ms.some((p95) => p95 > GPU_RELEASE_CPU_P95_LIMIT_MS)) failures.push("at least one CPU encode+submit run p95 exceeds 9 ms");
  if (candidateStartupMetrics.p50 > coldStartupLimitMs) failures.push("cold startup regression exceeds max(+50 ms, +5%)");
  if (gzipDeltaBytes > GPU_RELEASE_GZIP_DELTA_LIMIT_BYTES) failures.push("gzip payload delta exceeds 64 KiB");
  const rafOver25MsRatio = raf.filter((sample) => sample > GPU_RELEASE_RAF_JANK_MS).length / raf.length;
  const advisories: string[] = [];
  if (rafMetrics.p95 > GPU_RELEASE_RAF_JANK_MS) advisories.push("rAF p95 exceeds 25 ms");
  if (rafOver25MsRatio > 0) advisories.push("rAF samples above 25 ms observed");
  return {
    status: "evaluated",
    host: row.host,
    passed: failures.length === 0,
    failures,
    advisories,
    metrics: {
        cpuEncodeSubmitMs: cpuMetrics,
        cpuRunP95Ms,
        worstCpuRunP95Ms: Math.max(...cpuRunP95Ms),
      rafIntervalMs: rafMetrics,
      rafOver25MsRatio,
        baselineColdStartupMs: baselineStartupMetrics,
        candidateColdStartupMs: candidateStartupMetrics,
      coldStartupLimitMs,
      gzipDeltaBytes,
    },
  };
}

export function evaluateWebGpuReleaseGate(input: unknown): ReleaseGateReport {
  if (!input || typeof input !== "object") {
    return { schemaVersion: 1, passed: false, complete: false, rows: [], errors: ["input must be an object"] };
  }
  const candidate = input as Partial<ReleaseGateInput>;
  const errors: string[] = [];
  if (candidate.schemaVersion !== 1) errors.push("schemaVersion must equal 1");
  if (!Array.isArray(candidate.rows) || candidate.rows.length === 0) errors.push("rows must be a non-empty array");
  if (errors.length > 0) return { schemaVersion: 1, passed: false, complete: false, rows: [], errors };

  const rows = candidate.rows!.map((row): ReleaseRowResult => {
    if (!row || typeof row !== "object") return { status: "invalid", host: "unknown", errors: ["row must be an object"] };
    const host = typeof row.host === "string" && row.host.trim() ? row.host : "unknown";
    if (row.status === "unavailable") {
      return typeof row.reason === "string" && row.reason.trim()
        ? { status: "unavailable", host, reason: row.reason, passed: null }
        : { status: "invalid", host, errors: ["unavailable row requires a reason"] };
    }
    if (row.status !== "measured") return { status: "invalid", host, errors: ["row status must be measured or unavailable"] };
    return evaluateMeasured({ ...row, host } as MeasuredReleaseRowInput);
  });
  const suppliedHosts = candidate.rows!
    .map((row) => (row as { host?: unknown } | null)?.host)
    .filter((host): host is string => typeof host === "string");
  for (const host of GPU_RELEASE_HOSTS) {
    const count = suppliedHosts.filter((candidateHost) => candidateHost === host).length;
    if (count === 0) errors.push(`missing required host row: ${host}`);
    if (count > 1) errors.push(`duplicate host row: ${host}`);
  }
  for (const host of suppliedHosts) {
    if (!(GPU_RELEASE_HOSTS as readonly string[]).includes(host)) errors.push(`unsupported host row: ${host}`);
  }
  const complete = errors.length === 0 && rows.every((row) => row.status === "evaluated");
  const passed = complete && rows.every((row) => row.status === "evaluated" && row.passed);
  return { schemaVersion: 1, passed, complete, rows, errors };
}
