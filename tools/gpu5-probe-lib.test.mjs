import assert from "node:assert/strict";
import test from "node:test";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import {
  GPU5_MEASURED_FRAMES,
  GPU5_RUNS,
  GPU5_SCHEMA,
  GPU5_WARMUP_FRAMES,
  completeReport,
  buildRunEvidence,
  emptyReport,
  nearestRank,
  summarizeRun,
  summarizeStartup,
  validateProbeArtifact,
} from "./gpu5-probe-lib.mjs";

test("nearest-rank percentile is deterministic and strict", () => {
  assert.equal(nearestRank([9, 1, 5, 3], 0.5), 3);
  assert.equal(nearestRank([9, 1, 5, 3], 0.95), 9);
  assert.throws(() => nearestRank([1, Number.NaN], 0.95), /finite/);
  assert.throws(() => nearestRank([], 0.95), /non-empty/);
});

test("startup summary requires ten cold profiles", () => {
  const samples = Array.from({ length: 10 }, (_, index) => ({
    dom_content_loaded_ms: index + 1,
    app_ready_ms: index + 11,
    lightweight_ready_ms: index + 21,
  }));
  assert.deepEqual(summarizeStartup(samples), {
    runs: 10,
    coldStartupMs: [21, 22, 23, 24, 25, 26, 27, 28, 29, 30],
    raw: {
      domContentLoadedMs: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
      appReadyMs: [11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
      lightweightReadyMs: [21, 22, 23, 24, 25, 26, 27, 28, 29, 30],
    },
    dom_content_loaded_ms: { median: 5, p95: 10 },
    app_ready_ms: { median: 15, p95: 20 },
    lightweight_ready_ms: { median: 25, p95: 30 },
  });
  assert.throws(() => summarizeStartup(samples.slice(1)), /exactly 10/);
});

test("run summary discards exactly 60 warmup frames and retains 600", () => {
  const summary = summarizeRun([
    ...Array(GPU5_WARMUP_FRAMES).fill(1000),
    ...Array(GPU5_MEASURED_FRAMES - 1).fill(4),
    8,
  ]);
  assert.equal(summary.samples, 600);
  assert.equal(summary.cpu_encode_submit_ms.p95, 4);
  assert.equal(summary.cpu_encode_submit_ms.max, 8);
  assert.equal(summary.pass, true);
});

test("9 ms p95 is a hard inclusive gate", () => {
  const pass = summarizeRun(Array(660).fill(9));
  const fail = summarizeRun([...Array(60).fill(0), ...Array(569).fill(1), ...Array(31).fill(9.01)]);
  assert.equal(pass.pass, true);
  assert.equal(fail.pass, false);
});

test("unavailable and complete reports have one stable schema", () => {
  const base = emptyReport({
    hostLabel: "ci", userAgent: "test",
    automationPolicyOverride: true,
    startup: { runs: 10, coldStartupMs: Array(10).fill(1), dom_content_loaded_ms: {}, app_ready_ms: {}, lightweight_ready_ms: {} },
    fallback: { svg_present_before_gpu: true, svg_present_now: true, gpu_presented: false },
  });
  assert.equal(base.schema, GPU5_SCHEMA);
  assert.equal(base.automation_policy_override, true);
  assert.equal(base.protocol.comparison_requires_separate_invocations, true);
  assert.equal(base.outcome, "webgpu-unavailable");
  const run = buildRunEvidence(Array(660).fill(1), Array(660).fill(16));
  assert.equal(run.warmupCpuEncodeSubmitMs.length, 60);
  assert.equal(run.measuredCpuEncodeSubmitMs.length, 600);
  assert.equal(run.measuredRafIntervalMs.length, 600);
  const report = completeReport({ ...base, webgpu_available: true }, Array(GPU5_RUNS).fill(run));
  const emitted = JSON.parse(JSON.stringify(report));
  assert.equal(emitted.startup.coldStartupMs.length, 10);
  assert.equal(emitted.runs.length, 3);
  for (const emittedRun of emitted.runs) {
    assert.equal(emittedRun.warmupCpuEncodeSubmitMs.length, 60);
    assert.equal(emittedRun.measuredCpuEncodeSubmitMs.length, 600);
    assert.equal(emittedRun.measuredRafIntervalMs.length, 600);
  }
  assert.equal(report.runs.length, 3);
  assert.equal(report.pass, true);
  assert.equal(report.outcome, "pass");
  const incomplete = completeReport({ ...base, webgpu_available: true }, [
    { ...run, measuredRafIntervalMs: run.measuredRafIntervalMs.slice(1) }, run, run,
  ]);
  assert.equal(incomplete.evidence_complete, false);
  assert.equal(incomplete.pass, false);
  assert.equal(incomplete.outcome, "incomplete");
});

test("artifact validation fails closed on missing mode and incomplete raw startup arrays", () => {
  const startup = summarizeStartup(Array.from({ length: 10 }, (_, index) => ({
    dom_content_loaded_ms: index, app_ready_ms: index + 1, lightweight_ready_ms: index + 2,
  })));
  const artifact = {
    ...emptyReport({ hostLabel: "web", userAgent: "test", startup, fallback: {} }),
    mode: "lightweight", evidence_complete: true, pass: true, outcome: "lightweight-baseline-recorded",
  };
  assert.deepEqual(validateProbeArtifact(artifact, "lightweight"), []);
  assert.match(validateProbeArtifact({ ...artifact, mode: undefined }, "lightweight").join("\n"), /mode/);
  const incomplete = structuredClone(artifact);
  incomplete.startup.raw.appReadyMs.pop();
  assert.match(validateProbeArtifact(incomplete, "lightweight").join("\n"), /raw\.appReadyMs/);
});

test("unavailable artifacts are accepted only when they fail closed", () => {
  const startup = summarizeStartup(Array.from({ length: 10 }, (_, index) => ({
    dom_content_loaded_ms: index, app_ready_ms: index + 1, lightweight_ready_ms: index + 2,
  })));
  const artifact = { ...emptyReport({ hostLabel: "ios", userAgent: "test", startup, fallback: {} }), mode: "webgpu" };
  assert.deepEqual(validateProbeArtifact(artifact, "webgpu"), []);
  assert.match(validateProbeArtifact({ ...artifact, pass: true }, "webgpu").join("\n"), /fail closed/);
  assert.match(validateProbeArtifact({ ...artifact, webgpu_available: undefined }, "webgpu").join("\n"), /availability must be explicit/);
  assert.match(validateProbeArtifact({ ...artifact, outcome: "pass" }, "webgpu").join("\n"), /outcome is invalid/);
});

test("lightweight artifacts cannot masquerade as GPU evidence", () => {
  const startup = summarizeStartup(Array.from({ length: 10 }, (_, index) => ({
    dom_content_loaded_ms: index, app_ready_ms: index + 1, lightweight_ready_ms: index + 2,
  })));
  const artifact = {
    ...emptyReport({ hostLabel: "web", userAgent: "test", startup, fallback: {} }),
    mode: "lightweight", evidence_complete: true, pass: true, outcome: "lightweight-baseline-recorded",
  };
  assert.match(validateProbeArtifact({ ...artifact, webgpu_available: true }, "lightweight").join("\n"), /must not claim WebGPU/);
  assert.match(validateProbeArtifact({ ...artifact, pass: false }, "lightweight").join("\n"), /recorded baseline/);
});

test("probe CLI rejects missing and malformed modes as usage errors", async (t) => {
  const site = await mkdtemp(join(tmpdir(), "kero-gpu5-mode-"));
  t.after(() => rm(site, { recursive: true, force: true }));
  const cli = new URL("./gpu5-probe.mjs", import.meta.url).pathname;
  for (const args of [[site], [site, "--mode", "maybe"]]) {
    const result = spawnSync(process.execPath, [cli, ...args], { encoding: "utf8" });
    assert.equal(result.status, 2);
    assert.match(result.stderr, /mode must be explicitly set/);
  }
});
