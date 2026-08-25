//! The chart contract (CAP-3): one JSON shape for every computed curve.
//!
//! A chart is a claim, so the contract carries what a claim needs —
//! axes with units, the series, and the provenance line a reader is
//! owed. Producers (diagrams, the study runner, titration curves)
//! emit this; renderers (the CLI's SVG, the web) consume it; neither
//! side grows a private dialect. The predominance grid the Pourbaix
//! command emits is this contract's sibling and will fold in as a
//! `Regions` kind when its second producer appears.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chart {
    pub title: String,
    pub x: Axis,
    pub y: Axis,
    pub series: Vec<Series>,
    /// Where these numbers come from — engine, dataset, model. A chart
    /// without provenance is a picture, not a result.
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Axis {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Series {
    /// A connected curve.
    Line { name: String, points: Vec<[f64; 2]> },
    /// Unconnected points.
    Scatter { name: String, points: Vec<[f64; 2]> },
    /// A shaded uncertainty band between two envelopes sharing the
    /// same x values: `lower` and `upper` are each (x, y) polylines.
    /// CAP-8's artefact — the computed error bar made visible.
    Band {
        name: String,
        lower: Vec<[f64; 2]>,
        upper: Vec<[f64; 2]>,
    },
}

impl Series {
    pub fn name(&self) -> &str {
        match self {
            Series::Line { name, .. }
            | Series::Scatter { name, .. }
            | Series::Band { name, .. } => name,
        }
    }
    /// The points that bound the series for axis scaling: both
    /// envelopes for a band.
    pub fn points(&self) -> Vec<[f64; 2]> {
        match self {
            Series::Line { points, .. } | Series::Scatter { points, .. } => points.clone(),
            Series::Band { lower, upper, .. } => {
                lower.iter().chain(upper.iter()).copied().collect()
            }
        }
    }
}

/// The charts a step's events earn — the producer side of the contract,
/// shared by every host so the CLI, the wasm bench, and the shell emit
/// the identical claim for the identical chemistry.
///
/// Today one producer: a `Titrated` event's recorded curve becomes the
/// live titration chart (GUI-021/CAP-12). Each point is one solver
/// equilibration after one burette increment; nothing is interpolated.
pub fn charts_for_events(events: &[crate::Event]) -> Vec<Chart> {
    let mut charts = Vec::new();
    for event in events {
        if let crate::Event::Titrated {
            vessel,
            titrant,
            concentration,
            curve,
            ..
        } = event
        {
            // One reading is a number, not a curve; the chart starts at two.
            if curve.len() < 2 {
                continue;
            }
            charts.push(Chart {
                title: format!(
                    "titration of v{} with {} ({} M)",
                    vessel.0 + 1,
                    titrant,
                    concentration
                ),
                x: Axis {
                    label: "titrant added".into(),
                    unit: Some("mL".into()),
                },
                y: Axis {
                    label: "pH".into(),
                    unit: None,
                },
                series: vec![Series::Line {
                    name: "pH".into(),
                    points: curve.iter().map(|&(ml, ph)| [ml, ph]).collect(),
                }],
                provenance: "each point: one aqueous-solver equilibration after one \
                             burette increment (titrate); nothing interpolated"
                    .into(),
            });
        }
    }
    charts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_titrated_event_becomes_the_titration_chart() {
        let event = crate::Event::Titrated {
            vessel: crate::VesselId(0),
            titrant: crate::SpeciesId::new("NaOH"),
            concentration: 0.1,
            steps: 2,
            total_volume: crate::units::Liters(0.002),
            final_ph: 7.2,
            curve: vec![(0.0, 2.9), (1.0, 3.4), (2.0, 7.2)],
        };
        let charts = charts_for_events(&[event]);
        assert_eq!(charts.len(), 1);
        let c = &charts[0];
        assert_eq!(c.x.unit.as_deref(), Some("mL"));
        assert_eq!(
            c.series[0].points(),
            vec![[0.0, 2.9], [1.0, 3.4], [2.0, 7.2]]
        );
        assert!(!c.provenance.is_empty());

        // A single reading is not a curve; no chart is claimed.
        let short = crate::Event::Titrated {
            vessel: crate::VesselId(0),
            titrant: crate::SpeciesId::new("NaOH"),
            concentration: 0.1,
            steps: 0,
            total_volume: crate::units::Liters(0.0),
            final_ph: 2.9,
            curve: vec![(0.0, 2.9)],
        };
        assert!(charts_for_events(&[short]).is_empty());
    }

    #[test]
    fn contract_round_trips() {
        let c = Chart {
            title: "t".into(),
            x: Axis {
                label: "x".into(),
                unit: None,
            },
            y: Axis {
                label: "T".into(),
                unit: Some("°C".into()),
            },
            series: vec![Series::Line {
                name: "bubble".into(),
                points: vec![[0.0, 100.0], [1.0, 78.4]],
            }],
            provenance: "test".into(),
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: Chart = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
        assert!(json.contains("\"kind\":\"line\""), "{json}");
    }
}
