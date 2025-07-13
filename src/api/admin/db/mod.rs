// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use rand::{Rng, distr::Alphanumeric};
use sqlx::{query_as, types::Uuid};

use crate::database::{Database, Invite};

/// Create an invite.
pub(super) async fn create_invite(
	owner: Option<&Uuid>,
	code: Option<&str>,
	uses_max: i32,
	db: &Database,
) -> Result<Invite, crate::errors::SonataDbError> {
	let code = {
		if let Some(code) = code {
			code
		} else {
			&rand::rng().sample_iter(&Alphanumeric).take(16).map(char::from).collect::<String>()
		}
	};
	Ok(query_as!(
		Invite,
		"INSERT INTO invite_links
        (
            invite_link_owner,
            usages_current, usages_maximum,
            invite,
            invalid
        )
        VALUES ($1, 0, $2, $3, $4)
        RETURNING
            invite_link_owner,
            usages_current,
            usages_maximum,
            invite AS invite_code,
            invalid",
		owner,
		uses_max,
		code,
		false
	)
	.fetch_one(&db.pool)
	.await?)
}
