use poem::{Route, post};

/// The login endpoint
mod login;
/// The register endpoint
mod register;

#[cfg_attr(coverage_nightly, coverage(off))]
/// Route handler for the auth module
pub(super) fn setup_routes() -> Route {
	Route::new().at("/register", post(register::register))
}
