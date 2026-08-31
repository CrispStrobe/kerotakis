export const GPU5_SCHEMA = "kerotakis.gpu5-device-probe.v1";
export const GPU5_WARMUP_FRAMES = 60;
export const GPU5_MEASURED_FRAMES = 600;
export const GPU5_RUNS = 3;
export const GPU5_CPU_BUDGET_MS = 9;
export const GPU5_APP_METRICS_SCHEMA = "kerotakis.webgpu-metrics.v1";
export const GPU5_APP_METRICS_REQUEST_EVENT = "kerotakis:webgpu-metrics-request";
export const GPU5_APP_METRICS_MAX_SESSIONS = 32;
export const GPU5_APP_METRICS_MAX_FRAME_SAMPLES = 120;

const finiteNonNegative = (value) => typeof value === "number" && Number.isFinite(value) && value >= 0;
const exactSamples = (value, count) => Array.isArray(value) && value.length === count && value.every(finiteNonNegative);

export function validateApplicationMetrics(report) {
  const errors = [];
  if (!report || typeof report !== "object" || Array.isArray(report)) return ["application metrics must be an object"];
  if (report.schema !== GPU5_APP_METRICS_SCHEMA) errors.push(`application metrics schema must be ${GPU5_APP_METRICS_SCHEMA}`);
  if (!Number.isSafeInteger(report.activeSessions) || report.activeSessions < 1 || report.activeSessions > GPU5_APP_METRICS_MAX_SESSIONS) {
    errors.push(`application metrics activeSessions must be within 1..${GPU5_APP_METRICS_MAX_SESSIONS}`);
  }
  if (!Array.isArray(report.sessions) || report.sessions.length !== report.activeSessions) {
    errors.push("application metrics sessions must match activeSessions");
  }
  for (const field of ["successfulPresentations", "presentationFailures", "submittedFrames"]) {
    if (!Number.isSafeInteger(report[field]) || report[field] < 0) errors.push(`application metrics ${field} must be a non-negative integer`);
  }
  if (Number.isSafeInteger(report.successfulPresentations) && report.successfulPresentations < 1) errors.push("application metrics must prove a successful presentation");
  if (Number.isSafeInteger(report.submittedFrames) && report.submittedFrames < 1) errors.push("application metrics must prove a submitted frame");
  if (Array.isArray(report.sessions)) report.sessions.forEach((session, index) => {
    if (!session || typeof session !== "object") return errors.push(`application metrics session ${index + 1} must be an object`);
    if (!["string", "number"].includes(typeof session.identity)) errors.push(`application metrics session ${index + 1} identity is invalid`);
    if (!Number.isSafeInteger(session.retainedFrameSamples) || session.retainedFrameSamples < 0 || session.retainedFrameSamples > GPU5_APP_METRICS_MAX_FRAME_SAMPLES) {
      errors.push(`application metrics session ${index + 1} retained samples exceed the bound`);
    }
    if (!Number.isSafeInteger(session.submittedFrames) || session.submittedFrames < session.retainedFrameSamples) {
      errors.push(`application metrics session ${index + 1} submitted frame count is invalid`);
    }
    for (const field of ["successfulPresentations", "presentationFailures"]) {
      if (!Number.isSafeInteger(session[field]) || session[field] < 0) errors.push(`application metrics session ${index + 1} ${field} is invalid`);
    }
  });
  if (Array.isArray(report.sessions)) {
    for (const field of ["successfulPresentations", "presentationFailures", "submittedFrames"]) {
      if (Number.isSafeInteger(report[field]) && report.sessions.every((session) => Number.isSafeInteger(session?.[field]))) {
        const sum = report.sessions.reduce((total, session) => total + session[field], 0);
        if (sum !== report[field]) errors.push(`application metrics aggregate ${field} does not match sessions`);
      }
    }
    if (!report.sessions.some((session) => finiteNonNegative(session?.firstPresentationLatencyMs))) {
      errors.push("application metrics must include first presentation latency");
    }
  }
  return errors;
}

/** Validate the emitted artifact rather than trusting its summary/declaration fields. */
export function validateProbeArtifact(report, expectedMode) {
  const errors = [];
  if (!report || typeof report !== "object" || Array.isArray(report)) return ["probe must be an object"];
  if (report.schema !== GPU5_SCHEMA) errors.push(`probe schema must be ${GPU5_SCHEMA}`);
  if (report.mode !== "lightweight" && report.mode !== "webgpu") errors.push("probe mode must be lightweight or webgpu");
  if (expectedMode && report.mode !== expectedMode) errors.push(`probe mode must be ${expectedMode}`);
  if (typeof report.host !== "string" || report.host.trim() === "") errors.push("probe host must be a non-empty string");
  if (!exactSamples(report.startup?.coldStartupMs, 10)) errors.push("startup coldStartupMs must contain 10 finite samples");
  for (const [name, values] of Object.entries({
    domContentLoadedMs: report.startup?.raw?.domContentLoadedMs,
    appReadyMs: report.startup?.raw?.appReadyMs,
    lightweightReadyMs: report.startup?.raw?.lightweightReadyMs,
  })) if (!exactSamples(values, 10)) errors.push(`startup raw.${name} must contain 10 finite samples`);

  if (report.mode === "lightweight") {
    if (report.webgpu_available !== false) errors.push("lightweight probe must not claim WebGPU availability");
    if (report.evidence_complete !== true) errors.push("lightweight probe must declare complete evidence");
    if (report.pass !== true) errors.push("lightweight probe must pass only as a recorded baseline");
    if (report.outcome !== "lightweight-baseline-recorded") errors.push("lightweight probe outcome is invalid");
    if (!Array.isArray(report.runs) || report.runs.length !== 0) errors.push("lightweight probe must not contain GPU runs");
  } else if (report.mode === "webgpu" && typeof report.webgpu_available !== "boolean") {
    errors.push("WebGPU probe availability must be explicit");
  } else if (report.webgpu_available === true) {
    if (report.evidence_complete !== true) errors.push("available WebGPU probe must declare complete evidence");
    if (!Array.isArray(report.runs) || report.runs.length !== GPU5_RUNS) errors.push(`available WebGPU probe must contain ${GPU5_RUNS} runs`);
    else report.runs.forEach((run, index) => {
      if (!exactSamples(run?.warmupCpuEncodeSubmitMs, GPU5_WARMUP_FRAMES)) errors.push(`run ${index + 1} warmup samples are incomplete`);
      if (!exactSamples(run?.measuredCpuEncodeSubmitMs, GPU5_MEASURED_FRAMES)) errors.push(`run ${index + 1} CPU samples are incomplete`);
      if (!exactSamples(run?.measuredRafIntervalMs, GPU5_MEASURED_FRAMES)) errors.push(`run ${index + 1} rAF samples are incomplete`);
    });
    if (report.fallback?.svg_present_before_gpu !== true) errors.push("available WebGPU probe must prove SVG fallback before GPU presentation");
    errors.push(...validateApplicationMetrics(report.application_metrics));
    if (typeof report.pass !== "boolean" || !["pass", "fail"].includes(report.outcome)) errors.push("available WebGPU probe must declare a pass or fail outcome");
    else if ((report.outcome === "pass") !== report.pass) errors.push("available WebGPU probe pass and outcome disagree");
  } else {
    if (report.evidence_complete !== false || report.pass !== false) errors.push("unavailable WebGPU probe must fail closed");
    if (!Array.isArray(report.runs) || report.runs.length !== 0) errors.push("unavailable WebGPU probe must not contain runs");
    if (!["webgpu-unavailable", "headless-policy-active"].includes(report.outcome)) {
      errors.push("unavailable WebGPU probe outcome is invalid");
    }
  }
  return errors;
}

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
  const raw = {
    domContentLoadedMs: samples.map((sample) => sample.dom_content_loaded_ms),
    appReadyMs: samples.map((sample) => sample.app_ready_ms),
    lightweightReadyMs: samples.map((sample) => sample.lightweight_ready_ms),
  };
  return {
    runs: samples.length,
    /** Direct release-gate startup endpoint: ordinary lightweight bench ready. */
    coldStartupMs: raw.lightweightReadyMs,
    raw,
    dom_content_loaded_ms: field("dom_content_loaded_ms"),
    app_ready_ms: field("app_ready_ms"),
    lightweight_ready_ms: field("lightweight_ready_ms"),
  };
}

export function buildRunEvidence(cpuSamples, rafSamples) {
  const needed = GPU5_WARMUP_FRAMES + GPU5_MEASURED_FRAMES;
  if (!Array.isArray(cpuSamples) || cpuSamples.length < needed) throw new Error(`probe needs ${needed} CPU frame samples`);
  if (!Array.isArray(rafSamples) || rafSamples.length < needed) throw new Error(`probe needs ${needed} rAF samples`);
  const warmupCpuEncodeSubmitMs = cpuSamples.slice(0, GPU5_WARMUP_FRAMES);
  const measuredCpuEncodeSubmitMs = cpuSamples.slice(GPU5_WARMUP_FRAMES, needed);
  const measuredRafIntervalMs = rafSamples.slice(GPU5_WARMUP_FRAMES, needed);
  // Strict validation is shared with the summaries; raw values remain intact.
  const cpuSummary = summarizeRun(cpuSamples);
  const rafSummary = {
    samples: measuredRafIntervalMs.length,
    median: nearestRank(measuredRafIntervalMs, 0.5),
    p95: nearestRank(measuredRafIntervalMs, 0.95),
    max: nearestRank(measuredRafIntervalMs, 1),
  };
  return {
    warmupCpuEncodeSubmitMs,
    measuredCpuEncodeSubmitMs,
    measuredRafIntervalMs,
    summary: { ...cpuSummary, raf_interval_ms_advisory: rafSummary },
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
      release_gate_pairing: "pair lightweight.startup.coldStartupMs with webgpu.startup.coldStartupMs and webgpu.runs; add frontend-asset-budget baseline/candidate gzip totals",
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
  const evidenceComplete = base.startup?.runs === 10
    && base.startup?.coldStartupMs?.length === 10
    && runs.every((run) =>
      run.warmupCpuEncodeSubmitMs?.length === GPU5_WARMUP_FRAMES
        && run.measuredCpuEncodeSubmitMs?.length === GPU5_MEASURED_FRAMES
        && run.measuredRafIntervalMs?.length === GPU5_MEASURED_FRAMES,
  );
  const passed = evidenceComplete
    && runs.every((run) => run.summary?.pass)
    && base.fallback.svg_present_before_gpu === true
    && validateApplicationMetrics(base.application_metrics).length === 0;
  return {
    ...base,
    webgpu_available: true,
    evidence_complete: evidenceComplete,
    runs,
    pass: passed,
    outcome: !evidenceComplete ? "incomplete" : passed ? "pass" : "fail",
  };
}
