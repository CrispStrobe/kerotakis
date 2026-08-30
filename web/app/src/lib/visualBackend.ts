/** GUI-098: select optional GPU presentation without changing scene truth. */

export type VisualBackend = "lightweight" | "webgpu";
export type VisualBackendReason =
  | "enabled"
  | "effect-not-approved"
  | "webgpu-unavailable"
  | "device-lost"
  | "reduced-motion"
  | "headless"
  | "backgrounded";

export interface VisualBackendCapabilities {
  /** True only after a named WGSL effect passes GUI-098's acceptance gate. */
  effectApproved: boolean;
  webGpuAvailable: boolean;
  deviceHealthy: boolean;
  reducedMotion: boolean;
  headless: boolean;
  backgrounded: boolean;
}

export interface VisualBackendDecision {
  backend: VisualBackend;
  reason: VisualBackendReason;
}

/** Accessibility and execution constraints always win over GPU capability. */
export function selectVisualBackend(capabilities: VisualBackendCapabilities): VisualBackendDecision {
  if (capabilities.reducedMotion) return { backend: "lightweight", reason: "reduced-motion" };
  if (capabilities.headless) return { backend: "lightweight", reason: "headless" };
  if (capabilities.backgrounded) return { backend: "lightweight", reason: "backgrounded" };
  if (!capabilities.effectApproved) return { backend: "lightweight", reason: "effect-not-approved" };
  if (!capabilities.webGpuAvailable) return { backend: "lightweight", reason: "webgpu-unavailable" };
  if (!capabilities.deviceHealthy) return { backend: "lightweight", reason: "device-lost" };
  return { backend: "webgpu", reason: "enabled" };
}

/** Feature detection only; the renderer owns adapter and device lifetime. */
export function hasWebGpu(globalObject: unknown = globalThis): boolean {
  if (typeof globalObject !== "object" || globalObject === null) return false;
  const navigatorValue = Reflect.get(globalObject, "navigator");
  if (typeof navigatorValue !== "object" || navigatorValue === null) return false;
  return Reflect.get(navigatorValue, "gpu") !== undefined;
}
