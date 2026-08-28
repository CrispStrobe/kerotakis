import notice from "./generated/notice.json";

declare const __KERO_COMMIT__: string;
declare const __KERO_REF__: string;
declare const __KERO_BUILT_AT__: string;

/** What this build is, as far as it can honestly say.
 *
 * Stamped by vite.config.ts at build time. `unknown` is a real answer —
 * a tarball with no `.git` builds fine and cannot name its commit — and
 * saying so is better than inventing a version that would send someone
 * looking at the wrong code.
 */
export const BUILD = {
  commit: typeof __KERO_COMMIT__ === "string" ? __KERO_COMMIT__ : "unknown",
  ref: typeof __KERO_REF__ === "string" ? __KERO_REF__ : "",
  builtAt: typeof __KERO_BUILT_AT__ === "string" ? __KERO_BUILT_AT__ : "",
} as const;

export const REPO = "https://github.com/CrispStrobe/kerotakis";
export const COPYRIGHT = "Copyright © 2026 Christian Ströbele and contributors";
export const THIRD_PARTY_LICENSES = `${import.meta.env.BASE_URL}legal/third-party-licenses.html`;

/** A link to the exact source this build came from, when it knows. */
export function commitUrl(): string | null {
  return BUILD.commit === "unknown" ? null : `${REPO}/commit/${BUILD.commit}`;
}

/** The third-party components, verbatim from NOTICE. See tools/about-notice.mjs. */
export const NOTICE_SECTIONS: { title: string; entries: string[] }[] = notice.sections;

/** Human-readable build time, or "" when this build was not stamped. */
export function builtAt(locale: string): string {
  if (!BUILD.builtAt) return "";
  const when = new Date(BUILD.builtAt);
  if (Number.isNaN(when.getTime())) return "";
  try {
    return when.toLocaleString(locale || undefined, {
      dateStyle: "medium",
      timeStyle: "short",
    });
  } catch {
    // An unknown locale tag must not take the dialog down.
    return when.toISOString();
  }
}

/**
 * Reload after dropping everything that can keep an old build alive.
 *
 * A stale service worker serves the previous bundle indefinitely, and
 * "reload the page" does not fix it — which makes a shipped bug look
 * unfixed after it has been fixed. Notes and preferences live in
 * localStorage and are deliberately untouched: this clears caches, not
 * the reader's work.
 */
export async function hardReload(): Promise<void> {
  try {
    if (typeof caches !== "undefined") {
      const names = await caches.keys();
      await Promise.all(names.map((name) => caches.delete(name)));
    }
  } catch {
    /* CacheStorage is absent in some webviews and private modes. */
  }

  try {
    if (typeof navigator !== "undefined" && navigator.serviceWorker) {
      const regs = await navigator.serviceWorker.getRegistrations();
      await Promise.all(regs.map((reg) => reg.unregister()));
    }
  } catch {
    /* No service worker in the native shell; nothing to unregister. */
  }

  if (typeof window === "undefined" || !window.location) return;
  // A changing query also defeats an HTTP cache, which CacheStorage
  // deletion does not reach.
  const url = new URL(window.location.href);
  url.searchParams.set("reload", String(Date.now()));
  window.location.replace(url.href);
}
