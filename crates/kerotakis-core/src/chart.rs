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

#[cfg(test)]
mod tests {
    use super::*;

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
