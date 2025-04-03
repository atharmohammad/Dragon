#![allow(unused)]
mod config;
mod db;
mod error;
mod indexer;
mod routes;
mod types;
mod watch_list;
mod web;

use axum::{
    Router,
    routing::{get, post},
};
use indexer::webhook_client::HeliusWebhookClient;
use routes::static_routes;
use std::sync::Arc;
use tokio::sync::Mutex;
use web::{create_webhook_hanlder, index};

#[tokio::main]
async fn main() {
    let webhook_client = Arc::new(HeliusWebhookClient::new().unwrap()); //panic early

    let routes = Router::new()
        .route("/index", post(index))
        .route("/create", post(create_webhook_hanlder))
        .with_state(Arc::clone(&webhook_client));

    let app = Router::new()
        .merge(routes)
        .fallback_service(static_routes());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3005").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
