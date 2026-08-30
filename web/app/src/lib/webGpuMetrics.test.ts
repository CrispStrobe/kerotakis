import { describe, expect, it } from "vitest";

import { createWebGpuPresentationMetrics, WEB_GPU_METRICS_CAPACITY } from "./webGpuMetrics";

describe("WebGPU presentation metrics", () => {
  it("records configure-to-first-presentation latency only once", () => {
    let clock = 100;
    const metrics = createWebGpuPresentationMetrics({ now: () => clock });
    expect(metrics.startSession()).toBe(1);
    clock = 125;
    metrics.recordPresentationSuccess();
    clock = 180;
    metrics.recordPresentationSuccess();
    expect(metrics.snapshot()).toMatchObject({
      session: 1, configureStartedAtMs: 100, firstPresentationLatencyMs: 25, successfulPresentations: 2,
    });
  });

  it("measures CPU submission durations and counts failures", () => {
    let clock = 10;
    const metrics = createWebGpuPresentationMetrics({ now: () => clock });
    metrics.startSession();
    const frameStart = metrics.startFrame();
    clock = 13.5;
    metrics.recordFrameSubmitted(frameStart);
    metrics.recordPresentationFailure();
    expect(metrics.snapshot()).toMatchObject({ submittedFrames: 1, presentationFailures: 1, frameCpuSubmissionP95Ms: 3.5, frameCpuSubmissionMaxMs: 3.5 });
  });

  it("retains only the newest fixed-capacity ring samples after wraparound", () => {
    const metrics = createWebGpuPresentationMetrics({ capacity: 4 });
    [100, 200, 1, 2, 3, 4].forEach((value) => metrics.recordFrameSubmissionDuration(value));
    expect(metrics.snapshot()).toMatchObject({ submittedFrames: 6, retainedFrameSamples: 4, frameCpuSubmissionP95Ms: 4, frameCpuSubmissionMaxMs: 4 });
  });

  it("uses deterministic nearest-rank p95 calculated on snapshot", () => {
    const metrics = createWebGpuPresentationMetrics({ capacity: 20 });
    for (let value = 20; value >= 1; value -= 1) metrics.recordFrameSubmissionDuration(value);
    expect(metrics.snapshot().frameCpuSubmissionP95Ms).toBe(19);
    metrics.recordFrameSubmissionDuration(100);
    // The oldest sample (20) is evicted; nearest-rank p95 remains 19.
    expect(metrics.snapshot().frameCpuSubmissionP95Ms).toBe(19);
  });

  it("clamps backwards, non-finite, negative, and throwing timing inputs", () => {
    const readings = [Number.NaN, 5, 3, Number.POSITIVE_INFINITY];
    const metrics = createWebGpuPresentationMetrics({ now: () => {
      const value = readings.shift();
      if (value === undefined) throw new Error("clock unavailable");
      return value;
    } });
    metrics.startSession();
    const start = metrics.startFrame();
    metrics.recordFrameSubmitted(start);
    metrics.recordFrameSubmissionDuration(-4);
    metrics.recordFrameSubmissionDuration(Number.NaN);
    metrics.recordPresentationSuccess();
    expect(metrics.snapshot()).toMatchObject({ configureStartedAtMs: 0, firstPresentationLatencyMs: 0, submittedFrames: 3, frameCpuSubmissionP95Ms: 0, frameCpuSubmissionMaxMs: 0 });
  });

  it("resets counters in place and starts monotonically numbered sessions", () => {
    const metrics = createWebGpuPresentationMetrics({ capacity: 0, now: () => 7 });
    metrics.recordFrameSubmissionDuration(2);
    metrics.recordPresentationFailure();
    metrics.reset();
    expect(metrics.snapshot()).toMatchObject({ session: 0, submittedFrames: 0, retainedFrameSamples: 0, presentationFailures: 0 });
    expect(metrics.startSession()).toBe(1);
    metrics.recordFrameSubmissionDuration(9);
    expect(metrics.startSession()).toBe(2);
    expect(metrics.snapshot()).toMatchObject({ session: 2, submittedFrames: 0 });
    expect(WEB_GPU_METRICS_CAPACITY).toBe(120);
  });
});
