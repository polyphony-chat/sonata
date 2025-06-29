use argon2::Argon2;
use argon2::PasswordHasher;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;

use crate::StdResult;
use crate::database::Database;

/// DOCUMENTME
pub fn hash_auth_token(auth_token: &str) -> StdResult<String> {
    let argon_hasher = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    Ok(argon_hasher
        .hash_password(auth_token.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string())
}

pub(crate) async fn valid_token_in_db(db: Database, token: &str) -> crate::errors::SonataDbError {
    todo!()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use argon2::{PasswordHash, PasswordVerifier};

    use super::*;

    #[test]
    fn eq_tokens() {
        let token = "hi!ilovetheworld";
        let hash = hash_auth_token(token).unwrap();
        let pw_hash = PasswordHash::new(&hash).unwrap();
        Argon2::default().verify_password(token.as_bytes(), &pw_hash).unwrap();
    }
}
