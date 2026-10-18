# First current-equivalent ammonia coupling failure

A new smaller control was frozen and executed once on source `5b054e6c`, binary `6c617d2b`, actual lock `66886813`. Runtime and harness are identical to accepted `0a5a92f1`; whole-tree equality and a new-main build are not claimed. This is distinct from the unchanged original GS005 protocol. Both pre-issue design revisions are preserved.

The neutral arm retains about2.86 micromol NH3 gas. Acid-first and acid-after arms remove nearly all free aqueous NH3, yet retain over90% of that gas. Nitrogen and chloride close within the frozen1e-8 mol audit resolution; exit0 does not establish the failed gas/protonation forecast. Saved-only `review_saved.py` authenticates raw streams and records the mismatch; its approximate Henry diagnostic is not calibration.

Repair lane: couple finite NH3 gas to native NH3/NH4+ equilibrium while retaining inert carrier gases and core-only/dry fallback. Missing database phase or complex-bound ligand support must be explicit; introduce no open ammonia reservoir. Add both feed orders, multiple finite volumes/doses, initially absent/supplied gas, pressure control, conservation and unchanged-state refusal controls. No scientific catalog addition.
