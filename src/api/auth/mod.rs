use poem::Route;

mod login;
mod register;

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn setup_routes() -> Route {
	Route::new()
}
