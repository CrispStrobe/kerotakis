//! DATA-004: Load a deterministic runtime pack.
//!
//! The pack format is: `KREG` (4 bytes) + version (u32 LE) + SHA-256 (32 bytes)
//! + compact JSON payload. The content hash covers only the payload, so a
//! correct pack is self-verifying. JSON is used because the schema's tagged
//! enums (Uncertainty, Method, ModelSubject) are not compatible with binary
//! formats that lack self-describing type information.

use crate::RegistryDocument;

pub const PACK_MAGIC: &[u8; 4] = b"KREG";
pub const PACK_VERSION: u32 = 1;

const HEADER_LEN: usize = 4 + 4 + 32; // magic + version + SHA-256

#[derive(Debug)]
pub enum PackError {
    TooShort,
    BadMagic,
    UnsupportedVersion(u32),
    HashMismatch,
    DeserializeFailed(serde_json::Error),
}

impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort => write!(f, "pack file too short"),
            Self::BadMagic => write!(f, "not a KREG pack file"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported pack version {v}"),
            Self::HashMismatch => write!(f, "pack content hash mismatch"),
            Self::DeserializeFailed(e) => write!(f, "pack deserialization failed: {e}"),
        }
    }
}

impl std::error::Error for PackError {}

/// Serialize a registry document to the pack payload format (compact JSON).
pub fn serialize_pack_payload(doc: &RegistryDocument) -> Vec<u8> {
    serde_json::to_vec(doc).expect("JSON serialization should not fail")
}

/// Load a registry document from a compiled `.pack` byte slice.
///
/// Verifies the magic, version, and content hash before deserializing.
pub fn load_pack(data: &[u8]) -> Result<RegistryDocument, PackError> {
    if data.len() < HEADER_LEN {
        return Err(PackError::TooShort);
    }
    if &data[..4] != PACK_MAGIC {
        return Err(PackError::BadMagic);
    }
    let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
    if version != PACK_VERSION {
        return Err(PackError::UnsupportedVersion(version));
    }
    let expected_hash = &data[8..40];
    let payload = &data[HEADER_LEN..];

    use sha2::{Digest, Sha256};
    let actual_hash = Sha256::digest(payload);
    if actual_hash.as_slice() != expected_hash {
        return Err(PackError::HashMismatch);
    }

    serde_json::from_slice(payload).map_err(PackError::DeserializeFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_pack(doc: &RegistryDocument) -> Vec<u8> {
        let payload = serialize_pack_payload(doc);
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(&payload);
        let mut pack = Vec::new();
        pack.extend_from_slice(PACK_MAGIC);
        pack.extend_from_slice(&PACK_VERSION.to_le_bytes());
        pack.extend_from_slice(&hash);
        pack.extend_from_slice(&payload);
        pack
    }

    #[test]
    fn round_trip_empty_document() {
        let doc = RegistryDocument::empty();
        let pack = build_pack(&doc);
        let loaded = load_pack(&pack).unwrap();
        assert_eq!(loaded, doc);
    }

    #[test]
    fn bad_magic_rejected() {
        let mut pack = vec![0u8; 100];
        pack[..4].copy_from_slice(b"NOPE");
        assert!(matches!(load_pack(&pack), Err(PackError::BadMagic)));
    }

    #[test]
    fn hash_mismatch_rejected() {
        let doc = RegistryDocument::empty();
        let pack = build_pack(&doc);
        let mut corrupted = pack;
        let last = corrupted.len() - 1;
        corrupted[last] ^= 0xff;
        assert!(matches!(
            load_pack(&corrupted),
            Err(PackError::HashMismatch)
        ));
    }
}
