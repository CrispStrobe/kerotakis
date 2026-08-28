<script lang="ts">
  import { t } from "../i18n.svelte";

  export type RoomStyle = "discovery" | "research" | "orbital";

  let {
    value,
    onchange,
    onclose,
  }: {
    value: RoomStyle;
    onchange: (value: RoomStyle) => void;
    onclose: () => void;
  } = $props();

  const rooms: { id: RoomStyle; name: string; detail: string; icon: string }[] = [
    { id: "discovery", name: "discovery studio", detail: "Warm workshop colours for playful, open-ended investigation.", icon: "✦" },
    { id: "research", name: "research laboratory", detail: "A calm, neutral room for dense measurement and professional work.", icon: "⌁" },
    { id: "orbital", name: "orbital laboratory", detail: "A bright space-station lab with cool panels and blue light.", icon: "◉" },
  ];
</script>

<div class="scrim" role="presentation" onclick={onclose} onkeydown={(event) => event.key === "Escape" && onclose()}>
  <dialog open aria-modal="true" aria-labelledby="room-title" onclick={(event) => event.stopPropagation()} onkeydown={(event) => event.stopPropagation()}>
    <header>
      <span class="mark" aria-hidden="true">▱</span>
      <span><small>{t("laboratory environment")}</small><h2 id="room-title">{t("choose a lab room")}</h2></span>
      <button class="close" aria-label={t("close")} onclick={onclose}>×</button>
    </header>

    <p class="lead">{t("Change the room without changing your vessels, evidence, or chemistry.")}</p>

    <div class="rooms" role="radiogroup" aria-label={t("laboratory environment")}>
      {#each rooms as room (room.id)}
        <button
          class="room-card"
          class:selected={value === room.id}
          data-room={room.id}
          role="radio"
          aria-checked={value === room.id}
          onclick={() => onchange(room.id)}
        >
          <span class="preview" aria-hidden="true">
            <i class="window">{room.icon}</i>
            <i class="shelf"></i>
            <i class="counter"></i>
            <i class="vessel"></i>
          </span>
          <span class="copy"><strong>{t(room.name)}</strong><small>{t(room.detail)}</small></span>
          <span class="choice" aria-hidden="true">{value === room.id ? "✓" : "○"}</span>
        </button>
      {/each}
    </div>
  </dialog>
</div>

<style>
  .scrim { position: fixed; inset: 0; z-index: 82; display: grid; place-items: center; padding: 1rem; background: var(--scrim); backdrop-filter: blur(12px) saturate(1.15); }
  dialog { position: static; width: min(52rem, 94vw); margin: 0; padding: 0; overflow: hidden; border: 1px solid color-mix(in srgb, var(--instrument) 44%, var(--edge)); border-radius: 24px; color: var(--ink); background: var(--surface); box-shadow: 0 28px 80px var(--overlay-shadow); }
  header { display: flex; align-items: center; gap: .75rem; padding: 1rem 1.1rem; background: linear-gradient(110deg, color-mix(in srgb, var(--instrument) 15%, var(--surface)), color-mix(in srgb, var(--action) 10%, var(--surface))); }
  header > span:nth-child(2) { display: grid; gap: .05rem; }
  header small { color: var(--instrument); font-size: .58rem; font-weight: 850; letter-spacing: .11em; text-transform: uppercase; }
  h2 { margin: 0; font-size: 1.18rem; }
  .mark { width: 42px; height: 42px; display: grid; place-items: center; border-radius: 13px; color: var(--on-accent); background: linear-gradient(145deg, var(--instrument), var(--primary)); font-size: 1.25rem; }
  .close { width: 38px; height: 38px; margin-left: auto; border: 1px solid var(--edge); border-radius: 50%; color: var(--ink); background: var(--surface); cursor: pointer; font: inherit; font-size: 1.2rem; }
  .lead { margin: 0; padding: 1rem 1.1rem .3rem; color: var(--dim); font-size: .8rem; }
  .rooms { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: .75rem; padding: .8rem 1.1rem 1.2rem; }
  .room-card { min-width: 0; display: grid; grid-template-rows: 7.3rem 1fr; position: relative; padding: 0; overflow: hidden; border: 1px solid var(--edge); border-radius: 17px; color: var(--ink); background: var(--surface-raised); text-align: left; cursor: pointer; font: inherit; transition: transform 160ms ease, border-color 160ms ease, box-shadow 160ms ease; }
  .room-card:hover, .room-card:focus-visible { transform: translateY(-2px); border-color: var(--primary); box-shadow: 0 12px 25px var(--shadow); }
  .room-card.selected { border: 2px solid var(--primary); box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 12%, transparent); }
  .preview { position: relative; overflow: hidden; background: linear-gradient(var(--room-discovery-wall) 0 66%, var(--room-discovery-trim) 66% 72%, var(--room-discovery-front) 72%); }
  [data-room="research"] .preview { background: linear-gradient(var(--room-research-wall) 0 66%, var(--room-research-trim) 66% 72%, var(--room-research-front) 72%); }
  [data-room="orbital"] .preview { background: linear-gradient(145deg, var(--room-orbital-wall) 0 25%, var(--room-orbital-glow) 25% 29%, color-mix(in srgb, var(--room-orbital-wall) 30%, var(--surface)) 29% 66%, var(--room-orbital-trim) 66% 72%, var(--room-orbital-front) 72%); }
  .window { position: absolute; inset: .8rem .9rem auto; height: 2.9rem; display: grid; place-items: center; border: 3px solid var(--room-window-edge); border-radius: 12px; color: var(--discovery); background: var(--room-window); font-style: normal; font-size: 1.25rem; }
  .shelf { position: absolute; left: 1rem; right: 1rem; top: 4.15rem; height: .32rem; border-radius: 99px; background: var(--room-fixture); }
  .counter { position: absolute; inset: auto 0 0; height: 2.05rem; background: color-mix(in srgb, var(--room-window) 40%, transparent); }
  .vessel { position: absolute; width: 1.55rem; height: 1.8rem; left: calc(50% - .75rem); bottom: .45rem; border: 3px solid var(--room-glass-edge); border-top: 0; border-radius: 0 0 7px 7px; }
  .copy { min-width: 0; display: grid; align-content: start; gap: .25rem; padding: .75rem 2rem .85rem .75rem; }
  .copy strong { font-size: .78rem; }
  .copy small { color: var(--dim); font-size: .65rem; line-height: 1.4; }
  .choice { position: absolute; right: .65rem; bottom: .65rem; color: var(--primary); font-weight: 900; }
  @media (max-width: 650px) { .rooms { grid-template-columns: 1fr; max-height: 66dvh; overflow: auto; } .room-card { grid-template-columns: 7rem 1fr; grid-template-rows: 7rem; } }
</style>
