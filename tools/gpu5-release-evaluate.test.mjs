import assert from "node:assert/strict";
import test from "node:test";
import { mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { PROBE_SCHEMA, RELEASE_HOSTS, evaluateReleaseMatrix, mapProbeArtifacts } from "./gpu5-release-evaluate.mjs";

const values = (count, value) => Array(count).fill(value);
const probe = (mode, cpu = 9) => ({
  schema: PROBE_SCHEMA, mode,
  host: "test-host",
  evidence_complete: true,
  webgpu_available: mode === "webgpu", outcome: mode === "webgpu" ? "pass" : "lightweight-baseline-recorded",
  pass: true,
  fallback: { svg_present_before_gpu: true, svg_present_now: mode !== "webgpu", gpu_presented: mode === "webgpu" },
  startup: {
    coldStartupMs: values(10, mode === "webgpu" ? 1050 : 1000),
    raw: { domContentLoadedMs: values(10, 1), appReadyMs: values(10, 2), lightweightReadyMs: values(10, 3) },
  },
  runs: mode === "webgpu" ? Array.from({ length: 3 }, () => ({
    warmupCpuEncodeSubmitMs: values(60, 50), measuredCpuEncodeSubmitMs: values(600, cpu), measuredRafIntervalMs: values(600, 16),
  })) : [],
});
const asset = (gzipBytes) => ({ version: 1, totals: { all: { gzipBytes } } });
const row = (host, cpu = 9) => mapProbeArtifacts(host, probe("lightweight"), probe("webgpu", cpu), asset(100000), asset(165536));

test("maps raw paired probes and assets without trusting probe summaries", () => {
  const result = row("web");
  assert.equal(result.status, "evaluated");
  assert.equal(result.passed, true);
  assert.deepEqual(result.metrics.cpuRunP95Ms, [9, 9, 9]);
  assert.equal(result.metrics.gzipDeltaBytes, 65536);
});

test("one bad run cannot hide under pooled p95", () => {
  const candidate = probe("webgpu", 1);
  candidate.runs[0].measuredCpuEncodeSubmitMs = [...values(540, 1), ...values(60, 12)];
  const result = mapProbeArtifacts("web", probe("lightweight"), candidate, asset(1), asset(1));
  assert.equal(result.metrics.cpuEncodeSubmitMs.p95, 1);
  assert.deepEqual(result.metrics.cpuRunP95Ms, [12, 1, 1]);
  assert.equal(result.passed, false);
});

test("matrix requires all five unique hosts and unavailable never passes", () => {
  assert.equal(evaluateReleaseMatrix(RELEASE_HOSTS.map((host) => row(host))).passed, true);
  assert.match(evaluateReleaseMatrix([row("web")]).errors.join("\n"), /missing required host row: android/);
  assert.match(evaluateReleaseMatrix([...RELEASE_HOSTS.map((host) => row(host)), row("web")]).errors.join("\n"), /duplicate host row: web/);
  const unavailableProbe = probe("webgpu"); unavailableProbe.webgpu_available = false; unavailableProbe.outcome = "webgpu-unavailable";
  const unavailable = RELEASE_HOSTS.map((host) => host === "ios" ? mapProbeArtifacts(host, probe("lightweight"), unavailableProbe, asset(1), asset(1)) : row(host));
  assert.equal(evaluateReleaseMatrix(unavailable).passed, false);
  assert.equal(evaluateReleaseMatrix(unavailable).complete, false);
});

test("rejects undersampled startup and measured arrays", () => {
  const baseline = probe("lightweight"); baseline.startup.coldStartupMs.pop();
  assert.equal(mapProbeArtifacts("web", baseline, probe("webgpu"), asset(1), asset(1)).status, "invalid");
  const candidate = probe("webgpu"); candidate.runs[0].measuredCpuEncodeSubmitMs.pop();
  assert.equal(mapProbeArtifacts("web", probe("lightweight"), candidate, asset(1), asset(1)).status, "invalid");
});

test("rejects artifacts that do not declare complete evidence", () => {
  const baseline = probe("lightweight"); baseline.evidence_complete = false;
  assert.match(mapProbeArtifacts("web", baseline, probe("webgpu"), asset(1), asset(1)).errors.join("\n"), /baseline: lightweight probe must declare complete evidence/);
  const candidate = probe("webgpu"); candidate.evidence_complete = false;
  assert.match(mapProbeArtifacts("web", probe("lightweight"), candidate, asset(1), asset(1)).errors.join("\n"), /candidate: available WebGPU probe must declare complete evidence/);
});

test("rejects missing modes and incomplete raw arrays before mapping", () => {
  const baseline = probe("lightweight"); delete baseline.mode;
  assert.match(mapProbeArtifacts("web", baseline, probe("webgpu"), asset(1), asset(1)).errors.join("\n"), /mode/);
  const candidate = probe("webgpu"); candidate.startup.raw.lightweightReadyMs.pop();
  assert.match(mapProbeArtifacts("web", probe("lightweight"), candidate, asset(1), asset(1)).errors.join("\n"), /raw\.lightweightReadyMs/);
});

test("rejects incomplete run arrays and absent fallback proof even when summaries claim pass", () => {
  const incomplete = probe("webgpu"); incomplete.runs[1].measuredRafIntervalMs.pop();
  assert.match(mapProbeArtifacts("web", probe("lightweight"), incomplete, asset(1), asset(1)).errors.join("\n"), /run 2 rAF samples are incomplete/);
  const noFallback = probe("webgpu"); noFallback.fallback.svg_present_before_gpu = false;
  assert.match(mapProbeArtifacts("web", probe("lightweight"), noFallback, asset(1), asset(1)).errors.join("\n"), /prove SVG fallback/);
});

test("CLI consumes four evidence files and does not claim matrix release from one host", async () => {
  const directory = await mkdtemp(join(tmpdir(), "kero-gpu5-gate-"));
  const files = { baseline: probe("lightweight"), candidate: probe("webgpu"), baselineAssets: asset(100), candidateAssets: asset(100) };
  for (const [name, value] of Object.entries(files)) await writeFile(join(directory, `${name}.json`), JSON.stringify(value));
  const result = spawnSync(process.execPath, [
    new URL("./gpu5-release-evaluate.mjs", import.meta.url).pathname, "--host", "web",
    "--baseline-probe", join(directory, "baseline.json"), "--candidate-probe", join(directory, "candidate.json"),
    "--baseline-assets", join(directory, "baselineAssets.json"), "--candidate-assets", join(directory, "candidateAssets.json"),
  ], { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  const report = JSON.parse(result.stdout);
  assert.equal(report.releasePassed, null);
  assert.equal(report.row.passed, true);

  const manifest = {
    schemaVersion: 1,
    rows: RELEASE_HOSTS.map((host) => ({
      host,
      baselineProbe: "baseline.json",
      candidateProbe: "candidate.json",
      baselineAssets: "baselineAssets.json",
      candidateAssets: "candidateAssets.json",
    })),
  };
  await writeFile(join(directory, "matrix.json"), JSON.stringify(manifest));
  const matrixResult = spawnSync(process.execPath, [
    new URL("./gpu5-release-evaluate.mjs", import.meta.url).pathname,
    "--matrix", join(directory, "matrix.json"),
  ], { encoding: "utf8" });
  assert.equal(matrixResult.status, 0, matrixResult.stderr);
  const matrixReport = JSON.parse(matrixResult.stdout);
  assert.equal(matrixReport.passed, true);
  assert.equal(matrixReport.complete, true);
  assert.deepEqual(matrixReport.rows.map((entry) => entry.host), RELEASE_HOSTS);
});
