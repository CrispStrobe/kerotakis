/** GUI-098 browser renderer seam: optional GPU pixels, never scene authority. */

import type { IgnitionFlameUniforms } from "./ignitionFlameUniforms";
import {
  IGNITION_FLAME_UNIFORM_BYTES,
  IGNITION_FLAME_UNIFORM_FLOATS,
  IGNITION_FLAME_WGSL,
  writeIgnitionFlameUniformBuffer,
} from "./ignitionFlameShader";
import type { VisualBackendDecision } from "./visualBackend";
import type { WebGpuDeviceLike, WebGpuLifecycleState } from "./webGpuLifecycle";
import type { WebGpuPresentationMetrics } from "./webGpuMetrics";

/** A pre-built GPU effect supplied by the approved shader/pipeline module. */
export interface WebGpuFrameSurface {
  configure(device: WebGpuDeviceLike): void | Promise<void>;
  width(): number;
  height(): number;
  /** The view is stable and reused for the lifetime of this adapter. */
  writeIgnitionUniforms(values: Float32Array): void;
  /** True only after commands were submitted and a canvas frame was presented. */
  present(): boolean;
  reset(): void;
}

export interface AnimationSchedulerLike {
  request(callback: () => void): number;
  cancel(handle: number): void;
}

export interface WebGpuRendererAdapter {
  /** Reconcile lifecycle and environment-policy output synchronously. */
  sync(
    lifecycle: WebGpuLifecycleState,
    decision: VisualBackendDecision,
    uniforms: IgnitionFlameUniforms,
  ): void;
  dispose(): void;
  fallbackVisible(): boolean;
}

export interface WebGpuRendererOptions {
  surface: WebGpuFrameSurface;
  scheduler: AnimationSchedulerLike;
  setFallbackVisible(visible: boolean): void;
  /** Monotonic presentation clock; injected so headless tests need no browser. */
  nowSeconds(): number;
  /** Optional bounded telemetry sink; failures never control rendering. */
  metrics?: WebGpuPresentationMetrics;
}

/**
 * Bridges the existing lifecycle/environment policy to one approved effect.
 *
 * The SVG renderer begins visible and remains visible during acquisition,
 * pipeline setup, and the first submission. Device loss or any policy
 * constraint restores it synchronously. The animation callback and uniform
 * storage are allocated once; the hot path performs no IPC, readback, or
 * adapter-owned allocation.
 */
export function createWebGpuRendererAdapter(options: WebGpuRendererOptions): WebGpuRendererAdapter {
  const packed = new Float32Array(IGNITION_FLAME_UNIFORM_FLOATS);
  let device: WebGpuDeviceLike | undefined;
  let currentUniforms: IgnitionFlameUniforms | undefined;
  let frameHandle: number | undefined;
  let configureGeneration = 0;
  let renderGeneration = 0;
  let configuring = false;
  let fallback = true;
  let disposed = false;

  const metric = (record: (metrics: WebGpuPresentationMetrics) => void): void => {
    if (!options.metrics) return;
    try { record(options.metrics); } catch { /* telemetry cannot affect presentation */ }
  };
  const startMetricFrame = (): number | undefined => {
    if (!options.metrics) return undefined;
    try { return options.metrics.startFrame(); } catch { return undefined; }
  };

  const showFallback = (visible: boolean): void => {
    if (fallback === visible) return;
    fallback = visible;
    try { options.setFallbackVisible(visible); } catch { /* fallback state still fails closed */ }
  };

  const cancelFrame = (): void => {
    if (frameHandle === undefined) return;
    try { options.scheduler.cancel(frameHandle); } catch { /* a generation guard also invalidates it */ }
    frameHandle = undefined;
  };

  const deactivate = (): void => {
    showFallback(true);
    renderGeneration += 1;
    cancelFrame();
    if (device !== undefined) {
      try { options.surface.reset(); } catch { /* fallback is already visible */ }
      device = undefined;
    }
    configureGeneration += 1;
    configuring = false;
  };

  const startFrames = (): void => {
    if (disposed || device === undefined || currentUniforms?.active !== true) return;
    const ownGeneration = ++renderGeneration;
    // Allocated once per activation; every frame reuses this callback and buffer.
    const frame = (): void => {
      frameHandle = undefined;
      if (disposed || ownGeneration !== renderGeneration || device === undefined || currentUniforms?.active !== true) return;
      try {
        const frameStartedAtMs = startMetricFrame();
        writeIgnitionFlameUniformBuffer(
          packed,
          currentUniforms,
          options.nowSeconds(),
          options.surface.width(),
          options.surface.height(),
        );
        options.surface.writeIgnitionUniforms(packed);
        // Before handoff, a surface may legitimately skip its first texture;
        // SVG remains correct and we retry. Once GPU pixels are visible, a
        // failed presentation is a lost/invalid surface and must restore SVG.
        if (options.surface.present()) {
          if (frameStartedAtMs !== undefined) metric((metrics) => metrics.recordFrameSubmitted(frameStartedAtMs));
          metric((metrics) => metrics.recordPresentationSuccess());
          showFallback(false);
        } else if (!fallback) {
          metric((metrics) => metrics.recordPresentationFailure());
          deactivate();
          return;
        }
        frameHandle = options.scheduler.request(frame);
      } catch {
        metric((metrics) => metrics.recordPresentationFailure());
        deactivate();
      }
    };
    try {
      frameHandle = options.scheduler.request(frame);
    } catch {
      deactivate();
    }
  };

  // The fallback is a correctness layer, not an acquisition error screen.
  try { options.setFallbackVisible(true); } catch { /* internal state remains fallback-first */ }

  return {
    sync(lifecycle, decision, uniforms): void {
      if (disposed) return;
      if (decision.backend !== "webgpu" || lifecycle.status !== "ready") {
        deactivate();
        return;
      }

      currentUniforms = uniforms;
      if (!uniforms.active) {
        deactivate();
        return;
      }

      if (device !== lifecycle.device) {
        deactivate();
        device = lifecycle.device;
        metric((metrics) => { metrics.startSession(); });
        const ownGeneration = ++configureGeneration;
        let configured: void | Promise<void>;
        try {
          configured = options.surface.configure(device);
        } catch {
          deactivate();
          return;
        }
        if (configured && typeof configured.then === "function") {
          configuring = true;
          void configured.then(
            () => {
              if (!disposed && ownGeneration === configureGeneration && device === lifecycle.device) {
                configuring = false;
                startFrames();
              }
            },
            () => {
              if (ownGeneration === configureGeneration) deactivate();
            },
          );
          return;
        }
      }
      if (!configuring && frameHandle === undefined) startFrames();
    },

    dispose(): void {
      if (disposed) return;
      disposed = true;
      deactivate();
    },

    fallbackVisible: () => fallback,
  };
}

interface GpuCompilationInfoLike { messages: ArrayLike<{ type: string; message: string }> }
interface GpuShaderModuleLike { getCompilationInfo: () => Promise<GpuCompilationInfoLike> }
interface GpuBufferLike { destroy?: () => void }
interface GpuPipelineLike { getBindGroupLayout(index: number): unknown }
interface BrowserGpuDeviceLike extends WebGpuDeviceLike {
  createShaderModule(descriptor: unknown): GpuShaderModuleLike;
  createBuffer(descriptor: unknown): GpuBufferLike;
  createRenderPipeline(descriptor: unknown): GpuPipelineLike;
  createBindGroup(descriptor: unknown): unknown;
  createCommandEncoder(): {
    beginRenderPass(descriptor: unknown): { setPipeline(value: unknown): void; setBindGroup(index: number, value: unknown): void; draw(count: number): void; end(): void };
    finish(): unknown;
  };
  queue: { writeBuffer(buffer: unknown, offset: number, data: Float32Array): void; submit(commands: unknown[]): void };
}
export interface WebGpuCanvasContextLike {
  configure(descriptor: unknown): void;
  getCurrentTexture(): { createView(): unknown };
  unconfigure?(): void;
}
export interface WebGpuCanvasLike {
  readonly width: number;
  readonly height: number;
  getContext(kind: "webgpu"): WebGpuCanvasContextLike | null;
}

/** Minimal browser WebGPU implementation for the approved ignition shader. */
export function createBrowserIgnitionFlameSurface(canvas: WebGpuCanvasLike, format: string): WebGpuFrameSurface {
  const context = canvas.getContext("webgpu");
  if (!context) throw new Error("webgpu canvas context unavailable");
  const values = new Float32Array(IGNITION_FLAME_UNIFORM_FLOATS);
  const submissions: unknown[] = [undefined];
  const attachment = { view: undefined as unknown, loadOp: "clear", storeOp: "store", clearValue: { r: 0, g: 0, b: 0, a: 0 } };
  const renderPassDescriptor = { colorAttachments: [attachment] };
  let gpu: BrowserGpuDeviceLike | undefined;
  let buffer: GpuBufferLike | undefined;
  let pipeline: GpuPipelineLike | undefined;
  let bindGroup: unknown;
  let surfaceGeneration = 0;
  const clearResources = (): void => {
    buffer?.destroy?.();
    gpu = undefined;
    buffer = undefined;
    pipeline = undefined;
    bindGroup = undefined;
  };

  return {
    async configure(device): Promise<void> {
      const ownGeneration = ++surfaceGeneration;
      clearResources();
      const candidate = device as BrowserGpuDeviceLike;
      try {
        const module = candidate.createShaderModule({ code: IGNITION_FLAME_WGSL });
        const compiler = Reflect.get(module, "getCompilationInfo");
        if (typeof compiler !== "function") {
          throw new Error("WebGPU shader compilation information is unavailable");
        }
        const info = await Reflect.apply(compiler, module, []);
        if (ownGeneration !== surfaceGeneration) return;
        const errors = Array.from(info.messages).filter((message) => message.type === "error");
        if (errors.length > 0) throw new Error(`ignition flame WGSL failed: ${errors.map((error) => error.message).join("; ")}`);
        buffer = candidate.createBuffer({ size: IGNITION_FLAME_UNIFORM_BYTES, usage: 0x40 | 0x08 });
        pipeline = candidate.createRenderPipeline({
          layout: "auto",
          vertex: { module, entryPoint: "vertex_main" },
          fragment: {
            module,
            entryPoint: "fragment_main",
            targets: [{ format, blend: { color: { srcFactor: "one", dstFactor: "one-minus-src-alpha" }, alpha: { srcFactor: "one", dstFactor: "one-minus-src-alpha" } } }],
          },
          primitive: { topology: "triangle-list" },
        });
        bindGroup = candidate.createBindGroup({ layout: pipeline.getBindGroupLayout(0), entries: [{ binding: 0, resource: { buffer } }] });
        context.configure({ device, format, alphaMode: "premultiplied" });
        gpu = candidate;
      } catch (error) {
        if (ownGeneration === surfaceGeneration) clearResources();
        throw error;
      }
    },
    width: () => canvas.width,
    height: () => canvas.height,
    writeIgnitionUniforms(next): void { values.set(next); },
    present(): boolean {
      if (!gpu || !buffer || !pipeline || !bindGroup) return false;
      gpu.queue.writeBuffer(buffer, 0, values);
      attachment.view = context.getCurrentTexture().createView();
      const encoder = gpu.createCommandEncoder();
      const pass = encoder.beginRenderPass(renderPassDescriptor);
      pass.setPipeline(pipeline);
      pass.setBindGroup(0, bindGroup);
      pass.draw(3);
      pass.end();
      submissions[0] = encoder.finish();
      gpu.queue.submit(submissions);
      return true;
    },
    reset(): void {
      surfaceGeneration += 1;
      clearResources();
      context.unconfigure?.();
    },
  };
}

/** Browser scheduler kept structural for SSR/headless tests. */
export function browserAnimationScheduler(globalObject: unknown = globalThis): AnimationSchedulerLike | null {
  if (typeof globalObject !== "object" || globalObject === null) return null;
  const request = Reflect.get(globalObject, "requestAnimationFrame");
  const cancel = Reflect.get(globalObject, "cancelAnimationFrame");
  if (typeof request !== "function" || typeof cancel !== "function") return null;
  return {
    request: (callback) => Reflect.apply(request, globalObject, [callback]),
    cancel: (handle) => Reflect.apply(cancel, globalObject, [handle]),
  };
}
