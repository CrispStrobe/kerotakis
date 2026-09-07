//! Reviewed reference-temperature ligand slice shared by aqueous databases.
//!
//! Native mass action, charge balance and activity corrections compute the
//! distribution for arbitrary totals. These are not scenario-specific yields.
//! See data/thermo/usbm-ic9429-complexes.dat and its provenance review. The
//! source supplies 25 C constants only: callers must disclose that temperature
//! derivatives, reaction heat, rates and complex spectra remain unknown.

pub const SOURCE_ID: &str = "usbm-ic9429-complexes-25c";
pub const REFERENCE_TEMPERATURE_K: f64 = 298.15;
pub const EXTENSION: &[u8] = include_bytes!("../../../data/thermo/usbm-ic9429-complexes.dat");

/// Translate exact native pseudo-component names to physical species names.
///
/// Thiocyanate is an independently conserved ligand in the native database,
/// not a chemical element. Public reports must expose its actual formula so
/// downstream composition and spectral-coverage checks can identify it.
pub fn native_to_physical(name: &str) -> &str {
    match name {
        "Thiocyanate-" => "SCN-",
        "Fe(Thiocyanate)+2" => "Fe(SCN)+2",
        "Fe(Thiocyanate)2+" => "Fe(SCN)2+",
        _ => name,
    }
}

/// Native or physical names whose constants belong to this reviewed slice.
pub fn uses_reference_complexes(name: &str) -> bool {
    matches!(
        native_to_physical(name),
        "CuNH3+2" | "Cu(NH3)2+2" | "Cu(NH3)3+2" | "Cu(NH3)4+2" | "Fe(SCN)+2" | "Fe(SCN)2+"
    )
}

/// Append before the final END; bytes after END are not read by PHREEQC.
pub fn append_to(database: &[u8]) -> Vec<u8> {
    let insert_at = super::databases::find_last_end(database).unwrap_or(database.len());
    let mut bytes = Vec::with_capacity(database.len() + EXTENSION.len() + 1);
    bytes.extend_from_slice(&database[..insert_at]);
    bytes.push(b'\n');
    bytes.extend_from_slice(EXTENSION);
    bytes.extend_from_slice(&database[insert_at..]);
    bytes
}

/// True only at the measured reference temperature (numerical tolerance, not
/// an invented experimental validity interval).
pub fn temperature_covered(temperature_k: f64) -> bool {
    temperature_k.is_finite() && (temperature_k - REFERENCE_TEMPERATURE_K).abs() < 1e-8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_aliases_are_exact_and_preserve_reference_coverage() {
        for (native, physical) in [
            ("Thiocyanate-", "SCN-"),
            ("Fe(Thiocyanate)+2", "Fe(SCN)+2"),
            ("Fe(Thiocyanate)2+", "Fe(SCN)2+"),
        ] {
            assert_eq!(native_to_physical(native), physical);
            assert_eq!(native_to_physical(physical), physical);
            assert_eq!(
                uses_reference_complexes(native),
                uses_reference_complexes(physical)
            );
        }
        for name in ["Fe(SCN)+2", "Fe(SCN)2+", "Cu(NH3)4+2"] {
            assert!(uses_reference_complexes(name));
        }
        for name in [
            "Thiocyanate",
            "SCN-",
            "Fe(Thiocyanate)3",
            "XThiocyanate-",
            "Cu+2",
        ] {
            assert_eq!(native_to_physical(name), name);
            assert!(!uses_reference_complexes(name));
        }
    }

    #[test]
    fn extension_precedes_end_and_does_not_claim_temperature_derivatives() {
        for ending in ["END\n", "END", "  END\r\n"] {
            let source = format!("SOLUTION_SPECIES\nH+ = H+\n log_k 0\n{ending}");
            let text = String::from_utf8(append_to(source.as_bytes())).unwrap();
            assert!(text.find("Cu(NH3)4+2").unwrap() < text.rfind("END").unwrap());
            assert!(!text.contains("delta_h"));
        }
        assert!(temperature_covered(298.15));
        for t in [298.0, 300.0, f64::NAN, f64::INFINITY] {
            assert!(!temperature_covered(t));
        }
    }
}
