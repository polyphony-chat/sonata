use poem::error::ResponseError;
use poem::http::StatusCode;

/// Generic error type.
pub(crate) type StdError = Box<dyn std::error::Error + 'static>;
/// Generic result type.
pub(crate) type StdResult<T> = Result<T, StdError>;

#[derive(Debug, thiserror::Error)]
/// Error type for errors that concern the HTTP API. Implements [poem::error::ResponseError].
pub(crate) enum SonataApiError {
    #[error(transparent)]
    /// Generic error variant, supporting any type implementing [std::error::Error].
    StdError(StdError),
    /// A DB-related error.
    #[error(transparent)]
    DbError(SonataDbError),
}

#[derive(Debug, thiserror::Error)]
/// Error type for errors that concern interactions with the Database. Implements [poem::error::ResponseError].
pub(crate) enum SonataGatewayError {
    #[error(transparent)]
    /// Generic error variant, supporting any type implementing [std::error::Error].
    StdError(StdError),
}

#[derive(Debug, thiserror::Error)]
/// Error type for errors that concern the WebSocket Gateway.
pub(crate) enum SonataDbError {
    #[error(transparent)]
    /// Generic error variant, supporting any type implementing [std::error::Error].
    StdError(StdError),
    #[error(transparent)]
    /// An [sqlx::Error]
    Sqlx(sqlx::Error),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ResponseError for SonataApiError {
    fn status(&self) -> poem::http::StatusCode {
        match self {
            SonataApiError::StdError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SonataApiError::DbError(sonata_db_error) => sonata_db_error.status(),
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ResponseError for SonataDbError {
    fn status(&self) -> poem::http::StatusCode {
        match self {
            SonataDbError::StdError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SonataDbError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
