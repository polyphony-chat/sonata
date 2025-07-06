use bigdecimal::num_bigint::BigUint;
use rand::TryRngCore;
use sqlx::types::BigDecimal;
use sqlx::{Decode, Encode};

// TODO: This could be in polyproto instead

#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, Hash)]
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
        Self::normalize_first_byte(&mut buf);
        Ok(Self(BigDecimal::from_biguint(BigUint::from_bytes_be(&buf), 0)))
    }

    /// Derive [Self] from 20 bytes. These bytes should be sourced from a CSPRNG or another information
    /// source with high entropy.
    ///
    /// ## Important
    ///
    /// Serial numbers with an MSB which is larger than 127 in decimal are likely not considered valid
    /// in all x509 implementations. This is because RFC 5280 is ambiguous about whether numbers
    /// which result in 21 octets of ASN.1 Uint are considered valid. Sonata does not consider
    /// serial numbers with an MSB of > 127 valid for encoding, but it does consider them valid for
    /// decoding. It follows the `x509-cert` crate in this regard.
    pub fn new_from_bytes(bytes: [u8; 20]) -> Self {
        Self(BigDecimal::from_biguint(BigUint::from_bytes_be(&bytes), 0))
    }

    /// ASN.1 and its consequences have been a disaster for the human race.
    ///
    /// ## Preamble
    ///
    /// This is just my understanding of the underlying problem. Feel free to correct me, if I am
    /// wrong.
    ///
    /// ## Situation
    ///
    /// The x509-cert crate says:
    ///
    /// ```txt
    /// The user might give us a 20 byte unsigned integer with a high MSB,
    /// which we'd then encode with 21 octets to preserve the sign bit.
    /// RFC 5280 is ambiguous about whether this is valid, so we limit
    /// `SerialNumber` *encodings* to 20 bytes or fewer while permitting
    /// `SerialNumber` *decodings* to have up to 21 bytes below.
    /// ```
    ///
    /// This means that the first octet of a 20-octet array may not be larger than 128 in decimal,
    /// if the resulting [SerialNumber] is to be a valid x509 serial number regardless of how
    /// you may interpret the spec.
    ///
    /// ## The (hacky) solution
    ///
    /// To adjust for this, we simply take the modulo 128 of the first octet in the array. This way,
    /// we have a lossy projection from the CSPRNG generated first-maybe-valid-octet to a
    /// definitely-valid first octet.
    ///
    /// ## Is this cryptographically safe?
    ///
    /// I don't know. :3
    ///
    /// But I suspect that it is sufficient for this purpose, because serial numbers are not meant
    /// to be cryptographically safe on their own; they should likely just be random *enough*.
    /// In my head, the worst case is that instead of 160 bits of entropy, there will still be 159
    /// bits of entropy left, and that is still a lot of entropy.
    fn normalize_first_byte(buf: &mut [u8; 20]) {
        buf[0] %= 128;
    }
}

impl From<polyproto::types::x509_cert::SerialNumber> for SerialNumber {
    fn from(value: polyproto::types::x509_cert::SerialNumber) -> Self {
        Self(BigDecimal::from_biguint(BigUint::from_bytes_be(value.as_bytes()), 0))
    }
}

impl From<SerialNumber> for polyproto::types::x509_cert::SerialNumber {
    fn from(value: SerialNumber) -> Self {
        Self::from_bytes_be(value.0.into_bigint_and_scale().0.to_bytes_be().1.as_slice()).unwrap()
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

    #[test]
    fn as_bytes_polyproto_eq_from_be_bytes() {
        let serial_number = super::SerialNumber::new_from_bytes([0; 20]);
        let p2_serial_number =
            polyproto::types::x509_cert::SerialNumber::from(serial_number.clone());
        let converted_back = super::SerialNumber::from(p2_serial_number);
        assert_eq!(converted_back, serial_number);
        for _ in 0..5000 {
            let serial_number = super::SerialNumber::try_generate_random(&mut rng()).unwrap();
            let p2_serial_number =
                polyproto::types::x509_cert::SerialNumber::from(serial_number.clone());
            let converted_back = super::SerialNumber::from(p2_serial_number);
            assert_eq!(converted_back, serial_number)
        }
    }
}
