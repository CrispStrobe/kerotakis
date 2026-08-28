import { i18n, t } from "./i18n.svelte";

const decimal = "([+−-]?\\d+(?:\\.\\d+)?)";

/**
 * Nothing. The engine translates its own prose now.
 *
 * This file used to rewrite the engine's English into German in the shell,
 * by matching the sentences `render.rs` emits. `render.rs` has carried its
 * own catalogue for a while, which killed almost all of it; the hazard
 * notes were the last holdout, because the shell reads `hazard` and
 * `real_world` straight off the serialised event rather than through the
 * renderer, so no catalogue on the Rust side could reach them.
 *
 * `localize_events` in the wasm wrapper now translates those on the way
 * out, at the first point that knows the language. So this is a passthrough,
 * kept only so the call sites need not all change at once.
 *
 * Do not put a translation here. A German string written in the shell is
 * not a missing translation, it is one that only German can ever have —
 * there is nowhere to put French. The engine catalogue takes a language as
 * a file.
 */
export function engineText(text: string): string {
  return text;
}
