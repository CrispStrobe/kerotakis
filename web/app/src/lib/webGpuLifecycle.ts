/** GUI-098 WebGPU acquisition with deterministic lightweight fallback. */

import { selectVisualBackend, type VisualBackendDecision } from "./visualBackend";

export interface WebGpuDeviceLike {
  lost: Promise<unknown>;
  destroy?: () => void;
}

export interface WebGpuAdapterLike {
  requestDevice(): Promise<WebGpuDeviceLike>;
}

export interface WebGpuProviderLike {
  requestAdapter(): Promise<WebGpuAdapterLike | null>;
}

export type WebGpuFallbackReason =
  | "adapter-unavailable"
  | "adapter-request-failed"
  | "device-request-failed"
  | "device-lost"
  | "stopped";

export type WebGpuLifecycleState =
  | { status: "idle" }
  | { status: "requesting" }
  | { status: "ready"; device: WebGpuDeviceLike }
  | { status: "fallback"; reason: WebGpuFallbackReason };

export interface WebGpuLifecycle {
  start(): Promise<WebGpuLifecycleState>;
  stop(): void;
  state(): WebGpuLifecycleState;
}

export interface WebGpuLifecycleOptions {
  provider: WebGpuProviderLike;
  onChange?: (state: WebGpuLifecycleState) => void;
}

export function createWebGpuLifecycle(options: WebGpuLifecycleOptions): WebGpuLifecycle {
  let current: WebGpuLifecycleState = { status: "idle" };
  let generation = 0;
  let pending: Promise<WebGpuLifecycleState> | undefined;
  const destroyed = new WeakSet<object>();

  const emit = (state: WebGpuLifecycleState): WebGpuLifecycleState => {
    current = state;
    options.onChange?.(state);
    return state;
  };
  const destroy = (device: WebGpuDeviceLike): void => {
    const identity = device as object;
    if (destroyed.has(identity)) return;
    destroyed.add(identity);
    device.destroy?.();
  };

  const start = (): Promise<WebGpuLifecycleState> => {
    if (current.status === "ready") return Promise.resolve(current);
    if (current.status === "requesting" && pending) return pending;
    const ownGeneration = ++generation;
    emit({ status: "requesting" });
    pending = (async (): Promise<WebGpuLifecycleState> => {
      let adapter: WebGpuAdapterLike | null;
      try {
        adapter = await options.provider.requestAdapter();
      } catch {
        return ownGeneration === generation
          ? emit({ status: "fallback", reason: "adapter-request-failed" })
          : current;
      }
      if (ownGeneration !== generation) return current;
      if (!adapter) return emit({ status: "fallback", reason: "adapter-unavailable" });

      let device: WebGpuDeviceLike;
      try {
        device = await adapter.requestDevice();
      } catch {
        return ownGeneration === generation
          ? emit({ status: "fallback", reason: "device-request-failed" })
          : current;
      }
      if (ownGeneration !== generation) {
        destroy(device);
        return current;
      }
      const ready = emit({ status: "ready", device });
      void device.lost.then(
        () => {
          if (ownGeneration !== generation || current.status !== "ready" || current.device !== device) return;
          destroy(device);
          emit({ status: "fallback", reason: "device-lost" });
        },
        () => {
          if (ownGeneration !== generation || current.status !== "ready" || current.device !== device) return;
          destroy(device);
          emit({ status: "fallback", reason: "device-lost" });
        },
      );
      return ready;
    })().finally(() => {
      if (ownGeneration === generation) pending = undefined;
    });
    return pending;
  };

  return {
    start,
    stop(): void {
      generation += 1;
      pending = undefined;
      if (current.status === "ready") destroy(current.device);
      if (current.status !== "fallback" || current.reason !== "stopped") {
        emit({ status: "fallback", reason: "stopped" });
      }
    },
    state: () => current,
  };
}

/** Structural browser adapter: no dependency on ambient WebGPU TS types. */
export function browserWebGpuProvider(globalObject: unknown = globalThis): WebGpuProviderLike | null {
  if (typeof globalObject !== "object" || globalObject === null) return null;
  const navigatorValue = Reflect.get(globalObject, "navigator");
  if (typeof navigatorValue !== "object" || navigatorValue === null) return null;
  const gpu = Reflect.get(navigatorValue, "gpu");
  if (typeof gpu !== "object" || gpu === null) return null;
  const requestAdapter = Reflect.get(gpu, "requestAdapter");
  if (typeof requestAdapter !== "function") return null;
  return { requestAdapter: () => Reflect.apply(requestAdapter, gpu, []) };
}

export interface MediaQueryListLike {
  readonly matches: boolean;
  addEventListener(type: "change", listener: () => void): void;
  removeEventListener(type: "change", listener: () => void): void;
}

export interface VisibilityDocumentLike {
  readonly visibilityState: string;
  addEventListener(type: "visibilitychange", listener: () => void): void;
  removeEventListener(type: "visibilitychange", listener: () => void): void;
}

export interface WebGpuEnvironmentPolicy {
  start(): void;
  setEffectApproved(approved: boolean): void;
  decision(): VisualBackendDecision;
  dispose(): void;
}

export interface WebGpuEnvironmentPolicyOptions {
  provider: WebGpuProviderLike;
  reducedMotion: MediaQueryListLike;
  document: VisibilityDocumentLike;
  effectApproved?: boolean;
  onChange?: (decision: VisualBackendDecision) => void;
}

/**
 * Owns optional GPU work while browser policy permits it. The lightweight
 * renderer remains authoritative until acquisition has completed.
 */
export function createWebGpuEnvironmentPolicy(
  options: WebGpuEnvironmentPolicyOptions,
): WebGpuEnvironmentPolicy {
  let approved = options.effectApproved ?? false;
  let active = false;
  let disposed = false;
  let current: VisualBackendDecision = { backend: "lightweight", reason: "effect-not-approved" };

  const publish = (next: VisualBackendDecision): void => {
    if (next.backend === current.backend && next.reason === current.reason) return;
    current = next;
    options.onChange?.(next);
  };

  const constrainedDecision = (): VisualBackendDecision => selectVisualBackend({
    effectApproved: approved,
    webGpuAvailable: true,
    deviceHealthy: true,
    reducedMotion: options.reducedMotion.matches,
    headless: false,
    backgrounded: options.document.visibilityState !== "visible",
  });

  const lifecycle = createWebGpuLifecycle({
    provider: options.provider,
    onChange(state) {
      if (disposed || constrainedDecision().backend !== "webgpu") return;
      if (state.status === "ready") publish({ backend: "webgpu", reason: "enabled" });
      if (state.status === "fallback" && state.reason === "device-lost") {
        publish({ backend: "lightweight", reason: "device-lost" });
      }
    },
  });

  const reconcile = (): void => {
    if (!active || disposed) return;
    const constraint = constrainedDecision();
    if (constraint.backend === "lightweight") {
      publish(constraint);
      lifecycle.stop();
      return;
    }
    if (lifecycle.state().status === "ready") {
      publish({ backend: "webgpu", reason: "enabled" });
      return;
    }
    // Acquisition and all acquisition failures retain the baseline renderer.
    publish({ backend: "lightweight", reason: "device-lost" });
    void lifecycle.start();
  };

  const environmentChanged = (): void => reconcile();

  return {
    start(): void {
      if (active || disposed) return;
      active = true;
      options.reducedMotion.addEventListener("change", environmentChanged);
      options.document.addEventListener("visibilitychange", environmentChanged);
      reconcile();
    },
    setEffectApproved(next): void {
      if (disposed || approved === next) return;
      approved = next;
      reconcile();
    },
    decision: () => current,
    dispose(): void {
      if (disposed) return;
      disposed = true;
      if (active) {
        options.reducedMotion.removeEventListener("change", environmentChanged);
        options.document.removeEventListener("visibilitychange", environmentChanged);
      }
      active = false;
      lifecycle.stop();
      publish({ backend: "lightweight", reason: "effect-not-approved" });
    },
  };
}
