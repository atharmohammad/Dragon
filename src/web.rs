use std::sync::Arc;

use crate::{
    config::config,
    error::{Error, Result},
    types::WebhookTransactionPayload,
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
    Json(payload): Json<Vec<WebhookTransactionPayload>>,
) -> Result<impl IntoResponse> {
    let config = config();
    println!("{:?}", payload);

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

#[cfg(test)]
mod test {
    use anyhow::Result;
    use tokio::{fs, io::AsyncReadExt};

    use crate::{error::Error, types::WebhookTransactionPayload};

    #[tokio::test]
    async fn test_parse_webhook_transaction_payload_ok() -> Result<()> {
        let mut file =
            fs::File::open("/Users/athar/Desktop/workspace/dragon/examples/raydium-swap.json")
                .await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let data = serde_json::from_str::<Vec<WebhookTransactionPayload>>(&contents);
        dbg!("{}", &data);
        assert!(data.is_ok());

        Ok(())
    }
}
