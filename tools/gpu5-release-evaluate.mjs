#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { validateProbeArtifact } from "./gpu5-probe-lib.mjs";

export const RELEASE_HOSTS = ["web", "android", "ios", "macos", "windows"];
export const CPU_P95_LIMIT_MS = 9;
export const RAF_ADVISORY_MS = 25;
export const GZIP_DELTA_LIMIT_BYTES = 64 * 1024;
export const PROBE_SCHEMA = "kerotakis.gpu5-device-probe.v1";
const WARMUP = 60, MEASURED = 600, RUNS = 3, STARTUPS = 10;

const finite = (value) => typeof value === "number" && Number.isFinite(value) && value >= 0;
const samples = (value, count) => Array.isArray(value) && value.length === count && value.every(finite);

export function nearestRank(values, percentile) {
  if (!Array.isArray(values) || values.length === 0 || !values.every(finite)) throw new Error("nearest-rank requires finite non-negative samples");
  if (!(percentile > 0 && percentile <= 1)) throw new Error("percentile must be in (0, 1]");
  return [...values].sort((a, b) => a - b)[Math.ceil(values.length * percentile) - 1];
}
const distribution = (values) => ({ p50: nearestRank(values, .5), p95: nearestRank(values, .95), max: nearestRank(values, 1) });
const gzip = (asset) => asset?.totals?.all?.gzipBytes;

/** Maps one lightweight/WebGPU probe pair plus asset reports to one raw row. */
export function mapProbeArtifacts(host, baselineProbe, candidateProbe, baselineAssets, candidateAssets) {
  if (!RELEASE_HOSTS.includes(host)) return { status: "invalid", host, errors: [`unsupported host: ${host}`] };
  const errors = [];
  errors.push(...validateProbeArtifact(baselineProbe, "lightweight").map((error) => `baseline: ${error}`));
  errors.push(...validateProbeArtifact(candidateProbe, "webgpu").map((error) => `candidate: ${error}`));
  if (baselineAssets?.version !== 1 || candidateAssets?.version !== 1) errors.push("asset reports must have version 1");
  if (errors.length) return { status: "invalid", host, errors };
  if (candidateProbe?.webgpu_available !== true) {
    return { status: "unavailable", host, reason: candidateProbe?.outcome || "WebGPU probe unavailable", passed: null };
  }
  if (candidateProbe?.evidence_complete !== true) {
    return { status: "invalid", host, errors: ["candidate probe must declare complete evidence"] };
  }
  return evaluateMeasuredRow({
    status: "measured", host,
    baselineColdStartupMs: baselineProbe?.startup?.coldStartupMs,
    candidateColdStartupMs: candidateProbe?.startup?.coldStartupMs,
    baselineGzipBytes: gzip(baselineAssets), candidateGzipBytes: gzip(candidateAssets),
    runs: candidateProbe?.runs,
  });
}

export function evaluateMeasuredRow(row) {
  const errors = [];
  if (!samples(row.baselineColdStartupMs, STARTUPS)) errors.push(`baseline startup must contain ${STARTUPS} finite samples`);
  if (!samples(row.candidateColdStartupMs, STARTUPS)) errors.push(`candidate startup must contain ${STARTUPS} finite samples`);
  if (!Number.isSafeInteger(row.baselineGzipBytes) || row.baselineGzipBytes < 0) errors.push("baseline gzip bytes must be a non-negative safe integer");
  if (!Number.isSafeInteger(row.candidateGzipBytes) || row.candidateGzipBytes < 0) errors.push("candidate gzip bytes must be a non-negative safe integer");
  if (!Array.isArray(row.runs) || row.runs.length !== RUNS) errors.push(`candidate probe must contain exactly ${RUNS} runs`);
  else row.runs.forEach((run, index) => {
    if (!samples(run?.warmupCpuEncodeSubmitMs, WARMUP)) errors.push(`run ${index + 1} warmup must contain ${WARMUP} finite samples`);
    if (!samples(run?.measuredCpuEncodeSubmitMs, MEASURED)) errors.push(`run ${index + 1} CPU measurement must contain ${MEASURED} finite samples`);
    if (!samples(run?.measuredRafIntervalMs, MEASURED)) errors.push(`run ${index + 1} rAF measurement must contain ${MEASURED} finite samples`);
  });
  if (errors.length) return { status: "invalid", host: row.host, errors };

  const cpu = row.runs.flatMap((run) => run.measuredCpuEncodeSubmitMs);
  const raf = row.runs.flatMap((run) => run.measuredRafIntervalMs);
  const cpuRunP95Ms = row.runs.map((run) => nearestRank(run.measuredCpuEncodeSubmitMs, .95));
  const baselineStartup = distribution(row.baselineColdStartupMs);
  const candidateStartup = distribution(row.candidateColdStartupMs);
  const startupLimitMs = baselineStartup.p50 + Math.max(50, baselineStartup.p50 * .05);
  const gzipDeltaBytes = row.candidateGzipBytes - row.baselineGzipBytes;
  const failures = [];
  if (cpuRunP95Ms.some((value) => value > CPU_P95_LIMIT_MS)) failures.push("at least one CPU encode+submit run p95 exceeds 9 ms");
  if (candidateStartup.p50 > startupLimitMs) failures.push("cold startup regression exceeds max(+50 ms, +5%)");
  if (gzipDeltaBytes > GZIP_DELTA_LIMIT_BYTES) failures.push("gzip payload delta exceeds 64 KiB");
  const rafMetrics = distribution(raf);
  const rafOver25MsRatio = raf.filter((value) => value > RAF_ADVISORY_MS).length / raf.length;
  const advisories = [];
  if (rafMetrics.p95 > RAF_ADVISORY_MS) advisories.push("rAF p95 exceeds 25 ms");
  if (rafOver25MsRatio > 0) advisories.push("rAF samples above 25 ms observed");
  return { status: "evaluated", host: row.host, passed: failures.length === 0, failures, advisories, metrics: {
    cpuEncodeSubmitMs: distribution(cpu), cpuRunP95Ms, worstCpuRunP95Ms: Math.max(...cpuRunP95Ms),
    rafIntervalMs: rafMetrics, rafOver25MsRatio,
    baselineColdStartupMs: baselineStartup, candidateColdStartupMs: candidateStartup,
    coldStartupLimitMs: startupLimitMs, gzipDeltaBytes,
  } };
}

export function evaluateReleaseMatrix(rows) {
  const errors = [];
  if (!Array.isArray(rows)) return { schemaVersion: 1, passed: false, complete: false, rows: [], errors: ["rows must be an array"] };
  for (const host of RELEASE_HOSTS) {
    const count = rows.filter((row) => row?.host === host).length;
    if (count === 0) errors.push(`missing required host row: ${host}`);
    if (count > 1) errors.push(`duplicate host row: ${host}`);
  }
  for (const row of rows) if (!RELEASE_HOSTS.includes(row?.host)) errors.push(`unsupported host row: ${row?.host ?? "unknown"}`);
  const complete = errors.length === 0 && rows.every((row) => row.status === "evaluated");
  return { schemaVersion: 1, passed: complete && rows.every((row) => row.passed), complete, rows, errors };
}

const json = async (path) => JSON.parse(await readFile(resolve(path), "utf8"));
const option = (argv, name) => { const index = argv.indexOf(`--${name}`); return index < 0 ? undefined : argv[index + 1]; };
const usage = "usage: node tools/gpu5-release-evaluate.mjs --host web --baseline-probe FILE --candidate-probe FILE --baseline-assets FILE --candidate-assets FILE\n   or: node tools/gpu5-release-evaluate.mjs --matrix FILE";

async function main(argv) {
  const matrixPath = option(argv, "matrix");
  let report;
  if (matrixPath) {
    const manifest = await json(matrixPath);
    if (manifest?.schemaVersion !== 1 || !Array.isArray(manifest.rows)) throw new Error("matrix manifest must have schemaVersion 1 and rows");
    const root = dirname(resolve(matrixPath));
    const readRelative = (path) => json(resolve(root, path));
    const rows = await Promise.all(manifest.rows.map(async (entry) => mapProbeArtifacts(
      entry.host,
      await readRelative(entry.baselineProbe), await readRelative(entry.candidateProbe),
      await readRelative(entry.baselineAssets), await readRelative(entry.candidateAssets),
    )));
    report = evaluateReleaseMatrix(rows);
    process.exitCode = report.passed ? 0 : 1;
  } else {
    const host = option(argv, "host"), baselineProbe = option(argv, "baseline-probe"), candidateProbe = option(argv, "candidate-probe");
    const baselineAssets = option(argv, "baseline-assets"), candidateAssets = option(argv, "candidate-assets");
    if (!host || !baselineProbe || !candidateProbe || !baselineAssets || !candidateAssets) throw new Error(usage);
    const row = mapProbeArtifacts(host, await json(baselineProbe), await json(candidateProbe), await json(baselineAssets), await json(candidateAssets));
    // A single host can be evaluated, but can never claim matrix release pass.
    report = { schemaVersion: 1, mode: "single-host", releasePassed: null, row };
    process.exitCode = row.status === "evaluated" && row.passed ? 0 : 1;
  }
  process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  main(process.argv.slice(2)).catch((error) => { process.stderr.write(`${error.message}\n`); process.exitCode = 2; });
}
