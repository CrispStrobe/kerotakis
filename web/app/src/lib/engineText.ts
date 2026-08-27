import { i18n, t } from "./i18n.svelte";

const decimal = "([+−-]?\\d+(?:\\.\\d+)?)";

/**
 * The solver contract remains language-neutral and replayable. This is the
 * presentation boundary for prose produced by today's English Rust renderer.
 * Keep patterns deliberately exact: an unknown scientific sentence is shown
 * unchanged instead of being half-translated into something misleading.
 */
export function engineText(text: string): string {
  if (i18n.locale !== "de") return text;

  const direct = t(text);
  if (direct !== text) return direct;

  let match: RegExpMatchArray | null;
  if ((match = text.match(/^The mini centrifuge spins (v\d+); the particles travel ([\d.]+)% of the tube path\.$/))) {
    return `Die Minizentrifuge dreht ${match[1]}; die Teilchen legen ${match[2]} % des Röhrchenwegs zurück.`;
  }
  if ((match = text.match(/^(v\d+): ([\d.]+) rpm for ([\d.]+) s — ([\d.]+) × g; ([\d.]+)% separation; balanced within ([\d.]+) g$/))) {
    return `${match[1]}: ${match[2]} U/min für ${match[3]} s — ${match[4]} × g; ${match[5]} % getrennt; auf ${match[6]} g austariert`;
  }
  if ((match = text.match(/^While you wait, particles in (v\d+) sink toward the bottom\.$/))) {
    return `Während du wartest, sinken Teilchen in ${match[1]} zum Boden.`;
  }
  if ((match = text.match(/^(v\d+): ([\d.]+)% of the suspended particles settle in ([\d.]+) s$/))) {
    return `${match[1]}: ${match[2]} % der schwebenden Teilchen setzen sich in ${match[3]} s ab`;
  }
  if ((match = text.match(/^(.+) vapour is hazardous to inhale$/))) {
    return `${t(match[1]!)}dampf ist beim Einatmen gefährlich`;
  }


  return text;
}
