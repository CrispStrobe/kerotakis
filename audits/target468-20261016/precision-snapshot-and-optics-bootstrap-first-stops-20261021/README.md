# Precision snapshot and optics bootstrap first stops

Phase source `3da21055bc8de53c63388f0c8ef850370546cac0` passes Clippy but stops during full native execution at the existing rounded-water snapshot. Actual phi is0.9325 and freezing point−0.3468 °C, as the original test commentary predicted for precise water activity. The old model snapshot was−0.3516 ±0.004 °C. Physical assertions before the snapshot pass; the physical comparison after it was not reached. Full native/core/fleet acceptance is pending.

A test-only successor narrows the model snapshot tenfold to−0.3468 ±0.0004 °C. All model-route, osmotic-coefficient and empirical bands remain unchanged. Stale rounding comments are corrected; dilute-model admission and primary-source verification limits remain unchanged. The prior30-control phase receipt retains its actual predecessor binding. New-source execution remains required.

Optics source `14d502b2262134b636d44504d446652c3cfe49eb` stops during baseline dependency setup: the Rust SUNDIALS submodule manifest is missing. No controls ran. Initialize pinned Git submodules before Cargo resolves the workspace; fetching C sources alone is insufficient. Preserve source, frozen forecasts and observers for the setup-only successor.

Raw first-stop streams and producer manifests are retained here. Neither lane is accepted, integrated or deployed. Coverage remains334/468, catalog338. [Maintained handoff](../../../docs/next-agent-handoff.md).
