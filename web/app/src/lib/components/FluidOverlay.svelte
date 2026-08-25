<script lang="ts">
  /**
   * GUI-065a on screen: the fluid grid painted over a vessel's liquid
   * while something is happening — an addition's plume, a stir's
   * vortex, layers finding their order — then frozen out and faded,
   * leaving the static render (which IS the engine's answer) in place.
   *
   * The component owns only the canvas and the clock; all meaning lives
   * in fluid.ts / fluidScene.ts, where it is unit-tested. Runs nothing
   * when the user prefers reduced motion, and nothing outside activity
   * windows — a Chromebook's battery is part of the design.
   */
  import type { SceneVessel } from "../host/EngineHost";
  import { relaxToward, step } from "../fluid";
  import { injectStir, simFromScene, paint, type FluidSpecies, type VesselSim } from "../fluidScene";
  import { mulberry32, pourDone, startPour, stepPour, type PourState } from "../pour";
  import type { Effect } from "../magnitudes";

  let {
    vessel,
    effects = [],
    lookup,
  }: {
    vessel: SceneVessel;
    /** The session's transient effects for this vessel. */
    effects?: Effect[];
    /** Species visual data: srgb + density (g/mL), from the shelf. */
    lookup: (key: string) => FluidSpecies;
  } = $props();

  const GRID_W = 50;
  const GRID_H = 70;
  /** Activity window length before freeze-out, ms. */
  const RUN_MS = 2600;
  const FADE_MS = 700;

  const reducedMotion =
    typeof matchMedia !== "undefined" &&
    matchMedia("(prefers-reduced-motion: reduce)").matches;

  let canvas: HTMLCanvasElement | undefined = $state();
  let visible = $state(false);
  let fading = $state(false);

  let sim: VesselSim | null = null;
  let pours: PourState[] = [];
  let raf = 0;
  let ranEffects = new Set<number>();
  // Seeded per run from the effect timestamp: deterministic, replayable.
  let rand: () => number = mulberry32(1);

  // Ambient density: the bottom layer's own — buoyancy is RELATIVE.
  function densities(): number[] {
    if (!sim) return [];
    const ambient = sim.species[0]?.density ?? 1;
    return sim.species.map((s) => s.density / ambient);
  }

  function startRun(kinds: string[]) {
    if (reducedMotion || !canvas) return;
    sim = simFromScene(vessel, GRID_W, GRID_H, 2, lookup);
    if (!sim) return;
    // What just happened enters PHYSICALLY: an add-like effect pours in
    // from above as droplets (GUI-065b — mass ledger-conserved into the
    // grid on surface handoff, splash included); a stir shears the bath.
    rand = mulberry32((kinds.length * 2654435761) ^ Date.now());
    pours = [];
    for (const kind of kinds) {
      if (kind === "swirl") injectStir(sim, 1.2);
      else pours.push(startPour(sim.grid.fields.length - 1, 1.5, 0.45 + 0.1 * Math.random()));
    }
    visible = true;
    fading = false;
    const t0 = performance.now();
    const ctx = canvas.getContext("2d")!;
    const image = ctx.createImageData(GRID_W, GRID_H);
    cancelAnimationFrame(raf);

    const frame = (t: number) => {
      if (!sim) return;
      const elapsed = t - t0;
      const dt = 0.05;
      const d = densities();
      const pouring = pours.some((pp) => !pourDone(pp));
      if (elapsed < RUN_MS || pouring) {
        for (const pp of pours) stepPour(pp, sim, dt, rand);
        pours = pours.filter((pp) => !pourDone(pp));
        step(sim.grid, d, dt, 14, 0.94);
        for (let s = 0; s < sim.grid.fields.length; s++) {
          // Relaxation ramps up through the window: free early, homing
          // late — and held off entirely while a pour is still landing.
          const rate = pouring ? 0.15 : 0.4 + 1.6 * (Math.min(elapsed, RUN_MS) / RUN_MS);
          relaxToward(sim.grid, s, sim.targets[s]!, rate, dt);
        }
      } else {
        // Freeze-out: motion over, relaxation completes, then fade.
        for (let s = 0; s < sim.grid.fields.length; s++) {
          relaxToward(sim.grid, s, sim.targets[s]!, 3, dt);
        }
        if (!fading && elapsed > RUN_MS + 400) {
          fading = true;
          setTimeout(() => {
            visible = false;
            sim = null;
          }, FADE_MS);
        }
      }
      paint(sim, image.data);
      ctx.putImageData(image, 0, 0);
      // Droplets in flight, in their species' colour, over the fields.
      for (const pp of pours) {
        for (const drop of pp.droplets) {
          const c = sim.species[drop.s]!.srgb;
          ctx.fillStyle = `rgba(${c[0]},${c[1]},${c[2]},${drop.ejecta ? 0.6 : 0.95})`;
          ctx.beginPath();
          ctx.arc(drop.x, drop.y, drop.ejecta ? 0.5 : 0.9, 0, Math.PI * 2);
          ctx.fill();
        }
      }
      raf = requestAnimationFrame(frame);
    };
    raf = requestAnimationFrame(frame);
  }

  // Effects arriving for this vessel trigger a run; each firing once.
  $effect(() => {
    const fresh = effects.filter(
      (e) =>
        !ranEffects.has(e.at) &&
        Date.now() - e.at < 3000 &&
        ["dissolve", "swirl", "drip", "precipitate"].includes(e.kind),
    );
    if (fresh.length === 0) return;
    for (const e of fresh) ranEffects.add(e.at);
    if (ranEffects.size > 64) ranEffects = new Set([...ranEffects].slice(-32));
    startRun(fresh.map((e) => e.kind));
  });

  $effect(() => () => cancelAnimationFrame(raf));
</script>

{#if visible}
  <foreignObject x="0" y="0" width="100" height="140" class:fading>
    <canvas bind:this={canvas} width={GRID_W} height={GRID_H}></canvas>
  </foreignObject>
{:else}
  <!-- The canvas must exist before a run can start. -->
  <foreignObject x="0" y="0" width="100" height="140" style="opacity:0">
    <canvas bind:this={canvas} width={GRID_W} height={GRID_H}></canvas>
  </foreignObject>
{/if}

<style>
  foreignObject {
    transition: opacity 0.7s ease-out;
    pointer-events: none;
  }
  foreignObject.fading {
    opacity: 0;
  }
  canvas {
    width: 100%;
    height: 100%;
    image-rendering: auto;
  }
</style>
