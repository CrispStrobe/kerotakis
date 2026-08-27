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
  if ((match = text.match(/^The lamp shines on (v\d+)\. The light is applied, but photolysis is not connected yet\.$/))) {
    return `Die Lampe bestrahlt ${match[1]}. Das Licht wirkt ein, aber die Photolyse ist noch nicht gekoppelt.`;
  }
  if ((match = text.match(/^(v\d+): lamp ([\d.]+) nm at ([\d.]+) W\/m² — photolysis (coupled|not yet coupled)$/))) {
    return `${match[1]}: Lampe ${match[2]} nm bei ${match[3]} W/m² — Photolyse ${match[4] === "coupled" ? "gekoppelt" : "noch nicht gekoppelt"}`;
  }
  if ((match = text.match(/^(v\d+): irradiate λ=([\d.]+) nm, Ė\/A=([\d.]+) W\/m²; photolysis_coupled=(true|false)$/))) {
    return `${match[1]}: Bestrahlung λ=${match[2]} nm, Ė/A=${match[3]} W/m²; Photolyse gekoppelt=${match[4] === "true" ? "ja" : "nein"}`;
  }
  if ((match = text.match(new RegExp(`^${decimal} g of (.+) builds up on the electrode in (v\\d+)\\.$`)))) {
    return `${match[1]} g ${t(match[2]!)} scheiden sich an der Elektrode in ${match[3]} ab.`;
  }
  if ((match = text.match(new RegExp(`^(v\\d+): ${decimal} A for ${decimal} s = ${decimal} C → ${decimal} mol e⁻ → ${decimal} mol (.+) = ${decimal} g$`)))) {
    return `${match[1]}: ${match[2]} A für ${match[3]} s = ${match[4]} C → ${match[5]} mol e⁻ → ${match[6]} mol ${t(match[7]!)} = ${match[8]} g`;
  }
  if ((match = text.match(new RegExp(
    `^(v\\d+): I = ${decimal} A; t = ${decimal} s; Q = It = ${decimal} C; n\\(e⁻\\) = Q/F = ${decimal} mol; n\\((.+)\\) = n\\(e⁻\\)/${decimal} = ${decimal} mol; m = ${decimal} g — only the ${decimal} is chemistry\\. Inert anode assumed: the water is oxidised there, so the oxygen leaves and the acid stays$`,
  )))) {
    return `${match[1]}: I = ${match[2]} A; t = ${match[3]} s; Q = It = ${match[4]} C; n(e⁻) = Q/F = ${match[5]} mol; n(${t(match[6]!)}) = n(e⁻)/${match[7]} = ${match[8]} mol; m = ${match[9]} g — nur die ${match[10]} stammt aus der Chemie. Inerte Anode angenommen: Dort wird Wasser oxidiert; der Sauerstoff entweicht und die Säure bleibt zurück`;
  }
  if ((match = text.match(/^(v\d+) (receives|releases) ([\d.]+) kJ of heat\. This energy step has no elapsed-time model yet\.$/))) {
    return `${match[1]} ${match[2] === "receives" ? "nimmt" : "gibt"} ${match[3]} kJ Wärme ${match[2] === "receives" ? "auf" : "ab"}. Dieser Energieschritt hat noch kein Zeitmodell.`;
  }
  if ((match = text.match(/^(v\d+): ([\d.]+) kJ requested; ([\d.]+) kJ (delivered|removed) — time model (coupled|not yet coupled)$/))) {
    return `${match[1]}: ${match[2]} kJ angefordert; ${match[3]} kJ ${match[4] === "delivered" ? "zugeführt" : "entzogen"} — Zeitmodell ${match[5] === "coupled" ? "gekoppelt" : "noch nicht gekoppelt"}`;
  }
  if ((match = text.match(/^(v\d+): thermal energy requested=([\d.]+) J, delivered=([\d.]+) J, heating=(true|false), time_coupled=(true|false)$/))) {
    return `${match[1]}: Wärmeenergie angefordert=${match[2]} J, übertragen=${match[3]} J, Erwärmung=${match[4] === "true" ? "ja" : "nein"}, Zeitmodell gekoppelt=${match[5] === "true" ? "ja" : "nein"}`;
  }
  if ((match = text.match(/^(titration of v\d+) with (.+) \(([^)]+) M\)$/))) {
    return `Titration von ${match[1]!.slice("titration of ".length)} mit ${t(match[2]!)} (${match[3]} M)`;
  }
  if ((match = text.match(new RegExp(`^no pop — H₂ mole fraction ${decimal}% is below the ${decimal}% ignition limit$`)))) {
    return `kein Knall — H₂-Molenbruch ${match[1]} % liegt unter der Zündgrenze von ${match[2]} %`;
  }
  if ((match = text.match(new RegExp(`^squeaky pop — ${decimal} mol H₂ ignited with ${decimal} mol O₂, producing ${decimal} mol H₂O; 2 H₂ \\+ O₂ → 2 H₂O$`)))) {
    return `quietschender Knall — ${match[1]} mol H₂ entzündeten sich mit ${match[2]} mol O₂ und erzeugten ${match[3]} mol H₂O; 2 H₂ + O₂ → 2 H₂O`;
  }
  if ((match = text.match(new RegExp(`^the glowing splint relights — O₂ mole fraction ${decimal}% \\(above the ${decimal}% enrichment threshold\\)$`)))) {
    return `der glimmende Span flammt auf — O₂-Molenbruch ${match[1]} % (über der Anreicherungsschwelle von ${match[2]} %)`;
  }
  if ((match = text.match(new RegExp(`^the splint does not relight — O₂ mole fraction ${decimal}% is below the ${decimal}% enrichment threshold$`)))) {
    return `der Span flammt nicht auf — O₂-Molenbruch ${match[1]} % liegt unter der Anreicherungsschwelle von ${match[2]} %`;
  }
  if ((match = text.match(new RegExp(`^limewater stays clear — CO₂ mole fraction ${decimal}% is below the ${decimal}% detection floor$`)))) {
    return `Kalkwasser bleibt klar — CO₂-Molenbruch ${match[1]} % liegt unter der Nachweisgrenze von ${match[2]} %`;
  }
  if ((match = text.match(new RegExp(`^limewater turns milky — CO₂ detected \\(mole fraction ${decimal}%\\); ${decimal} mol CO₂ consumed; CO₂ \\+ Ca\\(OH\\)₂ → CaCO₃↓ \\+ H₂O \\(curated stoichiometry, limewater not modelled as a vessel\\)$`)))) {
    return `Kalkwasser wird milchig — CO₂ nachgewiesen (Molenbruch ${match[1]} %); ${match[2]} mol CO₂ verbraucht; CO₂ + Ca(OH)₂ → CaCO₃↓ + H₂O (kuratierte Stöchiometrie; Kalkwasser nicht als Gefäß modelliert)`;
  }
  if ((match = text.match(new RegExp(`^damp red litmus turns blue — NH₃ detected \\(mole fraction ${decimal}%\\)$`)))) {
    return `feuchtes rotes Lackmuspapier wird blau — NH₃ nachgewiesen (Molenbruch ${match[1]} %)`;
  }
  if ((match = text.match(new RegExp(`^litmus stays red — NH₃ mole fraction ${decimal}% is below the ${decimal}% detection floor$`)))) {
    return `Lackmuspapier bleibt rot — NH₃-Molenbruch ${match[1]} % liegt unter der Nachweisgrenze von ${match[2]} %`;
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

  if ((match = text.match(/^missing argument: (.+)$/))) {
    return `Fehlendes Argument: ${match[1]}`;
  }
  if ((match = text.match(/^unknown relation '(.+)'$/))) {
    return `Unbekannte Beziehung „${match[1]}“`;
  }

  const relationText = /Nernst equation|Arrhenius equation|Eyring equation|Henderson[–-]Hasselbalch|Ionic strength|Lewis and Randall|Debye–Hückel|Van 't Hoff|WARNING: I >/.test(text);
  if (relationText) {
    return text
      .replaceAll("Nernst equation", "Nernst-Gleichung")
      .replaceAll("Arrhenius equation", "Arrhenius-Gleichung")
      .replaceAll("Eyring equation", "Eyring-Gleichung")
      .replaceAll("transition state theory", "Übergangszustandstheorie")
      .replaceAll("Henderson–Hasselbalch equation", "Henderson-Hasselbalch-Gleichung")
      .replaceAll("Ionic strength", "Ionenstärke")
      .replaceAll("Debye–Hückel limiting law", "Debye-Hückel-Grenzgesetz")
      .replaceAll("Van 't Hoff equation", "Van-'t-Hoff-Gleichung")
      .replaceAll("modified form", "modifizierte Form")
      .replaceAll("constants:", "Konstanten:")
      .replaceAll(" electrons", " Elektronen")
      .replaceAll(" ion(s):", " Ion(en):")
      .replace(/\bion (\d+):/g, "Ion $1:")
      .replaceAll("base fraction", "Basenanteil")
      .replaceAll("ratio K₂/K₁", "Verhältnis K₂/K₁")
      .replaceAll("endothermic", "endotherm")
      .replaceAll("exothermic", "exotherm")
      .replaceAll(" at T₁", " bei T₁")
      .replaceAll(" at 25 °C in water", " bei 25 °C in Wasser")
      .replaceAll("valid only for", "nur gültig für")
      .replaceAll("above that, use an extended or Pitzer model", "darüber ist ein erweitertes oder ein Pitzer-Modell nötig")
      .replaceAll("(and PHREEQC's activity coefficients are the real ones)", "(maßgeblich sind die Aktivitätskoeffizienten von PHREEQC)")
      .replaceAll("WARNING: I > 0.01 mol/kg — the limiting law is outside its validity domain", "WARNUNG: I > 0.01 mol/kg — das Grenzgesetz liegt außerhalb seines Gültigkeitsbereichs")
      .replaceAll("Outside validity domain (I > 0.01 mol/kg). The extended Debye–Hückel, Davies or Pitzer model would give a different (better) answer, and PHREEQC uses one of those.", "Außerhalb des Gültigkeitsbereichs (I > 0.01 mol/kg). Das erweiterte Debye-Hückel-, Davies- oder Pitzer-Modell liefert ein anderes (besseres) Ergebnis; PHREEQC verwendet eines davon.");
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
