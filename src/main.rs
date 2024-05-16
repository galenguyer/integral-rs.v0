use std::{env, net::SocketAddr, sync::Arc};

use axum::{routing, Extension, Router};
use sqlx::{Pool, Sqlite};

mod db;
mod extractors;
mod features;
mod routes;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    if std::env::args().nth(1) == Some("--version".to_string()) {
        println!(
            "{}",
            option_env!("CARGO_PKG_VERSION").unwrap_or_else(|| "unknown")
        );
        return;
    }

    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "integral=debug,tower_http=debug");
    }

    tracing_subscriber::fmt::init();

    tracing::info!(
        "Starting integral v{}",
        option_env!("CARGO_PKG_VERSION").unwrap_or_else(|| "unknown")
    );
    tracing::info!("SIGNUPS_ENABLED = {}", *features::SIGNUPS_ENABLED);

    let sqlite_pool: Arc<Pool<Sqlite>> = Arc::new(
        Pool::connect(&env::var("DATABASE_URL").expect("Missing DATABASE_URL"))
            .await
            .unwrap(),
    );

    let app = Router::new()
        .nest(
            "/api",
            Router::new().nest(
                "/v0",
                Router::new()
                    .route(
                        "/features",
                        routing::get(routes::v0::features::get_features),
                    )
                    .nest(
                        "/users",
                        Router::new()
                            .route("/whoami", routing::get(routes::v0::login::whoami))
                            .route("/login", routing::post(routes::v0::login::login))
                            .route("/signup", routing::post(routes::v0::login::create_user)),
                    )
                    .nest(
                        "/jobs",
                        Router::new().route(
                            "/",
                            routing::get(routes::v0::jobs::get_all_jobs)
                                .post(routes::v0::jobs::create_job),
                        ).route("/comments", routing::post(routes::v0::jobs::add_comment)),
                    ),
            ),
        )
        .layer(Extension(sqlite_pool));

    let bind_address: SocketAddr = env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| String::from("0.0.0.0:8080"))
        .parse()
        .unwrap();
    let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
