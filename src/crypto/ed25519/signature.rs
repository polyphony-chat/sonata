use std::str::FromStr;

use polyproto::{
    der::asn1::BitString,
    signature::Signature as SignatureTrait,
    spki::{AlgorithmIdentifierOwned, ObjectIdentifier, SignatureBitStringEncoding},
};

/// The official IANA Object Identifier (OID) for the Ed25519 signature
/// algorithm
const IANA_OID_ED25519: &str = "1.3.101.112";

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct DigitalSignature {
    pub(super) signature: ed25519_dalek::Signature,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl SignatureBitStringEncoding for DigitalSignature {
    fn to_bitstring(&self) -> polyproto::der::Result<BitString> {
        BitString::from_bytes(&self.as_signature().to_bytes())
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl std::fmt::Display for DigitalSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.signature.to_string())
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl SignatureTrait for DigitalSignature {
    type Signature = ed25519_dalek::Signature;

    fn as_signature(&self) -> &Self::Signature {
        &self.signature
    }

    fn algorithm_identifier() -> AlgorithmIdentifierOwned {
        #[allow(clippy::unwrap_used)]
        AlgorithmIdentifierOwned {
            oid: ObjectIdentifier::from_str(IANA_OID_ED25519).unwrap(),
            parameters: None,
        }
    }

    fn from_bytes(signature: &[u8]) -> Self {
        let mut signature_vec = signature.to_vec();
        signature_vec.resize(64, 0);
        let signature_array: [u8; 64] = {
            let mut array = [0; 64];
            array.copy_from_slice(&signature_vec[..]);
            array
        };
        Self { signature: ed25519_dalek::Signature::from_bytes(&signature_array) }
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.signature.to_vec()
    }
}
