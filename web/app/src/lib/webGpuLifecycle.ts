/** GUI-098 WebGPU acquisition with deterministic lightweight fallback. */

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
