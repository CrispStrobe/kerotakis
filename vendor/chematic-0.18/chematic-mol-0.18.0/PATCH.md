# Local patch to `chematic-mol` 0.18.0 — V2000 implicit hydrogens

This crate is part of the audited `vendor/chematic-0.18/` release tree that
the workspace root's `[patch.crates-io]` block points at. That tree is
vendored verbatim; **`src/mol2000.rs` is the one file in it that Kerotakis
has changed**, and this file records what and why.

```
$ diff -rq ~/.cargo/registry/src/*/chematic-mol-0.18.0 \
           vendor/chematic-0.18/chematic-mol-0.18.0
Files …/src/mol2000.rs and …/src/mol2000.rs differ
```

Upstream: [`chematic-mol` 0.18.0](https://crates.io/crates/chematic-mol/0.18.0)
(part of [`chematic`](https://github.com/kent-tokyo/chematic)),
`MIT OR Apache-2.0`.

- crates.io `.crate` sha256:
  `339ad56c77290c0e91e2c427b8b633bf94fa7e773696af09bbaef2e56fb8c23c`
- upstream git commit (from the crate's `.cargo_vcs_info.json`):
  `9cd0f8beac91c41dcd3304049925a0a35ca4eed3`, path `crates/chematic-mol`

It is a **patch, not a fork**. The intended end state is upstream taking the
same change and this file, along with the local delta, disappearing at the
next chematic bump.

---

## The bug

`write_mol` (and `write_mol_with_conformer`) never emit the V2000 atom
block's **valence field** (`vvv`, columns 49–51). Every atom line went out
as:

```
    0.0000    0.0000    0.0000 Mg  0  0  0  0  0  0  0  0  0  0  0
                                            ^^^ sss hhh bbb vvv …all zero
```

V2000 has no field that states "this atom has exactly N hydrogens" in a
structure file — `hhh` is a *query* field, read as "at least". The
structural channel is `vvv`, the atom's **total** valence (bonds plus
hydrogens), where `0` means "unspecified" and `15` is the sentinel for
"zero valence".

A `Molecule` parsed from a bracket SMILES atom (`[Mg]`, `[C]`, `[S]`,
`[Pb]`) carries `hydrogen_count: Some(0)` — an explicit, deliberate *zero*.
That is precisely the fact an unspecified `vvv` cannot express, so every
standard-valence reader downstream fills the atom back up to its normal
valence. Against the IUPAC InChI reference reader:

| SMILES | what the molfile made InChI see | should be |
| --- | --- | --- |
| `[Mg]` | `InChI=1S/Mg.2H` (magnesium + 2 H) | `InChI=1S/Mg` |
| `[Pb]` | `InChI=1S/Pb.2H` | `InChI=1S/Pb` |
| `[C]`  | `InChI=1S/CH4/h1H4` — **methane** | `InChI=1S/C` |
| `[S]`  | `InChI=1S/H2S/h1H2` — **hydrogen sulfide** | `InChI=1S/S` |

The molfile encoded a different substance than the molecule it was written
from. It is not an InChI quirk: any consumer applying standard valences
(RDKit, Open Babel, a CTfile-conformant viewer) reads the same wrong thing,
and a MOL→SMILES round trip through another toolkit returns `[MgH2]`, `C`,
`S`.

## The fix

One new private helper, `v2000_valence_field(mol, idx)`, used by both V2000
atom-block writers:

- Atoms with **no** explicit `hydrogen_count` (organic-subset SMILES atoms,
  and every atom read back from a MOL file — the format has no H-count
  channel to read) keep `vvv = 0`. Their hydrogen count *is* the
  standard-valence inference, so restating it would say nothing new while
  changing the bytes of every file this writer has ever produced.
- Atoms **with** an explicit count get `vvv = Σ(bond orders) + hydrogens`,
  or the `15` sentinel when that total is zero.
- An atom carrying an **aromatic or query** bond order has no integer
  valence to sum, so it declines to state `vvv` at all rather than guess.
- Totals above 14 are unrepresentable (15 is taken by the sentinel) and
  fall back to unspecified.

The field is written *into* the existing fixed-width layout, not appended:
the atom line is still 66 columns and every other field lands where it did.
chematic's own reader ignores `vvv` (unchanged), so nothing about
chematic→chematic round-tripping moves.

Five regression tests were added next to the writer's existing ones in
`src/mol2000.rs`:

- `test_bare_bracket_atom_writes_zero_valence_sentinel`
- `test_bracket_atom_valence_field_counts_bonds_and_hydrogens`
- `test_organic_subset_atoms_leave_the_valence_field_unspecified`
- `test_aromatic_bond_declines_to_state_a_valence`
- `test_valence_field_does_not_disturb_the_other_atom_columns`

Those five are **upstream-facing**: they travel with the patch, and this
workspace does not build them, because the vendored tree is reached through
`[patch.crates-io]` and is not a workspace member.

Running them takes a scratch copy, for two reasons that are both properties
of the published crate and neither caused by this patch:

1. The crate has no `[workspace]` of its own, so cargo walks up until it
   finds one — the repo's, or in a `git worktree` the **main** checkout's,
   which produces "current package believes it's in a workspace when it's
   not". An empty `[workspace]` table in the copy settles it.
2. `cargo publish` strips path-only dev-dependencies, and a `#[cfg(test)]`
   block in `src/cdxml.rs` uses `chematic_chem`. The *lib test binary* is
   one compilation unit, so that unrelated module's test block stops
   `--lib` from building until the dev-dependencies are restored — pointed
   at the sibling directories in this same vendored tree.

So: copy this directory somewhere scratch, append an empty `[workspace]`
table, add `chematic-chem` / `chematic-fp` / `chematic-smarts` as path
dev-dependencies pointing at their siblings in `vendor/chematic-0.18/`,
add a `[patch.crates-io]` block pointing the rest of the chematic crates at
those siblings too, and then

```
cargo test --manifest-path <copy>/Cargo.toml --lib mol2000
```

What runs in this repo's ordinary gate, with no copy needed, is
`crates/kerotakis-org/tests/write_mol_repro.rs`, which covers the same
behaviour end to end through the official InChI library.

## Sending it upstream

`src/mol2000.rs` is the whole diff and the tests come with it, so

```
diff -u ~/.cargo/registry/src/*/chematic-mol-0.18.0/src/mol2000.rs \
        vendor/chematic-0.18/chematic-mol-0.18.0/src/mol2000.rs
```

is the patch to file against <https://github.com/kent-tokyo/chematic>.

## What this does *not* fix

CAP-13 deferred eleven species on chematic `write_mol` limitations. This
patch is the implicit-H class (four species) only. Six remain open, with
live repro cases in `crates/kerotakis-org/tests/write_mol_repro.rs`:

- **kekulisation** (2): methyl orange, bromothymol blue.
- **connectivity / charge** (4): Cu(OH)₂, MnO₄⁻, Pb²⁺, Pb(NO₃)₂ — InChI
  disconnects the metal (`Cu.2H2O/q+2;;/p-2`, `Mn.4O`) and the charge layer
  does not survive.

The eleventh, phenolphthalein, turned out not to be a chematic bug: given
the closed lactone rather than the open acid form, this writer already
produces a molfile the official library keys correctly. It was a curated
SMILES mistake blamed on the library.
