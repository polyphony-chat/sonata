pub(crate) mod private_key;
pub(crate) mod public_key;
pub(crate) mod signature;

use argon2::password_hash::rand_core;
use ed25519_dalek::SigningKey;
pub(crate) use private_key::*;
pub(crate) use public_key::*;
pub(crate) use signature::*;

/// Generate a `ed25519` keypair using an [rand_core::OsRng].
pub(crate) fn generate_keypair() -> (DigitalPrivateKey, DigitalPublicKey) {
    let signing_key = SigningKey::generate(&mut rand_core::OsRng);
    let verifying_key = signing_key.verifying_key();
    let dpuk = DigitalPublicKey { key: verifying_key };
    let dppk = DigitalPrivateKey { key: signing_key, pubkey: dpuk.clone() };
    (dppk, dpuk)
}
