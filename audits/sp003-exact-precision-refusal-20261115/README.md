# SP003 exact original: precision refusal, no new credit

**Actual reader result: SP-003-C1 FAIL, SP-003-C2 PASS; qualification FALSE,431→431/468.** All435 catalog entries remain untouched. Original `new120:sp-003` script SHA256 `203a94f77d183e9be5adfd54dcb62a1024bd25979e94d036cba16e97c52df085`; all original commands, cut fractions, stage counts, forecast and criteria are preserved.

Source `daeb04f0e1005da08747d371be1eb5959ee93514`, tree `9a5c1e08467964db6be1941635bf9138ee078fa2`, adds only exact-original/control fixtures and opt-in refusal logging to actual main6068. It does not alter a solver algorithm, iteration budget, guard, tolerance or phase law. No PR is opened for this diagnostic source. The actual Rust core physics/honesty stack executes the original on one fresh bench; no aqueous chemical engine or new physical data is needed by its water/ethanol separation operators. This is not a new full-source chemical qualification.

## Actual original outcome

Both matched feeds contain2.7754648903691366molwater and0.10853285289457118molethanol, at298.15K and101325Pa; the VLE model receives101.325kPa. The one-stage receiver contains0.21927613980246935molwater+0.0691236345239015molethanol, a0.28839977432637087mol cut. The four-stage receiver remains empty. Its complete source retains the original2.7754648903691366molwater+0.10853285289457118molethanol; the refused operator leaves the exact whole-bench state unchanged. All10 command ledgers close C/H/O/charge below1.7763568394002505e-15mol/eq. No extra ethanol is created, but there is no second product to compare: C1 is FAIL, not an unsupported output counted as a negative separation.

## Exact causal trace and ownership boundary

At provisional integration step629, the four-stage model retains2.73428661615462154molwater and2.77555756156289135e-17molethanol. Accumulated ethanol condensate is0.108532852894571152mol; the next positive6.93889390390722838e-18mol ethanol increment rounds away when added to that binary64 accumulator. The water increment is1.38567164285836562e-5mol and0.13868864721728455mol of the requested cut is still pending. The unchanged `e_over + de == e_over` guard therefore refuses rather than stalling or silently clipping the component. Provisional pot/condensate amounts are **not committed physical vessel stocks**. Internal phase temperature at the refusal was not logged and remains explicitly null; the exact bench T/P above is recorded, not substituted for the boiling phase temperature.

The affected ownership is `StillCut`'s scalar accumulated overhead and the residue derived by subtraction, followed by the bench's scalar withdrawal. A numerical completion needs a component-owned split representation that retains positive residues independently and preserves the donor/receiver ownership through the writer; merely deleting the guard, changing the original cut/stages, widening a tolerance, or assigning the missing residue to zero is not a fix. No such representation/model-writer change is implemented or validated here. Do not retry this unchanged case.

## Bounded execution and controls

The first local source-bound command exited101 before scientific execution because the clean worktree's SUNDIALS submodule was absent. After that setup correction, one nonblocking shared-lock admission was busy and launched no build. Both exact setup/resource receipts remain in the private operating handoff. One cached CPU closure then completed; no second closure or original retry was issued.

The actual original test exits101 at the first C1 assertion (nonempty matched products). Its independent small-cut/invalid-composition control passes, and five unchanged precision/failure ownership tests pass. The complete independent original reader ran, authenticates59 small producer files and the independently stream-hashed executable `a39d4ae79e4350ea5df4b77650aa9fc2763e0e4bd039d4f42790bca36512fd26`, verifies exact source/tree/compiler/lock, and rejects four mutations. Full raw producer evidence remains private; public logs are labelled excerpts with full raw hashes. Re-running the reader requires that producer bundle rather than these excerpts.

## PR884 ordinary acceptance, separate source

The same bounded CPU closure first runs six ordinary core suites on exact PR884 head `5db4a1260112c79b7ed610e5a8399b151dfd7363`: buoyancy5, density6, float/sink5, measurement8, paraffin7, scene13, for **44PASS/0FAIL**. Their six executable artifacts were independently stream-hashed. No PO002 original was repeated and no second PO002 credit is earned. Full source/platform/fleet acceptance remains separate; PR884 is draft and has no merge authorization. Existing normal CI/fleet runs remain in place; no `ci:full`, replacement agent, EC009 water retry, release upload or distribution is part of this work.
