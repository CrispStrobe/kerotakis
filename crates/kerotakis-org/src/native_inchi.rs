//! Build the official IUPAC InChI input structure directly from a chematic
//! molecule, without the V2000 molfile round-trip.
//!
//! The molfile detour was CAP-13's original bridge and it loses three things
//! the format cannot carry from a coordinate-less molecule: an explicit
//! "this atom has no hydrogens" (V2000 expresses that only through the
//! valence field, which chematic's writer never emits), the isotope
//! (never emitted), and every stereo descriptor (a 0-coordinate molfile
//! has no geometry, and chematic's writer additionally mis-encodes a
//! SMILES `/` `\` E/Z direction as a tetrahedral wedge). The official
//! library's own 0D input structure carries all three.

use chematic::core::{
    implicit_hcount, AtomIdx, BondOrder as CBond, Chirality, Molecule as CMol, STEREO_H_SENTINEL,
};
use inchi::{Atom as IAtom, BondOrder as IBond, ImplicitH, Molecule as IMol, Parity, Stereo};

/// Standard InChI (and its key) for `smiles`, via the official library's 0D
/// structure input.
pub fn native_inchikey_from_smiles(smiles: &str) -> Result<String, crate::OrgError> {
    let inchi = native_inchi_from_smiles(smiles)?;
    inchi::inchikey(&inchi).map_err(|e| crate::OrgError::InchiFailed(format!("InChIKey: {e}")))
}

/// The standard InChI string itself, for the callers that want the layers
/// rather than the hash.
pub fn native_inchi_from_smiles(smiles: &str) -> Result<String, crate::OrgError> {
    let mol = chematic::smiles::parse(smiles)
        .map_err(|e| crate::OrgError::InchiFailed(format!("SMILES parse: {e}")))?;
    let mol = kekulized(mol);
    let structure = to_inchi_molecule(&mol)?;
    let out = structure
        .to_inchi(())
        .map_err(|e| crate::OrgError::InchiFailed(format!("official InChI: {e}")))?;
    Ok(out.inchi().to_string())
}

/// Kekulize if we can; an aromatic ring system that will not kekulize is
/// still handed over with `Alternating` bonds rather than refused.
fn kekulized(mol: CMol) -> CMol {
    let mut m = mol;
    let _ = chematic::perception::kekulize_inplace(&mut m);
    m
}

fn to_inchi_molecule(mol: &CMol) -> Result<IMol, crate::OrgError> {
    let mut out = IMol::new();

    for (idx, atom) in mol.atoms() {
        let mut a = IAtom::new(atom.element.symbol()).charge(atom.charge);
        if let Some(iso) = atom.isotope {
            a = a.isotope(iso);
        }
        // `implicit_hcount` returns the bracket atom's explicit count verbatim
        // and infers the organic-subset count otherwise — exactly the number
        // the SMILES means. Handing it over as `Exactly` is what stops the
        // library re-deriving `[Mg]` as MgH2.
        a = a.implicit_hydrogens(ImplicitH::Exactly(implicit_hcount(mol, idx)));
        out.add_atom(a);
    }

    for (_bidx, bond) in mol.bonds() {
        let order = match bond.order {
            CBond::Double => IBond::Double,
            CBond::Triple => IBond::Triple,
            CBond::Aromatic => IBond::Alternating,
            // `Up`/`Down` are SMILES `/` `\` — single bonds carrying an E/Z
            // direction, not wedges. The direction is read back below.
            _ => IBond::Single,
        };
        out.add_bond(bond.atom1.0 as usize, bond.atom2.0 as usize, order)
            .map_err(|e| crate::OrgError::InchiFailed(format!("bond: {e}")))?;
    }

    for stereo in tetrahedral_stereo(mol) {
        out.add_stereo(stereo);
    }
    for stereo in double_bond_stereo(mol) {
        out.add_stereo(stereo);
    }

    Ok(out)
}

/// SMILES `@`/`@@` → InChI 0D tetrahedral parity.
///
/// Both conventions are the same function of an ordered 4-tuple of
/// neighbours: `@` means "looking from the first neighbour, the other three
/// run anticlockwise", and InChI's is "parity is even if the last three run
/// clockwise seen from the first". So `@` is odd, `@@` is even, and any
/// transposition of the tuple flips it.
///
/// A stereocentre with an implicit hydrogen has only three graph neighbours.
/// InChI's documented spelling for that case puts the *central atom itself*
/// in the vacant slot and views from it (`inchi_api.h`, "3 neighbors"), so
/// the sentinel is replaced by the centre and swapped to the front, flipping
/// the parity once per swap.
fn tetrahedral_stereo(mol: &CMol) -> Vec<Stereo> {
    let mut out = Vec::new();
    for (idx, atom) in mol.atoms() {
        let base = match atom.chirality {
            Chirality::CounterClockwise => Parity::Odd,
            Chirality::Clockwise => Parity::Even,
            // Square-planar has no 0D InChI spelling; `None` is nothing to say.
            _ => continue,
        };
        let Some(order) = mol.stereo_neighbor_order(idx) else {
            continue;
        };
        if order.len() != 4 {
            continue;
        }
        let mut neighbors = [0usize; 4];
        let mut sentinel_at = None;
        let mut ambiguous = false;
        for (slot, &entry) in order.iter().enumerate() {
            if entry == STEREO_H_SENTINEL {
                // Two vacant slots is not a stereocentre we can spell.
                if sentinel_at.is_some() {
                    ambiguous = true;
                    break;
                }
                sentinel_at = Some(slot);
                neighbors[slot] = idx.0 as usize;
            } else {
                neighbors[slot] = entry as usize;
            }
        }
        if ambiguous {
            continue;
        }
        let mut parity = base;
        if let Some(slot) = sentinel_at.filter(|slot| *slot != 0) {
            neighbors.swap(0, slot);
            parity = flip(parity);
        }
        // Distinct indices only — a malformed order would otherwise be
        // handed to the library as a stereocentre it cannot interpret.
        let mut seen = neighbors;
        seen.sort_unstable();
        if seen.windows(2).any(|w| w[0] == w[1]) {
            continue;
        }
        out.push(Stereo::Tetrahedral {
            center: idx.0 as usize,
            neighbors,
            parity,
        });
    }
    out
}

fn flip(p: Parity) -> Parity {
    match p {
        Parity::Odd => Parity::Even,
        Parity::Even => Parity::Odd,
        other => other,
    }
}

/// SMILES `/` `\` around a double bond → InChI 0D double-bond parity.
///
/// A directional bond written `p/q` puts `p` low and `q` high; `p\q` is the
/// reverse. For a double bond `a=b` with a directed substituent `x` on `a`
/// and `y` on `b`, the two are cis when their sides agree. InChI's
/// `{X, A, B, Y}` parity is even for the trans arrangement and odd for cis
/// (`inchi_api.h`, "double bond").
fn double_bond_stereo(mol: &CMol) -> Vec<Stereo> {
    let mut out = Vec::new();
    for (_bidx, bond) in mol.bonds() {
        if bond.order != CBond::Double {
            continue;
        }
        let a = bond.atom1;
        let b = bond.atom2;
        let Some((x, x_high)) = directed_substituent(mol, a, b) else {
            continue;
        };
        let Some((y, y_high)) = directed_substituent(mol, b, a) else {
            continue;
        };
        let parity = if x_high == y_high {
            Parity::Odd // cis
        } else {
            Parity::Even // trans
        };
        out.push(Stereo::DoubleBond {
            ends: [x.0 as usize, a.0 as usize, b.0 as usize, y.0 as usize],
            parity,
        });
    }
    out
}

/// The first neighbour of `end` (other than `across`) reached by a `/` or `\`
/// bond, and whether it sits on the high side of the double-bond axis.
fn directed_substituent(mol: &CMol, end: AtomIdx, across: AtomIdx) -> Option<(AtomIdx, bool)> {
    for (nbr, bidx) in mol.neighbors(end) {
        if nbr == across {
            continue;
        }
        let bond = mol.bond(bidx);
        let dir = match bond.order {
            CBond::Up | CBond::Down => bond.order,
            // After kekulization the direction is stashed off the bond order.
            _ => match mol.bond_direction(bidx) {
                Some(d @ (CBond::Up | CBond::Down)) => d,
                _ => continue,
            },
        };
        // `p/q`: p low, q high. `p\q`: p high, q low.
        let nbr_is_first = bond.atom1 == nbr;
        let high = match (dir, nbr_is_first) {
            (CBond::Up, true) => false,
            (CBond::Up, false) => true,
            (CBond::Down, true) => true,
            (CBond::Down, false) => false,
            _ => continue,
        };
        return Some((nbr, high));
    }
    None
}
