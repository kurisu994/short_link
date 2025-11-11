use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

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
use uuid::Uuid;

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
        .layer(middleware::from_fn(print_request_response))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
        ]))
        .fallback(prepare::handler_404)
        .with_state(state);

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
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("short_link=info"));

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
    let (parts, body) = req.into_parts();
    // 打印参数 fixme 需要完善
    let uid = Uuid::new_v4()
        .to_string()
        .split("-")
        .last()
        .unwrap_or("")
        .to_string();
    let bytes = buffer_and_print(&format!("request[{}]", uid), body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    // 打印响应
    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print(&format!("response[{}]", uid), body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

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
        tracing::trace!("{direction} body = {body:?}");
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
