/**
 * One line of quest prose, at the reader's detail level and in the reader's
 * language.
 *
 * A quest's `title`, `goal` and claim titles are maps keyed by register —
 * `lv1`, `lv2`, `lv3` — because the same fact is worth saying three ways
 * depending on how much chemistry the reader has. That is a different axis
 * from language, and the two used to be confused: the picker read
 * `title[register]` and stopped there, so a German shell listed English
 * quests under a German placeholder.
 *
 * German lives INSIDE the key, `lv2_de`, for the reason the catalogue's
 * register prose gives: these are maps keyed by level, not records of named
 * fields, so there is no field to hang a `_de` sibling off. Every fallback
 * below keeps its translated twin ahead of it, so a level someone has
 * translated still reads German even when its neighbour has not — the
 * degradation is one cell at a time, never all-or-nothing.
 *
 * The last resort is any value at all rather than the empty string: a
 * reader who asked for `lv3` and finds only `lv1` is better served by the
 * simple sentence than by a blank row.
 */
export function registerText(
  map: Readonly<Record<string, string>> | undefined | null,
  register: string,
  locale: string,
  fallback = "",
): string {
  if (!map) return fallback;
  const translated = locale === "en" ? undefined : `_${locale}`;
  const pick = (key: string): string | undefined =>
    (translated ? map[`${key}${translated}`] : undefined) ?? map[key];
  const bare = register.replace("lv", "");
  return pick(register)
    ?? pick(bare)
    ?? pick("lv2")
    ?? pick("2")
    // Not `Object.values(map)[0]`: that would happily return the German
    // half of a pair to an English reader, or a `_de` value as if it were
    // the neutral one. Only unsuffixed cells are candidates here.
    ?? Object.entries(map).find(([key]) => !/_[a-z]{2}$/.test(key))?.[1]
    ?? fallback;
}
