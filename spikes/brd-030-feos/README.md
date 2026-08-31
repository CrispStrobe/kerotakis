# BRD-030 — direct feos integration spike

A **decision-gate** artefact, not a delivery. Nothing here is wired into
routing, into the CLI, into wasm, or into CI. The verdict lives in
[`provenance/brd-030-feos-spike.md`](../../provenance/brd-030-feos-spike.md);
this directory is the evidence behind it.

## Why it is outside the workspace

`Cargo.toml` here declares its own `[workspace]`. That is deliberate:

* the kerotakis workspace's `members` list is untouched, so no production
  build, `Cargo.lock`, `cargo deny` run, or NOTICE generation can pick feos
  up by accident;
* the spike keeps its own lockfile, added by BRD-031 after its absence was
  caught in audit, and pins feos/feos-core to exactly `0.10.1`;
* deleting this directory removes the dependency completely.

It depends on `kerotakis-thermo` by path, read-only. No file under
`crates/` was modified for this spike.

## Layout

| path | what it is |
|---|---|
| `corpus.json` | the comparison corpus, read by **both** sides so all three engines answer the same question |
| `src/adapter.rs` | the adapter prototype — feos dressed as `kerotakis_thermo::fluid::FluidModel` for one calculation family (bubble point) |
| `src/main.rs` | driver: emits kerotakis-thermo, feos and adapter results as TSV |
| `oracle.py` | the referee: the Python `thermo` package (MIT), extending the CAP-19 pattern in `tools/gen-thermo-fixtures.py` |
| `compare.py` | joins the three and classifies every disagreement |
| `wasm-probe-feos/`, `wasm-probe-base/` | two cdylibs whose `.wasm` size difference is what feos costs a browser bundle |
| `fixtures/` | the checked-in outputs |

## Parameters are not checked in

`fetch-parameters.sh` downloads feos's published parameter JSONs into a
gitignored `parameters/`. They are third-party tables transcribed from
journal publications; clearing them for shipping is BRD-031's job, and this
spike deliberately does not pre-empt it by committing them. The report
records what each file is, where it came from, and the checksum the
measurements were taken against.

## Reproducing

```sh
export CARGO_TARGET_DIR=/somewhere/with/room
./fetch-parameters.sh
cargo build --release --locked
../../../target/release/brd030-feos-spike emit > fixtures/engines.tsv
python3 oracle.py > fixtures/oracle.tsv          # needs `thermo` (MIT)
python3 compare.py                                # writes fixtures/discrepancies.tsv
cargo run --release --locked -- bench             # order-of-magnitude timings
cargo build --release --locked --lib --target wasm32-unknown-unknown
cargo build --release --locked --target wasm32-unknown-unknown -p wasm-probe-feos -p wasm-probe-base
```
