import { describe, expect, it, vi } from "vitest";
import { createWebGpuPresentationMetrics } from "./webGpuMetrics";
import {
  attachWebGpuMetricsReporter,
  browserWebGpuMetricsReporterTarget,
  createWebGpuMetricsRegistry,
  WEB_GPU_METRICS_MAX_SESSIONS,
  WEB_GPU_METRICS_REPORT_SCHEMA,
  WEB_GPU_METRICS_REQUEST_EVENT,
} from "./webGpuMetricsRegistry";

describe("WebGPU metrics registry", () => {
  it("aggregates active renderer sessions and removes disposed entries", () => {
    const registry = createWebGpuMetricsRegistry();
    const first = registry.open("vessel-1");
    const second = registry.open("vessel-2");
    first.metrics.startSession();
    first.metrics.recordFrameSubmissionDuration(2);
    first.metrics.recordPresentationSuccess();
    second.metrics.startSession();
    second.metrics.recordPresentationFailure();

    expect(registry.snapshot()).toMatchObject({
      schema: WEB_GPU_METRICS_REPORT_SCHEMA,
      activeSessions: 2,
      successfulPresentations: 1,
      presentationFailures: 1,
      submittedFrames: 1,
    });
    first.dispose();
    expect(registry.snapshot().sessions.map(({ identity }) => identity)).toEqual(["vessel-2"]);
  });

  it("reports only on request and removes the request listener", () => {
    let listener: ((event: unknown) => void) | undefined;
    const target = {
      addEventListener: vi.fn((_type: string, next: (event: unknown) => void) => { listener = next; }),
      removeEventListener: vi.fn(),
    };
    const registry = createWebGpuMetricsRegistry();
    registry.open("vessel");
    const respond = vi.fn();
    const detach = attachWebGpuMetricsReporter(target, registry);
    expect(target.addEventListener).toHaveBeenCalledWith(WEB_GPU_METRICS_REQUEST_EVENT, expect.any(Function));
    listener?.({ detail: { respond } });
    expect(respond).toHaveBeenCalledWith(expect.objectContaining({ schema: WEB_GPU_METRICS_REPORT_SCHEMA, activeSessions: 1 }));
    detach();
    detach();
    listener?.({ detail: { respond } });
    expect(respond).toHaveBeenCalledTimes(1);
    expect(target.removeEventListener).toHaveBeenCalledTimes(1);
  });

  it("is SSR safe and contains hostile reporter targets and responders", () => {
    expect(browserWebGpuMetricsReporterTarget(undefined)).toBeNull();
    expect(browserWebGpuMetricsReporterTarget({})).toBeNull();
    const target = { addEventListener: () => { throw new Error("add"); }, removeEventListener: vi.fn() };
    expect(() => attachWebGpuMetricsReporter(target, createWebGpuMetricsRegistry())()).not.toThrow();
  });

  it("bounds simultaneous tracked sessions while every handle remains usable", () => {
    const registry = createWebGpuMetricsRegistry({ capacity: 2 });
    const first = registry.open(1);
    registry.open(2);
    const third = registry.open(3);
    first.metrics.startSession();
    third.metrics.startSession();
    expect(registry.snapshot().sessions.map(({ identity }) => identity)).toEqual([2, 3]);
    expect(registry.snapshot().activeSessions).toBeLessThanOrEqual(WEB_GPU_METRICS_MAX_SESSIONS);
  });

  it("contains metric factories, observers, and disposal", () => {
    const registry = createWebGpuMetricsRegistry({
      createMetrics: () => createWebGpuPresentationMetrics({ now: () => 4 }),
    });
    const session = registry.open("safe");
    session.metrics.startSession();
    expect(() => registry.report(() => { throw new Error("probe failed"); })).not.toThrow();
    registry.dispose();
    expect(registry.snapshot().activeSessions).toBe(0);
    expect(() => session.dispose()).not.toThrow();
  });

  it("contains hostile factories, snapshots, and reset hooks", () => {
    const registry = createWebGpuMetricsRegistry({
      createMetrics: () => { throw new Error("factory"); },
    });
    expect(() => registry.open("factory-fallback")).not.toThrow();

    const broken = createWebGpuMetricsRegistry({
      createMetrics: () => ({
        startSession: () => 1, startFrame: () => 0,
        recordFrameSubmitted() {}, recordFrameSubmissionDuration() {},
        recordPresentationSuccess() {}, recordPresentationFailure() {},
        reset: () => { throw new Error("reset"); },
        snapshot: () => { throw new Error("snapshot"); },
      }),
    });
    const session = broken.open("broken");
    expect(broken.snapshot()).toMatchObject({ activeSessions: 1, submittedFrames: 0 });
    expect(() => session.dispose()).not.toThrow();
    expect(() => broken.dispose()).not.toThrow();
  });

  it("creates detached snapshots that observers cannot mutate", () => {
    const observer = vi.fn((report: { sessions: readonly unknown[] }) =>
      (report.sessions as unknown[]).splice(0));
    const registry = createWebGpuMetricsRegistry();
    registry.open("vessel");
    registry.report(observer);
    expect(registry.snapshot().activeSessions).toBe(1);
  });
});
