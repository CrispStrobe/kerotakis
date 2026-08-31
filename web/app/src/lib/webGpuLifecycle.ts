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
  /** Browser-selected presentation format, when this provider can resolve it safely. */
  preferredCanvasFormat?(): string | null;
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
    try { device.destroy?.(); } catch { /* lifecycle still transitions to fallback */ }
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
  try {
    if (typeof globalObject !== "object" || globalObject === null) return null;
    const navigatorValue = Reflect.get(globalObject, "navigator");
    if (typeof navigatorValue !== "object" || navigatorValue === null) return null;
    const gpu = Reflect.get(navigatorValue, "gpu");
    if (typeof gpu !== "object" || gpu === null) return null;
    const requestAdapter = Reflect.get(gpu, "requestAdapter");
    if (typeof requestAdapter !== "function") return null;
    return {
      requestAdapter: () => Reflect.apply(requestAdapter, gpu, []),
      preferredCanvasFormat(): string | null {
        try {
          const resolver = Reflect.get(gpu, "getPreferredCanvasFormat");
          if (typeof resolver !== "function") return null;
          const format: unknown = Reflect.apply(resolver, gpu, []);
          return format === "bgra8unorm" || format === "rgba8unorm" ? format : null;
        } catch {
          return null;
        }
      },
    };
  } catch {
    return null;
  }
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
  snapshot(): WebGpuEnvironmentSnapshot;
  subscribe(listener: (snapshot: WebGpuEnvironmentSnapshot) => void): () => void;
  dispose(): void;
}

/** One coherent view for consumers which need both acquisition and policy state. */
export interface WebGpuEnvironmentSnapshot {
  readonly lifecycle: WebGpuLifecycleState;
  readonly decision: VisualBackendDecision;
  readonly preferredCanvasFormat: string | null;
}

export interface WebGpuEnvironmentPolicyOptions {
  provider: WebGpuProviderLike;
  reducedMotion: MediaQueryListLike;
  document: VisibilityDocumentLike;
  effectApproved?: boolean;
  /** Explicit execution context; headless callers never acquire a device. */
  headless?: boolean;
  onChange?: (decision: VisualBackendDecision) => void;
  /** Published after each lifecycle or policy transition, with no split reads. */
  onSnapshot?: (snapshot: WebGpuEnvironmentSnapshot) => void;
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
  const preferredCanvasFormat = (() => {
    try {
      const format = options.provider.preferredCanvasFormat?.();
      return format === "bgra8unorm" || format === "rgba8unorm" ? format : null;
    } catch {
      return null;
    }
  })();
  let current: VisualBackendDecision = { backend: "lightweight", reason: "effect-not-approved" };
  let currentLifecycle: WebGpuLifecycleState = { status: "idle" };
  let currentSnapshot: WebGpuEnvironmentSnapshot = {
    lifecycle: currentLifecycle,
    decision: current,
    preferredCanvasFormat,
  };
  const subscribers = new Set<(snapshot: WebGpuEnvironmentSnapshot) => void>();

  const constrainedDecision = (): VisualBackendDecision => selectVisualBackend({
    effectApproved: disposed ? false : approved,
    webGpuAvailable: true,
    deviceHealthy: true,
    reducedMotion: options.reducedMotion.matches,
    headless: options.headless ?? false,
    backgrounded: options.document.visibilityState !== "visible",
  });

  const decisionFor = (state: WebGpuLifecycleState): VisualBackendDecision => {
    const constraint = constrainedDecision();
    if (constraint.backend === "lightweight") return constraint;
    if (state.status === "ready") return { backend: "webgpu", reason: "enabled" };
    return { backend: "lightweight", reason: "device-lost" };
  };

  const publish = (state: WebGpuLifecycleState, next: VisualBackendDecision): void => {
    const decisionChanged = next.backend !== current.backend || next.reason !== current.reason;
    currentLifecycle = state;
    current = next;
    currentSnapshot = { lifecycle: state, decision: next, preferredCanvasFormat };
    if (decisionChanged) {
      try { options.onChange?.(next); } catch { /* observers cannot break fail-closed policy */ }
    }
    try { options.onSnapshot?.(currentSnapshot); } catch { /* snapshot is already committed */ }
    for (const subscriber of subscribers) {
      try { subscriber(currentSnapshot); } catch { /* observers cannot break policy */ }
    }
  };

  const decisionEquals = (left: VisualBackendDecision, right: VisualBackendDecision): boolean =>
    left.backend === right.backend && left.reason === right.reason;

  const lifecycle = createWebGpuLifecycle({
    provider: options.provider,
    onChange(state) {
      publish(state, decisionFor(state));
    },
  });

  const reconcile = (): void => {
    if (!active || disposed) return;
    const constraint = constrainedDecision();
    if (constraint.backend === "lightweight") {
      lifecycle.stop();
      // stop() is deliberately idempotent and may have no transition to emit.
      if (currentLifecycle !== lifecycle.state() || !decisionEquals(current, constraint)) {
        publish(lifecycle.state(), constraint);
      }
      return;
    }
    if (lifecycle.state().status === "ready") {
      publish(lifecycle.state(), { backend: "webgpu", reason: "enabled" });
      return;
    }
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
    snapshot: () => currentSnapshot,
    subscribe(listener): () => void {
      if (disposed) {
        try { listener(currentSnapshot); } catch { /* observers cannot break disposed policy */ }
        return () => undefined;
      }
      subscribers.add(listener);
      try { listener(currentSnapshot); } catch { /* subscription remains usable */ }
      let subscribed = true;
      return () => {
        if (!subscribed) return;
        subscribed = false;
        subscribers.delete(listener);
      };
    },
    dispose(): void {
      if (disposed) return;
      disposed = true;
      if (active) {
        options.reducedMotion.removeEventListener("change", environmentChanged);
        options.document.removeEventListener("visibilitychange", environmentChanged);
      }
      active = false;
      lifecycle.stop();
      const fallback: VisualBackendDecision = { backend: "lightweight", reason: "effect-not-approved" };
      if (currentLifecycle !== lifecycle.state() || current.backend !== fallback.backend || current.reason !== fallback.reason) {
        publish(lifecycle.state(), fallback);
      }
      subscribers.clear();
    },
  };
}
