use std::net::SocketAddr;
use std::sync::Arc;

use axum::http::Method;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use idgen::{IdGeneratorOptions, YitIdHelper};

use crate::{
    pojo::AppError,
    pojo::Message,
    types::{HandlerResult, IState, MessageResult, RedirectResponse, RedirectResult},
};

mod demo;
mod handle;
mod idgen;
mod pojo;
mod prepare;
mod types;
mod utils;

#[tokio::main]
async fn main() {
    let options = IdGeneratorOptions::default();
    YitIdHelper::set_id_generator(options);
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "short_link=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    if let Err(err) = run_server().await {
        tracing::error!("Server error: {}", err);
    }
}

async fn run_server() -> Result<(), axum::Error> {
    let state = prepare::create_state().await;
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
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(prepare::shutdown_signal())
        .await
        .unwrap();

    Ok(())
}

fn api_router() -> Router<Arc<IState>> {
    Router::new().merge(demo::router())
}
