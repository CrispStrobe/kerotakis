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
  if ((match = text.match(/^A fresh beaker appears on the bench: (v\d+)\.$/))) {
    return `Ein frisches Becherglas erscheint auf dem Labortisch: ${match[1]}.`;
  }
  if ((match = text.match(/^(v\d+): new vessel$/))) return `${match[1]}: neues Gefäß`;
  if ((match = text.match(/^The empty (v\d+) goes back into storage\.$/))) {
    return `Das leere Gefäß ${match[1]} kommt zurück in den Schrank.`;
  }
  if ((match = text.match(/^(v\d+): empty vessel removed$/))) return `${match[1]}: leeres Gefäß entfernt`;
  if ((match = text.match(/^You add (.+) to (v\d+)\.$/))) {
    return `Du gibst ${t(match[1]!)} in ${match[2]}.`;
  }
  if ((match = text.match(new RegExp(`^(v\\d+): \\+${decimal} mol (.+?) — ${decimal} mol now in vessel$`)))) {
    return `${match[1]}: +${match[2]} mol ${t(match[3]!)} — jetzt ${match[4]} mol im Gefäß`;
  }
  if ((match = text.match(new RegExp(`^(v\\d+): \\+${decimal} mol (.+)$`)))) {
    return `${match[1]}: +${match[2]} mol ${t(match[3]!)}`;
  }
  if ((match = text.match(/^The magnetic stirrer spins (v\d+) for ([\d.]+) seconds\.$/))) {
    return `Der Magnetrührer dreht ${match[1]} ${match[2]} Sekunden lang.`;
  }
  if ((match = text.match(/^(v\d+): magnetic stirrer ([\d.]+) rpm for ([\d.]+) s — bar tip ([\d.]+) m\/s; ([\d.]+)% resuspension$/))) {
    return `${match[1]}: Magnetrührer ${match[2]} U/min für ${match[3]} s — Rührstabspitze ${match[4]} m/s; ${match[5]} % wieder aufgeschwemmt`;
  }
  if ((match = text.match(/^You grind the (.+) in (v\d+) into a finer powder\.$/))) {
    return `Du mahlst ${t(match[1]!)} in ${match[2]} zu einem feineren Pulver.`;
  }
  if ((match = text.match(/^(v\d+): (.+) ground to ([\d.]+) µm — about ([\d.]+) m² surface area$/))) {
    return `${match[1]}: ${t(match[2]!)} auf ${match[3]} µm gemahlen — etwa ${match[4]} m² Oberfläche`;
  }
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
  if ((match = text.match(new RegExp(`^(v\\d+): (.+) in (.+) — ${decimal} mol dissolved \\(handbook limit\\), ${decimal} mol left as solid$`)))) {
    return `${match[1]}: ${t(match[2]!)} in ${t(match[3]!)} — ${match[4]} mol gelöst (Handbuchgrenze), ${match[5]} mol bleiben als Feststoff`;
  }
  if ((match = text.match(/^The (.+) just sits at the bottom of the (.+) — it will not dissolve\.$/))) {
    return `${t(match[1]!)} bleibt am Boden von ${t(match[2]!)} liegen — es löst sich nicht.`;
  }
  if ((match = text.match(/^The (.+) disappears into the (.+)\.$/))) {
    return `${t(match[1]!)} löst sich vollständig in ${t(match[2]!)}.`;
  }
  if ((match = text.match(/^A little of the (.+) dissolves in the (.+); the rest sits on the bottom\.$/))) {
    return `Ein wenig ${t(match[1]!)} löst sich in ${t(match[2]!)}; der Rest bleibt am Boden.`;
  }
  if ((match = text.match(/^(.+) vapour is hazardous to inhale$/))) {
    return `${t(match[1]!)}dampf ist beim Einatmen gefährlich`;
  }

  // Numeric state lines need only their stable vocabulary translated.
  if (/^v\d+ \([^)]+\) — /.test(text)) {
    return text
      .replace("(beaker)", "(Becherglas)")
      .replace("(flask)", "(Kolben)")
      .replace("(tube)", "(Reagenzglas)")
      .replace("(cylinder)", "(Messzylinder)")
      .replace("(crucible)", "(Tiegel)")
      .replace(" mL liquid", " mL Flüssigkeit")
      .replace("open to atmosphere", "offen zur Atmosphäre")
      .replace("sealed ", "verschlossener ")
      .replace(" headspace", " Gasraum");
  }

  return text;
}
