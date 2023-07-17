use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use axum::http::Method;
use axum::Router;
use chrono::Local;
use tower_http::cors::{Any, CorsLayer};
use tracing::Level;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use idgen::{IdGeneratorOptions, YitIdHelper};

use crate::{
    pojo::AppError,
    pojo::Message,
    service::{link_base_service, link_service},
    types::{IState, RedirectResult},
};

mod config;
mod handle;
mod idgen;
mod pojo;
mod prepare;
mod service;
mod types;
mod utils;

#[tokio::main]
async fn main() {
    print_banner();
    init_log();
    YitIdHelper::set_id_generator(IdGeneratorOptions::default());
    let state = prepare::create_state().await;
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
    tracing::info!(
        " - Local:   http://{}:{}",
        "127.0.0.1",
        8008);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(prepare::shutdown_signal())
        .await
        .unwrap();
    Ok(())
}

fn api_router() -> Router<Arc<IState>> {
    Router::new()
        .merge(handle::api::router())
        .merge(handle::admin::router())
}

struct LocalTimer;

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S.%3f"))
    }
}

fn init_log() {
    let info_file = tracing_appender::rolling::daily("./logs", "short-link-all.log")
        .with_max_level(Level::INFO);
    let error_file = tracing_appender::rolling::daily("./logs", "short-link-err.log")
        .with_max_level(Level::ERROR);
    let all_files = info_file.and(error_file);
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::from_str(&"short_link=info")
                .unwrap_or_else(|_| "short_link=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_ansi(true)
                .with_file(false)
                .with_timer(LocalTimer)
                .with_writer(io::stdout),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_ansi(false)
                .with_file(false)
                .with_timer(LocalTimer)
                .with_writer(all_files),
        )
        .init()
}

fn print_banner() {
    let banner = r#"
    ███████╗██╗  ██╗ ██████╗ ██████╗ ████████╗    ██╗     ██╗███╗   ██╗██╗  ██╗
    ██╔════╝██║  ██║██╔═══██╗██╔══██╗╚══██╔══╝    ██║     ██║████╗  ██║██║ ██╔╝
    ███████╗███████║██║   ██║██████╔╝   ██║       ██║     ██║██╔██╗ ██║█████╔╝
    ╚════██║██╔══██║██║   ██║██╔══██╗   ██║       ██║     ██║██║╚██╗██║██╔═██╗
    ███████║██║  ██║╚██████╔╝██║  ██║   ██║       ███████╗██║██║ ╚████║██║  ██╗
    ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝       ╚══════╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝
"#;
    println!("{}", banner);
}

