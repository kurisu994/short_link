use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::http::{Method, Request};
use axum::Router;
use axum::{
    body::{Body, Bytes},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    serve,
};
use chrono::Local;
use http_body_util::BodyExt;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use idgen::{IdGeneratorOptions, YitIdHelper};

use crate::{
    pojo::AppError,
    pojo::Message,
    service::{link_base_service, link_service, cleanup_service},
    types::{IState, RedirectResult},
};

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn generate_request_id() -> String {
    let counter = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("req_{:x}", counter)
}

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
        .layer(middleware::from_fn(print_request_response))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
        ]))
        .fallback(prepare::handler_404)
        .with_state(state.clone());

    // 启动定时清理过期链接任务
    let cleanup_state = state.clone();
    tokio::spawn(async move {
        cleanup_service::cleanup_expired_links_task(cleanup_state).await;
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8008));
    tracing::info!(" - Local:   http://{}:{}", "127.0.0.1", 8008);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    serve(listener, app)
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
    // 创建日志目录（如果不存在）
    std::fs::create_dir_all("./logs").ok();

    // 创建按天滚动的日志文件
    let file_appender = tracing_appender::rolling::daily("./logs", "short-link.log");

    // 控制台输出和文件输出
    let (console_writer, file_writer) = (io::stdout, file_appender);

    // 设置日志级别过滤器
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("short_link=info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_ansi(true)
                .with_file(false)
                .with_timer(LocalTimer)
                .with_writer(console_writer),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_ansi(false)
                .with_file(false)
                .with_timer(LocalTimer)
                .with_writer(file_writer),
        )
        .init()
}

async fn print_request_response(
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start_time = std::time::Instant::now();
    let (parts, body) = req.into_parts();
    let uid = generate_request_id();
    let method = parts.method.clone();
    let uri = parts.uri.clone();

    tracing::info!("request[{}] - {} {}", uid, method, uri);

    let should_log_body = std::env::var("RUST_LOG")
        .unwrap_or_default()
        .contains("debug") ||
        uri.path().starts_with("/link/");

    let bytes = if should_log_body {
        buffer_and_print(&format!("request[{}]", uid), body).await?
    } else {
        body.collect().await.map_err(|err| {
            (StatusCode::BAD_REQUEST, format!("failed to read request body: {err}"))
        })?.to_bytes()
    };

    let req = Request::from_parts(parts, Body::from(bytes));
    let res = next.run(req).await;

    let duration = start_time.elapsed();
    let (response_parts, body) = res.into_parts();
    let status = response_parts.status;

    tracing::info!("response[{}] - {} - {:?}", uid, status, duration);

    let bytes = if should_log_body {
        buffer_and_print(&format!("response[{}]", uid), body).await?
    } else {
        body.collect().await.map_err(|err| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("failed to read response body: {err}"))
        })?.to_bytes()
    };

    let res = Response::from_parts(response_parts, Body::from(bytes));
    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(body) {
            if let Ok(formatted_json) = serde_json::to_string_pretty(&json_value) {
                tracing::info!("{direction} body (JSON):\n{}", formatted_json);
            } else {
                tracing::info!("{direction} body = {}", body);
            }
        } else {
            tracing::info!("{direction} body = {}", body);
        }
    }

    Ok(bytes)
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
