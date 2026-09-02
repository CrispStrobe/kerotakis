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
  import type { WebGpuEnvironmentSnapshot } from "../webGpuLifecycle";
  import type { WebGpuMetricsRegistry, WebGpuMetricsSession } from "../webGpuMetricsRegistry";
  import {
    browserAnimationScheduler,
    createBrowserIgnitionFlameSurface,
    createWebGpuRendererAdapter,
    type AnimationSchedulerLike,
    type WebGpuRendererAdapter,
  } from "../webGpuRenderer";

  /** One coherent policy/lifecycle observation; callers must not tear the pair. */
  export type IgnitionFlameGpuSnapshot = WebGpuEnvironmentSnapshot;

  let {
    effect: flameEffect,
    vesselIdentity,
    gpu,
    metricsRegistry,
    onfallbackchange,
  }: {
    effect: Effect | null | undefined;
    vesselIdentity: number | string;
    gpu: IgnitionFlameGpuSnapshot;
    metricsRegistry: WebGpuMetricsRegistry;
    /** Reports whether the sibling SVG correctness layer must be visible. */
    onfallbackchange?: (visible: boolean) => void;
  } = $props();

  let canvas = $state<HTMLCanvasElement>();
  let adapter = $state<WebGpuRendererAdapter | null>(null);
  let scheduler = $state<AnimationSchedulerLike | null>(null);
  let mounted = $state(false);
  let gpuPresented = $state(false);
  let metricsSession: WebGpuMetricsSession | null = null;

  const publishFallback = (visible: boolean): void => {
    gpuPresented = !visible;
    try { onfallbackchange?.(visible); } catch { /* visual observers cannot break fallback */ }
  };

  const resizeBackingStore = (): void => {
    if (!canvas) return;
    const size = ignitionFlameCanvasSize(Reflect.get(globalThis, "devicePixelRatio"));
    canvas.width = size.width;
    canvas.height = size.height;
  };

  const initializeSurface = (): void => {
    if (!mounted || adapter || !canvas || !scheduler || !gpu.preferredCanvasFormat) return;
    try {
      const surface = createBrowserIgnitionFlameSurface(canvas, gpu.preferredCanvasFormat);
      adapter = createWebGpuRendererAdapter({
        surface,
        scheduler,
        metrics: metricsSession?.metrics,
        setFallbackVisible: publishFallback,
        nowSeconds: () => {
          const performanceValue = Reflect.get(globalThis, "performance") as object | undefined;
          const now = performanceValue && Reflect.get(performanceValue, "now");
          return typeof now === "function"
            ? Number(Reflect.apply(now, performanceValue, [])) / 1000
            : Date.now() / 1000;
        },
      });
    } catch {
      publishFallback(true);
    }
  };

  $effect(() => {
    initializeSurface();
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
    mounted = true;
    metricsSession = metricsRegistry.open(vesselIdentity);
    resizeBackingStore();
    scheduler = browserAnimationScheduler();
    initializeSurface();
    if (!canvas || !scheduler || !gpu.preferredCanvasFormat) publishFallback(true);
    const addEventListener = Reflect.get(globalThis, "addEventListener");
    const removeEventListener = Reflect.get(globalThis, "removeEventListener");
    if (typeof addEventListener === "function") {
      Reflect.apply(addEventListener, globalThis, ["resize", resizeBackingStore]);
    }
    return () => {
      if (typeof removeEventListener === "function") {
        Reflect.apply(removeEventListener, globalThis, ["resize", resizeBackingStore]);
      }
      adapter?.dispose();
      metricsSession?.dispose();
      metricsSession = null;
      adapter = null;
      scheduler = null;
      mounted = false;
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
