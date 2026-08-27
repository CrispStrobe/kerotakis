import { i18n, t } from "./i18n.svelte";

const decimal = "([+−-]?\\d+(?:\\.\\d+)?)";

/**
 * What the engine catalogue does not reach.
 *
 * This file used to translate the engine's prose in the shell, by
 * regex-matching the English the Rust renderer emits. `render.rs` now
 * carries its own message catalogue — 199 keys, every sentence — so almost
 * everything here became dead code: a pattern that matches English can
 * never fire on a line the engine already rendered in German.
 *
 * One pattern remains, and it earns its place. The hazard notes come from
 * `bench.rs`, which has about twenty user-facing strings and no catalogue
 * of its own yet. That is the honest reason this file still exists, and
 * the thing to fix if you want it gone.
 *
 * Do not add to it. A German string written here is not a missing
 * translation, it is a translation only German can ever have — there is
 * nowhere to put French. The engine catalogue takes a language as a file.
 */
export function engineText(text: string): string {
  if (i18n.locale !== "de") return text;

  const direct = t(text);
  if (direct !== text) return direct;

  let match: RegExpMatchArray | null;
  if ((match = text.match(/^(.+) vapour is hazardous to inhale$/))) {
    return `${t(match[1]!)}dampf ist beim Einatmen gefährlich`;
  }


  return text;
}
