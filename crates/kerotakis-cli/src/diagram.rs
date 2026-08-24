//! `kero diagram pourbaix` — the computed predominance map, drawn (CAP-4).
//!
//! The chemistry lives in `kerotakis_phreeqc::pourbaix`; this file only
//! renders: coloured cells for the dominant form, crisp boundaries where
//! neighbouring cells disagree, the computed water-stability lines, a
//! legend, and the provenance caption a chart owes its reader. Hand-rolled
//! SVG on purpose — the contract is small and a chart library would be the
//! first framework in the product.

use kerotakis_phreeqc::pourbaix::{
    curated_elements, diagram, outside_water_stability, water_stability_lines, PourbaixDiagram,
};
use kerotakis_phreeqc::PhreeqcEquilibrator;
use std::fmt::Write as _;

/// A small, distinguishable palette; labels are assigned colours in
/// first-appearance order. Neutral greys are reserved for refusals.
const PALETTE: &[&str] = &[
    "#4e79a7", "#f28e2b", "#59a14f", "#e15759", "#b07aa1", "#76b7b2", "#edc948", "#9c755f",
];

/// `kero diagram txy` — the ethanol–water temperature–composition
/// envelope, the McCabe–Thiele backdrop, computed point by point with
/// full UNIFAC γ(x, T) and emitted as the CAP-3 chart contract before
/// rendering. The bubble and dew curves meet at the azeotrope because
/// the thermodynamics says so, not because the plot was drawn that way.
pub fn run_txy(args: &[String]) -> Result<(), String> {
    let mut out_path: Option<String> = None;
    let mut json = false;
    let mut n = 120usize;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" | "-o" => {
                i += 1;
                out_path = Some(args.get(i).ok_or("--out needs a file path")?.to_string());
            }
            "--json" => json = true,
            "--points" => {
                i += 1;
                n = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .ok_or("--points needs a number")?;
            }
            other => return Err(format!("unknown argument '{other}'")),
        }
        i += 1;
    }
    let n = n.clamp(16, 2000);
    let mut bubble = Vec::with_capacity(n + 1);
    let mut dew = Vec::with_capacity(n + 1);
    for k in 0..=n {
        let x = k as f64 / n as f64;
        if let Some(bp) = kerotakis_thermo::vle::ethanol_water_bubble_point(
            x,
            kerotakis_thermo::vle::ATMOSPHERE_KPA,
        ) {
            bubble.push([x, bp.t_celsius]);
            dew.push([bp.y[0], bp.t_celsius]);
        }
    }
    dew.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let chart = kerotakis_core::chart::Chart {
        title: "Ethanol–water T–x–y at 1 atm — computed".to_string(),
        x: kerotakis_core::chart::Axis {
            label: "mole fraction ethanol".to_string(),
            unit: None,
        },
        y: kerotakis_core::chart::Axis {
            label: "temperature".to_string(),
            unit: Some("°C".to_string()),
        },
        series: vec![
            kerotakis_core::chart::Series::Line {
                name: "bubble (liquid)".to_string(),
                points: bubble,
            },
            kerotakis_core::chart::Series::Line {
                name: "dew (vapour)".to_string(),
                points: dew,
            },
        ],
        provenance: "bubble points from UNIFAC γ(x, T) (Fredenslund 1975 parameters) over                      Stull-fit Antoine constants; dew curve is the same points read at their                      vapour composition; cross-checked against the Python thermo package to                      a part in a million (tests/thermo_oracle.rs)"
            .to_string(),
    };
    if json {
        println!(
            "{}",
            serde_json::to_string(&chart).map_err(|e| e.to_string())?
        );
    }
    let path = out_path.unwrap_or_else(|| "txy-ethanol-water.svg".to_string());
    std::fs::write(&path, crate::chart_svg::render(&chart))
        .map_err(|e| format!("cannot write {path}: {e}"))?;
    eprintln!("wrote {path} — {} computed points per curve", n + 1);
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), String> {
    let mut element: Option<&str> = None;
    let mut grid = (48usize, 40usize);
    let mut out_path: Option<String> = None;
    let mut json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--grid" => {
                i += 1;
                let spec = args.get(i).ok_or("--grid needs NxM, e.g. --grid 48x40")?;
                let (a, b) = spec
                    .split_once(['x', 'X'])
                    .ok_or("--grid needs NxM, e.g. --grid 48x40")?;
                grid = (
                    a.parse().map_err(|_| format!("bad grid '{spec}'"))?,
                    b.parse().map_err(|_| format!("bad grid '{spec}'"))?,
                );
            }
            "--out" | "-o" => {
                i += 1;
                out_path = Some(args.get(i).ok_or("--out needs a file path")?.to_string());
            }
            "--json" => json = true,
            other if element.is_none() && !other.starts_with('-') => {
                element = Some(other);
            }
            other => return Err(format!("unknown argument '{other}'")),
        }
        i += 1;
    }
    let Some(element) = element else {
        return Err(format!(
            "usage: kero diagram pourbaix <element> [--grid NxM] [--out FILE.svg] [--json]\n\
             curated elements: {}",
            curated_elements().join(", ")
        ));
    };

    let mut eq = PhreeqcEquilibrator::new().map_err(|e| e.to_string())?;
    eprintln!(
        "computing {}×{} pe–pH grid for {element} ({} solves)…",
        grid.0,
        grid.1,
        grid.0 * grid.1
    );
    let d = diagram(&mut eq, element, grid.0, grid.1)?;

    if json {
        println!("{}", to_json(&d));
    }
    let path = out_path.unwrap_or_else(|| format!("pourbaix-{}.svg", d.element.to_lowercase()));
    std::fs::write(&path, render_svg(&d)).map_err(|e| format!("cannot write {path}: {e}"))?;
    eprintln!(
        "wrote {path} — {} regions, {} refused cells, engine calls {}",
        d.distinct().len(),
        d.refused,
        eq.engine_calls()
    );
    Ok(())
}

/// The chart contract, machine-readable: axes, cells, legend, provenance.
/// This is CAP-3's seed — when the study runner grows charts, this shape
/// moves to a shared module rather than growing a second dialect.
fn to_json(d: &PourbaixDiagram) -> String {
    let labels: Vec<serde_json::Value> = d
        .labels
        .iter()
        .map(|l| match l {
            Some(s) => serde_json::Value::String(s.clone()),
            None => serde_json::Value::Null,
        })
        .collect();
    serde_json::json!({
        "chart": "predominance-grid",
        "title": format!("Pourbaix diagram: {}", d.element),
        "x": {"label": "pH", "values": d.ph},
        "y": {"label": "pe", "values": d.pe},
        "cells": labels,
        "legend": d.distinct(),
        "refused": d.refused,
        "t_celsius": d.t_celsius,
        "database": d.db_tag,
        "provenance": d.provenance,
    })
    .to_string()
}

fn render_svg(d: &PourbaixDiagram) -> String {
    let (nx, ny) = (d.ph.len(), d.pe.len());
    let (px, py) = (760.0f64, 560.0f64); // plot area
    let (ml, mt, mr, mb) = (64.0, 46.0, 230.0, 78.0);
    let (w, h) = (ml + px + mr, mt + py + mb);
    let cw = px / nx as f64;
    let ch = py / ny as f64;
    let x_of = |ph: f64| ml + (ph - d.ph[0]) / (d.ph[nx - 1] - d.ph[0]) * px;
    let y_of = |pe: f64| mt + (d.pe[ny - 1] - pe) / (d.pe[ny - 1] - d.pe[0]) * py;

    let legend = d.distinct();
    let colour = |label: &str| -> &str {
        legend
            .iter()
            .position(|l| l == label)
            .map(|i| PALETTE[i % PALETTE.len()])
            .unwrap_or("#999999")
    };

    let mut s = String::new();
    write!(s, r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.0} {h:.0}" font-family="system-ui, sans-serif">"##).unwrap();
    write!(
        s,
        r##"<rect width="{w:.0}" height="{h:.0}" fill="#ffffff"/>"##
    )
    .unwrap();
    write!(
        s,
        r##"<text x="{ml}" y="26" font-size="18" font-weight="600" fill="#111">Pourbaix diagram: {} — computed, {} database, {:.0} °C</text>"##,
        d.element, d.db_tag, d.t_celsius
    )
    .unwrap();

    // Cells. Refusals outside the water-stability field are the pale
    // wash physics predicts; refusals inside it are hatched dark so a
    // hole can never pass for a region.
    for (j, &pe) in d.pe.iter().enumerate() {
        for (i, &ph) in d.ph.iter().enumerate() {
            let x = ml + i as f64 * cw;
            let y = mt + (ny - 1 - j) as f64 * ch;
            match d.label(j, i) {
                Some(label) => {
                    write!(
                        s,
                        r##"<rect x="{x:.2}" y="{y:.2}" width="{cw:.2}" height="{ch:.2}" fill="{}"/>"##,
                        colour(label)
                    )
                    .unwrap();
                }
                None if outside_water_stability(ph, pe) => {
                    write!(
                        s,
                        r##"<rect x="{x:.2}" y="{y:.2}" width="{cw:.2}" height="{ch:.2}" fill="#eef0f2"/>"##
                    )
                    .unwrap();
                }
                None => {
                    write!(
                        s,
                        r##"<rect x="{x:.2}" y="{y:.2}" width="{cw:.2}" height="{ch:.2}" fill="#3b3b3b"/>"##
                    )
                    .unwrap();
                }
            }
        }
    }

    // Boundaries where neighbours disagree — the region edges.
    write!(s, r##"<g stroke="#222" stroke-width="1.1">"##).unwrap();
    for j in 0..ny {
        for i in 0..nx {
            let here = d.label(j, i);
            let x = ml + i as f64 * cw;
            let y = mt + (ny - 1 - j) as f64 * ch;
            if i + 1 < nx && here != d.label(j, i + 1) {
                write!(
                    s,
                    r##"<line x1="{0:.2}" y1="{1:.2}" x2="{0:.2}" y2="{2:.2}"/>"##,
                    x + cw,
                    y,
                    y + ch
                )
                .unwrap();
            }
            if j + 1 < ny && here != d.label(j + 1, i) {
                write!(
                    s,
                    r##"<line x1="{0:.2}" y1="{2:.2}" x2="{1:.2}" y2="{2:.2}"/>"##,
                    x,
                    x + cw,
                    y
                )
                .unwrap();
            }
        }
    }
    write!(s, "</g>").unwrap();

    // Water-stability lines, computed, dashed.
    let (upper, lower) = water_stability_lines(&d.ph);
    for line in [&upper, &lower] {
        let pts: Vec<String> = line
            .iter()
            .filter(|(_, pe)| *pe >= d.pe[0] && *pe <= d.pe[ny - 1])
            .map(|(ph, pe)| format!("{:.1},{:.1}", x_of(*ph), y_of(*pe)))
            .collect();
        if pts.len() >= 2 {
            write!(
                s,
                r##"<polyline points="{}" fill="none" stroke="#1a4d8f" stroke-width="1.6" stroke-dasharray="7 5"/>"##,
                pts.join(" ")
            )
            .unwrap();
        }
    }

    // Axes.
    write!(
        s,
        r##"<g font-size="12" fill="#333"><text x="{:.0}" y="{:.0}">pH</text><text x="16" y="{:.0}" transform="rotate(-90 16 {:.0})">pe</text></g>"##,
        ml + px / 2.0,
        h - 22.0,
        mt + py / 2.0,
        mt + py / 2.0
    )
    .unwrap();
    for t in 0..=14 {
        let x = x_of(t as f64);
        write!(
            s,
            r##"<text x="{x:.0}" y="{:.0}" font-size="11" fill="#555" text-anchor="middle">{t}</text>"##,
            mt + py + 16.0
        )
        .unwrap();
    }
    let mut pe_t = -10i32;
    while pe_t <= 20 {
        let y = y_of(pe_t as f64);
        write!(
            s,
            r##"<text x="{:.0}" y="{y:.0}" font-size="11" fill="#555" text-anchor="end">{pe_t}</text>"##,
            ml - 8.0
        )
        .unwrap();
        pe_t += 5;
    }

    // Legend.
    let lx = ml + px + 18.0;
    write!(
        s,
        r##"<text x="{lx:.0}" y="{:.0}" font-size="13" font-weight="600" fill="#111">dominant form</text>"##,
        mt + 8.0
    )
    .unwrap();
    for (i, label) in legend.iter().enumerate() {
        let y = mt + 26.0 + i as f64 * 22.0;
        write!(
            s,
            r##"<rect x="{lx:.0}" y="{:.0}" width="14" height="14" fill="{}"/><text x="{:.0}" y="{:.0}" font-size="12" fill="#333">{}</text>"##,
            y - 11.0,
            PALETTE[i % PALETTE.len()],
            lx + 20.0,
            y,
            svg_escape(label)
        )
        .unwrap();
    }
    let mut ly = mt + 26.0 + legend.len() as f64 * 22.0;
    write!(
        s,
        r##"<rect x="{lx:.0}" y="{:.0}" width="14" height="14" fill="#eef0f2"/><text x="{:.0}" y="{ly:.0}" font-size="12" fill="#333">outside water stability</text>"##,
        ly - 11.0,
        lx + 20.0
    )
    .unwrap();
    if d.refused > 0 {
        ly += 22.0;
        write!(
            s,
            r##"<rect x="{lx:.0}" y="{:.0}" width="14" height="14" fill="#3b3b3b"/><text x="{:.0}" y="{ly:.0}" font-size="12" fill="#333">engine refused (in-field)</text>"##,
            ly - 11.0,
            lx + 20.0
        )
        .unwrap();
    }
    // Dashed-line legend entry.
    ly += 24.0;
    write!(
        s,
        r##"<line x1="{lx:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="#1a4d8f" stroke-width="1.6" stroke-dasharray="7 5"/><text x="{:.0}" y="{ly:.0}" font-size="12" fill="#333">water stability (O2 / H2)</text>"##,
        ly - 5.0,
        lx + 14.0,
        ly - 5.0,
        lx + 20.0
    )
    .unwrap();

    // Provenance caption: a chart is a claim. Two clamped lines so the
    // claim never runs off its own frame.
    let mut prov = d.provenance.replace(char::is_whitespace, " ");
    while prov.contains("  ") {
        prov = prov.replace("  ", " ");
    }
    if prov.chars().count() > 150 {
        prov = prov.chars().take(149).collect::<String>() + "…";
    }
    write!(
        s,
        r##"<text x="{ml:.0}" y="{:.0}" font-size="10.5" fill="#666">Every cell is a PHREEQC solve at fixed (pH, pe); regions are computed dominance, not lookup.</text>"##,
        h - 22.0,
    )
    .unwrap();
    write!(
        s,
        r##"<text x="{ml:.0}" y="{:.0}" font-size="10.5" fill="#666">{}</text>"##,
        h - 8.0,
        svg_escape(&prov)
    )
    .unwrap();
    write!(s, "</svg>").unwrap();
    s
}

fn svg_escape(t: &str) -> String {
    t.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
