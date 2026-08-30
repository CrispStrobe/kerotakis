import assert from "node:assert/strict";
import test from "node:test";
import {
  GPU5_MEASURED_FRAMES,
  GPU5_RUNS,
  GPU5_SCHEMA,
  GPU5_WARMUP_FRAMES,
  completeReport,
  emptyReport,
  nearestRank,
  summarizeRun,
  summarizeStartup,
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
    startup: { runs: 10, dom_content_loaded_ms: {}, app_ready_ms: {}, lightweight_ready_ms: {} },
    fallback: { svg_present_before_gpu: true, svg_present_now: true, gpu_presented: false },
  });
  assert.equal(base.schema, GPU5_SCHEMA);
  assert.equal(base.automation_policy_override, true);
  assert.equal(base.protocol.comparison_requires_separate_invocations, true);
  assert.equal(base.outcome, "webgpu-unavailable");
  const run = {
    ...summarizeRun(Array(660).fill(1)),
    raf_interval_ms_advisory: { samples: 600, median: 16, p95: 17, max: 20 },
  };
  const report = completeReport({ ...base, webgpu_available: true }, Array(GPU5_RUNS).fill(run));
  assert.equal(report.runs.length, 3);
  assert.equal(report.pass, true);
  assert.equal(report.outcome, "pass");
  const incomplete = completeReport({ ...base, webgpu_available: true }, [
    { ...run, raf_interval_ms_advisory: { samples: 599, incomplete: true } }, run, run,
  ]);
  assert.equal(incomplete.evidence_complete, false);
  assert.equal(incomplete.pass, false);
  assert.equal(incomplete.outcome, "incomplete");
});
