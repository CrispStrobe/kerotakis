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
/// The charts a run earns, with their furniture in the reader's language.
///
/// The axis labels and the title were English literals, so a French
/// titration drew a French curve under `titrant added (mL)` and a title
/// that said `titration of v1 with NaOH`. The numbers were never the
/// problem; the words around them were the last English on the screen.
///
/// `pH` and `pe` stay as they are: they are notation, like `2H⁺/H₂`, and
/// translating notation is how a chart stops being readable.
pub fn charts_for_events(events: &[crate::Event], locale: crate::i18n::Locale) -> Vec<Chart> {
    let titrant_added = locale
        .lookup("chart.titrant-added")
        .unwrap_or("titrant added");
    let mut charts = Vec::new();
    for event in events {
        if let crate::Event::Titrated {
            vessel,
            titrant,
            concentration,
            curve,
            pe_curve,
            ..
        } = event
        {
            // One reading is a number, not a curve; the chart starts at two.
            if curve.len() >= 2 {
                charts.push(Chart {
                    title: locale.fill(
                        "chart.titration-title",
                        "titration of {vessel} with {titrant} ({molarity} M)",
                        &[
                            ("vessel", &format!("v{}", vessel.0 + 1)),
                            ("titrant", titrant.0.as_str()),
                            ("molarity", &locale.number(concentration.to_string())),
                        ],
                    ),
                    x: Axis {
                        label: titrant_added.into(),
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
                    provenance: locale
                        .lookup("chart.one-point-one-equilibration")
                        .unwrap_or(
                            "each point: one aqueous-solver equilibration after one \
                             burette increment (titrate); nothing interpolated",
                        )
                        .into(),
                });
            }
            // EXP-39: the redox curve is its own chart rather than a
            // second series, because pH and pe do not share a y axis and
            // the contract carries one axis per chart. It is a Scatter,
            // not a Line: the curve is sparse wherever the engine
            // withheld a potential, and joining across that gap would
            // draw a line through the very point that has no value.
            if pe_curve.len() >= 2 {
                charts.push(Chart {
                    title: locale.fill(
                        "chart.redox-titration-title",
                        "redox titration of {vessel} with {titrant} ({molarity} M)",
                        &[
                            ("vessel", &format!("v{}", vessel.0 + 1)),
                            ("titrant", titrant.0.as_str()),
                            ("molarity", &locale.number(concentration.to_string())),
                        ],
                    ),
                    x: Axis {
                        label: titrant_added.into(),
                        unit: Some("mL".into()),
                    },
                    y: Axis {
                        label: "pe".into(),
                        unit: None,
                    },
                    series: vec![Series::Scatter {
                        name: "pe".into(),
                        points: pe_curve.iter().map(|&(ml, pe)| [ml, pe]).collect(),
                    }],
                    provenance: "each point: the pe the aqueous engine pinned after one \
                                 burette increment (titrate); steps where the electron \
                                 balance had no root carry no point at all"
                        .into(),
                });
            }
        }
    }
    charts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_titrated_event_becomes_the_titration_chart() {
        fn event2() -> crate::Event {
            crate::Event::Titrated {
                vessel: crate::VesselId(0),
                titrant: crate::SpeciesId::new("NaOH"),
                concentration: 0.1,
                steps: 2,
                total_volume: crate::units::Liters(0.002),
                final_ph: 7.2,
                curve: vec![(0.0, 2.9), (1.0, 3.4), (2.0, 7.2)],
                pe_curve: Vec::new(),
                endpoint_reached: Some(true),
                endpoint: crate::ops::Endpoint::Ph,
            }
        }
        let event = crate::Event::Titrated {
            vessel: crate::VesselId(0),
            titrant: crate::SpeciesId::new("NaOH"),
            concentration: 0.1,
            steps: 2,
            total_volume: crate::units::Liters(0.002),
            final_ph: 7.2,
            curve: vec![(0.0, 2.9), (1.0, 3.4), (2.0, 7.2)],
            pe_curve: Vec::new(),
            endpoint_reached: Some(true),
            endpoint: crate::ops::Endpoint::Ph,
        };
        let charts = charts_for_events(&[event], crate::i18n::Locale::EN);
        assert_eq!(charts.len(), 1);
        let c = &charts[0];
        assert_eq!(c.x.unit.as_deref(), Some("mL"));
        assert_eq!(
            c.series[0].points(),
            vec![[0.0, 2.9], [1.0, 3.4], [2.0, 7.2]]
        );
        assert!(!c.provenance.is_empty());

        // The words around the curve follow the reader, in every
        // shipped language. `pH` and `pe` do not: they are notation, and
        // a chart whose axis says `pH` in five languages is the one that
        // stays readable. Driven off the catalogue rather than naming a
        // language, so a fourth is covered by existing.
        for locale in crate::i18n::Locale::available() {
            if locale.is_english() {
                continue;
            }
            let mine = charts_for_events(&[event2()], locale);
            let mine = &mine[0];
            assert_ne!(
                mine.title,
                c.title,
                "{}: the chart title is still the English one",
                locale.code()
            );
            assert_ne!(
                mine.x.label,
                c.x.label,
                "{}: the x axis is still labelled in English",
                locale.code()
            );
            assert_eq!(
                mine.y.label,
                c.y.label,
                "{}: pH is notation and must not be translated",
                locale.code()
            );
            assert_eq!(mine.x.unit, c.x.unit, "{}: mL is a unit", locale.code());
        }

        // A single reading is not a curve; no chart is claimed.
        let short = crate::Event::Titrated {
            vessel: crate::VesselId(0),
            titrant: crate::SpeciesId::new("NaOH"),
            concentration: 0.1,
            steps: 0,
            total_volume: crate::units::Liters(0.0),
            final_ph: 2.9,
            curve: vec![(0.0, 2.9)],
            pe_curve: Vec::new(),
            endpoint_reached: Some(false),
            endpoint: crate::ops::Endpoint::Ph,
        };
        assert!(charts_for_events(&[short], crate::i18n::Locale::EN).is_empty());
    }

    /// EXP-39: a redox titration earns a second chart, and the gap where
    /// the engine withheld a potential stays a gap.
    #[test]
    fn a_pe_curve_becomes_its_own_chart() {
        let event = crate::Event::Titrated {
            vessel: crate::VesselId(0),
            titrant: crate::SpeciesId::new("KMnO4"),
            concentration: 0.02,
            steps: 3,
            total_volume: crate::units::Liters(0.003),
            final_ph: 1.1,
            curve: vec![(0.0, 1.0), (1.0, 1.05), (2.0, 1.08), (3.0, 1.1)],
            // No point at 2.0 mL: that is the equivalence point.
            pe_curve: vec![(0.0, 4.1), (1.0, 5.2), (3.0, 14.9)],
            endpoint_reached: Some(true),
            endpoint: crate::ops::Endpoint::ColourPersists,
        };
        let charts = charts_for_events(&[event], crate::i18n::Locale::EN);
        assert_eq!(charts.len(), 2, "a pH chart and a pe chart");
        let redox = &charts[1];
        assert_eq!(redox.y.label, "pe");
        assert_eq!(redox.series[0].points().len(), 3);
        assert!(
            matches!(redox.series[0], Series::Scatter { .. }),
            "the withheld point must not be joined across"
        );

        // A single pinned potential is not a curve either.
        let one = crate::Event::Titrated {
            vessel: crate::VesselId(0),
            titrant: crate::SpeciesId::new("KMnO4"),
            concentration: 0.02,
            steps: 3,
            total_volume: crate::units::Liters(0.003),
            final_ph: 1.1,
            curve: vec![(0.0, 1.0), (1.0, 1.05)],
            pe_curve: vec![(1.0, 5.2)],
            endpoint_reached: Some(false),
            endpoint: crate::ops::Endpoint::ColourPersists,
        };
        assert_eq!(charts_for_events(&[one], crate::i18n::Locale::EN).len(), 1);
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
