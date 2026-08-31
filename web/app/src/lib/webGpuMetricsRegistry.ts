import {
  createWebGpuPresentationMetrics,
  type WebGpuMetricsSnapshot,
  type WebGpuPresentationMetrics,
} from "./webGpuMetrics";

export const WEB_GPU_METRICS_MAX_SESSIONS = 32;

export interface WebGpuMetricsSessionSnapshot extends WebGpuMetricsSnapshot {
  readonly identity: number | string;
  readonly token: number;
}

export interface WebGpuMetricsRegistrySnapshot {
  readonly activeSessions: number;
  readonly successfulPresentations: number;
  readonly presentationFailures: number;
  readonly submittedFrames: number;
  readonly sessions: readonly WebGpuMetricsSessionSnapshot[];
}

export interface WebGpuMetricsSession {
  readonly metrics: WebGpuPresentationMetrics;
  snapshot(): WebGpuMetricsSessionSnapshot;
  dispose(): void;
}

export interface WebGpuMetricsRegistry {
  open(identity: number | string): WebGpuMetricsSession;
  snapshot(): WebGpuMetricsRegistrySnapshot;
  /** Delivers a detached snapshot; diagnostics cannot disrupt rendering. */
  report(observer: (snapshot: WebGpuMetricsRegistrySnapshot) => void): WebGpuMetricsRegistrySnapshot;
  dispose(): void;
}

export interface WebGpuMetricsRegistryOptions {
  readonly capacity?: number;
  readonly createMetrics?: () => WebGpuPresentationMetrics;
}

type Entry = {
  readonly identity: number | string;
  readonly token: number;
  readonly metrics: WebGpuPresentationMetrics;
};

export function createWebGpuMetricsRegistry(
  options: WebGpuMetricsRegistryOptions = {},
): WebGpuMetricsRegistry {
  const requested = options.capacity ?? WEB_GPU_METRICS_MAX_SESSIONS;
  const capacity = Number.isSafeInteger(requested) && requested > 0
    ? requested
    : WEB_GPU_METRICS_MAX_SESSIONS;
  const createMetrics = options.createMetrics ?? createWebGpuPresentationMetrics;
  const entries = new Map<number, Entry>();
  let nextToken = 1;
  let disposed = false;

  const sessionSnapshot = (entry: Entry): WebGpuMetricsSessionSnapshot => {
    try {
      return { identity: entry.identity, token: entry.token, ...entry.metrics.snapshot() };
    } catch {
      return {
        identity: entry.identity,
        token: entry.token,
        session: 0,
        configureStartedAtMs: 0,
        firstPresentationLatencyMs: null,
        successfulPresentations: 0,
        presentationFailures: 0,
        submittedFrames: 0,
        retainedFrameSamples: 0,
        frameCpuSubmissionP95Ms: null,
        frameCpuSubmissionMaxMs: null,
      };
    }
  };

  const snapshot = (): WebGpuMetricsRegistrySnapshot => {
    const sessions = Array.from(entries.values(), sessionSnapshot);
    return {
      activeSessions: sessions.length,
      successfulPresentations: sessions.reduce((sum, item) => sum + item.successfulPresentations, 0),
      presentationFailures: sessions.reduce((sum, item) => sum + item.presentationFailures, 0),
      submittedFrames: sessions.reduce((sum, item) => sum + item.submittedFrames, 0),
      sessions,
    };
  };

  return {
    open(identity): WebGpuMetricsSession {
      let metrics: WebGpuPresentationMetrics;
      try { metrics = createMetrics(); } catch { metrics = createWebGpuPresentationMetrics(); }
      const token = nextToken++;
      const entry = { identity, token, metrics };
      if (!disposed) {
        if (entries.size >= capacity) {
          const oldest = entries.keys().next().value as number | undefined;
          if (oldest !== undefined) entries.delete(oldest);
        }
        entries.set(token, entry);
      }
      let sessionDisposed = false;
      return {
        metrics,
        snapshot: () => sessionSnapshot(entry),
        dispose(): void {
          if (sessionDisposed) return;
          sessionDisposed = true;
          try { metrics.reset(); } catch { /* diagnostics cleanup is best-effort */ }
          entries.delete(token);
        },
      };
    },
    snapshot,
    report(observer): WebGpuMetricsRegistrySnapshot {
      const report = snapshot();
      try { observer(report); } catch { /* diagnostics never own presentation */ }
      return report;
    },
    dispose(): void {
      disposed = true;
      for (const entry of entries.values()) {
        try { entry.metrics.reset(); } catch { /* diagnostics cleanup is best-effort */ }
      }
      entries.clear();
    },
  };
}
