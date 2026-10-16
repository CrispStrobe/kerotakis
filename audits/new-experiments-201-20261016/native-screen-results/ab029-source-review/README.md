# AB-029 source diagnosis and narrow repair draft

The saved failure exposes a concrete aqueous-transfer ownership defect. Filter
moves dissolved contents into an empty receiver while leaving its measured
hydroxide/proton and solute-charge baseline behind. The next StepStart captures
zeros in the receiver; PHREEQC prices measured hydroxide separately from analytical
H/O portions. This can charge transported, pre-existing hydroxide as newly formed.

[Review and writer map](review.json) bind the actual c446 runtime source and saved
witness. The broad audit documentation tree is not an integrated runtime baseline.
Its numerical mechanics must not be ported as prerequisites.

The missing measured-hydroxide reference scale is about 1.41657 J, or 0.003399 K
on the local water heat-capacity scale. Its proximity to the 0.003402842 K final
residual is evidence to investigate, **not proof of the exact 64-pass recurrence**.
No per-iteration native temperature/heat/OH trace was saved. Combined matter/charge
closes, and the failed target pH is honestly withheld. Same temperature alone does
not establish energy closure.

[Draft PR809](https://github.com/CrispStrobe/kerotakis/pull/809), signed head
`25956e619cc7c874e0099377a49a23d5791efb57`, proposes a shared measured
snapshot carry for Filter and proportional Decant only. It moves source/destination
baseline coordinates exactly once after admission, invalidates solution caches,
and creates no new chemical portions. Filter also preserves nontransferred gas
instead of silently deleting it. This retains existing admission/headspace scope;
it introduces no pressure-work or gas-export model. Mix, Drain, Extract and typed
transport require their own semantic review.

[Draft file hashes and validation state](draft-status.json) identify the source
proposal. Ten core regression groups and two native acceptance tests are written,
including the exact original AB-029 script and repeated solved-base pours without
phantom cooling. Formatting/source checks passed; compilation, native/core/Wasm
acceptance and the applicable fleet remain pending. No tolerance, iteration or
bracket change is proposed. No local engine or build ran.
