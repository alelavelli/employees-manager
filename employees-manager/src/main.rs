use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use employees_manager::{
    middleware::{add_cors_middleware, add_logging_middleware},
    router::{ADMIN_ROUTER, SDK_ROUTER, WEB_APP_ROUTER},
    service::{db::get_database_service, environment::ENVIRONMENT},
};
use tracing_subscriber::fmt::writer::MakeWriterExt;

#[tokio::main]
async fn main() {
    let logfile = tracing_appender::rolling::hourly(".logs", "application_logs");
    let (non_blocking, _guard) = tracing_appender::non_blocking(logfile);
    let stdout = std::io::stdout.with_max_level(ENVIRONMENT.logging.level);

    // initialize tracing logging with level defined by the environment service
    tracing_subscriber::fmt()
        .with_max_level(ENVIRONMENT.logging.level)
        .with_ansi(true)
        .with_writer(stdout.and(non_blocking))
        .init();

    // initialize database service
    get_database_service().await;

    // build our application two routes, one for the sdk and the other for web application
    let mut app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(handler))
        // SDK v0 user
        .nest("/sdk/v0", SDK_ROUTER.to_owned())
        // Web application router
        .nest("/api", WEB_APP_ROUTER.to_owned())
        // Admin panel router
        .nest("/api/admin", ADMIN_ROUTER.to_owned());

    // add 404 for unknown path
    app = app.fallback(handler_404);
    // Add middlewares to our application.
    // Layers are accessed from bottom to up, hence the order is very important
    app = add_logging_middleware(app);
    app = add_cors_middleware(app);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("Ok!")
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
