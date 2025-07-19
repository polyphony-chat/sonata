// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use poem::{Endpoint, Middleware, http::StatusCode};

use crate::database::tokens::{TokenStore, hash_auth_token};

/// Authentication middleware, implementing [Endpoint] via
/// [AuthenticationMiddlewareImpl]
pub struct AuthenticationMiddleware;

#[cfg_attr(coverage_nightly, coverage(off))]
impl<E: Endpoint> Middleware<E> for AuthenticationMiddleware {
    type Output = AuthenticationMiddlewareImpl<E>;

    fn transform(&self, ep: E) -> Self::Output {
        Self::Output { ep }
    }
}

/// Struct for middleware functionality implementation
pub struct AuthenticationMiddlewareImpl<E> {
    /// I copied this from symfonia and don't know exactly what this is, but we
    /// need it, so...
    ep: E,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<E: Endpoint> Endpoint for AuthenticationMiddlewareImpl<E> {
    type Output = E::Output;

    async fn call(&self, mut req: poem::Request) -> poem::Result<Self::Output> {
        let auth = req
            .header("Authorization")
            .ok_or(poem::error::Error::from_status(StatusCode::UNAUTHORIZED))?;

        let token_store = req.data::<TokenStore>().unwrap();
        let hashed_user_token = hash_auth_token(auth);
        // We first get the serial_number of the cert that this token is associated
        // with...
        let user_serial_number = token_store
            .get_token_serial_number(&hashed_user_token)
            .await
            .map_err(|_| poem::error::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?
            .ok_or(poem::error::Error::from_status(StatusCode::UNAUTHORIZED))?;
        // ...then we check, if this token is actually valid
        let valid_token_in_db_for_user = token_store
            .get_token_userid(&user_serial_number)
            .await
            .map_err(|_| poem::error::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?
            .ok_or(poem::error::Error::from_status(StatusCode::UNAUTHORIZED))?;
        if valid_token_in_db_for_user.token == hashed_user_token.into() {
            req.set_data(valid_token_in_db_for_user);
        } else {
            return Err(poem::error::Error::from_status(StatusCode::UNAUTHORIZED));
        }

        self.ep.call(req).await
    }
}
