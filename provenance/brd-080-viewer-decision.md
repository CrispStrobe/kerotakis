# BRD-080 provisional molecular viewer decision

**Evidence-backed provisional selection:** use **3Dmol.js 2.5.5** for the
bounded BRD-081 molecule viewer adapter if the remaining Svelte and physical-
mobile acceptance checks pass. Do not ship Mol* and do not ship both. Crystal
and orbital data contracts may land in BRD-081, but their production rendering
remains gated by real authoritative assets (including BRD-060 for crystals).

This record applies BREADTH.md's rule: choose the smaller adequate viewer;
Mol* wins only if its macromolecular or volume capability justifies its cost.
Both candidates reached their ready state with one canvas and semantic rows for
the same five local format probes in Chrome, so current evidence does not invoke
the exception. This does not establish pixel-level scientific correctness.

## Exact candidates and primary evidence

| Candidate | Exact source | Code licence and notices | Engine constraint |
|---|---|---|---|
| 3Dmol.js 2.5.5 | [release/tag](https://github.com/3dmol/3Dmol.js/releases/tag/2.5.5), commit `c26e390544b6388f86e50387cd4565759b4da0df`, [exact manifest](https://raw.githubusercontent.com/3dmol/3Dmol.js/2.5.5/package.json) | [BSD-3-Clause distribution notice](https://raw.githubusercontent.com/3dmol/3Dmol.js/2.5.5/LICENSE), which also preserves incorporated GLmol (MIT or LGPL-3), Three.js (MIT), and jQuery (MIT) notices | Node `>=16.16.0`, npm `>=8.11`; browser UMD/global entry |
| Mol* 5.11.0 | [release/tag](https://github.com/molstar/molstar/releases/tag/v5.11.0), commit `7fc2ec55517e3da840ffe9fb09dab7d3065efec2`, [exact manifest](https://raw.githubusercontent.com/molstar/molstar/v5.11.0/package.json) | [MIT](https://raw.githubusercontent.com/molstar/molstar/v5.11.0/LICENSE); every installed manifest in the exact lock was independently allowlisted | Node `>=22.0.0`; the repository's Node 20 development host warns, while the decision CI uses Node 22 |

The official 3Dmol [format/API documentation](https://3dmol.org/doc/) covers
SDF/XYZ, PDB, CIF and cube data, selection, labels, surfaces and unit cells.
Mol*'s official [format matrix](https://molstar.org/docs/plugin/file-formats/)
covers the same bounded probes and its [bundle guidance](https://molstar.org/docs/plugin/instance/#bundle-size)
warns that the default Viewer includes all extensions. Capability statements
were nevertheless tested through the disposable route rather than accepted
from documentation alone.

## Reproducible measurements

Run from the repository root:

```sh
npm ci --prefix spikes/brd080 --ignore-scripts --no-audit --no-fund
npm test --prefix spikes/brd080
node spikes/brd080/evidence.mjs > /tmp/brd080-evidence.json
npm run build --prefix spikes/brd080
npm run test:browser --prefix spikes/brd080
```

The committed evidence is schema `kerotakis.brd080-evidence.v1`:

- lock SHA-256 `e27f1b431bce76ba053ebb2760564b5525bd74384360039d4fb50ae690bc7539`;
- evidence SHA-256 `721ab64692d052640dbf84d1c86a6fadfbec7d90916990b06ecdddb1f2d1470e`
  (regenerated with Node 22.23.2 and canonically checked against the lock and
  fixture manifest);
- fixture-manifest SHA-256 `810427932c6037f7d385149df4fe9ac6bcc732b3c8bcee1605334c44c5bf9dfa`;
- five ordered project-authored probes: molecule/SDF, crystal/CIF,
  protein/PDB, signed volume/cube and two-frame trajectory/XYZ;
- 222 installed production package instances in the combined isolated lock,
  each with exact version, integrity and accepted licence. This is deliberately
  not the production application lock.

| Measurement | 3Dmol.js | Mol* | Consequence |
|---|---:|---:|---|
| Isolated candidate artifacts, raw | 586,724 B | 5,345,087 B | Mol* is 9.1x larger |
| Isolated candidate artifacts, deterministic gzip-9 | 168,749 B | 1,968,375 B | Mol* is 11.7x larger |
| Actual route renderer JS, gzip | 169,550 B + 2,100 B adapter | 1,478,420 B + 2,240 B adapter | Both lazy; Mol* also emits 16,450 B gzip CSS and 620,413 raw image assets |
| Exact production closure | 6 package instances | 216 package instances | Mol* adds server/tooling-looking dependencies and a much larger audit surface |
| Ten-path Chrome smoke | pass | pass | Same local fixtures; ready state, one canvas and semantic rows after every replacement |
| External requests | 0 | 0 | No request escaped the locally served route during this test |

Chrome also records maximum backing-store pixels and JS heap in a
`kerotakis.brd080-browser-smoke.v1` line. That value is a headless Chromium
proxy, **not** physical Android/iOS RAM or GPU memory. The final checkpoint
[run/job](https://github.com/CrispStrobe/kerotakis/actions/runs/33396831056/job/99503232314)
recorded 10 paths, 25 local requests, zero external requests, 4,915,200 maximum
canvas pixels and 70,737,452 maximum JS-heap bytes. This CI log is operational
evidence, not a hash-backed committed artifact. BRD-081 must retain the
2,000,000-byte source, 20,000-atom, 40,000-bond, 120-frame, 262,144-grid-point,
1280x960 and DPR-2 bounds; physical mobile release measurements remain a
separate acceptance item rather than an inferred pass.

The comparison is now a strict isolated Svelte 5 component and is deployed at
the disposable [Vercel origin](https://dist-a3br8svxu-crispstrobes-projects.vercel.app).
Playwright's Pixel 7 profile exercised keyboard selection, labels, bounded
resize and reduced-motion reload across all ten paths over HTTPS, observing
nine same-origin requests and zero external requests. The committed hosted
record identifies Chromium 151 and SwiftShader explicitly: this is deployed-
origin and mobile-layout evidence, not physical Android/iOS RAM or GPU evidence.
The latter must satisfy `tools/brd080-device-evidence.mjs` with one real Android
and one real iOS evidence row before this decision becomes final.

## Functional and accessibility findings

Both adapters reach ready state for the molecule, crystal, protein, cube and
short trajectory paths. Focused adapter tests cover selection, visual labels,
resize, reduced-motion options and repeated disposal; the hardened Chrome lane
also snapshots selection, labels and bounded resize for each real load path.
The tests do not prove WebGL-context or animation-frame release. Both are WebGL
canvases without evidence of a complete native screen-reader/keyboard
representation. The project-owned
semantic table and plain-language description are therefore required product
behavior, not optional fallback prose.

3Dmol's costs are real: the package entry is UMD and mutates a browser global,
contains an `eval` that Vite warns about, and exposes no documented complete
`destroy()` call. The adapter must remain SSR-lazy, avoid `download`/`get`,
stop animation, clear objects and remove created nodes. Its global-DPR sizing
also requires the tested inverse logical-size wrapper to keep the backing
store at DPR 2 without desynchronizing the GL viewport.

Mol* offers substantially deeper macromolecular and volume machinery, but the
bounded school-viewer requirements did not need it. Even with remote UI/state,
volume streaming and remote-capable extensions disabled, the actual build
pulls the large Viewer graph, SCSS, background assets and browser-externalized
Node paths from its MP4 dependency. That complexity has no compensating
accepted observable in this slice.

## Decision and BRD-081 boundary

3Dmol is the smaller adequate provisional selection. The disposable Svelte
route and deployed Playwright checkpoint pass; BRD-080 remains open until the
physical constrained-mobile Android and iOS RAM/GPU runs pass.
Only then may BRD-081a land a renderer-neutral `ScientificView` contract and
accessible semantic fallback; its crystal slice remains separately blocked on
BRD-060. The production adapter must preserve the limits and offline/teardown
tests above. No conformer, unit cell, bond, orbital, surface,
protein structure or trajectory may be invented from a formula or registry
name. Mol* remains a rejected decision-spike dependency and must be removed
with the disposable spike when BRD-081 no longer needs comparison evidence.
