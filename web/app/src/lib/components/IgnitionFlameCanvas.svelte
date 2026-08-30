<script module lang="ts">
  export const IGNITION_FLAME_LOGICAL_WIDTH = 48;
  export const IGNITION_FLAME_LOGICAL_HEIGHT = 56;
  export const IGNITION_FLAME_MAX_DPR = 2;

  /** Fixed logical target with a hard physical-pixel ceiling. */
  export function ignitionFlameCanvasSize(devicePixelRatio: unknown): { width: number; height: number } {
    const value = typeof devicePixelRatio === "number" && Number.isFinite(devicePixelRatio)
      ? devicePixelRatio
      : 1;
    const dpr = Math.max(1, Math.min(IGNITION_FLAME_MAX_DPR, value));
    return {
      width: Math.round(IGNITION_FLAME_LOGICAL_WIDTH * dpr),
      height: Math.round(IGNITION_FLAME_LOGICAL_HEIGHT * dpr),
    };
  }
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import type { Effect } from "../magnitudes";
  import { ignitionFlameUniforms } from "../ignitionFlameUniforms";
  import type { VisualBackendDecision } from "../visualBackend";
  import type { WebGpuLifecycleState } from "../webGpuLifecycle";
  import {
    browserAnimationScheduler,
    createBrowserIgnitionFlameSurface,
    createWebGpuRendererAdapter,
    type WebGpuRendererAdapter,
  } from "../webGpuRenderer";

  /** One coherent policy/lifecycle observation; callers must not tear the pair. */
  export interface IgnitionFlameGpuSnapshot {
    lifecycle: WebGpuLifecycleState;
    decision: VisualBackendDecision;
  }

  let {
    effect: flameEffect,
    vesselIdentity,
    gpu,
    onfallbackchange,
  }: {
    effect: Effect | null | undefined;
    vesselIdentity: number | string;
    gpu: IgnitionFlameGpuSnapshot;
    /** Reports whether the sibling SVG correctness layer must be visible. */
    onfallbackchange?: (visible: boolean) => void;
  } = $props();

  let canvas = $state<HTMLCanvasElement>();
  let adapter = $state<WebGpuRendererAdapter | null>(null);
  let gpuPresented = $state(false);

  const publishFallback = (visible: boolean): void => {
    gpuPresented = !visible;
    onfallbackchange?.(visible);
  };

  const resizeBackingStore = (): void => {
    if (!canvas) return;
    const size = ignitionFlameCanvasSize(globalThis.devicePixelRatio);
    canvas.width = size.width;
    canvas.height = size.height;
  };

  $effect(() => {
    adapter?.sync(
      gpu.lifecycle,
      gpu.decision,
      ignitionFlameUniforms({
        effect: flameEffect,
        vesselIdentity,
        reducedMotion: gpu.decision.reason === "reduced-motion",
      }),
    );
  });

  onMount(() => {
    resizeBackingStore();
    const scheduler = browserAnimationScheduler();
    if (!canvas || !scheduler) {
      publishFallback(true);
      return;
    }
    try {
      const navigatorValue = Reflect.get(globalThis, "navigator") as object | undefined;
      const gpu = navigatorValue ? Reflect.get(navigatorValue, "gpu") : undefined;
      const preferred = gpu && typeof gpu === "object"
        ? Reflect.get(gpu, "getPreferredCanvasFormat")
        : undefined;
      const format = typeof preferred === "function"
        ? String(Reflect.apply(preferred, gpu, []))
        : "bgra8unorm";
      const surface = createBrowserIgnitionFlameSurface(canvas, format);
      adapter = createWebGpuRendererAdapter({
        surface,
        scheduler,
        setFallbackVisible: publishFallback,
        nowSeconds: () => performance.now() / 1000,
      });
    } catch {
      publishFallback(true);
    }
    globalThis.addEventListener("resize", resizeBackingStore);
    return () => {
      globalThis.removeEventListener("resize", resizeBackingStore);
      adapter?.dispose();
      adapter = null;
      publishFallback(true);
    };
  });
</script>

<canvas
  bind:this={canvas}
  class:presented={gpuPresented}
  aria-hidden="true"
  data-visual-backend={gpuPresented ? "webgpu" : "lightweight"}
></canvas>

<style>
  canvas {
    display: block;
    width: 48px;
    height: 56px;
    opacity: 0;
    pointer-events: none;
  }
  canvas.presented { opacity: 1; }
</style>
