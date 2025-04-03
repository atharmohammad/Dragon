pub mod error;
mod utils;
pub mod webhook_client;

use solana_sdk::{program_memory::sol_memcmp, program_pack, pubkey, pubkey::Pubkey};
use std::{
    collections::{BTreeSet, HashMap},
    mem,
    str::FromStr,
};
use utils::filter_target_token_data_from_tx;

use crate::{
    indexer::error::{Error, Result},
    types::WebhookTransactionPayload,
    watch_list,
};

pub struct Indexer {
    current_block: i64,
    tx_map: HashMap<(String, String, i64, i64), WebhookTransactionPayload>,
    block_tx_map: HashMap<i64, Vec<WebhookTransactionPayload>>,
}

impl Indexer {
    pub async fn index(self: &mut Self, payload: Vec<WebhookTransactionPayload>) -> Result<()> {
        for tx in payload {
            self.search_sandwich_attack(&tx);
            // todo: silently log error
            self.map_tx_with_block(tx)?;
        }

        Ok(())
    }

    pub fn search_sandwich_attack(self: &Self, tx: &WebhookTransactionPayload) -> Result<()> {
        let message = &tx.transaction.message;
        let watch_list = watch_list::TargetPool::new()?
            .iter()
            .map(|target_pool| target_pool.address.to_string())
            .collect::<Vec<String>>();

        let transaction_accounts = &message.account_keys;
        let swap_instruction = message
            .instructions
            .iter()
            .find(|ix| watch_list.contains(&transaction_accounts[ix.program_id_index]));

        if swap_instruction.is_none() {
            todo!()
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
            // return;
        }

        let user_target_token_account = if diff < 0 {
            user_destination_token_account
        } else {
            user_source_token_account
        };

        let (token_mint, user_pre_token_balance, user_post_token_balance) =
            filter_target_token_data_from_tx(&user_target_token_account, &tx)?;

        /*
          1. Check for the key (owner, mint, block, price) for both the tokens in map to find the frontrun or backrun tx (As transaction coming in webhook could be out of order).
          2. If found -> then save it to db and clear the key from map.
          3. If not found -> insert into map.
        */

        Ok(())
    }

    pub fn map_tx_with_block(self: &mut Self, tx: WebhookTransactionPayload) -> Result<()> {
        if self.block_tx_map.contains_key(&tx.block_time) {
            self.block_tx_map
                .get_mut(&self.current_block)
                .ok_or(Error::FailedToIndexInBlockTxMapVector)?
                .push(tx);
        } else {
            let block_time = tx.block_time;
            let vec: Vec<WebhookTransactionPayload> = Vec::from([tx]);

            self.block_tx_map.insert(block_time, vec);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    #[test]
    fn deserailize_raydium_swap_ok() -> Result<()> {
        todo!()
    }
}
