/**
 * The bench as an installable, offline-first app.
 *
 * The service worker lives at the payload root (`web/sw.js`) because it
 * precaches both documents and the one engine payload they share. Until
 * GUI-063 only the console page registered it, so opening `/app/` — the
 * URL the README advertises — installed nothing: no offline, no install
 * prompt. This module is the bench's half of that.
 *
 * Two states are worth surfacing and nothing else is:
 *
 *   `updateReady`   a new deploy is cached and waiting. The worker does
 *                   NOT skip the wait on its own, because activating
 *                   deletes the previous cache and this page may still ask
 *                   for a content-hashed chunk that lived only there.
 *                   `applyUpdate()` is the user saying "now".
 *   `installable`   the browser offered `beforeinstallprompt`. Chromium
 *                   only; Safari installs from the share sheet and never
 *                   fires it, which is why nothing here treats the absence
 *                   as an error.
 *
 * In the Tauri shell there is no service worker and nothing to install —
 * the app IS installed — so registration is skipped entirely.
 */

import { isTauri } from "./host/TauriHost";
import { resolvePayloadBase } from "./host/WorkerHost";

/** The `beforeinstallprompt` event, which no lib.dom.d.ts declares. */
type InstallPromptEvent = Event & {
  prompt(): Promise<void>;
  userChoice: Promise<{ outcome: "accepted" | "dismissed" }>;
};

class Pwa {
  /** A newer build is cached and waiting for this page to let go. */
  updateReady = $state(false);
  /** The browser will show an install prompt if asked. */
  installable = $state(false);
  /** Running from the home screen / app window rather than a tab. */
  installed = $state(false);

  #waiting: ServiceWorker | null = null;
  #prompt: InstallPromptEvent | null = null;

  /** Swap to the waiting build and reload once it has taken over. */
  async applyUpdate(): Promise<void> {
    const waiting = this.#waiting ?? (await navigator.serviceWorker?.ready)?.waiting;
    if (!waiting) {
      location.reload();
      return;
    }
    // `controllerchange` fires once the new worker has claimed this page;
    // reloading before that would just re-run the old assets.
    navigator.serviceWorker.addEventListener(
      "controllerchange",
      () => location.reload(),
      { once: true },
    );
    waiting.postMessage({ kero: "skip-waiting" });
  }

  /** Show the browser's install prompt. No-op where there is none. */
  async install(): Promise<void> {
    const prompt = this.#prompt;
    if (!prompt) return;
    this.#prompt = null;
    this.installable = false;
    await prompt.prompt();
    await prompt.userChoice;
  }

  /** Wire everything up. Safe to call once, from `onMount`. */
  register(): void {
    if (typeof window === "undefined" || isTauri()) return;

    this.installed =
      window.matchMedia?.("(display-mode: standalone)").matches ||
      // iOS Safari's own flag; it implements neither display-mode nor
      // beforeinstallprompt for home-screen apps.
      (navigator as { standalone?: boolean }).standalone === true;

    window.addEventListener("beforeinstallprompt", (event) => {
      event.preventDefault();
      this.#prompt = event as InstallPromptEvent;
      this.installable = true;
    });
    window.addEventListener("appinstalled", () => {
      this.installable = false;
      this.#prompt = null;
    });

    if (!("serviceWorker" in navigator) || !location.protocol.startsWith("http")) return;

    navigator.serviceWorker.addEventListener("message", (event) => {
      if (event.data?.kero === "update-ready") this.updateReady = true;
    });

    // The worker sits at the payload root — the same directory the engine
    // wasm and databases come from, which is what `resolvePayloadBase`
    // already resolves — so its scope covers the console page and the
    // bench alike. A script may claim any scope at or below its own
    // directory, so no Service-Worker-Allowed header is needed.
    const base = resolvePayloadBase();
    void navigator.serviceWorker
      .register(new URL("sw.js", base).href, { scope: base })
      .then((registration) => {
        // Only ever raised, never cleared: the worker's own `update-ready`
        // message can arrive before `updatefound` does, and a `false` here
        // would take it back down again.
        const watch = (worker: ServiceWorker | null) => {
          if (!worker) return;
          this.#waiting = worker;
          const settle = () => {
            if (worker.state === "installed" && navigator.serviceWorker.controller) {
              this.updateReady = true;
            }
          };
          settle();
          worker.addEventListener("statechange", settle);
        };
        watch(registration.waiting);
        registration.addEventListener("updatefound", () => watch(registration.installing));
      })
      // An offline first load, a file:// preview, a host without the
      // worker deployed — none of these should break the bench.
      .catch(() => {});
  }
}

export const pwa = new Pwa();
