# BRD-080 MacBook + physical iPad handover

This handover is for the next agent who has a MacBook and a physical iPad that
can join the same private WLAN. It produces the **iOS/iPadOS half** of the
BRD-080 physical-device evidence envelope. It cannot close BRD-080 by itself:
`tools/brd080-device-evidence.mjs` also requires one independently measured
physical Android row. Bluetooth may help with pairing or a hotspot, but it is
not a Safari Web Inspector, asset-serving, or measurement transport. Use WLAN
or USB for inspection and record which one was used.

## Evidence target

Test the exact production build from repository commit
`a53d3423eddd055dd8680d1d7c1c5129e1eb512c`:

- Vercel deployment: `dpl_8QBNH3njt4mUeiEGLjDFFuPWhfcw`
- immutable origin:
  <https://kerotakis-brd080-viewer-spike-5edk0wsvr-crispstrobes-projects.vercel.app>
- candidate: 3Dmol.js 2.5.5, source commit
  `c26e390544b6388f86e50387cd4565759b4da0df`
- artifact-manifest SHA-256:
  `14fdb0c39580f06d1dfafeff7d6937b91306a3072a5178c9476d90718b7c1b9c`
- served `index.html` SHA-256:
  `e6d496914a0f83fdc1fc3968c50da3479b06d1a0ac3b1a56011a82eb598245e3`

Before measuring, confirm the immutable origin returns HTTP 200 and inspect the
deployment metadata. If it is unavailable, create a fresh deployment from the
same reviewed commit, record the new immutable URL and deployment ID, rebuild
the route manifest, and rerun the hosted Playwright gate. Never substitute a
moving alias or a mutable local HTTP server in the evidence record.

## Definition of done

The iPad checkpoint is accepted only when all of the following are true:

- Xcode tooling identifies a physical iPad, not Simulator.
- The exact repository commit, immutable HTTPS origin, deployment ID, route
  manifest, iPad model, iPadOS version/build, Mobile Safari version and WebKit
  engine are recorded.
- Three complete cycles exercise all five selected-candidate paths: water SDF,
  NaCl CIF, peptide PDB, orbital cube and trajectory XYZ.
- Every path completes load, touch selection, labels, resize/orientation,
  reduced-motion reload and viewer disposal/replacement.
- The semantic table stays populated, exactly one viewer canvas remains after
  replacement, maximum backing allocation is at most 4,915,200 pixels, and
  context losses and workload errors both equal zero.
- Safari's page requests are all same-origin, `data:` or `blob:`; the recorded
  external-request count is zero.
- Baseline, peak and final settled bytes are attributable to the exact Safari
  WebContent process. `performance.memory` is not reported as device RAM.
- Raw screenshots, console/network output, collector output, OS memory samples
  and the Instruments trace are retained and hash-referenced using relative
  paths from the evidence directory.
- The reviewable bundle contains no raw UDID/serial, Apple ID, personal device
  name, WLAN secret, cookie, pairing record, provisioning profile or token.
- The iOS row passes structural and artifact validation. The complete envelope
  is expected to remain fail-closed until its physical Android row is added.

## 1. Prepare the MacBook and iPad

On the MacBook install/currently select:

- current macOS Safari;
- Xcode and its command-line tools (`xcode-select -p`);
- Node.js for the repository validator;
- Instruments/`xctrace`; and
- enough non-synced local disk for large Instruments traces.

In Safari enable **Settings → Advanced → Show features for web developers**.
On the physical iPad enable Developer Mode and **Settings → Safari → Advanced
→ Web Inspector**. Temporarily disable Auto-Lock and Low Power Mode. Use a
fresh Safari session without personal tabs, VPN, content blocker, Private Relay
or custom DNS filtering unless one of those is deliberately part of the test
environment.

Connect the iPad by USB for its initial pairing, accept **Trust This Computer**,
and verify both commands show the physical device:

```sh
xcrun devicectl list devices
xcrun xctrace list devices
```

In Xcode's **Devices and Simulators** window select the iPad and enable network
connection. Put both devices on the same private WPA2/WPA3 WLAN with client
isolation disabled. Disconnect USB only after the iPad remains visible. If
wireless inspection becomes unreliable, reconnect USB and restart the affected
cycle; never replace the run with desktop responsive mode.

## 2. Establish exact route provenance

Use a clean checkout of the recorded route commit:

```sh
git fetch origin main
git switch --detach a53d3423eddd055dd8680d1d7c1c5129e1eb512c
npm ci --prefix spikes/brd080 --ignore-scripts --no-audit --no-fund
npm test --prefix spikes/brd080
npm run build --prefix spikes/brd080
```

Create a canonical local route manifest:

```sh
cd spikes/brd080/dist
find . -type f ! -path './.vercel/*' ! -name '.gitignore' -print0 |
  LC_ALL=C sort -z |
  xargs -0 shasum -a 256 > ../../../route-assets.txt
cd ../../..
shasum -a 256 route-assets.txt
```

Rerun deployed-origin automation before physical testing:

```sh
BRD080_ORIGIN=https://kerotakis-brd080-viewer-spike-5edk0wsvr-crispstrobes-projects.vercel.app \
  npm run test:hosted --prefix spikes/brd080
```

Archive the command output, deployment inspection, route manifest, exact Git
SHA and timestamps. The canonical artifact-manifest digest at the top hashes a
different manifest serialization, so its digest is not expected to equal the
plain `shasum` listing. Compare the listed deployable paths and per-file hashes
to that reviewed manifest. Stop and explain any missing, extra or changed
deployable file before collecting device data. Never include `.vercel` link
metadata or `.gitignore` in the route artifact.

## 3. Attach Safari Web Inspector

1. On the iPad open only the immutable Vercel URL in Mobile Safari.
2. On the Mac choose **Safari → Develop → _iPad name_ → _exact page_**.
3. Confirm the inspected URL and page title.
4. Open Console, Network, Timelines, Storage and Elements.
5. Clear console and network records immediately before each cycle.
6. In Elements confirm the semantic table and one viewer canvas are present.

Bluetooth-only attachment is unsupported. A Mac-provided Wi-Fi hotspot is
acceptable if its topology is recorded and the page still loads from Vercel.

## 4. Run the three-cycle workload

For each of three complete cycles, select **3Dmol** and exercise fixtures in
this order: water, NaCl, peptide, orbital, trajectory. For every fixture:

1. Load it and wait for `status=ready`.
2. Select the first atom using the physical touchscreen.
3. Enable labels and confirm the semantic table remains populated.
4. In Web Inspector run the bounded stress resize:

   ```js
   await globalThis.__brd080.resize(5000, 5000, 9);
   globalThis.__brd080.snapshot();
   ```

5. Rotate portrait → landscape → portrait. Confirm readiness and the canvas
   bound after every orientation.
6. Enable reduced motion and wait for the replacement viewer to become ready.
7. Disable labels and switch to the next fixture, forcing disposal/replacement.
8. Confirm no old canvas accumulates and exactly one live canvas remains.

At the end of every cycle capture a screenshot showing the viewer and semantic
table, save the bridge snapshot, preserve Console and Network output, and record
canvas width/height, DPR, WebGL identity/limits and context-loss count. Use this
probe without inventing an unmasked GPU identity when Safari withholds it:

```js
const canvas = document.querySelector("#viewer canvas");
const gl = canvas?.getContext("webgl2") ?? canvas?.getContext("webgl");
const debug = gl?.getExtension("WEBGL_debug_renderer_info");
({
  vendor: debug ? gl.getParameter(debug.UNMASKED_VENDOR_WEBGL) : gl.getParameter(gl.VENDOR),
  renderer: debug ? gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER),
  webglVersion: gl.getParameter(gl.VERSION),
  maxTextureSize: gl.getParameter(gl.MAX_TEXTURE_SIZE),
  maxRenderbufferSize: gl.getParameter(gl.MAX_RENDERBUFFER_SIZE),
  canvasWidth: canvas.width,
  canvasHeight: canvas.height,
  snapshot: globalThis.__brd080.snapshot()
});
```

Any off-origin request, second canvas, context loss, oversize backing store or
workload error fails that run. Preserve the failed artifacts, fix or explain the
cause, and collect a new clean three-cycle run rather than editing the record.

## 5. Measure attributed OS memory

Use Instruments against the physical iPad and the WebContent process belonging
to the inspected tab. Close unrelated Safari tabs/apps first. Activity Monitor
or VM Tracker supplies process footprint; Allocations may additionally explain
allocation behavior. If process attribution remains ambiguous, stop and use a
development-signed WKWebView harness instead of claiming Safari-wide memory.

Start recording before the first fixture and mark timestamps for settled
baseline, every load/selection/label/rotation/reduced-motion/disposal step,
peak, and final settled state. After cycle three, leave the final replacement
idle for 30 seconds before the settled sample. Export the trace and a
machine-readable CSV/table. Record:

- `baselineBytes`: attributed WebContent footprint before the workload;
- `peakBytes`: maximum attributed footprint over all three cycles;
- `settledBytes`: footprint after the final 30-second idle;
- `deltaPeakBytes = peakBytes - baselineBytes`; and
- `deltaSettledBytes = max(0, settledBytes - baselineBytes)`.

Do not combine an Instruments footprint and JavaScript heap into one value.

## 6. Assemble, redact and validate evidence

Collect outside the repository while the run is active:

```text
brd080-physical-YYYYMMDD/
  evidence.json
  route-assets.txt
  deployment.txt
  ios/
    device-redacted.json
    collector.json
    console.txt
    network-export/
    memory.trace
    memory.csv
    screenshots/
```

Copy `tools/fixtures/brd080-device-evidence.template.json` to `evidence.json`.
Construct one row with `platform: "ios"` and `physical: true`, following every
field enforced by `tools/brd080-device-evidence.mjs`. Hash the device identifier
with HMAC-SHA-256 and an offline lab secret; do not publish the secret or raw
identifier. Review every artifact for personal data or credentials, then add
relative paths and SHA-256 digests:

```sh
find ios -type f -print0 | LC_ALL=C sort -z | xargs -0 shasum -a 256
node tools/brd080-device-evidence.mjs \
  /path/to/brd080-physical-YYYYMMDD/evidence.json
```

The one-row bundle must fail only because the physical Android row is absent.
Do not weaken the validator or mark BRD-080 complete. Transfer the redacted,
hash-valid bundle to the reviewer; keep any unredacted original encrypted and
access-restricted according to local policy.

## Failure triage

- **iPad absent from Develop:** recheck trust, Web Inspector, Developer Mode,
  Xcode pairing and WLAN; fall back to USB.
- **Origin unavailable/authenticated:** verify the exact Vercel deployment;
  never fall back to mutable HTTP.
- **External request:** preserve the Network initiator and fail the run.
- **Renderer says headless/SwiftShader/simulator/generic:** reject the row.
- **GPU identity is masked:** record the masked value honestly and flag it for
  review; do not infer an Apple GPU model.
- **Memory attribution ambiguous:** close other tabs and repeat, or use a signed
  WKWebView harness.
- **Wireless disconnect:** invalidate the affected cycle and restart all three
  cycles after reconnecting.
- **Arithmetic mismatch:** correct attribution or arithmetic from raw samples;
  never alter measurements merely to satisfy validation.

After the iPad row is accepted, the next owner must collect the matching
physical Android row described in `tools/brd080-device-evidence.md`, validate
the complete two-row envelope, independently review the raw artifacts, and only
then update the provisional decision and BRD-080 status to final.
