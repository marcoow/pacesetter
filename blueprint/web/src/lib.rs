use anyhow::Context;
use axum::{serve, http::StatusCode};
use {{crate_name}}_config::{Config, get_env, load_config};
use std::fmt::{Debug, Display};
use tokio::net::TcpListener;
use tracing::info;
use tracing_panic::panic_hook;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub mod controllers;
pub mod middlewares;
#[doc(hidden)]
pub mod routes;
pub mod state;

#[doc(hidden)]
pub async fn run() -> anyhow::Result<()> {
    let env = get_env().context("Cannot get environment!")?;
    let config: Config = load_config(&env).context("Cannot load config!")?;

    let app_state = state::app_state(config.clone()).await;
    let app = routes::routes(app_state);

    let addr = config.server.addr();
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on {}", &addr);
    serve(listener, app.into_make_service()).await?;

    Ok(())
}

/// Helper function to create an internal error response while
/// taking care to log the error itself.
///
/// This is useful to avoid duplication in web endpoints – in the case of unrecoverable errors,
/// there's only really two things to do anyway, which you'll want to do in every such case:
///
/// 1. create an error-level tracing event
/// 2. respond with an axum::http::StatusCode::INTERNAL_SERVER_ERROR status code
///
/// Example
/// ```rust
/// pub async fn read_all(
///     State(app_state): State<AppState>,
/// ) -> Result<Json<Vec<Task>>, StatusCode> {
///     let tasks = tasks::load_all(&app_state.db_pool)
///         .await
///         .map_err(internal_error)?;
///
///     Ok(Json(tasks))
/// }
/// ```
pub fn internal_error<E>(e: E) -> StatusCode
where
    // Some "error-like" types (e.g. `anyhow::Error`) don't implement the error trait, therefore
    // we "downgrade" to simply requiring `Debug` and `Display`, the traits
    // we actually need for logging purposes.
    E: Debug + Display,
{
    tracing::error!(err.msg = %e, err.details = ?e, "Internal server error");
    // We don't want to leak internal implementation details to the client
    // via the error response, so we just return an opaque internal server.
    StatusCode::INTERNAL_SERVER_ERROR
}

/// Initializes tracing.
///
/// This function
///
/// * registers a [`tracing_subscriber::fmt::Subscriber`]
/// * registers a [`tracing_panic::panic_hook`]
///
/// The function respects the `RUST_LOG` if set or defaults to filtering spans and events with level [`tracing_subscriber::filter::LevelFilter::INFO`] and higher.
#[doc(hidden)]
pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    std::panic::set_hook(Box::new(panic_hook));
}
