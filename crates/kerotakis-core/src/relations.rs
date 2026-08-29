//! Named physical relations — the equations a learner can ask, vary and see
//! explained.
//!
//! Each relation is the single source of truth: in-solver implementations
//! (Nernst in `displacement.rs`, Arrhenius in `kinetics.rs`,
//! Henderson–Hasselbalch in `indicator.rs`) call these shared functions, and
//! `kero calc` exposes them directly. Every result carries register text at
//! three detail levels and a provenance string naming the equation and its
//! domain.

use crate::constants;
use crate::units::Kelvin;

/// The result of evaluating a named relation.
#[derive(Debug, Clone)]
pub struct RelationResult {
    pub value: f64,
    pub unit: &'static str,
    pub provenance: &'static str,
    pub lv1: String,
    pub lv2: String,
    pub lv3: String,
}

// ── Nernst equation ──────────────────────────────────────────────────

const NERNST_PROVENANCE: &str = "Nernst equation (W. Nernst, 1889); constants: CODATA 2018";

/// RT ln(10) / F at this temperature — the Nernst slope, V per decade
/// of activity. 0.05916 V at 25 °C.
pub fn nernst_slope(temperature: Kelvin) -> f64 {
    constants::GAS_CONSTANT * temperature.0 * std::f64::consts::LN_10 / constants::FARADAY
}

/// Nernst equation: E = E° + (RT ln 10)/(nF) · log₁₀(a).
pub fn nernst(e0: f64, n: f64, activity: f64, temperature: Kelvin) -> RelationResult {
    let slope = nernst_slope(temperature);
    let value = e0 + slope / n * activity.log10();
    RelationResult {
        value,
        unit: "V",
        provenance: NERNST_PROVENANCE,
        lv1: format!("{value:+.4} V"),
        lv2: format!(
            "E = E° + (RT ln 10)/(nF) · log₁₀(a) = {:+.4} + {:.5}/{:.0} · log₁₀({:.4e}) = {:+.4} V",
            e0, slope, n, activity, value
        ),
        lv3: format!(
            "Nernst equation: E = E° + (RT ln 10)/(nF) · log₁₀(a)\n\
             \x20 E° = {:+.6} V\n\
             \x20 n  = {:.0} electrons\n\
             \x20 a  = {:.6e}\n\
             \x20 T  = {:.2} K ({:.1} °C)\n\
             \x20 RT ln 10 / F = {:.6} V/decade\n\
             \x20 E  = {:+.6} V\n\
             \x20 {}",
            e0,
            n,
            activity,
            temperature.0,
            temperature.to_celsius(),
            slope,
            value,
            NERNST_PROVENANCE
        ),
    }
}

// ── Arrhenius equation ───────────────────────────────────────────────

const ARRHENIUS_PROVENANCE: &str =
    "Arrhenius equation (S. Arrhenius, 1889); modified form k = A·T^b·exp(−Ea/RT); constants: CODATA 2018";

/// Modified Arrhenius rate constant: k(T) = A·T^b·exp(−Ea/RT).
///
/// This is the single source of truth for every rate constant in the
/// project. `kinetics::RateLaw::rate_constant()` delegates here.
pub fn arrhenius(
    pre_exponential: f64,
    temperature_exponent: f64,
    activation_energy: f64,
    temperature_k: f64,
) -> f64 {
    let temperature_k = temperature_k.max(1.0);
    pre_exponential
        * temperature_k.powf(temperature_exponent)
        * (-activation_energy / (constants::GAS_CONSTANT * temperature_k)).exp()
}

/// Full Arrhenius evaluation with register text.
pub fn arrhenius_result(
    pre_exponential: f64,
    temperature_exponent: f64,
    activation_energy: f64,
    temperature_k: f64,
) -> RelationResult {
    let k = arrhenius(
        pre_exponential,
        temperature_exponent,
        activation_energy,
        temperature_k,
    );
    let lv2 = if temperature_exponent == 0.0 {
        format!(
            "k = A·exp(−Ea/RT) = {:.4e}·exp(−{:.0}/({:.4}×{:.2})) = {k:.6e}",
            pre_exponential,
            activation_energy,
            constants::GAS_CONSTANT,
            temperature_k
        )
    } else {
        format!(
            "k = A·T^b·exp(−Ea/RT) = {:.4e}·{:.2}^{:.2}·exp(−{:.0}/({:.4}×{:.2})) = {k:.6e}",
            pre_exponential,
            temperature_k,
            temperature_exponent,
            activation_energy,
            constants::GAS_CONSTANT,
            temperature_k
        )
    };
    RelationResult {
        value: k,
        unit: "(rate constant units)",
        provenance: ARRHENIUS_PROVENANCE,
        lv1: format!("k = {k:.6e}"),
        lv2,
        lv3: format!(
            "Arrhenius equation: k = A·T^b·exp(−Ea/RT)\n\
             \x20 A  = {:.6e}\n\
             \x20 b  = {:.6}\n\
             \x20 Ea = {:.1} J/mol ({:.1} kJ/mol)\n\
             \x20 T  = {:.2} K ({:.1} °C)\n\
             \x20 k  = {:.6e}\n\
             \x20 {}",
            pre_exponential,
            temperature_exponent,
            activation_energy,
            activation_energy / 1000.0,
            temperature_k,
            temperature_k - 273.15,
            k,
            ARRHENIUS_PROVENANCE
        ),
    }
}

// ── Eyring equation ──────────────────────────────────────────────────

const EYRING_PROVENANCE: &str =
    "Eyring equation (H. Eyring, 1935); transition state theory; constants: CODATA 2018";

/// Eyring (transition state theory) rate constant:
/// k = (kB·T / h) · exp(−ΔG‡ / RT).
///
/// `delta_g_dagger` is the Gibbs energy of activation in J/mol.
pub fn eyring(delta_g_dagger: f64, temperature_k: f64) -> f64 {
    let temperature_k = temperature_k.max(1.0);
    (constants::BOLTZMANN * temperature_k / constants::PLANCK)
        * (-delta_g_dagger / (constants::GAS_CONSTANT * temperature_k)).exp()
}

pub fn eyring_result(delta_g_dagger: f64, temperature_k: f64) -> RelationResult {
    let k = eyring(delta_g_dagger, temperature_k);
    let prefactor = constants::BOLTZMANN * temperature_k / constants::PLANCK;
    RelationResult {
        value: k,
        unit: "s⁻¹",
        provenance: EYRING_PROVENANCE,
        lv1: format!("k = {k:.6e} s⁻¹"),
        lv2: format!(
            "k = (kB·T/h)·exp(−ΔG‡/RT) = {:.4e}·exp(−{:.0}/({:.4}×{:.2})) = {k:.6e} s⁻¹",
            prefactor,
            delta_g_dagger,
            constants::GAS_CONSTANT,
            temperature_k
        ),
        lv3: format!(
            "Eyring equation (transition state theory): k = (kB·T/h)·exp(−ΔG‡/RT)\n\
             \x20 ΔG‡ = {:.1} J/mol ({:.1} kJ/mol)\n\
             \x20 T   = {:.2} K ({:.1} °C)\n\
             \x20 kB·T/h = {:.6e} s⁻¹\n\
             \x20 k   = {:.6e} s⁻¹\n\
             \x20 {}",
            delta_g_dagger,
            delta_g_dagger / 1000.0,
            temperature_k,
            temperature_k - 273.15,
            prefactor,
            k,
            EYRING_PROVENANCE
        ),
    }
}

// ── Henderson–Hasselbalch ────────────────────────────────────────────

const HH_PROVENANCE: &str =
    "Henderson–Hasselbalch equation (L.J. Henderson, 1908; K.A. Hasselbalch, 1917)";

/// Fraction of the base (conjugate) form at a given pH:
/// f = 1 / (1 + 10^(pKa − pH)).
///
/// This is the Henderson–Hasselbalch equation rearranged. The indicator
/// module's `base_fraction` delegates here.
pub fn henderson_hasselbalch_fraction(pka: f64, ph: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf(pka - ph))
}

/// pH from pKa and the ratio [A⁻]/[HA]:
/// pH = pKa + log₁₀([A⁻]/[HA]).
pub fn henderson_hasselbalch_ph(pka: f64, ratio: f64) -> f64 {
    pka + ratio.log10()
}

pub fn henderson_hasselbalch_result(pka: f64, c_acid: f64, c_base: f64) -> RelationResult {
    let ratio = c_base / c_acid;
    let ph = henderson_hasselbalch_ph(pka, ratio);
    let fraction = henderson_hasselbalch_fraction(pka, ph);
    RelationResult {
        value: ph,
        unit: "",
        provenance: HH_PROVENANCE,
        lv1: format!("pH = {ph:.2}"),
        lv2: format!(
            "pH = pKa + log₁₀([A⁻]/[HA]) = {:.2} + log₁₀({:.4}/{:.4}) = {ph:.4}",
            pka, c_base, c_acid
        ),
        lv3: format!(
            "Henderson–Hasselbalch: pH = pKa + log₁₀([A⁻]/[HA])\n\
             \x20 pKa    = {:.4}\n\
             \x20 [HA]   = {:.6e} M\n\
             \x20 [A⁻]   = {:.6e} M\n\
             \x20 ratio  = {:.6}\n\
             \x20 pH     = {:.4}\n\
             \x20 base fraction = {:.4}\n\
             \x20 {}",
            pka, c_acid, c_base, ratio, ph, fraction, HH_PROVENANCE
        ),
    }
}

// ── Ionic strength ───────────────────────────────────────────────────

const IONIC_STRENGTH_PROVENANCE: &str = "Lewis and Randall (1921); I = ½ Σ mᵢ zᵢ²";

/// Ionic strength: I = ½ Σ mᵢ zᵢ².
///
/// `species` is a slice of (charge, molality) pairs.
pub fn ionic_strength(species: &[(f64, f64)]) -> f64 {
    0.5 * species.iter().map(|(z, m)| m * z * z).sum::<f64>()
}

pub fn ionic_strength_result(species: &[(f64, f64)]) -> RelationResult {
    let value = ionic_strength(species);
    let terms: String = species
        .iter()
        .map(|(z, m)| format!("{m:.4}×{:.0}²", z.abs()))
        .collect::<Vec<_>>()
        .join(" + ");
    RelationResult {
        value,
        unit: "mol/kg",
        provenance: IONIC_STRENGTH_PROVENANCE,
        lv1: format!("I = {value:.4} mol/kg"),
        lv2: format!("I = ½({terms}) = {value:.6} mol/kg"),
        lv3: format!(
            "Ionic strength: I = ½ Σ mᵢ zᵢ²\n\
             \x20 {} ion(s):\n{}\
             \x20 I = {:.6} mol/kg\n\
             \x20 {}",
            species.len(),
            species
                .iter()
                .enumerate()
                .map(|(i, (z, m))| format!(
                    "\x20   ion {}: z = {:+.0}, m = {:.6e} mol/kg\n",
                    i + 1,
                    z,
                    m
                ))
                .collect::<String>(),
            value,
            IONIC_STRENGTH_PROVENANCE
        ),
    }
}

// ── Debye–Hückel limiting law ────────────────────────────────────────

const DEBYE_HUCKEL_PROVENANCE: &str = "Debye–Hückel limiting law (P. Debye and E. Hückel, 1923); \
     A = 0.5091 (mol/kg)^(−½) at 25 °C in water; \
     valid only for I ≲ 0.01 mol/kg — above that, use an extended or \
     Pitzer model (and PHREEQC's activity coefficients are the real ones)";

/// Debye–Hückel A parameter at 25 °C in water: 0.5091 (mol/kg)^(−½).
///
/// Temperature dependence requires ε(T) and ρ(T) (CAP-6); this value
/// is accurate at 25 °C ± 10 °C and degrades outside that window.
pub const A_DH_25C: f64 = 0.5091;

/// Debye–Hückel limiting law: log₁₀(γᵢ) = −A zᵢ² √I.
///
/// Returns log₁₀ of the single-ion activity coefficient. This is the
/// textbook approximation, valid only for I ≲ 0.01 mol/kg; it
/// systematically overpredicts non-ideality at higher concentrations.
/// PHREEQC's activity coefficients (Davies, Pitzer, SIT) are the ones
/// the engine actually uses; this is the equation a learner derives,
/// and the disagreement is a lesson.
pub fn debye_huckel_log_gamma(charge: f64, ionic_strength_val: f64) -> f64 {
    -A_DH_25C * charge * charge * ionic_strength_val.sqrt()
}

pub fn debye_huckel_result(charge: f64, ionic_strength_val: f64) -> RelationResult {
    let log_gamma = debye_huckel_log_gamma(charge, ionic_strength_val);
    let gamma = 10f64.powf(log_gamma);
    let validity = if ionic_strength_val > 0.01 {
        " WARNING: I > 0.01 mol/kg — the limiting law is outside its validity domain"
    } else {
        ""
    };
    RelationResult {
        value: gamma,
        unit: "",
        provenance: DEBYE_HUCKEL_PROVENANCE,
        lv1: format!("γ = {gamma:.4}{validity}"),
        lv2: format!(
            "log₁₀(γ) = −A z² √I = −{:.4} × {:.0}² × √{:.6} = {:.4}; γ = {gamma:.4}{validity}",
            A_DH_25C, charge, ionic_strength_val, log_gamma
        ),
        lv3: format!(
            "Debye–Hückel limiting law: log₁₀(γᵢ) = −A zᵢ² √I\n\
             \x20 A = {:.4} (mol/kg)^(−½) at 25 °C\n\
             \x20 z = {:+.0}\n\
             \x20 I = {:.6} mol/kg\n\
             \x20 log₁₀(γ) = {:.6}\n\
             \x20 γ = {:.6}\n\
             {}\
             \x20 {}",
            A_DH_25C,
            charge,
            ionic_strength_val,
            log_gamma,
            gamma,
            if ionic_strength_val > 0.01 {
                "\x20 ⚠ Outside validity domain (I > 0.01 mol/kg). The extended Debye–Hückel, Davies \
                 or Pitzer model would give a different (better) answer, and PHREEQC uses one of those.\n"
            } else {
                ""
            },
            DEBYE_HUCKEL_PROVENANCE
        ),
    }
}

// ── Van 't Hoff equation ─────────────────────────────────────────────

const VAN_T_HOFF_PROVENANCE: &str = "Van 't Hoff equation (J.H. van 't Hoff, 1884); \
     ln(K₂/K₁) = −(ΔH°/R)(1/T₂ − 1/T₁); constants: CODATA 2018";

/// Van 't Hoff equation: K₂ = K₁ · exp[−(ΔH°/R)(1/T₂ − 1/T₁)].
///
/// `delta_h` is the standard enthalpy of reaction in J/mol (positive =
/// endothermic). Assumes ΔH° is constant over the temperature range.
pub fn van_t_hoff(delta_h: f64, k1: f64, t1: f64, t2: f64) -> f64 {
    k1 * ((delta_h / constants::GAS_CONSTANT) * (1.0 / t1 - 1.0 / t2)).exp()
}

pub fn van_t_hoff_result(delta_h: f64, k1: f64, t1: f64, t2: f64) -> RelationResult {
    let k2 = van_t_hoff(delta_h, k1, t1, t2);
    let ratio = k2 / k1;
    RelationResult {
        value: k2,
        unit: "",
        provenance: VAN_T_HOFF_PROVENANCE,
        lv1: format!("K₂ = {k2:.6e}"),
        lv2: format!(
            "K₂ = K₁·exp[−(ΔH°/R)(1/T₂ − 1/T₁)] = {:.4e}·exp[−({:.0}/{:.4})({:.6} − {:.6})] = {k2:.6e}",
            k1, delta_h, constants::GAS_CONSTANT, 1.0 / t2, 1.0 / t1
        ),
        lv3: format!(
            "Van 't Hoff equation: ln(K₂/K₁) = −(ΔH°/R)(1/T₂ − 1/T₁)\n\
             \x20 ΔH° = {:.1} J/mol ({:.1} kJ/mol, {})\n\
             \x20 K₁  = {:.6e} at T₁ = {:.2} K ({:.1} °C)\n\
             \x20 T₂  = {:.2} K ({:.1} °C)\n\
             \x20 K₂  = {:.6e} (ratio K₂/K₁ = {:.4})\n\
             \x20 {}",
            delta_h, delta_h / 1000.0,
            if delta_h > 0.0 { "endothermic" } else { "exothermic" },
            k1, t1, t1 - 273.15,
            t2, t2 - 273.15,
            k2, ratio, VAN_T_HOFF_PROVENANCE
        ),
    }
}

// ── Registry of relations ────────────────────────────────────────────

pub struct RelationInfo {
    pub name: &'static str,
    pub equation: &'static str,
    pub args: &'static str,
    /// What question this relation answers, in one sentence (GUI-087).
    pub purpose: &'static str,
    pub purpose_de: &'static str,
    /// Where it stops being true. A formula shipped without its validity
    /// range teaches a learner to apply it outside that range, which is the
    /// most common way these get misused.
    pub validity: &'static str,
    pub validity_de: &'static str,
    /// Who published it, and when (GUI-096). Deliberately the leading
    /// clause of this relation's `*_PROVENANCE` constant rather than a
    /// second citation written beside it: `sources_are_the_provenance_they_came_from`
    /// asserts the containment, so the catalogue cannot cite one paper
    /// while the computed result cites another.
    pub source: &'static str,
    pub source_de: &'static str,
}

pub const RELATIONS: &[RelationInfo] = &[
    RelationInfo {
        name: "nernst",
        equation: "E = E° + (RT ln 10)/(nF) · log₁₀(a)",
        args: "e0=<V> n=<electrons> a=<activity> T=<K>",
        purpose: "How far a half-cell's potential shifts when the species are not at unit activity.",
        purpose_de: "Wie weit sich das Potenzial einer Halbzelle verschiebt, wenn die Teilchen nicht in der Aktivität 1 vorliegen.",
        validity: "Needs activities, not concentrations — using concentrations is the usual source of the textbook 59 mV that a real cell does not deliver. Assumes equilibrium at the electrode and no current drawn, and that 59 mV per decade is the slope at 25 °C alone: RT ln 10 / F moves with temperature.",
        validity_de: "Braucht Aktivitäten, nicht Konzentrationen — mit Konzentrationen entstehen die 59 mV aus dem Schulbuch, die eine reale Zelle nicht liefert. Setzt Gleichgewicht an der Elektrode und stromlose Messung voraus; die 59 mV pro Dekade gelten nur bei 25 °C, denn RT ln 10 / F ändert sich mit der Temperatur.",
        source: "Nernst equation (W. Nernst, 1889)",
        source_de: "Nernst-Gleichung (W. Nernst, 1889)",
    },
    RelationInfo {
        name: "arrhenius",
        equation: "k = A·T^b·exp(−Ea/RT)",
        args: "A=<prefactor> Ea=<J/mol> T=<K> [b=<exponent>]",
        purpose: "How much faster a reaction runs when it is warmer.",
        purpose_de: "Um wie viel schneller eine Reaktion abläuft, wenn es wärmer ist.",
        validity: "Empirical: Ea and A are fitted to data, not derived, and both are treated as constant over the fitted range. Valid over that range; extrapolating far outside it, or across a change of mechanism, is not.",
        validity_de: "Empirisch: Ea und A werden an Messdaten angepasst, nicht hergeleitet, und beide gelten im angepassten Bereich als temperaturunabhängig. Gültig in diesem Bereich; weite Extrapolation oder ein Mechanismuswechsel machen sie ungültig.",
        source: "Arrhenius equation (S. Arrhenius, 1889)",
        source_de: "Arrhenius-Gleichung (S. Arrhenius, 1889)",
    },
    RelationInfo {
        name: "eyring",
        equation: "k = (kB·T/h)·exp(−ΔG‡/RT)",
        args: "dG=<J/mol> T=<K>",
        purpose: "The same temperature dependence, derived from the transition state rather than fitted.",
        purpose_de: "Dieselbe Temperaturabhängigkeit, hergeleitet über den Übergangszustand statt angepasst.",
        validity: "Assumes thermal equilibrium between reactants and the activated complex, and a transmission coefficient of one. Where that fails, Arrhenius fitted to data is the more honest answer.",
        validity_de: "Setzt ein thermisches Gleichgewicht zwischen Edukten und aktiviertem Komplex sowie einen Transmissionskoeffizienten von eins voraus. Wo das nicht gilt, ist der an Daten angepasste Arrhenius-Ansatz die ehrlichere Antwort.",
        source: "Eyring equation (H. Eyring, 1935); transition state theory",
        source_de: "Eyring-Gleichung (H. Eyring, 1935); Theorie des Übergangszustands",
    },
    RelationInfo {
        name: "henderson-hasselbalch",
        equation: "pH = pKa + log₁₀([A⁻]/[HA])",
        args: "pKa=<value> cA=<mol/L> cB=<mol/L>",
        purpose: "The pH of a buffer from the ratio of its two forms.",
        purpose_de: "Der pH-Wert eines Puffers aus dem Verhältnis seiner beiden Formen.",
        validity: "An approximation: it assumes the acid and base concentrations are the ones you weighed in, so it drifts near the ends of the buffer range, in dilute solution, and wherever water's own autoprotolysis matters. The bench solves the full charge balance instead.",
        validity_de: "Eine Näherung: sie nimmt an, dass die Konzentrationen von Säure und Base den eingewogenen entsprechen. Deshalb weicht sie an den Rändern des Pufferbereichs, in verdünnter Lösung und dort ab, wo die Autoprotolyse des Wassers zählt. Die Bank löst stattdessen die vollständige Ladungsbilanz.",
        source: "Henderson–Hasselbalch equation (L.J. Henderson, 1908; K.A. Hasselbalch, 1917)",
        source_de: "Henderson–Hasselbalch-Gleichung (L.J. Henderson, 1908; K.A. Hasselbalch, 1917)",
    },
    RelationInfo {
        name: "ionic-strength",
        equation: "I = ½ Σ mᵢ zᵢ²",
        args: "<z>:<m> <z>:<m> ... (charge:molality pairs)",
        purpose: "How crowded with charge a solution is — the quantity every activity correction starts from.",
        purpose_de: "Wie stark eine Lösung mit Ladung besetzt ist — die Größe, bei der jede Aktivitätskorrektur beginnt.",
        validity: "A definition, so it is always true; the caution belongs to what you do with it. Sum over every ion actually present, not only the ones you added.",
        validity_de: "Eine Definition und damit immer gültig; die Einschränkung liegt darin, was man damit tut. Über alle tatsächlich vorhandenen Ionen summieren, nicht nur über die zugegebenen.",
        source: "Lewis and Randall (1921)",
        source_de: "Lewis und Randall (1921)",
    },
    RelationInfo {
        name: "debye-huckel",
        equation: "log₁₀(γ) = −A z² √I",
        args: "z=<charge> I=<mol/kg>",
        purpose: "How far an ion's activity falls below its concentration, because of the other ions around it.",
        purpose_de: "Wie weit die Aktivität eines Ions unter seiner Konzentration liegt, wegen der übrigen Ionen ringsum.",
        validity: "The limiting law holds only in dilute solution, roughly below I = 0.01 mol/kg. Above that it overcorrects, and above about 0.5 mol/kg a specific-ion-interaction model such as Pitzer is required — which is why the bench routes concentrated solutions to a different database. A = 0.5091 is the value for water at 25 °C; it depends on the solvent's permittivity and density, so it moves with both.",
        validity_de: "Das Grenzgesetz gilt nur in verdünnter Lösung, etwa unterhalb I = 0,01 mol/kg. Darüber überkorrigiert es, und oberhalb von etwa 0,5 mol/kg braucht es ein ionenspezifisches Wechselwirkungsmodell wie Pitzer — deshalb leitet die Bank konzentrierte Lösungen an eine andere Datenbank weiter. A = 0,5091 gilt für Wasser bei 25 °C; der Wert hängt von Permittivität und Dichte des Lösungsmittels ab und ändert sich mit beiden.",
        source: "Debye–Hückel limiting law (P. Debye and E. Hückel, 1923)",
        source_de: "Debye–Hückel-Grenzgesetz (P. Debye und E. Hückel, 1923)",
    },
    RelationInfo {
        name: "van-t-hoff",
        equation: "K₂ = K₁·exp[−(ΔH°/R)(1/T₂ − 1/T₁)]",
        args: "dH=<J/mol> K1=<value> T1=<K> T2=<K>",
        purpose: "Where an equilibrium moves when the temperature changes.",
        purpose_de: "Wohin sich ein Gleichgewicht verschiebt, wenn sich die Temperatur ändert.",
        validity: "Assumes the reaction enthalpy is constant over the interval. Across a wide temperature range, or a phase change, it is not, and the prediction drifts.",
        validity_de: "Nimmt an, dass die Reaktionsenthalpie über das Intervall konstant ist. Über einen weiten Temperaturbereich oder einen Phasenwechsel hinweg gilt das nicht, und die Vorhersage weicht ab.",
        source: "Van 't Hoff equation (J.H. van 't Hoff, 1884)",
        source_de: "Van-'t-Hoff-Gleichung (J.H. van 't Hoff, 1884)",
    },
];

/// Parse a `key=value` argument, returning (key, value).
pub fn parse_arg(arg: &str) -> Option<(&str, f64)> {
    let (key, val) = arg.split_once('=')?;
    val.parse::<f64>().ok().map(|v| (key, v))
}

/// Evaluate a named relation from string arguments.
pub fn evaluate(name: &str, args: &[String]) -> Result<RelationResult, String> {
    let get = |key: &str| -> Result<f64, String> {
        args.iter()
            .filter_map(|a| parse_arg(a))
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v)
            .ok_or_else(|| format!("missing argument: {key}"))
    };
    let get_or = |key: &str, default: f64| -> f64 {
        args.iter()
            .filter_map(|a| parse_arg(a))
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v)
            .unwrap_or(default)
    };

    match name {
        "nernst" => {
            let e0 = get("e0")?;
            let n = get("n")?;
            let a = get("a")?;
            let t = get("T")?;
            Ok(nernst(e0, n, a, Kelvin(t)))
        }
        "arrhenius" => {
            let a = get("A")?;
            let ea = get("Ea")?;
            let t = get("T")?;
            let b = get_or("b", 0.0);
            Ok(arrhenius_result(a, b, ea, t))
        }
        "eyring" => {
            let dg = get("dG")?;
            let t = get("T")?;
            Ok(eyring_result(dg, t))
        }
        "henderson-hasselbalch" => {
            let pka = get("pKa")?;
            let ca = get("cA")?;
            let cb = get("cB")?;
            if ca <= 0.0 || cb <= 0.0 {
                return Err("concentrations must be positive".into());
            }
            Ok(henderson_hasselbalch_result(pka, ca, cb))
        }
        "ionic-strength" => {
            let species: Vec<(f64, f64)> = args
                .iter()
                .filter_map(|a| {
                    let (z, m) = a.split_once(':')?;
                    Some((z.parse::<f64>().ok()?, m.parse::<f64>().ok()?))
                })
                .collect();
            if species.is_empty() {
                return Err("provide charge:molality pairs (e.g. 1:0.1 -1:0.1)".into());
            }
            Ok(ionic_strength_result(&species))
        }
        "debye-huckel" => {
            let z = get("z")?;
            let i = get("I")?;
            Ok(debye_huckel_result(z, i))
        }
        "van-t-hoff" => {
            let dh = get("dH")?;
            let k1 = get("K1")?;
            let t1 = get("T1")?;
            let t2 = get("T2")?;
            Ok(van_t_hoff_result(dh, k1, t1, t2))
        }
        _ => Err(format!("unknown relation '{name}'")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nernst_slope_at_25c() {
        let slope = nernst_slope(Kelvin::STANDARD);
        assert!((slope - 0.05916).abs() < 0.0001, "slope at 25 °C: {slope}");
    }

    #[test]
    fn nernst_standard_hydrogen_electrode() {
        let result = nernst(0.0, 2.0, 1.0, Kelvin::STANDARD);
        assert!(
            result.value.abs() < 1e-12,
            "SHE at unit activity: {}",
            result.value
        );
    }

    #[test]
    fn nernst_copper_at_dilute() {
        let result = nernst(0.3419, 2.0, 0.01, Kelvin::STANDARD);
        let expected = 0.3419 + 0.05916 / 2.0 * (-2.0);
        assert!(
            (result.value - expected).abs() < 1e-4,
            "Cu²⁺/Cu at 0.01 M: {} vs expected {expected}",
            result.value
        );
    }

    #[test]
    fn arrhenius_matches_hand_calculation() {
        let k = arrhenius(1.0e10, 0.0, 50_000.0, 298.15);
        let expected = 1.0e10 * (-50_000.0 / (constants::GAS_CONSTANT * 298.15)).exp();
        assert!((k - expected).abs() / expected < 1e-12);
    }

    #[test]
    fn arrhenius_ten_degree_rule() {
        let k1 = arrhenius(1.0, 0.0, 50_000.0, 298.15);
        let k2 = arrhenius(1.0, 0.0, 50_000.0, 308.15);
        let ratio = k2 / k1;
        assert!((1.9..2.3).contains(&ratio), "ten-degree ratio: {ratio:.3}");
    }

    #[test]
    fn eyring_at_room_temperature() {
        let k = eyring(65_000.0, 298.15);
        let prefactor = constants::BOLTZMANN * 298.15 / constants::PLANCK;
        let expected = prefactor * (-65_000.0 / (constants::GAS_CONSTANT * 298.15)).exp();
        assert!(
            (k - expected).abs() / expected < 1e-12,
            "Eyring: {k} vs {expected}"
        );
        assert!(k > 0.0);
    }

    #[test]
    fn henderson_hasselbalch_at_pka() {
        let f = henderson_hasselbalch_fraction(4.76, 4.76);
        assert!(
            (f - 0.5).abs() < 1e-12,
            "at pH = pKa, fraction should be 0.5: {f}"
        );
    }

    #[test]
    fn henderson_hasselbalch_ph_from_ratio() {
        let ph = henderson_hasselbalch_ph(4.76, 10.0);
        assert!((ph - 5.76).abs() < 1e-12, "pKa + log₁₀(10) = 5.76: {ph}");
    }

    #[test]
    fn henderson_hasselbalch_equal_concentrations() {
        let result = henderson_hasselbalch_result(4.76, 0.1, 0.1);
        assert!(
            (result.value - 4.76).abs() < 1e-12,
            "equal [HA] and [A⁻] gives pH = pKa: {}",
            result.value
        );
    }

    #[test]
    fn ionic_strength_nacl_0_1m() {
        let i = ionic_strength(&[(1.0, 0.1), (-1.0, 0.1)]);
        assert!((i - 0.1).abs() < 1e-12, "0.1 M NaCl: I = {i}");
    }

    #[test]
    fn ionic_strength_cacl2_0_1m() {
        let i = ionic_strength(&[(2.0, 0.1), (-1.0, 0.2)]);
        assert!((i - 0.3).abs() < 1e-12, "0.1 M CaCl₂: I = {i}");
    }

    #[test]
    fn debye_huckel_monovalent_dilute() {
        let log_gamma = debye_huckel_log_gamma(1.0, 0.001);
        let gamma = 10f64.powf(log_gamma);
        assert!(
            (gamma - 0.965).abs() < 0.01,
            "γ(z=1, I=0.001) = {gamma}, expected ~0.965"
        );
    }

    #[test]
    fn debye_huckel_divalent() {
        let log_gamma = debye_huckel_log_gamma(2.0, 0.01);
        let gamma = 10f64.powf(log_gamma);
        assert!(
            gamma < 0.7,
            "divalent at I=0.01 should be well below 1: {gamma}"
        );
    }

    #[test]
    fn van_t_hoff_endothermic_increases_k() {
        let k2 = van_t_hoff(50_000.0, 1.0, 298.15, 373.15);
        assert!(
            k2 > 1.0,
            "endothermic: K should increase with temperature: {k2}"
        );
    }

    #[test]
    fn van_t_hoff_exothermic_decreases_k() {
        let k2 = van_t_hoff(-50_000.0, 1e14, 298.15, 373.15);
        assert!(
            k2 < 1e14,
            "exothermic: K should decrease with temperature: {k2}"
        );
    }

    #[test]
    fn van_t_hoff_same_temperature_returns_k1() {
        let k2 = van_t_hoff(50_000.0, 42.0, 298.15, 298.15);
        assert!((k2 - 42.0).abs() < 1e-10, "same T: K₂ = K₁: {k2}");
    }

    #[test]
    fn evaluate_dispatches_nernst() {
        let args: Vec<String> = ["e0=0.0", "n=2", "a=1.0", "T=298.15"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let r = evaluate("nernst", &args).unwrap();
        assert!(r.value.abs() < 1e-12);
    }

    #[test]
    fn evaluate_rejects_unknown() {
        let r = evaluate("made-up", &[]);
        assert!(r.is_err());
    }

    /// One worked example per catalogue entry, so the two tests below can
    /// evaluate every relation the toolbox offers rather than one of them.
    /// A relation added without a line here fails `every_relation_has_a_worked_example`.
    const SAMPLES: &[(&str, &[&str])] = &[
        ("nernst", &["e0=0.3419", "n=2", "a=0.01", "T=298.15"]),
        ("arrhenius", &["A=1e10", "Ea=50000", "T=298.15"]),
        ("eyring", &["dG=65000", "T=298.15"]),
        ("henderson-hasselbalch", &["pKa=4.76", "cA=0.1", "cB=0.1"]),
        ("ionic-strength", &["1:0.1", "-1:0.1"]),
        ("debye-huckel", &["z=2", "I=0.005"]),
        (
            "van-t-hoff",
            &["dH=-57000", "K1=1e14", "T1=298.15", "T2=373.15"],
        ),
    ];

    #[test]
    fn every_relation_has_a_worked_example() {
        assert_eq!(
            SAMPLES.len(),
            RELATIONS.len(),
            "every catalogue entry needs a sample argument list"
        );
        for r in RELATIONS {
            assert!(
                SAMPLES.iter().any(|(name, _)| *name == r.name),
                "no worked example for '{}'",
                r.name
            );
        }
    }

    /// GUI-096: the toolbox shows what a relation is for, where it holds
    /// and where it came from, BEFORE anything is computed — so every one
    /// of those sentences has to exist in every language the engine ships,
    /// not only in the one the fields are named after.
    #[test]
    fn every_relation_says_what_it_is_for_in_both_languages() {
        for r in RELATIONS {
            for (label, en, de) in [
                ("purpose", r.purpose, r.purpose_de),
                ("validity", r.validity, r.validity_de),
                ("source", r.source, r.source_de),
            ] {
                assert!(
                    !en.trim().is_empty(),
                    "{}: {label} is empty in English",
                    r.name
                );
                assert!(
                    !de.trim().is_empty(),
                    "{}: {label} is empty in German",
                    r.name
                );
                // An English sentence copied into a `_de` field renders as
                // a German page with an English paragraph in it, which no
                // emptiness check would catch.
                assert_ne!(en, de, "{}: {label}_de is still the English", r.name);
            }
        }
    }

    /// The catalogue must not cite one paper while the computed result
    /// cites another. Each `source` is the leading clause of the relation's
    /// own provenance line, so this is the check that keeps them one string
    /// rather than two that drift.
    #[test]
    fn sources_are_the_provenance_they_came_from() {
        for (name, args) in SAMPLES {
            let info = RELATIONS
                .iter()
                .find(|r| r.name == *name)
                .unwrap_or_else(|| panic!("no catalogue entry for '{name}'"));
            let owned: Vec<String> = args.iter().map(|a| (*a).to_string()).collect();
            let result = evaluate(name, &owned)
                .unwrap_or_else(|e| panic!("'{name}' failed to evaluate: {e}"));
            assert!(
                result.provenance.contains(info.source),
                "'{name}': catalogue cites {:?}, the result cites {:?}",
                info.source,
                result.provenance
            );
        }
    }
}
