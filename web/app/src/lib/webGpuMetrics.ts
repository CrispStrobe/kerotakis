/** Bounded, framework-neutral presentation telemetry for the WebGPU hot path. */

export const WEB_GPU_METRICS_CAPACITY = 120;

export interface WebGpuMetricsSnapshot {
  readonly session: number;
  readonly configureStartedAtMs: number;
  readonly firstPresentationLatencyMs: number | null;
  readonly successfulPresentations: number;
  readonly presentationFailures: number;
  readonly submittedFrames: number;
  readonly retainedFrameSamples: number;
  readonly frameCpuSubmissionP95Ms: number | null;
  readonly frameCpuSubmissionMaxMs: number | null;
}

export interface WebGpuPresentationMetrics {
  /** Clears prior measurements and records the start of a new configure session. */
  startSession(): number;
  /** Captures a timestamp that can be passed to recordFrameSubmitted. */
  startFrame(): number;
  /** Records CPU encode/submission time without allocating on the hot path. */
  recordFrameSubmitted(startedAtMs: number): void;
  /** Records a precomputed CPU encode/submission duration. */
  recordFrameSubmissionDuration(durationMs: number): void;
  /** Records a presentation and, once per session, configure-to-first-present latency. */
  recordPresentationSuccess(): void;
  recordPresentationFailure(): void;
  /** Clears measurements without starting/configuring a new session. */
  reset(): void;
  /** Allocates and sorts only here, never while recording a frame. */
  snapshot(): WebGpuMetricsSnapshot;
}

export interface WebGpuPresentationMetricsOptions {
  readonly capacity?: number;
  readonly now?: () => number;
}

const safeTimestamp = (value: number): number => Number.isFinite(value) && value >= 0 ? value : 0;
const safeDuration = (value: number): number => Number.isFinite(value) && value >= 0 ? value : 0;
const elapsed = (start: number, end: number): number =>
  Number.isFinite(start) && Number.isFinite(end) ? safeDuration(end - start) : 0;

function defaultNow(): number {
  return typeof performance === "object" && typeof performance.now === "function"
    ? performance.now()
    : Date.now();
}

/**
 * Creates a collector whose recording methods only mutate scalar values and a
 * preallocated ring buffer. Snapshot is intentionally the sole sorting path.
 */
export function createWebGpuPresentationMetrics(
  options: WebGpuPresentationMetricsOptions = {},
): WebGpuPresentationMetrics {
  const requestedCapacity = options.capacity ?? WEB_GPU_METRICS_CAPACITY;
  const capacity = Number.isSafeInteger(requestedCapacity) && requestedCapacity > 0
    ? requestedCapacity
    : WEB_GPU_METRICS_CAPACITY;
  const now = options.now ?? defaultNow;
  const samples = new Float64Array(capacity);
  let session = 0;
  let configured = false;
  let configureStartedAtMs = 0;
  let firstPresentationLatencyMs: number | null = null;
  let successfulPresentations = 0;
  let presentationFailures = 0;
  let submittedFrames = 0;
  let sampleCount = 0;
  let writeIndex = 0;

  const clear = (): void => {
    configured = false;
    configureStartedAtMs = 0;
    firstPresentationLatencyMs = null;
    successfulPresentations = 0;
    presentationFailures = 0;
    submittedFrames = 0;
    sampleCount = 0;
    writeIndex = 0;
  };

  const readNow = (): number => {
    try { return now(); } catch { return Number.NaN; }
  };

  const recordDuration = (durationMs: number): void => {
    samples[writeIndex] = safeDuration(durationMs);
    writeIndex = (writeIndex + 1) % capacity;
    sampleCount = Math.min(sampleCount + 1, capacity);
    submittedFrames += 1;
  };

  return {
    startSession(): number {
      clear();
      session += 1;
      configured = true;
      configureStartedAtMs = safeTimestamp(readNow());
      return session;
    },
    startFrame: readNow,
    recordFrameSubmitted(startedAtMs: number): void {
      recordDuration(elapsed(startedAtMs, readNow()));
    },
    recordFrameSubmissionDuration: recordDuration,
    recordPresentationSuccess(): void {
      successfulPresentations += 1;
      if (firstPresentationLatencyMs === null && configured) {
        firstPresentationLatencyMs = elapsed(configureStartedAtMs, readNow());
      }
    },
    recordPresentationFailure(): void {
      presentationFailures += 1;
    },
    reset: clear,
    snapshot(): WebGpuMetricsSnapshot {
      // Ring order is immaterial for a percentile; copying also keeps the
      // returned report detached from subsequent frame writes.
      const ordered = Array.from(samples.subarray(0, sampleCount)).sort((a, b) => a - b);
      const p95Index = ordered.length === 0 ? -1 : Math.ceil(ordered.length * 0.95) - 1;
      return {
        session,
        configureStartedAtMs,
        firstPresentationLatencyMs,
        successfulPresentations,
        presentationFailures,
        submittedFrames,
        retainedFrameSamples: sampleCount,
        frameCpuSubmissionP95Ms: p95Index < 0 ? null : ordered[p95Index]!,
        frameCpuSubmissionMaxMs: ordered.length === 0 ? null : ordered[ordered.length - 1]!,
      };
    },
  };
}
