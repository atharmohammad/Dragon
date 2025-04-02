use std::sync::Arc;

use crate::{
    config::config,
    error::{Error, Result},
};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::webhook_client::HeliusWebhookClient;

#[axum::debug_handler]
pub async fn index(
    State(webhook_client): State<Arc<HeliusWebhookClient>>,
    Json(payload): Json<Value>,
) -> Result<impl IntoResponse> {
    let config = config();
    // TODO: Safely handle errors
    println!("received payload: {}", payload);

    /*
     * Use helius webhooks to listen to swaps, and then look for tx before and after it.
     *
     * 1. filter the swap tx
     * 2. map txs to their blocks
     * 3. implement a sliding window search
     */
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct WebhookForCreate {
    url: String,
}

#[axum::debug_handler]
pub async fn create_webhook_hanlder(
    State(webhook_client): State<Arc<HeliusWebhookClient>>,
    Json(data): Json<WebhookForCreate>,
) -> Result<Json<Value>> {
    let webhook = webhook_client.create_webhook(&data.url).await?;

    Ok(Json(json!({
        "webhook": webhook
    })))
}
