pub mod error;
pub mod types;
pub mod webhook_client;

mod utils;
mod watch_list;

use solana_sdk::{program_memory::sol_memcmp, program_pack, pubkey, pubkey::Pubkey};
use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    mem,
    str::FromStr,
    sync::Arc,
};
use tokio::sync::Mutex;
use types::{SandwichAttackVector, WebhookTransactionPayload};
use utils::filter_target_token_data_from_tx;

use crate::indexer::error::{Error, Result};

pub struct Indexer {
    tx_buffers: Arc<Mutex<HashMap<i64, Vec<WebhookTransactionPayload>>>>,
    block_queue: Arc<Mutex<VecDeque<i64>>>,
}

impl Indexer {
    pub fn new() -> Self {
        let tx_buffers: Arc<Mutex<HashMap<i64, Vec<WebhookTransactionPayload>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let block_queue = Arc::new(Mutex::new(VecDeque::<i64>::new()));

        Indexer {
            tx_buffers,
            block_queue,
        }
    }

    pub async fn index(self: &mut Self, payload: Vec<WebhookTransactionPayload>) -> Result<()> {
        {
            let mut tx_buffers = self.tx_buffers.lock().await;
            for tx in payload {
                if tx_buffers.contains_key(&tx.block_time) {
                    tx_buffers
                        .get_mut(&tx.block_time)
                        .ok_or(Error::FailedToIndexInBlockTxMapVector)?
                        .push(tx);
                } else {
                    let block_time = tx.block_time;
                    let vec: Vec<WebhookTransactionPayload> = Vec::from([tx]);

                    tx_buffers.insert(block_time, vec);
                }
            }
        }

        Ok(())
    }

    pub async fn get_buffer_to_process(self: &Self) -> Option<Vec<WebhookTransactionPayload>> {
        let block = {
            let mut block_queue = self.block_queue.lock().await;
            block_queue.pop_front()
        };

        let block = block?;
        let buffer = {
            let mut tx_buffers = self.tx_buffers.lock().await;
            tx_buffers.remove(&block)
        };

        buffer
    }
}

pub async fn process_tx_buffer(buffer: Vec<WebhookTransactionPayload>) {
    let tx_map = HashMap::<(String, String, i64, i64), WebhookTransactionPayload>::new();

    todo!()
}

pub fn search_sandwich_attack(
    tx: &WebhookTransactionPayload,
) -> Option<&WebhookTransactionPayload> {
    let message = &tx.transaction.message;
    let watch_list = watch_list::TargetPool::new()
        .ok()?
        .iter()
        .map(|target_pool| target_pool.address.to_string())
        .collect::<Vec<String>>();

    let transaction_accounts = &message.account_keys;
    let swap_instruction = message
        .instructions
        .iter()
        .find(|ix| watch_list.contains(&transaction_accounts[ix.program_id_index]));

    if swap_instruction.is_none() {
        return None;
    }

    let swap_instruction = swap_instruction.unwrap();
    let mut swap_accounts: Vec<String> = Vec::new();

    swap_instruction
        .accounts
        .iter()
        .for_each(|index| swap_accounts.push(transaction_accounts[*index].clone()));

    let len = swap_accounts.len();
    let user = &swap_accounts[len - 1];
    let user_destination_token_account = &swap_accounts[len - 2];
    let user_source_token_account = &swap_accounts[len - 3];

    /*
       For V1:
       Filter out only swap between SOL and Token
       - get the signer/owner account index and use it to get pre and post sol balance

       1. If Swap Sol -> Token
       - get the signer/owner destination account and use it to get pre and post token balance

       2. If Swap Token -> Sol
       - get the signer/owner source token account and use it to get pre and post token balance

    */

    let user_pre_sol_balance = tx.meta.pre_balances[0];
    let user_post_sol_balance = tx.meta.post_balances[0];

    let diff = user_post_sol_balance - user_pre_sol_balance;
    if diff == 0 {
        return None;
    }

    let user_target_token_account = if diff < 0 {
        user_destination_token_account
    } else {
        user_source_token_account
    };

    let (token_mint, user_pre_token_balance, user_post_token_balance) =
        filter_target_token_data_from_tx(&user_target_token_account, &tx).ok()?;

    /*
      1. Check for the key (owner, mint, block, price) for both the tokens in map to find the frontrun or backrun tx (As transaction coming in webhook could be out of order).
      2. If found -> then save it to db and clear the key from map.
      3. If not found -> insert into map.
    */

    let abs_diff = (user_post_token_balance - user_pre_token_balance).abs();
    let key = (user.clone(), token_mint, tx.block_time, abs_diff);
    // let second_tx = self.tx_map.get(&key);

    // second_tx

    todo!()
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    #[test]
    fn deserailize_raydium_swap_ok() -> Result<()> {
        todo!()
    }
}
