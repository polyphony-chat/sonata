use ed25519_dalek::VerifyingKey;
use polyproto::{der::asn1::BitString, key::PublicKey, signature::Signature};

use crate::crypto::ed25519::DigitalSignature;

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct DigitalPublicKey {
	key: VerifyingKey,
}

impl PublicKey<DigitalSignature> for DigitalPublicKey {
	fn verify_signature(
		&self,
		signature: &DigitalSignature,
		data: &[u8],
	) -> Result<(), polyproto::errors::PublicKeyError> {
		match self.key.verify_strict(data, signature.as_signature()) {
			Ok(_) => Ok(()),
			Err(_) => Err(polyproto::errors::composite::PublicKeyError::BadSignature),
		}
	}

	fn public_key_info(&self) -> polyproto::certs::PublicKeyInfo {
		// Get the key as bytes
		let key_bytes = self.key.to_bytes();

		// Create a 32-byte array, copying bytes into it
		let mut key_array = [0u8; 32];
		for (dest, &src) in key_array.iter_mut().zip(key_bytes.iter()) {
			*dest = src;
		}

		#[allow(clippy::unwrap_used)]
		polyproto::certs::PublicKeyInfo {
			algorithm: DigitalSignature::algorithm_identifier(),
			// Unwrap is okay. As I understand it, BitString::from_bytes will only panic if the
			// length of the bytes is super long. We know, that that is not the case.
			public_key_bitstring: BitString::from_bytes(&key_array).unwrap(),
		}
	}

	fn try_from_public_key_info(
		public_key_info: polyproto::certs::PublicKeyInfo,
	) -> Result<Self, polyproto::errors::CertificateConversionError> {
		let mut key_vec = public_key_info.public_key_bitstring.raw_bytes().to_vec();
		key_vec.resize(32, 0);
		let signature_array: [u8; 32] = {
			let mut array = [0; 32];
			array.copy_from_slice(&key_vec[..]);
			array
		};
		Ok(Self {
			key: VerifyingKey::from_bytes(&signature_array).map_err(|e| {
				polyproto::errors::CertificateConversionError::InvalidInput(
					polyproto::errors::InvalidInput::Malformed(e.to_string()),
				)
			})?,
		})
	}
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
	use std::thread;

	use rand::RngCore;

	use super::*;

	#[tokio::test(flavor = "multi_thread")]
	async fn test_bitstring_from_32_random_bytes() {
		let num_cores = thread::available_parallelism().unwrap().get();
		let handles: Vec<_> = (0..num_cores)
			.map(|_| {
				tokio::task::spawn(async move {
					let mut rng = rand::rng();
					for _ in 0..100_000 {
						let mut random_bytes = [0u8; 32];
						rng.fill_bytes(&mut random_bytes);
						// This should never panic for any 32-byte array
						let _bitstring = BitString::from_bytes(&random_bytes).unwrap();
					}
				})
			})
			.collect();
		for handle in handles {
			handle.await.unwrap();
		}
	}
}
