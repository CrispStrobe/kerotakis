//! Domain packs: environmental, materials, and polymer (ADV-001/003/004).
//!
//! Each pack is a configuration that selects approved data, scenarios, and
//! models for a specific chemistry domain. Packs are additive — enabling
//! a pack does not disable the core.

use serde::{Deserialize, Serialize};

/// A model pack manifest — what data and models a pack provides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    /// Which data sources this pack draws from.
    pub data_sources: Vec<DataSource>,
    /// Which scenarios are included.
    pub scenarios: Vec<Scenario>,
    /// Content hash of the compiled pack.
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub name: String,
    pub licence: String,
    pub provenance: String,
    pub approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub description: String,
    /// .lab script content for this scenario.
    pub script: String,
}

// ── ADV-001: Environmental pack ───────────────────────────────────

/// Environmental chemistry pack: soils, treatment, weathering, ocean.
pub fn environmental_pack() -> PackManifest {
    PackManifest {
        name: "environmental".into(),
        version: "0.1.0".into(),
        description: "Soil chemistry, water treatment, weathering, and ocean acidification \
                       scenarios using approved PHREEQC databases"
            .into(),
        data_sources: vec![
            DataSource {
                name: "PHREEQC phreeqc.dat".into(),
                licence: "public domain (USGS)".into(),
                provenance: "United States Geological Survey, PHREEQC v3".into(),
                approved: true,
            },
            DataSource {
                name: "PHREEQC wateq4f.dat".into(),
                licence: "public domain (USGS)".into(),
                provenance: "WATEQ4F thermodynamic database".into(),
                approved: true,
            },
        ],
        scenarios: vec![
            Scenario {
                name: "ocean-acidification".into(),
                description: "Dissolve CO2 into seawater, observe pH drop and carbonate equilibria"
                    .into(),
                script: "new\nadd v1 water 55.5\nadd v1 NaCl 0.6\nheat v1 4184\nwait 1".into(),
            },
            Scenario {
                name: "soil-weathering".into(),
                description: "Feldspar dissolution in acidic rain".into(),
                script: "new\nadd v1 water 55.5\nwait 10".into(),
            },
        ],
        content_hash: None,
    }
}

// ── ADV-003: Materials/metallurgy pilot ───────────────────────────

/// Materials chemistry pack: iron/copper processes.
pub fn materials_pack() -> PackManifest {
    PackManifest {
        name: "materials".into(),
        version: "0.1.0".into(),
        description: "Iron and copper metallurgy using cleared CEA thermodynamic records".into(),
        data_sources: vec![DataSource {
            name: "NASA CEA (cleared subset)".into(),
            licence: "public domain (NASA)".into(),
            provenance: "NASA Glenn coefficients, subset cleared for runtime use".into(),
            approved: true,
        }],
        scenarios: vec![
            Scenario {
                name: "copper-smelting".into(),
                description: "Reduction of CuO with carbon at high temperature".into(),
                script: "new\nadd v1 CuO 0.1\nadd v1 C 0.1\nheat v1 50000\nwait 60".into(),
            },
            Scenario {
                name: "iron-rusting".into(),
                description: "Aqueous corrosion of iron in aerated brine".into(),
                script: "new\nadd v1 water 55.5\nadd v1 NaCl 0.1\nadd v1 Fe 0.01\nwait 3600"
                    .into(),
            },
        ],
        content_hash: None,
    }
}

// ── ADV-004: Polymer kinetics pilot ──────────────────────────────

/// Polymer kinetics pack: population moments and heat ledger.
pub fn polymer_pack() -> PackManifest {
    PackManifest {
        name: "polymer".into(),
        version: "0.1.0".into(),
        description: "Polymer kinetics with population moments tracking (Mn, Mw, PDI) \
                       and reaction heat accounting"
            .into(),
        data_sources: vec![DataSource {
            name: "project-authored network".into(),
            licence: "AGPL-3.0 (project-authored)".into(),
            provenance: "kerotakis project, hand-authored styrene polymerization kinetics".into(),
            approved: true,
        }],
        scenarios: vec![Scenario {
            name: "styrene-bulk".into(),
            description: "Bulk free-radical polymerization of styrene".into(),
            script: "new\nadd v1 styrene 1.0\nadd v1 BPO 0.01\nheat v1 20000\nwait 3600".into(),
        }],
        content_hash: None,
    }
}

/// Polymer population state for moment tracking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PolymerMoments {
    /// Zeroth moment: total number of polymer chains.
    pub mu_0: f64,
    /// First moment: total monomer units incorporated.
    pub mu_1: f64,
    /// Second moment: sum of (chain length)².
    pub mu_2: f64,
}

impl PolymerMoments {
    /// Number-average molecular weight (Mn = mu_1 / mu_0 * monomer_mw).
    pub fn mn(&self, monomer_mw: f64) -> f64 {
        if self.mu_0 > 0.0 {
            self.mu_1 / self.mu_0 * monomer_mw
        } else {
            0.0
        }
    }

    /// Weight-average molecular weight (Mw = mu_2 / mu_1 * monomer_mw).
    pub fn mw(&self, monomer_mw: f64) -> f64 {
        if self.mu_1 > 0.0 {
            self.mu_2 / self.mu_1 * monomer_mw
        } else {
            0.0
        }
    }

    /// Polydispersity index (PDI = Mw / Mn = mu_2 * mu_0 / mu_1²).
    pub fn pdi(&self) -> f64 {
        if self.mu_1 > 0.0 {
            self.mu_2 * self.mu_0 / (self.mu_1 * self.mu_1)
        } else {
            1.0
        }
    }

    /// Fractional conversion (assumes mu_1 tracks consumed monomer).
    pub fn conversion(&self, initial_monomer_moles: f64) -> f64 {
        if initial_monomer_moles > 0.0 {
            (self.mu_1 / initial_monomer_moles).min(1.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environmental_pack_has_scenarios() {
        let pack = environmental_pack();
        assert!(!pack.scenarios.is_empty());
        assert!(pack.data_sources.iter().all(|d| d.approved));
    }

    #[test]
    fn materials_pack_has_scenarios() {
        let pack = materials_pack();
        assert!(!pack.scenarios.is_empty());
    }

    #[test]
    fn polymer_pack_has_scenarios() {
        let pack = polymer_pack();
        assert!(!pack.scenarios.is_empty());
    }

    #[test]
    fn polymer_moments_mn_mw_pdi() {
        let m = PolymerMoments {
            mu_0: 100.0,  // 100 chains
            mu_1: 5000.0, // 5000 monomer units total
            mu_2: 300000.0,
        };
        let mw_monomer = 104.15; // styrene
        let mn = m.mn(mw_monomer);
        let mw = m.mw(mw_monomer);
        let pdi = m.pdi();
        assert!(mn > 0.0);
        assert!(mw > mn, "Mw should be ≥ Mn");
        assert!(pdi >= 1.0, "PDI should be ≥ 1");
    }

    #[test]
    fn polymer_moments_pdi_unity_for_monodisperse() {
        let m = PolymerMoments {
            mu_0: 100.0,
            mu_1: 5000.0,
            mu_2: 250000.0, // all chains same length: mu_2 = mu_1²/mu_0
        };
        assert!((m.pdi() - 1.0).abs() < 1e-10);
    }
}
