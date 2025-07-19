use ed25519_dalek::{SigningKey, ed25519::signature::Signer};
use polyproto::key::PrivateKey;

use crate::crypto::ed25519::{DigitalPublicKey, DigitalSignature};

#[derive(PartialEq, Eq, Clone, Debug)]
/// `ed25519` private key, also containing information about the corresponding
/// public key.
pub(crate) struct DigitalPrivateKey {
	/// The private key
	pub(crate) key: SigningKey,
	/// The corresponding public key
	pub(crate) pubkey: DigitalPublicKey,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl PrivateKey<DigitalSignature> for DigitalPrivateKey {
	type PublicKey = DigitalPublicKey;

	fn pubkey(&self) -> &Self::PublicKey {
		&self.pubkey
	}

	fn sign(&self, data: &[u8]) -> DigitalSignature {
		let signature = self.key.sign(data);
		DigitalSignature { signature }
	}
}
