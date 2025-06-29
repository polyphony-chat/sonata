// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use poem::http::StatusCode;
use poem::{Endpoint, Middleware};
use sqlx::PgPool;

pub struct AuthenticationMiddleware;

impl<E: Endpoint> Middleware<E> for AuthenticationMiddleware {
    type Output = AuthenticationMiddlewareImpl<E>;

    fn transform(&self, ep: E) -> Self::Output {
        Self::Output { ep }
    }
}

pub struct AuthenticationMiddlewareImpl<E> {
    ep: E,
}

impl<E: Endpoint> Endpoint for AuthenticationMiddlewareImpl<E> {
    type Output = E::Output;

    async fn call(&self, req: poem::Request) -> poem::Result<Self::Output> {
        let auth = req
            .header("Authorization")
            .ok_or(poem::error::Error::from_status(StatusCode::UNAUTHORIZED))?;

        let db = req.data::<PgPool>().unwrap();

        self.ep.call(req).await
    }
}
