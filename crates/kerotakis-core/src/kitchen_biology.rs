//! Bounded household-biology observables.
//!
//! These models deliberately stop short of molecular food simulation. They
//! calculate the three quantities the demonstrations ask for while making the
//! unrepresented state explicit: membrane water transfer, cut-surface colour,
//! and aggregate insoluble soap. Callers must conserve their own ledgers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsmosisDirection {
    IntoObject,
    OutOfObject,
    Balanced,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OsmosisPrediction {
    pub direction: OsmosisDirection,
    /// Fraction of the object's initial water exchanged during this interval.
    pub water_fraction: f64,
}

/// A semipermeable-membrane teaching model driven by total dissolved-particle
/// concentration. It does not claim membrane area, egg size, ion selectivity,
/// shell dissolution, tissue elasticity, or an equilibrium final mass.
pub fn egg_osmosis(
    internal_osmolarity: f64,
    external_osmolarity: f64,
    elapsed_seconds: f64,
) -> Option<OsmosisPrediction> {
    if !internal_osmolarity.is_finite()
        || !external_osmolarity.is_finite()
        || !elapsed_seconds.is_finite()
        || internal_osmolarity < 0.0
        || external_osmolarity < 0.0
        || elapsed_seconds < 0.0
    {
        return None;
    }
    let difference = internal_osmolarity - external_osmolarity;
    let direction = if difference.abs() < 1e-6 {
        OsmosisDirection::Balanced
    } else if difference > 0.0 {
        OsmosisDirection::IntoObject
    } else {
        OsmosisDirection::OutOfObject
    };
    // One deliberately stated classroom timescale: a 1 osmol/L contrast
    // approaches a 20% water exchange over 24 h. The exponential is bounded
    // and composes over successive waits.
    let drive = difference.abs().min(2.0);
    let water_fraction = 0.20 * (1.0 - (-elapsed_seconds / 86_400.0).exp()) * drive;
    Some(OsmosisPrediction {
        direction,
        water_fraction: water_fraction.clamp(0.0, 0.40),
    })
}

/// Fraction of a freshly cut apple surface visibly browned. Oxygen and time
/// are required; ascorbate inhibits the visible response without being called
/// a universal preservative or a complete polyphenol-oxidase mechanism.
pub fn apple_browning(
    elapsed_seconds: f64,
    oxygen_fraction: f64,
    ascorbate_moles_per_gram_apple: f64,
) -> Option<f64> {
    if !elapsed_seconds.is_finite()
        || !oxygen_fraction.is_finite()
        || !ascorbate_moles_per_gram_apple.is_finite()
        || elapsed_seconds < 0.0
        || !(0.0..=1.0).contains(&oxygen_fraction)
        || ascorbate_moles_per_gram_apple < 0.0
    {
        return None;
    }
    let oxygen_gate = (oxygen_fraction / 0.21).clamp(0.0, 1.0);
    let inhibition = 1.0 / (1.0 + ascorbate_moles_per_gram_apple / 2.0e-5);
    Some((1.0 - (-elapsed_seconds / 900.0).exp()) * oxygen_gate * inhibition)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoapScumPrediction {
    pub soap_bound_moles: f64,
    pub divalent_ion_bound_moles: f64,
    pub aggregate_mass_g: f64,
}

/// Two fatty-carboxylate anions bind each Ca²⁺ or Mg²⁺. `soap_moles` is an
/// explicit fatty-soap equivalent supplied by a reviewed recipe; this function
/// must not infer it from generic detergent or surfactant labels.
pub fn soap_scum(divalent_moles: f64, soap_moles: f64) -> Option<SoapScumPrediction> {
    if !divalent_moles.is_finite()
        || !soap_moles.is_finite()
        || divalent_moles < 0.0
        || soap_moles < 0.0
    {
        return None;
    }
    let bound_ion = divalent_moles.min(soap_moles / 2.0);
    let bound_soap = 2.0 * bound_ion;
    // Calcium stearate is the bounded visual surrogate (607.0 g/mol). A
    // magnesium-rich water differs slightly; callers must label this aggregate.
    Some(SoapScumPrediction {
        soap_bound_moles: bound_soap,
        divalent_ion_bound_moles: bound_ion,
        aggregate_mass_g: bound_ion * 607.0,
    })
}
