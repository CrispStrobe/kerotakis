# BRD-080 physical-device evidence protocol

For the concrete MacBook + physical iPad procedure, including wireless Safari
inspection, Instruments memory attribution, artifact layout and exact DoDs, see
[`brd080-macbook-ipad-handover.md`](brd080-macbook-ipad-handover.md). That
handover produces only the required iOS row; this protocol still requires one
physical Android row as well.

Vercel and Playwright establish that the exact deployed HTTPS route loads, stays
on-origin, and supports the scripted interactions. A desktop Playwright worker
with a mobile viewport is not physical-mobile RAM or GPU evidence. The validator
therefore requires one real Android row and one real iOS row and rejects
headless, simulator, emulator and SwiftShader identities.

Copy `tools/fixtures/brd080-device-evidence.template.json` beside a directory of
raw artifacts. Hash every referenced artifact with `sha256sum` (or
`shasum -a 256`) and validate the completed envelope with:

```sh
node tools/brd080-device-evidence.mjs path/to/evidence.json
```

The route artifact is a sorted manifest of the deployed build's files and
SHA-256 values. Record the exact repository route commit and the fixed upstream
3Dmol 2.5.5 source commit. For each
device perform three cycles over all five fixtures. Every cycle must load,
select an atom, toggle labels, resize/orient, enable reduced motion, and dispose
or replace the viewer. Preserve screenshots, browser console/network output,
the browser-side collector JSON and raw OS memory samples. The browser collector
records viewport/DPR, canvas backing pixels, WebGL vendor/renderer/version and
limits, context-loss events, completed interactions and requests; it must not
present `performance.memory` as device RAM.

## Android

Use a connected physical device. `adb devices -l` must report `device`; reject
serials whose `ro.kernel.qemu` is `1` or whose product/model contains
`sdk`, `generic` or `emulator`. Record model, manufacturer, Android release and
build from `adb shell getprop`, hashing the serial rather than publishing it.
Enable Chrome remote debugging, attach Playwright/CDP to the device Chrome, and
open the deployed Vercel URL. Identify the renderer PID belonging to that tab;
if attribution is ambiguous, stop. Sample `adb shell dumpsys meminfo PID` (and
`/proc/PID/status` when readable) before loading, after every workload step,
at peak, and after the final settled interval. Preserve the unedited output.

## iOS

Use a paired iPhone or iPad with Developer Mode. `xcrun devicectl list devices`
must identify a physical iOS/iPadOS device; a Simulator row is invalid. Drive
the deployed URL in Mobile Safari through Appium/XCUITest/WebDriverAgent, or in
a development-signed WKWebView harness. Use Instruments/`xctrace` to capture
memory for the exact Safari WebContent or harness process across the same three
cycles. Preserve the trace/export plus console/network and screenshots. The
existing `tools/install-ios-device.sh` provides the repository's paired-device
selection and signing pattern when a WKWebView harness is used.

Hosted device farms are acceptable only when the session is a named physical
device and the provider supplies downloadable raw identity, logs and performance
artifacts. Credentials and stable device identifiers must never be committed.
No absolute RAM ceiling is fabricated here: peak and settled deltas are recorded
for independent review. Completion does require all paths/interactions, bounded
canvas allocation, zero external requests, no context loss and hash-valid raw
evidence.
