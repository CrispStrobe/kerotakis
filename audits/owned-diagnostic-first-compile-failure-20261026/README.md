# Owned diagnostic candidate: first compiler failure

The first candidate `f61a2855` fails [CI](https://github.com/CrispStrobe/kerotakis/actions/runs/40343194391) and the [audit build](https://github.com/CrispStrobe/kerotakis/actions/runs/40343194372). Source-only locale and peer reviews did not establish Rust compilation. Five compiler errors identify an actual selected-output table omitted from the diagnostic helper signature; strict Clippy also rejects a new test assertion idiom.

The successor passes the actual cached table through both production paths and uses `expect_err` for the capability refusal. Numeric guards, raw detail and test assertions remain unchanged. Frozen [compiler](COMPILER-EXCERPT.log), [Clippy](CLIPPY-EXCERPT.log) and [source/log bindings](FIRST-FAILURE.json) preserve the failure. No candidate scientific or runtime acceptance is claimed. Complete new-head validation remains required.
