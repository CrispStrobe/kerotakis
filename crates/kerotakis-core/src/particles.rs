//! The submicroscopic view: what the particles are doing.
//!
//! Johnstone's triangle says chemistry lives at three levels — what you
//! see, the particles underneath, and the symbols we write — and that
//! novices fail because instruction moves between them without saying so.
//! This engine could already render two of the three. An earlier version of
//! the plan claimed the third, mapping lv3 onto "submicroscopic", and that
//! was wrong: a table of molalities and activity coefficients is not the
//! particle level, it is *deeper symbolic*. Still numbers, still formulae.
//! The triangle was open on one side.
//!
//! This closes it, and the reason it can be closed honestly is that the
//! engine already computes the census. `Ag⁺ 9.56e-6 · AgCl(aq) 3.21e-7` is
//! a solved answer about how many of each particle there are, so drawing
//! dots in those proportions is a *rendering of a result*, not an artist's
//! impression. That is the difference between this and a textbook diagram,
//! and it is the whole reason the picture is worth showing.
//!
//! **Scale is where a particle picture can lie**, so it is made explicit.
//! You cannot draw 10²³ of anything; you draw a few dozen and each one
//! stands for some amount. A species too dilute to earn a single dot is
//! therefore *named* rather than quietly dropped — the same discipline the
//! rest of the engine applies to everything it declines to model. A picture
//! that silently omits the neutral complex teaches that the complex is not
//! there.

use serde::{Deserialize, Serialize};

use crate::species::{self, Phase};
use crate::vessel::Vessel;

/// What kind of thing a particle is — which is what the picture is *for*.
/// A learner who cannot tell an ion from a molecule cannot read a solution.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Cation,
    Anion,
    /// Dissolved and uncharged: a neutral complex or a molecular solute.
    NeutralSolute,
    Solvent,
    Solid,
    Gas,
}

impl Kind {
    /// A glyph a terminal can draw. Shape carries the meaning, so the
    /// picture survives being printed in one colour.
    pub fn glyph(self) -> char {
        match self {
            Kind::Cation => '●',
            Kind::Anion => '○',
            Kind::NeutralSolute => '◍',
            Kind::Solvent => '·',
            Kind::Solid => '▪',
            Kind::Gas => '°',
        }
    }

    pub fn describe(self) -> &'static str {
        match self {
            Kind::Cation => "positive ion",
            Kind::Anion => "negative ion",
            Kind::NeutralSolute => "uncharged, dissolved",
            Kind::Solvent => "solvent",
            Kind::Solid => "solid",
            Kind::Gas => "gas",
        }
    }
}

/// One kind of particle, and how many of it to draw.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Population {
    pub label: String,
    pub kind: Kind,
    /// How many glyphs this population gets at the census's scale.
    pub drawn: usize,
    /// The underlying amount — mol/kgw where speciation was solved, mol
    /// otherwise.
    pub amount: f64,
}

/// Where the numbers came from. A picture drawn from solved speciation is a
/// different claim from one drawn off the inventory, and the viewer is
/// entitled to know which they are looking at.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// Ratios from the aqueous engine's species distribution.
    Speciation,
    /// Ratios from the vessel's inventory: no solution was characterised,
    /// so ion pairs and complexes are not resolved and the picture is
    /// coarser than it looks.
    Inventory,
}

/// A drawable census of one vessel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Census {
    pub populations: Vec<Population>,
    /// Amount each glyph stands for.
    pub per_glyph: f64,
    /// Present, and too dilute to earn a glyph at this scale. Named rather
    /// than dropped: an omitted species reads as an absent one.
    pub too_rare: Vec<(String, f64)>,
    pub source: Source,
}

/// Solvent is drawn sparsely on purpose: at true proportions water is
/// essentially all there is, and a picture that is 99.9% water teaches
/// nothing about the solute. This is the one place the picture is
/// deliberately not to scale, so it is a named constant and the renderer
/// says so.
const SOLVENT_GLYPHS: usize = 6;

fn kind_of(name: &str, phase: Phase) -> Kind {
    match phase {
        Phase::Solid => return Kind::Solid,
        Phase::Gas => return Kind::Gas,
        _ => {}
    }
    if name == "water" || name == "H2O" {
        return Kind::Solvent;
    }
    // Charge from the formula's trailing sign, which is how both the
    // registry and PHREEQC write it.
    match crate::stoich::parse_formula(name) {
        Ok(f) if f.charge > 0.0 => Kind::Cation,
        Ok(f) if f.charge < 0.0 => Kind::Anion,
        Ok(_) => Kind::NeutralSolute,
        // A name the formula parser cannot read is not thereby uncharged.
        // The census draws whatever the solve returned, and a solve returns
        // the DATABASE's spelling — `Acetate-`, `Mg(Acetate)+`, `Citrate-3`
        // — whose pseudo-elements are not elements, so the parse fails and
        // the old catch-all called every one of them neutral. Acetate ion
        // was drawn as an uncharged molecule in a picture whose entire
        // subject is which particles carry charge.
        //
        // The sign is still right there at the end of the name, which is
        // what the comment above already claims the rule is. Read it.
        Err(_) => match trailing_charge(name) {
            Some(q) if q > 0 => Kind::Cation,
            Some(q) if q < 0 => Kind::Anion,
            _ => Kind::NeutralSolute,
        },
    }
}

/// The charge written at the end of a species name: `-`, `+`, `-2`, `+3`.
///
/// `None` means no sign is written, which for these names means neutral —
/// PHREEQC and the registry both put the sign last or not at all, so an
/// absent sign is a statement rather than a gap.
fn trailing_charge(name: &str) -> Option<i32> {
    let magnitude = name.trim_end_matches(|c: char| c.is_ascii_digit());
    let digits = &name[magnitude.len()..];
    let n: i32 = if digits.is_empty() {
        1
    } else {
        digits.parse().ok()?
    };
    match magnitude.chars().last()? {
        '+' => Some(n),
        '-' => Some(-n),
        _ => None,
    }
}

/// Take a census of this vessel, drawing about `glyphs` particles in total.
///
/// Prefers the solved species distribution, because that is the answer to
/// the question the picture asks. Falls back to the inventory, and says so.
pub fn census(vessel: &Vessel, glyphs: usize) -> Census {
    let glyphs = glyphs.max(4);

    // (label, amount, kind) for whichever source we have.
    let (mut rows, source): (Vec<(String, f64, Kind)>, Source) = match &vessel.solution {
        Some(info) if !info.species.is_empty() => (
            info.species
                .iter()
                .filter(|s| s.name != "H2O")
                .map(|s| {
                    let kind = kind_of(&s.name, Phase::Aqueous);
                    (s.name.clone(), s.molality, kind)
                })
                .collect(),
            Source::Speciation,
        ),
        _ => (
            vessel
                .contents
                .iter()
                .filter(|p| !matches!(kind_of(&p.species.0, p.phase), Kind::Solvent))
                .map(|p| {
                    let label = species::lookup(&p.species)
                        .map(|d| d.formula.to_string())
                        .unwrap_or_else(|| p.species.0.clone());
                    (label, p.moles.0, kind_of(&p.species.0, p.phase))
                })
                .collect(),
            Source::Inventory,
        ),
    };

    // Solids and gases are inventory facts either way — the speciation
    // block only describes what is dissolved. They arrive in *moles* while
    // the speciation is in mol/kgw, so they are converted onto the same
    // basis before anything is scaled. Mixing the two put a whole
    // precipitate below the drawing threshold: 0.01 mol of silver chloride
    // in 200 mL is 0.05 mol/kgw, one of the larger populations in the
    // beaker, and it was being reported as too rare to draw.
    if source == Source::Speciation {
        let kgw = vessel
            .contents
            .iter()
            .filter(|p| matches!(kind_of(&p.species.0, p.phase), Kind::Solvent))
            .map(|p| p.moles.0 * 0.018_015)
            .sum::<f64>()
            .max(1e-9);
        for p in &vessel.contents {
            if matches!(p.phase, Phase::Solid | Phase::Gas) {
                let label = species::lookup(&p.species)
                    .map(|d| d.formula.to_string())
                    .unwrap_or_else(|| p.species.0.clone());
                rows.push((label, p.moles.0 / kgw, kind_of(&p.species.0, p.phase)));
            }
        }
    }

    rows.retain(|(_, amount, _)| *amount > 0.0);
    rows.sort_by(|a, b| b.1.total_cmp(&a.1));

    let total: f64 = rows.iter().map(|(_, a, _)| *a).sum();
    let has_solvent = vessel
        .contents
        .iter()
        .any(|p| matches!(kind_of(&p.species.0, p.phase), Kind::Solvent));
    let budget = if has_solvent {
        glyphs.saturating_sub(SOLVENT_GLYPHS).max(1)
    } else {
        glyphs
    };
    let per_glyph = if total > 0.0 {
        total / budget as f64
    } else {
        1.0
    };

    let mut populations = Vec::new();
    let mut too_rare = Vec::new();
    if has_solvent {
        populations.push(Population {
            label: "H2O".to_string(),
            kind: Kind::Solvent,
            drawn: SOLVENT_GLYPHS,
            amount: vessel
                .contents
                .iter()
                .filter(|p| matches!(kind_of(&p.species.0, p.phase), Kind::Solvent))
                .map(|p| p.moles.0)
                .sum(),
        });
    }
    for (label, amount, kind) in rows {
        let drawn = (amount / per_glyph).round() as usize;
        if drawn == 0 {
            too_rare.push((label, amount));
        } else {
            populations.push(Population {
                label,
                kind,
                drawn,
                amount,
            });
        }
    }

    Census {
        populations,
        per_glyph,
        too_rare,
        source,
    }
}

impl Census {
    /// Draw it, at the depth the register asks for.
    pub fn render(&self, register: crate::render::Register) -> String {
        let mut out = String::new();
        for p in &self.populations {
            let row: String = std::iter::repeat_n(p.kind.glyph(), p.drawn.min(40)).collect();
            match register.level() {
                1 => out.push_str(&format!("  {row}   {}\n", plain_name(&p.label))),
                2 => out.push_str(&format!("  {row}   {}  ({})\n", p.label, p.kind.describe())),
                _ => out.push_str(&format!(
                    "  {row}   {:<12} {:.4e}  {}\n",
                    p.label,
                    p.amount,
                    p.kind.describe()
                )),
            }
        }
        if register.level() >= 2 {
            out.push_str(&format!(
                "  one {} ≈ {:.3e} {}",
                Kind::Cation.glyph(),
                self.per_glyph,
                match self.source {
                    Source::Speciation => "mol/kgw",
                    Source::Inventory => "mol",
                }
            ));
            if self.populations.iter().any(|p| p.kind == Kind::Solvent) {
                out.push_str("; the water is drawn sparsely, not to scale");
            }
            out.push('\n');
        }
        if !self.too_rare.is_empty() {
            let names: Vec<String> = self
                .too_rare
                .iter()
                .take(4)
                .map(|(n, a)| match register.level() {
                    1 => plain_name(n).to_string(),
                    _ => format!("{n} ({a:.2e})"),
                })
                .collect();
            let more = match self.too_rare.len().saturating_sub(4) {
                0 => String::new(),
                n => format!(", and {n} more"),
            };
            out.push_str(&match register.level() {
                1 => format!(
                    "  also in there, too few to draw: {}{more}\n",
                    names.join(", ")
                ),
                _ => format!(
                    "  present below one glyph, so not drawn: {}{more}\n",
                    names.join(", ")
                ),
            });
        }
        if self.source == Source::Inventory && register.level() >= 2 {
            out.push_str(
                "  drawn from the inventory: no solution was characterised, so ion pairs and complexes are not resolved here\n",
            );
        }
        out
    }
}

/// A formula a nine-year-old can read out loud.
fn plain_name(formula: &str) -> &str {
    species::REGISTRY
        .iter()
        .find(|d| d.formula == formula || d.key == formula)
        .map(|d| d.name)
        .unwrap_or(formula)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn salty() -> Vessel {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.deposit(SpeciesId::new("water"), Moles(5.5343), Phase::Liquid);
        v.solution = Some(crate::vessel::SolutionInfo {
            redox: Vec::new(),
            pe: None,
            ph: 7.0,
            ionic_strength: 0.1,
            species: vec![
                crate::vessel::SpeciesDetail {
                    name: "Na+".to_string(),
                    molality: 0.1,
                    activity: 0.078,
                },
                crate::vessel::SpeciesDetail {
                    name: "Cl-".to_string(),
                    molality: 0.1,
                    activity: 0.078,
                },
                crate::vessel::SpeciesDetail {
                    name: "AgCl".to_string(),
                    molality: 3.2e-7,
                    activity: 3.3e-7,
                },
            ],
            provenance: None,
        });
        v
    }

    #[test]
    fn ions_are_drawn_in_computed_proportion() {
        let c = census(&salty(), 30);
        assert_eq!(c.source, Source::Speciation);
        let na = c.populations.iter().find(|p| p.label == "Na+").unwrap();
        let cl = c.populations.iter().find(|p| p.label == "Cl-").unwrap();
        assert_eq!(na.drawn, cl.drawn, "equal molality, equal dots");
        assert_eq!(na.kind, Kind::Cation);
        assert_eq!(cl.kind, Kind::Anion);
    }

    #[test]
    fn a_species_too_rare_to_draw_is_named_not_dropped() {
        // The whole point. AgCl(aq) at 3e-7 mol/kgw cannot earn a dot beside
        // 0.1 mol/kgw of sodium, and a picture that silently omitted it
        // would teach that the neutral complex is not there.
        let c = census(&salty(), 30);
        assert!(
            c.populations.iter().all(|p| p.label != "AgCl"),
            "too dilute to draw"
        );
        assert!(
            c.too_rare.iter().any(|(n, _)| n == "AgCl"),
            "but it must still be reported: {:?}",
            c.too_rare
        );
        let text = c.render(Register::LV3);
        assert!(text.contains("AgCl"), "{text}");
    }

    #[test]
    fn the_scale_is_stated() {
        let text = census(&salty(), 30).render(Register::LV2);
        assert!(text.contains("one ● ≈"), "{text}");
        assert!(
            text.contains("not to scale"),
            "water must be flagged: {text}"
        );
    }

    #[test]
    fn without_speciation_the_picture_says_so() {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.deposit(SpeciesId::new("water"), Moles(5.5343), Phase::Liquid);
        v.deposit(SpeciesId::new("NaCl"), Moles(0.1), Phase::Solid);
        let c = census(&v, 30);
        assert_eq!(c.source, Source::Inventory);
        let text = c.render(Register::LV2);
        assert!(text.contains("no solution was characterised"), "{text}");
    }

    #[test]
    fn solids_appear_beside_the_dissolved_species() {
        // And on the same basis. A precipitate arrives in moles while the
        // speciation is in mol/kgw; scaling both against one divisor buried
        // a whole 0.01 mol of silver chloride below the drawing threshold.
        let mut v = salty();
        v.deposit(SpeciesId::new("AgCl"), Moles(0.01), Phase::Solid);
        let c = census(&v, 30);
        let solid = c
            .populations
            .iter()
            .find(|p| p.kind == Kind::Solid)
            .unwrap_or_else(|| panic!("no solid drawn: {:?} / {:?}", c.populations, c.too_rare));
        assert!(solid.drawn > 0);
        // 0.01 mol in ~0.0997 kg of water is ~0.1 mol/kgw — comparable with
        // the sodium, so it must get a comparable number of glyphs.
        let na = c.populations.iter().find(|p| p.label == "Na+").unwrap();
        assert!(
            solid.drawn >= na.drawn / 2,
            "solid {} glyphs vs sodium {}",
            solid.drawn,
            na.drawn
        );
    }

    #[test]
    fn an_empty_vessel_draws_nothing_and_does_not_panic() {
        let v = Vessel::new(VesselId(0), "beaker");
        let c = census(&v, 30);
        assert!(c.populations.is_empty());
        assert!(c.render(Register::LV1).is_empty());
    }
}

#[cfg(test)]
mod charge_label_tests {
    use super::*;

    /// The census draws whatever the solve returned, and a solve returns the
    /// database's spelling. Those names carry pseudo-elements the formula
    /// parser cannot read, so the parse fails — and failing to read a name
    /// is not evidence that the thing is uncharged.
    ///
    /// Found by looking at a real census of vinegar, which drew `Acetate-`
    /// as "uncharged, dissolved" in a picture whose whole subject is which
    /// particles carry charge. Nobody would ever have filed it: the number
    /// of particles was right and only the label was false.
    #[test]
    fn database_spellings_are_classified_by_their_written_sign() {
        for (name, expected) in [
            // The ones that broke: PHREEQC pseudo-elements.
            ("Acetate-", Kind::Anion),
            ("Citrate-3", Kind::Anion),
            ("Mg(Acetate)+", Kind::Cation),
            ("H(Acetate)", Kind::NeutralSolute),
            // And the ones that already worked must keep working — this
            // path is a fallback, not a replacement.
            ("Na+", Kind::Cation),
            ("SO4-2", Kind::Anion),
            ("CH3COO-", Kind::Anion),
            ("NH4+", Kind::Cation),
            ("NH3", Kind::NeutralSolute),
            ("O2", Kind::NeutralSolute),
        ] {
            assert_eq!(
                kind_of(name, Phase::Aqueous),
                expected,
                "{name} classified wrongly"
            );
        }
    }

    /// A trailing digit is not a charge unless a sign precedes it, or every
    /// diatomic gas in the registry would acquire one.
    #[test]
    fn a_trailing_digit_alone_is_not_a_charge() {
        assert_eq!(trailing_charge("O2"), None);
        assert_eq!(trailing_charge("H(Acetate)"), None);
        assert_eq!(trailing_charge("Acetate-"), Some(-1));
        assert_eq!(trailing_charge("Citrate-3"), Some(-3));
        assert_eq!(trailing_charge("Fe+2"), Some(2));
    }
}
