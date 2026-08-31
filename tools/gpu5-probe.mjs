#!/usr/bin/env node
/** GPU-5 device probe: synchronous CPU encode+submit cost, never GPU readback. */
import { serve, browser, waitFor } from "./lib/headless.mjs";
import {
  GPU5_MEASURED_FRAMES,
  GPU5_RUNS,
  GPU5_WARMUP_FRAMES,
  buildRunEvidence,
  completeReport,
  emptyReport,
  summarizeStartup,
} from "./gpu5-probe-lib.mjs";

const argv = process.argv.slice(2);
const site = argv.find((value) => !value.startsWith("--"));
const option = (name, fallback) => {
  const index = argv.indexOf(`--${name}`);
  return index < 0 ? fallback : argv[index + 1];
};
const repeated = (name) => argv.flatMap((value, index) => value === `--${name}` ? [argv[index + 1]] : []).filter(Boolean);
if (!site) {
  console.error("usage: node tools/gpu5-probe.mjs <site-dir> --mode lightweight|webgpu [--host-label name] [--command '...'] [--trigger-command 'ignite v1']");
  console.error("run lightweight and webgpu separately; each invocation records ten fresh-profile startups");
  console.error("pair both startup.coldStartupMs arrays, webgpu.runs, and frontend-asset-budget gzip totals into each five-host release-gate row");
  process.exit(2);
}

const hostLabel = option("host-label", "unnamed-host");
const mode = option("mode");
if (mode !== "lightweight" && mode !== "webgpu") {
  console.error("--mode must be explicitly set to lightweight or webgpu");
  process.exit(2);
}
const setupCommands = repeated("command");
const triggerCommand = option("trigger-command", "ignite v1");
const timeoutMs = Number(option("timeout-ms", "90000"));
const startupRuns = Number(option("startup-runs", "10"));
if (startupRuns !== 10) throw new Error("GPU-5 release evidence requires exactly 10 cold startup runs");
const browserOptions = {
  disableGpu: false,
  headless: !argv.includes("--headed"),
  // This flag permits probing; capability acquisition still decides honestly.
  extraArgs: [
    "--enable-unsafe-webgpu",
    // Probe-only: CDP remote debugging may otherwise set webdriver=true,
    // which correctly activates the app's normal headless GPU veto.
    "--disable-blink-features=AutomationControlled",
  ],
};

const injection = `(() => {
  const state = globalThis.__keroGpu5 = {
    supported: Boolean(navigator.gpu), samples: [], domContentLoadedMs: null,
    webdriver: navigator.webdriver === true,
    appReadyMs: null, lightweightReadyMs: null, hookInstalled: false,
    rafIntervals: [], lastRaf: null,
    svgFallbackObserved: false, gpuPresentedObserved: false
  };
  addEventListener("DOMContentLoaded", () => { state.domContentLoadedMs = performance.now(); }, { once: true });
  const markReady = (timestamp) => {
    if (state.lastRaf !== null) state.rafIntervals.push(timestamp - state.lastRaf);
    state.lastRaf = timestamp;
    if (state.appReadyMs === null && document.querySelector("form.bar input")) state.appReadyMs = performance.now();
    if (state.lightweightReadyMs === null && document.querySelector(".bench")) state.lightweightReadyMs = performance.now();
    const svg = Boolean(document.querySelector("g.flame"));
    const gpu = Boolean(document.querySelector('canvas[data-visual-backend="webgpu"]'));
    if (svg && !state.gpuPresentedObserved) state.svgFallbackObserved = true;
    if (gpu) state.gpuPresentedObserved = true;
    requestAnimationFrame(markReady);
  };
  requestAnimationFrame(markReady);
  try {
    if (!state.supported || !globalThis.GPUDevice || !globalThis.GPUCommandEncoder || !globalThis.GPUQueue) return;
    const encoderStart = new WeakMap();
    const bufferStart = new WeakMap();
    const create = GPUDevice.prototype.createCommandEncoder;
    const finish = GPUCommandEncoder.prototype.finish;
    const submit = GPUQueue.prototype.submit;
    GPUDevice.prototype.createCommandEncoder = function(...args) {
      const encoder = Reflect.apply(create, this, args);
      encoderStart.set(encoder, performance.now());
      return encoder;
    };
    GPUCommandEncoder.prototype.finish = function(...args) {
      const buffer = Reflect.apply(finish, this, args);
      const start = encoderStart.get(this);
      if (start !== undefined) bufferStart.set(buffer, start);
      return buffer;
    };
    GPUQueue.prototype.submit = function(buffers) {
      let start = Infinity;
      for (const buffer of buffers) start = Math.min(start, bufferStart.get(buffer) ?? Infinity);
      const result = Reflect.apply(submit, this, [buffers]);
      if (Number.isFinite(start)) state.samples.push(performance.now() - start);
      return result;
    };
    state.hookInstalled = true;
  } catch { state.hookInstalled = false; }
})()`;

const submitCommand = (page, command) => page.evaluate(`(() => {
  const input = document.querySelector("form.bar input");
  const form = input?.closest("form");
  if (!input || !form) return false;
  const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
  setter?.call(input, ${JSON.stringify(command)});
  input.dispatchEvent(new Event("input", { bubbles: true }));
  form.requestSubmit();
  return true;
})()`);

const { server, origin } = await serve(site);
const coldStartupSamples = [];
for (let run = 0; run < startupRuns; run += 1) {
  const startupPage = await browser(browserOptions);
  try {
    await startupPage.cdp.send("Page.addScriptToEvaluateOnNewDocument", { source: injection }, startupPage.sessionId);
    await startupPage.goto(`${origin}/app/`);
    const ready = await waitFor(startupPage, `__keroGpu5.appReadyMs !== null && __keroGpu5.lightweightReadyMs !== null`, { timeout: 60000 });
    if (!ready) throw new Error(`cold startup run ${run + 1} did not reach both readiness markers`);
    coldStartupSamples.push(JSON.parse(await startupPage.evaluate(`JSON.stringify({
      dom_content_loaded_ms: __keroGpu5.domContentLoadedMs,
      app_ready_ms: __keroGpu5.appReadyMs,
      lightweight_ready_ms: __keroGpu5.lightweightReadyMs
    })`)));
  } finally {
    await startupPage.close?.();
  }
}
const page = await browser(browserOptions);
let exitCode = 0;
try {
  await page.cdp.send("Page.addScriptToEvaluateOnNewDocument", { source: injection }, page.sessionId);
  await page.goto(`${origin}/app/`);
  await waitFor(page, `document.querySelector("form.bar input")`, { timeout: 60000 });
  await page.evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find((item) => /enter Sandbox|Sandbox betreten/i.test(item.textContent || ""));
    button?.click();
  })()`);
  await waitFor(page, `document.querySelector(".bench")`, { timeout: 20000 });
  for (const command of setupCommands) {
    if (!(await submitCommand(page, command))) throw new Error(`cannot submit setup command: ${command}`);
    await new Promise((resolve) => setTimeout(resolve, 250));
  }

  const initial = JSON.parse(await page.evaluate(`JSON.stringify({
    supported: __keroGpu5.supported && __keroGpu5.hookInstalled,
    webdriver: __keroGpu5.webdriver,
    userAgent: navigator.userAgent,
    startup: ${JSON.stringify(summarizeStartup(coldStartupSamples))},
    fallback: {
      svg_present_before_gpu: __keroGpu5.svgFallbackObserved,
      svg_present_now: Boolean(document.querySelector("g.flame")),
      gpu_presented: Boolean(document.querySelector('canvas[data-visual-backend="webgpu"]'))
    }
  })`));
  const base = {
    ...emptyReport({
      hostLabel,
      userAgent: initial.userAgent,
      startup: initial.startup,
      fallback: initial.fallback,
      automationPolicyOverride: true,
    }),
    mode,
    measurement_override: "AutomationControlled disabled for CDP measurement only; normal app headless policy is unchanged",
  };
  if (mode === "lightweight") {
    console.log(JSON.stringify({
      ...base, evidence_complete: true, pass: true, outcome: "lightweight-baseline-recorded",
    }, null, 2));
    process.exitCode = 0;
  } else if (initial.webdriver) {
    console.log(JSON.stringify({ ...base, outcome: "headless-policy-active" }, null, 2));
    exitCode = 1;
  } else if (!initial.supported) {
    console.log(JSON.stringify(base, null, 2));
    exitCode = 1; // Honest unsupported evidence is valid evidence, but not a successful gate.
  } else {
    await submitCommand(page, triggerCommand);
    const presented = await waitFor(page, `__keroGpu5.samples.length > 0`, { timeout: 5000, step: 50 });
    if (!presented) {
      const fallback = JSON.parse(await page.evaluate(`JSON.stringify({
        svg_present_before_gpu: __keroGpu5.svgFallbackObserved,
        svg_present_now: Boolean(document.querySelector("g.flame")),
        gpu_presented: false
      })`));
      console.log(JSON.stringify({ ...base, fallback }, null, 2));
      exitCode = 1;
    } else {
    const runs = [];
    const needed = GPU5_WARMUP_FRAMES + GPU5_MEASURED_FRAMES;
    for (let run = 0; run < GPU5_RUNS; run += 1) {
      await page.evaluate(`__keroGpu5.samples.length = 0; __keroGpu5.rafIntervals.length = 0`);
      const deadline = Date.now() + timeoutMs;
      while (Date.now() < deadline) {
        await submitCommand(page, triggerCommand);
        if (await waitFor(page, `__keroGpu5.samples.length >= ${needed} && __keroGpu5.rafIntervals.length >= ${needed}`, { timeout: 1600, step: 50 })) break;
      }
      const measurement = JSON.parse(await page.evaluate(`JSON.stringify({ samples: __keroGpu5.samples, raf: __keroGpu5.rafIntervals })`));
      runs.push(buildRunEvidence(measurement.samples, measurement.raf));
    }
    const fallback = JSON.parse(await page.evaluate(`JSON.stringify({
      svg_present_before_gpu: __keroGpu5.svgFallbackObserved,
      svg_present_now: Boolean(document.querySelector("g.flame")),
      gpu_presented: Boolean(document.querySelector('canvas[data-visual-backend="webgpu"]'))
    })`));
    const report = completeReport({ ...base, fallback, webgpu_available: true }, runs);
    console.log(JSON.stringify(report, null, 2));
    exitCode = report.pass ? 0 : 1;
    }
  }
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  exitCode = 1;
} finally {
  await page.close?.();
  server.close();
}
process.exitCode = exitCode;
