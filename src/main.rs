#![allow(unused)]

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use routes::static_routes;
use tokio::sync::Mutex;
use web::{create_webhook_hanlder, index};
use webhook_client::HeliusWebhookClient;
mod config;
mod error;
mod routes;
mod watch_list;
mod web;
mod webhook_client;

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
