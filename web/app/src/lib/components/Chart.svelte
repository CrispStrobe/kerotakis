<script lang="ts">
  import {
    bandPath,
    extent,
    linePath,
    niceTicks,
    scale,
    seriesPoints,
    type ChartSpec,
  } from "../chart";

  let { spec }: { spec: ChartSpec } = $props();

  const W = 420;
  const H = 260;
  const M = { top: 26, right: 14, bottom: 42, left: 52 };

  const e = $derived(extent(spec));
  const sx = $derived(scale(e.x, [M.left, W - M.right]));
  const sy = $derived(scale(e.y, [H - M.bottom, M.top]));
  const xTicks = $derived(niceTicks(e.x[0], e.x[1], 6));
  const yTicks = $derived(niceTicks(e.y[0], e.y[1], 5));
  const fmt = (v: number) =>
    Math.abs(v) >= 1e4 || (v !== 0 && Math.abs(v) < 1e-3)
      ? v.toExponential(1)
      : String(Number(v.toPrecision(4)));
  const axisLabel = (a: { label: string; unit?: string }) =>
    a.unit ? `${a.label} (${a.unit})` : a.label;

  let svgEl: SVGSVGElement | undefined = $state();

  /**
   * A serialized copy with the theme baked in. The live SVG is styled by
   * the component's scoped CSS and theme variables — both lost on
   * serialization, leaving an invisible file. So the export walks the
   * clone and inlines each element's COMPUTED style: what you saved is
   * what you saw, in whichever theme you saw it.
   */
  const STYLE_PROPS = [
    "fill",
    "stroke",
    "stroke-width",
    "opacity",
    "font-size",
    "font-family",
    "text-anchor",
  ] as const;

  function styledClone(): SVGSVGElement | null {
    if (!svgEl) return null;
    const clone = svgEl.cloneNode(true) as SVGSVGElement;
    const live = svgEl.querySelectorAll<SVGElement>("*");
    const copied = clone.querySelectorAll<SVGElement>("*");
    live.forEach((el, i) => {
      const computed = getComputedStyle(el);
      const target = copied[i]!;
      for (const prop of STYLE_PROPS) {
        target.style.setProperty(prop, computed.getPropertyValue(prop));
      }
      target.removeAttribute("class");
    });
    // A background, so dark-theme exports do not land on transparency.
    const bg = document.createElementNS("http://www.w3.org/2000/svg", "rect");
    bg.setAttribute("width", String(W));
    bg.setAttribute("height", String(H));
    bg.style.fill = getComputedStyle(svgEl.parentElement!).backgroundColor;
    clone.insertBefore(bg, clone.firstChild);
    clone.setAttribute("width", String(W));
    clone.setAttribute("height", String(H));
    return clone;
  }

  const stem = () => spec.title.replace(/\s+/g, "-") || "chart";

  function save(href: string, name: string) {
    const a = document.createElement("a");
    a.href = href;
    a.download = name;
    a.click();
  }

  function exportSvg() {
    const clone = styledClone();
    if (!clone) return;
    const blob = new Blob([clone.outerHTML], { type: "image/svg+xml" });
    const url = URL.createObjectURL(blob);
    save(url, `${stem()}.svg`);
    URL.revokeObjectURL(url);
  }

  function exportPng() {
    const clone = styledClone();
    if (!clone) return;
    const url = URL.createObjectURL(
      new Blob([clone.outerHTML], { type: "image/svg+xml" }),
    );
    const img = new Image();
    img.onload = () => {
      const canvas = document.createElement("canvas");
      canvas.width = W * 2; // 2x for slide decks and print
      canvas.height = H * 2;
      canvas.getContext("2d")?.drawImage(img, 0, 0, W * 2, H * 2);
      URL.revokeObjectURL(url);
      save(canvas.toDataURL("image/png"), `${stem()}.png`);
    };
    img.onerror = () => URL.revokeObjectURL(url);
    img.src = url;
  }
</script>

<figure class="chart">
  <svg bind:this={svgEl} viewBox={`0 0 ${W} ${H}`} role="img" xmlns="http://www.w3.org/2000/svg">
    <title>{spec.title}</title>
    <text class="title" x={W / 2} y="15">{spec.title}</text>

    {#each yTicks as t (t)}
      <line class="grid" x1={M.left} x2={W - M.right} y1={sy(t)} y2={sy(t)} />
      <text class="tick" x={M.left - 6} y={sy(t) + 3} text-anchor="end">{fmt(t)}</text>
    {/each}
    {#each xTicks as t (t)}
      <line class="tickmark" x1={sx(t)} x2={sx(t)} y1={H - M.bottom} y2={H - M.bottom + 4} />
      <text class="tick" x={sx(t)} y={H - M.bottom + 15} text-anchor="middle">{fmt(t)}</text>
    {/each}
    <line class="axis" x1={M.left} x2={W - M.right} y1={H - M.bottom} y2={H - M.bottom} />
    <line class="axis" x1={M.left} x2={M.left} y1={M.top} y2={H - M.bottom} />
    <text class="label" x={(M.left + W - M.right) / 2} y={H - 6} text-anchor="middle">
      {axisLabel(spec.x)}
    </text>
    <text
      class="label"
      transform={`rotate(-90 12 ${(M.top + H - M.bottom) / 2})`}
      x="12"
      y={(M.top + H - M.bottom) / 2}
      text-anchor="middle"
    >
      {axisLabel(spec.y)}
    </text>

    {#each spec.series as s (s.name)}
      {#if s.kind === "band"}
        <path class="band" d={bandPath(s.lower, s.upper, sx, sy)}>
          <title>{s.name}</title>
        </path>
      {:else if s.kind === "scatter"}
        {#each s.points as [px, py], i (i)}
          <circle class="dot" cx={sx(px)} cy={sy(py)} r="2.2" />
        {/each}
      {:else}
        <path class="series" d={linePath(s.points, sx, sy)}>
          <title>{s.name}</title>
        </path>
      {/if}
    {/each}
  </svg>

  <figcaption>
    <button class="export" onclick={exportSvg}>save SVG</button>
    <button class="export" onclick={exportPng}>save PNG</button>
    <span class="prov">{spec.provenance}</span>
  </figcaption>

  <!-- The same data as a table, for screen readers and for checking. -->
  <details class="data">
    <summary>data</summary>
    {#each spec.series as s (s.name)}
      <table>
        <caption>{s.name} ({s.kind})</caption>
        <thead>
          <tr><th>{axisLabel(spec.x)}</th><th>{axisLabel(spec.y)}</th></tr>
        </thead>
        <tbody>
          {#each seriesPoints(s) as [x, y], i (i)}
            <tr><td>{fmt(x)}</td><td>{fmt(y)}</td></tr>
          {/each}
        </tbody>
      </table>
    {/each}
  </details>
</figure>

<style>
  .chart {
    margin: 0.4rem 0;
    border: 1px solid var(--edge);
    border-radius: 8px;
    background: var(--panel-raised);
    padding: 0.4rem;
    max-width: 30rem;
  }
  svg {
    width: 100%;
    height: auto;
    display: block;
  }
  text {
    fill: var(--ink);
    font-family: inherit;
  }
  .title {
    font-size: 11px;
    text-anchor: middle;
  }
  .tick {
    font-size: 8.5px;
    fill: var(--dim);
  }
  .label {
    font-size: 9.5px;
    fill: var(--dim);
  }
  .grid {
    stroke: var(--edge);
    stroke-width: 0.6;
  }
  .axis,
  .tickmark {
    stroke: var(--edge-strong);
    stroke-width: 1;
  }
  .series {
    fill: none;
    stroke: var(--hot);
    stroke-width: 1.8;
  }
  .dot {
    fill: var(--hot);
  }
  .band {
    fill: var(--hot);
    opacity: 0.18;
    stroke: none;
  }
  figcaption {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.2rem 0.2rem 0;
  }
  .export {
    background: var(--panel);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.72rem;
    padding: 0.15rem 0.5rem;
    cursor: pointer;
  }
  .prov {
    font-size: 0.7rem;
    color: var(--dim);
  }
  .data summary {
    font-size: 0.72rem;
    color: var(--dim);
    cursor: pointer;
    padding: 0.2rem;
  }
  table {
    font-size: 0.75rem;
    border-collapse: collapse;
    margin: 0.3rem 0;
  }
  th,
  td {
    border: 1px solid var(--edge);
    padding: 0.15rem 0.5rem;
    text-align: right;
  }
  caption {
    font-size: 0.72rem;
    color: var(--dim);
    text-align: left;
  }
</style>
