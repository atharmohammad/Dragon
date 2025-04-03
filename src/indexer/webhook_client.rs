use super::watch_list::TargetPool;
use crate::{
    config::config,
    indexer::error::{Error, Result},
};
use axum::{Json, extract::State, response::IntoResponse};
use helius::{
    Helius,
    types::{
        AccountWebhookEncoding, CreateWebhookRequest, EditWebhookRequest, HeliusEndpoints,
        TransactionStatus, TransactionType, Webhook, WebhookType,
    },
};
use reqwest::{Method, Url};
use serde_json::Value;
use std::{cell::RefCell, sync::Arc};

pub struct HeliusWebhookClient {
    pub helius: Helius,
    pub base_url: String,
    pub webhook_id: Option<String>,
}

impl HeliusWebhookClient {
    pub fn new() -> Result<Self> {
        let config = config();
        let helius = Helius::new(&config.HELIUS_API_KEY, helius::types::Cluster::Devnet)?;

        Ok(HeliusWebhookClient {
            base_url: String::from("https://api.helius.xyz"), // Api prefix for webhooks, incorrect in helius sdk.
            helius,
            webhook_id: config.WEBHOOK_ID.clone(),
        })
    }

    pub async fn get_webhook_by_id(self: &Self, id: &str) -> Result<Webhook> {
        let webhook_id = self.webhook_id.as_ref().ok_or(Error::WebhookIdMissing)?;
        let url: String = format!(
            "{}/v0/webhooks/{}?api-key={}",
            self.base_url, webhook_id, self.helius.config.api_key
        );
        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        let webhook = self
            .helius
            .rpc_client
            .handler
            .send::<_, Webhook>(Method::GET, parsed_url, None::<&()>)
            .await?;
        Ok(webhook)
    }

    pub async fn create_webhook(self: &Self, webhook_url: &str) -> Result<Webhook> {
        let watch_list = TargetPool::new()?;
        let account_addresses: Vec<String> = watch_list
            .iter()
            .map(|pool| pool.address.to_string())
            .collect();

        let request = CreateWebhookRequest {
            webhook_url: webhook_url.to_string(),
            transaction_types: Vec::from([TransactionType::Swap]),
            account_addresses,
            webhook_type: WebhookType::Raw,
            txn_status: TransactionStatus::Success,
            auth_header: None, // todo: add auth header
            encoding: AccountWebhookEncoding::JsonParsed,
        };

        let url: String = format!(
            "{}/v0/webhooks?api-key={}",
            self.base_url, self.helius.config.api_key
        );
        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        let webhook = self
            .helius
            .rpc_client
            .handler
            .send::<_, Webhook>(Method::POST, parsed_url, Some(&request))
            .await?;

        Ok(webhook)
    }

    pub async fn delete_webhook(self: &Self) -> Result<()> {
        let webhook_id = self.webhook_id.as_ref().ok_or(Error::WebhookIdMissing)?;
        let url: String = format!(
            "{}/v0/webhooks/{}?api-key={}",
            self.base_url, webhook_id, self.helius.config.api_key
        );
        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        self.helius
            .rpc_client
            .handler
            .send::<_, ()>(Method::DELETE, parsed_url, None::<&()>)
            .await?;

        Ok(())
    }

    pub async fn subscribe(self: &Self, address: String) -> Result<()> {
        let webhook_id = self.webhook_id.as_ref().ok_or(Error::WebhookIdMissing)?;

        let mut webhook = self.get_webhook_by_id(webhook_id).await?;
        webhook.account_addresses.push(address);

        let request = EditWebhookRequest {
            webhook_id: webhook.webhook_id,
            webhook_url: webhook.webhook_url,
            transaction_types: webhook.transaction_types,
            account_addresses: webhook.account_addresses,
            webhook_type: webhook.webhook_type,
            auth_header: webhook.auth_header,
            txn_status: webhook.txn_status,
            encoding: webhook.encoding,
        };
        let url: String = format!(
            "{}v0/webhooks/{}?api-key={}",
            self.base_url, request.webhook_id, self.helius.config.api_key
        );
        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        self.helius
            .rpc_client
            .handler
            .send::<_, Webhook>(Method::PUT, parsed_url, Some(&request))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use super::HeliusWebhookClient;

    #[test]
    fn test_initialize_webhook_client_ok() -> Result<()> {
        let res = HeliusWebhookClient::new();
        assert!(res.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_and_delete_webhook_ok() -> Result<()> {
        let mut client = HeliusWebhookClient::new()?;
        let fx_webhook_url = "http://localhost:3005/helius/test";
        let res = client.create_webhook(fx_webhook_url).await;

        assert!(res.is_ok());
        let webhook = res?;
        assert_eq!(webhook.webhook_url, fx_webhook_url);
        client.delete_webhook().await?;

        Ok(())
    }
}
