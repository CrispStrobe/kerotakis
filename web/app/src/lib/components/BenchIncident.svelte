<script lang="ts">
  import { onMount } from "svelte";
  import type { Effect } from "../magnitudes";
  import { t } from "../i18n.svelte";

  let { effect, x, y }: { effect: Effect; x: number; y: number } = $props();
  let visible = $state(true);
  const destination = $derived(`${effect.spill?.surface ?? "bench"} ${effect.spill?.location ?? "unknown"}`);
  const percent = $derived(((effect.spill?.fraction ?? 0) * 100).toFixed(1));
  const label = $derived(effect.kind === "break"
    ? t("vessel broke; contents contained at {destination}", { destination })
    : t("spill contained at {destination}: {percent}% transferred", { destination, percent }));

  onMount(() => {
    const remaining = Math.max(0, (effect.durationMs ?? 5000) - (Date.now() - effect.at));
    const expiry = window.setTimeout(() => (visible = false), remaining);
    return () => window.clearTimeout(expiry);
  });
</script>

{#if visible}
<section
  class="incident"
  class:breakage={effect.kind === "break"}
  style={`left:${x * 100}%;top:${y * 100}%;--incident-scale:${.7 + effect.magnitude * .8}`}
  role="status"
  aria-live="polite"
  aria-label={label}
>
  <span class="pool" aria-hidden="true"></span>
  {#if effect.kind === "break"}
    <span class="shard shard-a" aria-hidden="true"></span>
    <span class="shard shard-b" aria-hidden="true"></span>
    <span class="shard shard-c" aria-hidden="true"></span>
  {/if}
  <span class="incident-label">{label}</span>
</section>
{/if}

<style>
  .incident { position: absolute; z-index: 8; pointer-events: none; transform: translate(-50%, 1.7rem); }
  .pool { display: block; width: calc(5rem * var(--incident-scale)); height: calc(1.5rem * var(--incident-scale)); border: 2px solid color-mix(in srgb, var(--danger) 55%, var(--edge-strong)); border-radius: 48% 52% 45% 55%; background: color-mix(in srgb, var(--danger) 28%, var(--cool)); box-shadow: 0 .3rem .8rem color-mix(in srgb, var(--danger) 22%, transparent); animation: spill-spread .55s ease-out both; }
  .incident-label { display: block; width: max-content; max-width: 12rem; margin-top: .25rem; padding: .2rem .4rem; border-radius: .3rem; background: color-mix(in srgb, var(--surface) 92%, transparent); color: var(--danger); font-size: .68rem; font-weight: 750; }
  .shard { position: absolute; top: -.2rem; left: 50%; width: .65rem; height: .9rem; background: color-mix(in srgb, var(--cloud) 48%, transparent); border: 1px solid var(--edge-strong); clip-path: polygon(50% 0, 100% 100%, 0 78%); animation: shard-burst .62s cubic-bezier(.15,.7,.3,1) both; }
  .shard-a { --dx: -2.4rem; --dy: -1.5rem; --turn: -75deg; }
  .shard-b { --dx: .7rem; --dy: -2.1rem; --turn: 95deg; animation-delay: .04s; }
  .shard-c { --dx: 2.5rem; --dy: -.8rem; --turn: 155deg; animation-delay: .08s; }
  @keyframes spill-spread { from { transform: scale(.2); opacity: .25; } }
  @keyframes shard-burst { from { transform: translate(0, 0) rotate(0); opacity: 1; } to { transform: translate(var(--dx), var(--dy)) rotate(var(--turn)); opacity: .35; } }
  @media (prefers-reduced-motion: reduce) {
    .pool, .shard { animation: none; }
    .shard-a { transform: translate(-1rem, -.6rem) rotate(-35deg); }
    .shard-b { transform: translate(.2rem, -.8rem) rotate(20deg); }
    .shard-c { transform: translate(1.2rem, -.4rem) rotate(50deg); }
  }
</style>
