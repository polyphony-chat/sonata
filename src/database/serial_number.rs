use bigdecimal::num_bigint::BigUint;
use rand::TryRngCore;
use sqlx::types::BigDecimal;
use sqlx::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
/// A serial number for a [polyproto::certs::IdCert].
pub struct SerialNumber(BigDecimal);

impl SerialNumber {
    /// From a [ThreadRng], get 20 octets (160 bits) of entropy and construct a serial number
    /// out of it.
    ///
    /// ## Errors
    ///
    /// Will error, if the [ThreadRng] fails to generate randomness. Depending on the implementation
    /// of `ThreadRng`, this method may cause a panic in these cases.
    pub fn try_generate_random(
        rng: &mut rand::rngs::ThreadRng,
    ) -> Result<Self, crate::errors::StdError> {
        let mut buf = [0u8; 20];
        rng.try_fill_bytes(&mut buf)?;
        Ok(Self(BigDecimal::from_biguint(BigUint::from_bytes_be(&buf), 0)))
    }

    /// Derive [Self] from 20 bytes. These bytes should be sourced from a CSPRNG or another information
    /// source with high entropy.
    pub fn new_from_bytes(bytes: [u8; 20]) -> Self {
        Self(BigDecimal::from_biguint(BigUint::from_bytes_be(&bytes), 0))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use rand::rng;

    #[test]
    fn generate_random_serials() {
        let mut rng = rng();
        for _ in 0..5000 {
            super::SerialNumber::try_generate_random(&mut rng).unwrap();
        }
        for _ in 0..3 {
            dbg!(super::SerialNumber::try_generate_random(&mut rng).unwrap());
        }
    }

    #[test]
    fn from_bytes() {
        let bytes = [1u8; 20];
        dbg!(super::SerialNumber::new_from_bytes(bytes));
    }
}
