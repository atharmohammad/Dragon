use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookTransactionPayload {
    slot: i32,
    block_time: i64,
    index_within_block: i32,
    meta: TransactionMeta,
    transaction: Transaction,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionMeta {
    pre_token_balances: Vec<TokenBalance>,
    post_token_balances: Vec<TokenBalance>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    message: TransactionMessage,
    signatures: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionMessage {
    account_keys: Vec<String>,
    instructions: Vec<Instruction>,
    recent_blockhash: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instruction {
    accounts: Vec<usize>,
    data: String,
    program_id_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    account_index: usize,
    mint: String,
    owner: String,
    program_id: String,
    ui_token_amount: UiTokenAmount,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiTokenAmount {
    amount: String,
    decimals: u8,
}
