//! Render the chart contract to SVG (CAP-3). Hand-rolled on purpose —
//! the contract is small and a chart library would be the first
//! framework in the product. Same visual discipline as the Pourbaix
//! renderer: axes, legend, and a provenance caption, because a chart
//! is a claim.

use kerotakis_core::chart::{Chart, Series};
use std::fmt::Write as _;

const PALETTE: &[&str] = &[
    "#4e79a7", "#f28e2b", "#59a14f", "#e15759", "#b07aa1", "#76b7b2", "#edc948", "#9c755f",
];

pub fn render(chart: &Chart) -> String {
    let (px, py) = (720.0f64, 480.0f64);
    let (ml, mt, mr, mb) = (70.0, 48.0, 180.0, 78.0);
    let (w, h) = (ml + px + mr, mt + py + mb);

    let all: Vec<[f64; 2]> = chart.series.iter().flat_map(|s| s.points()).collect();
    let (mut x0, mut x1) = (f64::INFINITY, f64::NEG_INFINITY);
    let (mut y0, mut y1) = (f64::INFINITY, f64::NEG_INFINITY);
    for [x, y] in &all {
        x0 = x0.min(*x);
        x1 = x1.max(*x);
        y0 = y0.min(*y);
        y1 = y1.max(*y);
    }
    if !x0.is_finite() || x1 <= x0 {
        (x0, x1) = (0.0, 1.0);
    }
    if !y0.is_finite() || y1 <= y0 {
        (y0, y1) = (0.0, 1.0);
    }
    // A little headroom so curves don't kiss the frame.
    let (xpad, ypad) = ((x1 - x0) * 0.03, (y1 - y0) * 0.06);
    let (x0, x1, y0, y1) = (x0 - xpad, x1 + xpad, y0 - ypad, y1 + ypad);
    let xs = |x: f64| ml + (x - x0) / (x1 - x0) * px;
    let ys = |y: f64| mt + (y1 - y) / (y1 - y0) * py;

    let mut s = String::new();
    write!(s, r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.0} {h:.0}" font-family="system-ui, sans-serif">"##).unwrap();
    write!(
        s,
        r##"<rect width="{w:.0}" height="{h:.0}" fill="#ffffff"/>"##
    )
    .unwrap();
    write!(
        s,
        r##"<text x="{ml}" y="28" font-size="17" font-weight="600" fill="#111">{}</text>"##,
        esc(&chart.title)
    )
    .unwrap();

    // Frame and ticks: five per axis, values printed at the data scale.
    write!(
        s,
        r##"<rect x="{ml}" y="{mt}" width="{px}" height="{py}" fill="none" stroke="#ccc"/>"##
    )
    .unwrap();
    for i in 0..=5 {
        let fx = x0 + (x1 - x0) * i as f64 / 5.0;
        let fy = y0 + (y1 - y0) * i as f64 / 5.0;
        write!(
            s,
            r##"<text x="{:.0}" y="{:.0}" font-size="11" fill="#555" text-anchor="middle">{}</text>"##,
            xs(fx),
            mt + py + 16.0,
            trim_num(fx)
        )
        .unwrap();
        write!(
            s,
            r##"<text x="{:.0}" y="{:.0}" font-size="11" fill="#555" text-anchor="end">{}</text>"##,
            ml - 6.0,
            ys(fy) + 4.0,
            trim_num(fy)
        )
        .unwrap();
        write!(
            s,
            r##"<line x1="{ml}" y1="{0:.1}" x2="{1:.1}" y2="{0:.1}" stroke="#eee"/>"##,
            ys(fy),
            ml + px
        )
        .unwrap();
    }
    let xl = match &chart.x.unit {
        Some(u) => format!("{} ({u})", chart.x.label),
        None => chart.x.label.clone(),
    };
    let yl = match &chart.y.unit {
        Some(u) => format!("{} ({u})", chart.y.label),
        None => chart.y.label.clone(),
    };
    write!(
        s,
        r##"<text x="{:.0}" y="{:.0}" font-size="12" fill="#333" text-anchor="middle">{}</text>"##,
        ml + px / 2.0,
        h - 26.0,
        esc(&xl)
    )
    .unwrap();
    write!(
        s,
        r##"<text x="18" y="{0:.0}" font-size="12" fill="#333" transform="rotate(-90 18 {0:.0})" text-anchor="middle">{1}</text>"##,
        mt + py / 2.0,
        esc(&yl)
    )
    .unwrap();

    // Series + legend.
    for (i, series) in chart.series.iter().enumerate() {
        let colour = PALETTE[i % PALETTE.len()];
        match series {
            Series::Line { points, .. } => {
                let pts: Vec<String> = points
                    .iter()
                    .map(|[x, y]| format!("{:.2},{:.2}", xs(*x), ys(*y)))
                    .collect();
                write!(
                    s,
                    r##"<polyline points="{}" fill="none" stroke="{colour}" stroke-width="2"/>"##,
                    pts.join(" ")
                )
                .unwrap();
            }
            Series::Scatter { points, .. } => {
                for [x, y] in points {
                    write!(
                        s,
                        r##"<circle cx="{:.2}" cy="{:.2}" r="3" fill="{colour}"/>"##,
                        xs(*x),
                        ys(*y)
                    )
                    .unwrap();
                }
            }
            Series::Band { lower, upper, .. } => {
                // One closed polygon: along the lower envelope, back
                // along the upper — the shaded region between them.
                let path: Vec<String> = lower
                    .iter()
                    .chain(upper.iter().rev())
                    .map(|[x, y]| format!("{:.2},{:.2}", xs(*x), ys(*y)))
                    .collect();
                write!(
                    s,
                    r##"<polygon points="{}" fill="{colour}" fill-opacity="0.25" stroke="{colour}" stroke-width="1" stroke-opacity="0.5"/>"##,
                    path.join(" ")
                )
                .unwrap();
            }
        }
        let ly = mt + 16.0 + i as f64 * 20.0;
        write!(
            s,
            r##"<line x1="{0:.0}" y1="{1:.0}" x2="{2:.0}" y2="{1:.0}" stroke="{colour}" stroke-width="3"/><text x="{3:.0}" y="{4:.0}" font-size="12" fill="#333">{5}</text>"##,
            ml + px + 16.0,
            ly - 4.0,
            ml + px + 34.0,
            ml + px + 40.0,
            ly,
            esc(series.name())
        )
        .unwrap();
    }

    // Provenance caption, clamped to its frame.
    let mut prov = chart.provenance.replace(char::is_whitespace, " ");
    while prov.contains("  ") {
        prov = prov.replace("  ", " ");
    }
    if prov.chars().count() > 150 {
        prov = prov.chars().take(149).collect::<String>() + "…";
    }
    write!(
        s,
        r##"<text x="{ml}" y="{:.0}" font-size="10.5" fill="#666">{}</text>"##,
        h - 8.0,
        esc(&prov)
    )
    .unwrap();
    write!(s, "</svg>").unwrap();
    s
}

fn esc(t: &str) -> String {
    t.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn trim_num(v: f64) -> String {
    if v.abs() >= 100.0 {
        format!("{v:.0}")
    } else if v.abs() >= 1.0 {
        format!("{v:.1}")
    } else {
        format!("{v:.2}")
    }
}
