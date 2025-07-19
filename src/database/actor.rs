// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use sqlx::{query, query_as, types::Uuid};

use crate::{
    database::Database,
    errors::{Context, Errcode, Error},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorType {
    Local,
    Foreign,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Actor {
    pub unique_actor_identifier: Uuid,
    r#type: ActorType,
}

impl From<LocalActor> for Actor {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn from(value: LocalActor) -> Self {
        Self { unique_actor_identifier: value.unique_actor_identifier, r#type: ActorType::Local }
    }
}

#[derive(Debug, sqlx::Decode, sqlx::Encode, sqlx::FromRow)]
/// Actors from this home server. Does not include the `password_hash` column.
pub struct LocalActor {
    /// The unique actor identifer. Does not change, even if the `local_name`
    /// changes.
    pub unique_actor_identifier: Uuid,
    /// The "local name" part of the actor. See the polyproto specification for
    /// more information.
    pub local_name: String,
    /// Whether this actors' account is currently deactivated.
    pub is_deactivated: bool,
    /// Timestamp from when the actor has first registered on the server, or
    /// when this account has been created.
    pub joined_at_timestamp: chrono::NaiveDateTime,
}

impl LocalActor {
    /// Tries to find an actor from the [Database] where `local_name` is equal
    /// to `name`, returning `None`, if such an actor does not exist.
    ///
    /// ## Errors
    ///
    /// Will error on Database connection issues and on other errors with the
    /// database, all of which are not in scope for this function to handle.
    pub async fn by_local_name(db: &Database, name: &str) -> Result<Option<LocalActor>, Error> {
        Ok(query!(
            "
            SELECT uaid, local_name, deactivated, joined
            FROM local_actors
            WHERE local_name = $1
            LIMIT 1",
            name
        )
        .fetch_optional(&db.pool)
        .await?
        .map(|record| LocalActor {
            unique_actor_identifier: record.uaid,
            local_name: record.local_name,
            is_deactivated: record.deactivated,
            joined_at_timestamp: record.joined,
        }))
    }

    /// Returns the `password_hash` of an actor from the [Database] where
    /// `local_name` is equal to `name`, returning `None`, if such an actor
    /// does not exist.
    ///
    /// ## Errors
    ///
    /// Will error on Database connection issues and on other errors with the
    /// database, all of which are not in scope for this function to handle.
    pub async fn get_password_hash(db: &Database, name: &str) -> Result<Option<String>, Error> {
        Ok(query!(
            "
            SELECT password_hash
            FROM local_actors
            WHERE local_name = $1
            LIMIT 1",
            name
        )
        .fetch_optional(&db.pool)
        .await?
        .map(|record| record.password_hash))
    }

    /// Create a new [LocalActor] in the `local_actors` table of the [Database].
    /// Before creating, checks, if a user specified by `local_name` already
    /// exists in the table, returning an [Errcode::Duplicate]-type error, if
    /// this is the case.
    ///
    /// ## Errors
    ///
    /// Other than the above, this method will error, if something is wrong with
    /// the Database or Database connection.
    pub async fn create(
        db: &Database,
        local_name: &str,
        password_hash: &str,
    ) -> Result<LocalActor, Error> {
        if LocalActor::by_local_name(db, local_name).await?.is_some() {
            Err(Error::new(
                Errcode::Duplicate,
                Some(Context::new(Some("local_name"), Some(local_name), None, None)),
            ))
        } else {
            let uaid = query!("INSERT INTO actors (type) VALUES ('local') RETURNING uaid")
                .fetch_one(&db.pool)
                .await?;
            Ok(query_as!(
			LocalActor,
			"INSERT INTO local_actors (uaid, local_name, password_hash) VALUES ($1, $2, $3) RETURNING uaid AS unique_actor_identifier, local_name, deactivated AS is_deactivated, joined AS joined_at_timestamp",
			uaid.uaid,
			local_name,
			password_hash
		).fetch_one(&db.pool).await?)
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{Pool, Postgres};

    use super::*;

    #[sqlx::test(fixtures("../../fixtures/local_actor_tests.sql"))]
    async fn test_by_local_name_finds_existing_user(pool: Pool<Postgres>) {
        let db = Database { pool };

        let result = LocalActor::by_local_name(&db, "alice").await.unwrap();
        assert!(result.is_some());

        let actor = result.unwrap();
        assert_eq!(actor.local_name, "alice");
        assert_eq!(
            actor.unique_actor_identifier.to_string(),
            "00000000-0000-0000-0000-000000000001"
        );
        assert!(!actor.is_deactivated);
    }

    #[sqlx::test(fixtures("../../fixtures/local_actor_tests.sql"))]
    async fn test_by_local_name_finds_deactivated_user(pool: Pool<Postgres>) {
        let db = Database { pool };

        let result = LocalActor::by_local_name(&db, "deactivated_user").await.unwrap();
        assert!(result.is_some());

        let actor = result.unwrap();
        assert_eq!(actor.local_name, "deactivated_user");
        assert_eq!(
            actor.unique_actor_identifier.to_string(),
            "00000000-0000-0000-0000-000000000004"
        );
        assert!(actor.is_deactivated);
    }

    #[sqlx::test(fixtures("../../fixtures/local_actor_tests.sql"))]
    async fn test_by_local_name_finds_user_with_special_characters(pool: Pool<Postgres>) {
        let db = Database { pool };

        let result = LocalActor::by_local_name(&db, "user_with_underscores").await.unwrap();
        assert!(result.is_some());

        let actor = result.unwrap();
        assert_eq!(actor.local_name, "user_with_underscores");
        assert_eq!(
            actor.unique_actor_identifier.to_string(),
            "00000000-0000-0000-0000-000000000005"
        );
        assert!(!actor.is_deactivated);
    }

    #[sqlx::test(fixtures("../../fixtures/local_actor_tests.sql"))]
    async fn test_by_local_name_returns_none_for_nonexistent_user(pool: Pool<Postgres>) {
        let db = Database { pool };

        let result = LocalActor::by_local_name(&db, "nonexistent_user").await.unwrap();
        assert!(result.is_none());
    }

    #[sqlx::test(fixtures("../../fixtures/local_actor_tests.sql"))]
    async fn test_by_local_name_returns_none_for_empty_string(pool: Pool<Postgres>) {
        let db = Database { pool };

        let result = LocalActor::by_local_name(&db, "").await.unwrap();
        assert!(result.is_none());
    }

    #[sqlx::test(fixtures("../../fixtures/local_actor_tests.sql"))]
    async fn test_by_local_name_is_case_sensitive(pool: Pool<Postgres>) {
        let db = Database { pool };

        // Should find exact match
        let result_exact = LocalActor::by_local_name(&db, "alice").await.unwrap();
        assert!(result_exact.is_some());

        // Should not find case-different match
        let result_upper = LocalActor::by_local_name(&db, "ALICE").await.unwrap();
        assert!(result_upper.is_none());

        let result_mixed = LocalActor::by_local_name(&db, "Alice").await.unwrap();
        assert!(result_mixed.is_none());
    }

    #[sqlx::test(fixtures("../../fixtures/local_actor_tests.sql"))]
    async fn test_create_new_user_success(pool: Pool<Postgres>) {
        let db = Database { pool };

        let result = LocalActor::create(&db, "new_user", "hash").await;
        assert!(result.is_ok());

        let actor = result.unwrap();
        assert_eq!(actor.local_name, "new_user");
        assert!(!actor.is_deactivated);
        assert!(actor.unique_actor_identifier != sqlx::types::Uuid::nil());

        // Verify the user was actually created in the database
        let found = LocalActor::by_local_name(&db, "new_user").await.unwrap();
        assert!(found.is_some());
        let found_actor = found.unwrap();
        assert_eq!(found_actor.unique_actor_identifier, actor.unique_actor_identifier);
    }

    #[sqlx::test(fixtures("../../fixtures/local_actor_tests.sql"))]
    async fn test_create_duplicate_user_returns_error(pool: Pool<Postgres>) {
        let db = Database { pool };

        let result = LocalActor::create(&db, "alice", "hash").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            error => {
                assert_eq!(error.code, Errcode::Duplicate);
                assert!(error.context.is_some());
                let context = error.context.unwrap();
                assert_eq!(context.field_name, "local_name");
                assert_eq!(context.found, "alice");
            }
        }
    }

    #[sqlx::test(fixtures("../../fixtures/local_actor_tests.sql"))]
    async fn test_create_duplicate_deactivated_user_returns_error(pool: Pool<Postgres>) {
        let db = Database { pool };

        let result = LocalActor::create(&db, "deactivated_user", "hash").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            error => {
                assert_eq!(error.code, Errcode::Duplicate);
                assert!(error.context.is_some());
                let context = error.context.unwrap();
                assert_eq!(context.field_name, "local_name");
                assert_eq!(context.found, "deactivated_user");
            }
        }
    }

    #[sqlx::test(fixtures("../../fixtures/local_actor_tests.sql"))]
    async fn test_create_user_with_special_characters(pool: Pool<Postgres>) {
        let db = Database { pool };

        let result = LocalActor::create(&db, "user.with-special_chars", "hash").await;
        assert!(result.is_ok());

        let actor = result.unwrap();
        assert_eq!(actor.local_name, "user.with-special_chars");
        assert!(!actor.is_deactivated);

        let found = LocalActor::by_local_name(&db, "user.with-special_chars").await.unwrap();
        assert!(found.is_some());
    }

    #[sqlx::test(fixtures("../../fixtures/local_actor_tests.sql"))]
    async fn test_create_user_with_empty_name(pool: Pool<Postgres>) {
        let db = Database { pool };

        let result = LocalActor::create(&db, "", "hash").await;
        assert!(result.is_ok());

        let actor = result.unwrap();
        assert_eq!(actor.local_name, "");
        assert!(!actor.is_deactivated);

        let found = LocalActor::by_local_name(&db, "").await.unwrap();
        assert!(found.is_some());
    }

    #[sqlx::test(fixtures("../../fixtures/local_actor_tests.sql"))]
    async fn test_create_multiple_users_have_different_uuids(pool: Pool<Postgres>) {
        let db = Database { pool };

        let user1 = LocalActor::create(&db, "user1", "hash").await.unwrap();
        let user2 = LocalActor::create(&db, "user2", "hash").await.unwrap();
        let user3 = LocalActor::create(&db, "user3", "hash").await.unwrap();

        assert_ne!(user1.unique_actor_identifier, user2.unique_actor_identifier);
        assert_ne!(user1.unique_actor_identifier, user3.unique_actor_identifier);
        assert_ne!(user2.unique_actor_identifier, user3.unique_actor_identifier);

        assert_ne!(user1.local_name, user2.local_name);
        assert_ne!(user1.local_name, user3.local_name);
        assert_ne!(user2.local_name, user3.local_name);
    }

    #[sqlx::test(fixtures("../../fixtures/local_actor_tests.sql"))]
    async fn test_create_user_sets_joined_timestamp(pool: Pool<Postgres>) {
        let db = Database { pool };

        let before_create = chrono::Utc::now().naive_utc();
        let actor = LocalActor::create(&db, "timestamped_user", "hash").await.unwrap();
        let after_create = chrono::Utc::now().naive_utc();

        assert!(actor.joined_at_timestamp >= before_create);
        assert!(actor.joined_at_timestamp <= after_create);
    }
}
