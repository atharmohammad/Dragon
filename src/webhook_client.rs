use crate::{
    config::config,
    error::{Error, Result},
};
use helius::{
    Helius,
    types::{
        AccountWebhookEncoding, CreateWebhookRequest, EditWebhookRequest, TransactionStatus,
        TransactionType, Webhook, WebhookType,
    },
};

pub struct HeliusWebhookClient {
    pub rpc: Helius,
    pub webhook_id: Option<String>,
}

impl HeliusWebhookClient {
    pub fn new() -> Result<Self> {
        let config = config();
        let helius = Helius::new(&config.HELIUS_API_KEY, helius::types::Cluster::Devnet)?;

        Ok(HeliusWebhookClient {
            rpc: helius,
            webhook_id: None,
        })
    }

    pub async fn get_webhook_by_id(self: &Self, id: &str) -> Result<Webhook> {
        let webhook = self.rpc.get_webhook_by_id(id).await?;
        Ok(webhook)
    }

    pub async fn create_webhook(self: &mut Self, webhook_url: &str) -> Result<Webhook> {
        let request = CreateWebhookRequest {
            webhook_url: webhook_url.to_string(),
            transaction_types: Vec::from([TransactionType::Swap]),
            account_addresses: Vec::new(),
            webhook_type: WebhookType::EnhancedDevnet,
            txn_status: TransactionStatus::Success,
            auth_header: None, // todo: add auth header
            encoding: AccountWebhookEncoding::JsonParsed,
        };

        let webhook = self.rpc.create_webhook(request).await?;
        self.webhook_id = Some(webhook.webhook_id.clone());

        Ok(webhook)
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
        self.rpc.edit_webhook(request).await?;

        Ok(())
    }
}

pub async fn index() -> Result<()> {
    let config = config();
    // TODO: Safely handle errors

    /*
     * Use helius webhooks to listen to swaps, and then look for tx before and after it.
     *
     * 1. filter the swap tx
     * 2. map txs to their blocks
     * 3. implement a sliding window search
     */
    Ok(())
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
    async fn test_create_webhook_ok() -> Result<()> {
        let mut client = HeliusWebhookClient::new()?;
        let fx_webhook_url = "https://localhost:3005/helius/test";
        let res = client.create_webhook(fx_webhook_url).await;

        assert!(res.is_ok());
        let webhook = res?;
        assert_eq!(webhook.webhook_url, fx_webhook_url);
        //todo: delete webhook

        Ok(())
    }
}
