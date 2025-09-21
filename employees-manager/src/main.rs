use std::path::Path;
use std::sync::Arc;

use axum::response::Html;
use axum::routing::get_service;
use axum::{routing::get, Router};
use employees_manager::enums::FrontendMode;
use employees_manager::middleware::add_mongodb_transaction_middleware;
use employees_manager::service::db::DatabaseService;
use employees_manager::service::environment::EnvironmentVariables;
use employees_manager::AppState;
use employees_manager::{
    middleware::{add_cors_middleware, add_logging_middleware},
    router::{ADMIN_ROUTER, WEB_APP_ROUTER},
};

use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::fmt::writer::MakeWriterExt;

#[tokio::main]
async fn main() {
    let environment_service = EnvironmentVariables::new();
    let app_state = AppState::new(
        Arc::new(environment_service.clone()),
        Arc::new(
            DatabaseService::new(Some(&environment_service))
                .await
                .expect("Error in database initialization"),
        ),
    );

    println!(
        "Current working directory: {:?}",
        std::env::current_dir().unwrap()
    );
    let logfile = tracing_appender::rolling::hourly(".logs", "application_logs");
    let (non_blocking, _guard) = tracing_appender::non_blocking(logfile);
    let stdout = std::io::stdout.with_max_level(app_state.environment_service.get_logging_level());

    // initialize tracing logging with level defined by the environment service
    tracing_subscriber::fmt()
        .with_max_level(app_state.environment_service.get_logging_level())
        .with_ansi(true)
        .with_writer(stdout.and(non_blocking))
        .init();

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, build_app(app_state)).await.unwrap();
}

async fn health_handler() -> Html<&'static str> {
    Html("Ok!")
}

/// Build our application routes. According to frontend mode we change the root behavior.
/// When frontend mode is integrated, the root returns index.html and the other static content
/// via fallback service.
///
/// When frontend mode is external then the root returns standard 200 OK
fn build_app(state: AppState) -> Router {
    let mut app =
        if let FrontendMode::Integrated(path) = state.environment_service.get_frontend_mode() {
            tracing::info!("working with frontend mode `integrated` with path {path}");
            Router::new()
                .route(
                    "/",
                    get_service(ServeFile::new(Path::new(path).join("index.html"))),
                )
                .fallback_service(get_service(
                    ServeDir::new(path)
                        .not_found_service(ServeFile::new(Path::new(path).join("index.html"))),
                ))
                .route("/api/health", get(health_handler))
        } else {
            Router::new()
                // `GET /` goes to `root`
                .route("/", get(health_handler))
        };

    app = app
        // SDK v0 user
        //.nest("/sdk/v0", SDK_ROUTER.to_owned())
        // Web application router
        .nest("/api", WEB_APP_ROUTER.to_owned())
        // Admin panel router
        .nest("/api/admin", ADMIN_ROUTER.to_owned());

    // add 404 for unknown path
    //app = app.fallback(handler_404);
    // Add middlewares to our application.
    // Layers are accessed from bottom to up, hence the order is very important
    app = add_mongodb_transaction_middleware(state.clone(), app);
    app = add_logging_middleware(
        app,
        state.environment_service.get_logging_include_headers(),
        state.environment_service.get_logging_level(),
    );
    app = add_cors_middleware(app);

    app.with_state(state)
}
