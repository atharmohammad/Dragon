pub mod error;
mod watch_list;

use crate::{
    config::config,
    indexer::{
        error::{Error, Result},
        watch_list::TargetPool,
    },
};
use solana_client::rpc_client::RpcClient;
use solana_instruction::{AccountMeta, Instruction};
use solana_message::{VersionedMessage, compiled_instruction::CompiledInstruction};
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use solana_transaction_status_client_types::{
    EncodedConfirmedBlock, EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
    EncodedTransactionWithStatusMeta, UiTransactionStatusMeta,
};
use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    mem,
    str::FromStr,
    sync::Arc,
};
use tokio::sync::mpsc::Sender;

/**
 *
 * Picker -> Listener -> Processor -> Filter -> Save
 *
 * Picker is running on main thread
 * Listener is running in it's own thread
 * Processor in same thread as listener and as soon as one block gets processed it gets passed to filter queue and free itself to take new block from listener
 * Filter running in it's own thread and save after checks
 */

type ProcessedTransaction = (Vec<Instruction>, UiTransactionStatusMeta);

pub struct Indexer {
    rpc_client: Arc<RpcClient>,
    target_pools: Vec<TargetPool>,
}

impl Indexer {
    pub fn new() -> Self {
        let rpc_client = Arc::new(RpcClient::new(&config().RPC_URL));
        let target_pools = TargetPool::new();

        Indexer {
            rpc_client,
            target_pools,
        }
    }

    pub async fn process_blocks(
        &self,
        blocks: Vec<u64>,
        filter_sender: Sender<EncodedConfirmedBlock>,
    ) {
        // fetch block
        for block_id in blocks {
            if let Ok(block) = self.rpc_client.get_block(block_id) {
                if let Err(e) = filter_sender.send(block).await {
                    println!("Error sending block {} in filter channel", block_id)
                }
            }
        }
    }

    pub async fn find_mev_in_block(&self, block: EncodedConfirmedBlock) -> Result<()> {
        let mut target_transactions: Vec<ProcessedTransaction> = Vec::new();

        let transactions = block.transactions;
        for transaction in transactions {
            let meta = transaction.meta.unwrap();

            let decompiled_transaction = self.filter_transaction(transaction.transaction).await;
            if let Ok(decompiled_transaction) = decompiled_transaction {
                target_transactions.push((decompiled_transaction, meta));
            }
        }

        for i in 0..=target_transactions.len() - 3 {
            let window: Result<(&[ProcessedTransaction; 3])> =
                (&target_transactions[i..i + 3]).try_into().map_err(|_| {
                    Error::Custom("Failed to convert target transactions into slice of 3")
                });

            if let Ok(window) = window {
                self.check_mev(window, &self.target_pools[0].address);
            }
        }

        Ok(())
    }

    pub async fn filter_transaction(
        &self,
        transaction: EncodedTransaction,
    ) -> Result<Vec<Instruction>> {
        // check watch list
        let transaction = transaction
            .decode()
            .ok_or(Error::Custom("Failed to decode transaction"))?;

        let instruction = transaction.message.instructions();

        let instructions = instruction
            .iter()
            .map(|ix| Self::decompile_instruction(ix, &transaction.message))
            .collect::<Result<Vec<Instruction>>>()?;

        // Filter for target pools
        let is_target_pool_transaction = instructions.iter().any(|ix| {
            let program_id = ix.program_id;
            self.target_pools
                .iter()
                .any(|pool| pool.address.eq(&program_id))
        });

        if !is_target_pool_transaction {
            return Err(Error::TransactionNotFromTargetPools);
        }

        //TODO: Filter for swap transaction as well
        // Should we only keep swap ix ?
        // Should we only keep transactions that are sequential in batch of 3

        return Err(Error::TransactionNotFromTargetPools);
    }

    fn decompile_instruction(
        compiled_ix: &CompiledInstruction,
        message: &VersionedMessage,
    ) -> Result<Instruction> {
        let keys = message.static_account_keys();

        let account_metas = compiled_ix
            .accounts
            .iter()
            .map(|account_index| -> Result<AccountMeta> {
                let account_index =
                    usize::try_from(*account_index).map_err(|_| Error::NumericalOverflow)?;
                let is_signer = message.is_signer(account_index);

                // Safe to use for client side validation
                let is_writable = message.is_maybe_writable(account_index, None);

                Ok(AccountMeta {
                    pubkey: keys[account_index].clone(),
                    is_signer,
                    is_writable,
                })
            })
            .collect::<Result<Vec<AccountMeta>>>()?;

        let program_id_index =
            usize::try_from(compiled_ix.program_id_index).map_err(|_| Error::NumericalOverflow)?;

        Ok(Instruction {
            program_id: keys[program_id_index].clone(),
            accounts: account_metas,
            data: compiled_ix.data.clone(),
        })
    }

    pub fn check_mev(
        &self,
        transactions: &[ProcessedTransaction; 3],
        target_pool: &Pubkey,
    ) -> Result<()> {
        let [frontrun, victim, backrun] = transactions;

        let (frontrun_source, frontrun_amount_in, frontrun_amount_out) =
            Self::get_swap_data(&victim.0, target_pool, 9)?;

        let (victim_source, victim_amount_in, victim_amount_out) =
            Self::get_swap_data(&victim.0, target_pool, 9)?;

        let (backrun_source, backrun_amount_in, backrun_amount_out) =
            Self::get_swap_data(&victim.0, target_pool, 9)?;

        if frontrun_source != backrun_source {
            return Err(Error::Custom("Not a MEV"));
        }

        if frontrun_amount_out != backrun_amount_in {
            return Err(Error::Custom("Not a compelete MEV"));
        }

        let attacker_delta = backrun_amount_out - frontrun_amount_in;

        dbg!(format!(
            "
            'Attacker': {},
            'Victim': {},
            'Delta': {}
        ",
            frontrun_source.to_string(),
            victim_source.to_string(),
            attacker_delta
        ));

        Ok(())
    }

    fn get_swap_data(
        transaction: &Vec<Instruction>,
        target_pool: &Pubkey,
        expected_discriminator: u8,
    ) -> Result<(Pubkey, i128, i128)> {
        // Find swap instruction targeting the pool
        let instruction = transaction
            .iter()
            .find(|ix| &ix.program_id == target_pool)
            .ok_or_else(|| Error::Custom("No matching instruction found for target pool".into()))?;

        let data = &instruction.data;

        if data.is_empty() {
            return Err(Error::Custom("Instruction data is empty".into()));
        }

        let discriminator = data[0];
        if discriminator != expected_discriminator {
            return Err(Error::Custom(
                "Invalid discriminator, instruction doesn't belong to swap",
            ));
        }

        if data.len() < 17 {
            return Err(Error::Custom("Instruction data too short for swap".into()));
        }

        let amount_in = u64::from_le_bytes(
            data[1..9]
                .try_into()
                .map_err(|_| Error::Custom("Failed to parse amount_in".into()))?,
        );

        let min_amount_out = u64::from_be_bytes(
            data[9..17]
                .try_into()
                .map_err(|_| Error::Custom("Failed to parse min_amount_out".into()))?,
        );

        let source = instruction
            .accounts
            .last()
            .ok_or_else(|| Error::Custom("Missing source account in instruction".into()))?
            .clone();

        Ok((
            source.pubkey,
            i128::from(amount_in),
            i128::from(min_amount_out),
        ))
    }

    pub async fn save_in_db() -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    #[tokio::test]
    async fn test_check_mev_in_raydium_swap() -> Result<()> {
        todo!()
    }
}
