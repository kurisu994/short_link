use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use axum::http::Method;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use idgen::{IdGeneratorOptions, YitIdHelper};

use crate::{
    pojo::AppError,
    pojo::Message,
    service::{link_base_service, link_service},
    types::{HandlerResult, IState, MessageResult, RedirectResponse, RedirectResult},
};
use crate::config::Logging;

mod config;
mod demo;
mod handle;
mod idgen;
mod pojo;
mod prepare;
mod service;
mod types;
mod utils;

#[tokio::main]
async fn main() {
    YitIdHelper::set_id_generator(IdGeneratorOptions::default());
    let (state, logging) = prepare::create_state().await;

    init_log(logging);
    if let Err(err) = run_server(state).await {
        tracing::error!("Server error: {}", err);
    }
}

async fn run_server(state: Arc<IState>) -> Result<(), axum::Error> {
    let app = api_router()
        .layer(CorsLayer::new().allow_origin(Any).allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
        ]))
        .fallback(prepare::handler_404)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8008));
    tracing::info!("server start success, listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(prepare::shutdown_signal())
        .await
        .unwrap();

    Ok(())
}

fn api_router() -> Router<Arc<IState>> {
    Router::new()
        .merge(demo::router())
        .merge(handle::api::router())
        .merge(handle::admin::router())
}

fn init_log(logging: Logging) {
    let log_info = logging.level.unwrap_or("debug".to_string());
    let info_file = tracing_appender::rolling::daily("./logs", "short-link-all.log")
        .with_max_level(tracing::Level::INFO);
    let error_file = tracing_appender::rolling::daily("./logs", "short-link-err.log")
        .with_max_level(tracing::Level::ERROR);
    let all_files = info_file.and(error_file);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::from_str(&format!("short_link={}", log_info))
                .unwrap_or_else(|_| "short_link=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer()
            .pretty()
            .with_file(false)
            .with_writer(io::stdout)
        )
        .with(tracing_subscriber::fmt::layer()
            .pretty()
            .with_file(false)
            .with_writer(all_files)
        )
        .init();
}
