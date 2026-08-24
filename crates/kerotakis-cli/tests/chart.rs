//! The chart contract's renderer holds its own contract: every series
//! drawn and named, the provenance caption present, valid SVG shape.

use kerotakis_core::chart::{Axis, Chart, Series};

#[test]
fn renderer_draws_every_series_and_the_provenance() {
    let chart = Chart {
        title: "probe".into(),
        x: Axis {
            label: "x".into(),
            unit: None,
        },
        y: Axis {
            label: "T".into(),
            unit: Some("°C".into()),
        },
        series: vec![
            Series::Line {
                name: "bubble".into(),
                points: vec![[0.0, 100.0], [0.5, 80.0], [1.0, 78.4]],
            },
            Series::Scatter {
                name: "measured".into(),
                points: vec![[0.3, 82.0]],
            },
        ],
        provenance: "a chart is a claim".into(),
    };
    // The renderer is a private CLI module; test through the public
    // binary path: write JSON, run `kero chart`, read the SVG.
    let dir = std::env::temp_dir().join(format!("kero-chart-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let json_path = dir.join("c.json");
    let svg_path = dir.join("c.svg");
    std::fs::write(&json_path, serde_json::to_string(&chart).unwrap()).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "chart",
            json_path.to_str().unwrap(),
            "-o",
            svg_path.to_str().unwrap(),
        ])
        .output()
        .expect("kero chart runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let svg = std::fs::read_to_string(&svg_path).unwrap();
    assert!(svg.starts_with("<svg"), "valid SVG shell");
    assert!(svg.contains("polyline"), "the line series is drawn");
    assert!(svg.contains("circle"), "the scatter series is drawn");
    assert!(
        svg.contains("bubble") && svg.contains("measured"),
        "legend names both"
    );
    assert!(
        svg.contains("a chart is a claim"),
        "provenance caption present"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_band_renders_as_a_shaded_polygon() {
    let chart = Chart {
        title: "Endpoint uncertainty".into(),
        x: Axis {
            label: "acid".into(),
            unit: Some("mol".into()),
        },
        y: Axis {
            label: "endpoint".into(),
            unit: Some("L".into()),
        },
        series: vec![
            Series::Band {
                name: "p5–p95".into(),
                lower: vec![[0.005, 0.0048], [0.02, 0.0198]],
                upper: vec![[0.005, 0.0052], [0.02, 0.0202]],
            },
            Series::Line {
                name: "median".into(),
                points: vec![[0.005, 0.005], [0.02, 0.02]],
            },
        ],
        provenance: "test fixture".into(),
    };
    let dir = std::env::temp_dir().join(format!("kero-band-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let json_path = dir.join("b.json");
    let svg_path = dir.join("b.svg");
    std::fs::write(&json_path, serde_json::to_string(&chart).unwrap()).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "chart",
            json_path.to_str().unwrap(),
            "-o",
            svg_path.to_str().unwrap(),
        ])
        .output()
        .expect("kero chart runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let svg = std::fs::read_to_string(&svg_path).unwrap();
    assert!(svg.contains("polygon"), "the band is a filled polygon");
    assert!(svg.contains("fill-opacity"), "shaded, not solid");
    assert!(svg.contains("p5–p95"), "the band appears in the legend");
    assert!(svg.contains("polyline"), "the median line still draws");
}
