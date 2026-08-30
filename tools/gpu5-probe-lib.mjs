export const GPU5_SCHEMA = "kerotakis.gpu5-device-probe.v1";
export const GPU5_WARMUP_FRAMES = 60;
export const GPU5_MEASURED_FRAMES = 600;
export const GPU5_RUNS = 3;
export const GPU5_CPU_BUDGET_MS = 9;

export function nearestRank(values, percentile) {
  if (!Array.isArray(values) || values.length === 0) throw new Error("samples must be non-empty");
  if (!(percentile > 0 && percentile <= 1)) throw new Error("percentile must be in (0, 1]");
  const sorted = [...values].sort((a, b) => a - b);
  if (sorted.some((value) => typeof value !== "number" || !Number.isFinite(value) || value < 0)) {
    throw new Error("samples must be finite non-negative numbers");
  }
  return sorted[Math.max(1, Math.ceil(percentile * sorted.length)) - 1];
}

export function summarizeRun(samples, warmup = GPU5_WARMUP_FRAMES, measured = GPU5_MEASURED_FRAMES) {
  if (!Number.isInteger(warmup) || warmup < 0 || !Number.isInteger(measured) || measured < 1) {
    throw new Error("warmup and measured frame counts must be non-negative integers");
  }
  if (!Array.isArray(samples) || samples.length < warmup + measured) {
    throw new Error(`probe needs ${warmup + measured} CPU frame samples`);
  }
  const values = samples.slice(warmup, warmup + measured);
  const p95 = nearestRank(values, 0.95);
  return {
    samples: values.length,
    cpu_encode_submit_ms: {
      min: Math.min(...values),
      median: nearestRank(values, 0.5),
      p95,
      max: Math.max(...values),
    },
    budget_ms: GPU5_CPU_BUDGET_MS,
    pass: p95 <= GPU5_CPU_BUDGET_MS,
  };
}

export function summarizeStartup(samples) {
  if (!Array.isArray(samples) || samples.length !== 10) throw new Error("startup probe needs exactly 10 cold runs");
  const field = (name) => {
    const values = samples.map((sample) => sample[name]);
    return { median: nearestRank(values, 0.5), p95: nearestRank(values, 0.95) };
  };
  return {
    runs: samples.length,
    dom_content_loaded_ms: field("dom_content_loaded_ms"),
    app_ready_ms: field("app_ready_ms"),
    lightweight_ready_ms: field("lightweight_ready_ms"),
  };
}

export function emptyReport({ hostLabel, userAgent, startup, fallback, automationPolicyOverride = false }) {
  return {
    schema: GPU5_SCHEMA,
    host: hostLabel,
    user_agent: userAgent,
    automation_policy_override: automationPolicyOverride,
    protocol: {
      warmup_frames: GPU5_WARMUP_FRAMES,
      measured_frames: GPU5_MEASURED_FRAMES,
      runs: GPU5_RUNS,
      cpu_budget_ms: GPU5_CPU_BUDGET_MS,
      metric: "CPU time from createCommandEncoder through queue.submit return",
      gpu_readback: false,
      per_frame_await: false,
      probe_overhead: "WeakMap bookkeeping plus one numeric sample append per encoder submission and rAF",
      comparison_requires_separate_invocations: true,
    },
    startup,
    fallback,
    webgpu_available: false,
    evidence_complete: false,
    runs: [],
    pass: false,
    outcome: "webgpu-unavailable",
  };
}

export function completeReport(base, runs) {
  if (!Array.isArray(runs) || runs.length !== GPU5_RUNS) {
    throw new Error(`probe needs exactly ${GPU5_RUNS} runs`);
  }
  const evidenceComplete = base.startup?.runs === 10 && runs.every((run) =>
    run.samples === GPU5_MEASURED_FRAMES
      && run.raf_interval_ms_advisory?.samples === GPU5_MEASURED_FRAMES,
  );
  const passed = evidenceComplete
    && runs.every((run) => run.pass)
    && base.fallback.svg_present_before_gpu === true;
  return {
    ...base,
    webgpu_available: true,
    evidence_complete: evidenceComplete,
    runs,
    pass: passed,
    outcome: !evidenceComplete ? "incomplete" : passed ? "pass" : "fail",
  };
}
