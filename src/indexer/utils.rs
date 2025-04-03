use crate::indexer::error::{Error, Result};
use crate::types::WebhookTransactionPayload;
use solana_sdk::{program_memory::sol_memcmp, pubkey::Pubkey};
use std::str::FromStr;

pub fn filter_target_token_data_from_tx(
    target_token_account: &str,
    tx: &WebhookTransactionPayload,
) -> Result<(String, i64, i64)> {
    let pre_token_balance = tx.meta.pre_token_balances.iter().find(|balance| {
        let program_id = balance.program_id.as_str();
        let program_id = Pubkey::from_str(program_id).unwrap();

        let token_account = Pubkey::find_program_address(
            &[balance.mint.as_bytes(), balance.owner.as_bytes()],
            &program_id,
        )
        .0;

        cmp_pubkey(&token_account.to_bytes(), target_token_account.as_bytes())
    });

    let pre_token_balance = pre_token_balance.ok_or(Error::FailedToParseInput)?;
    let post_token_balance = tx
        .meta
        .post_token_balances
        .iter()
        .find(|balance| balance.account_index == pre_token_balance.account_index);

    let post_token_balance = post_token_balance.ok_or(Error::FailedToParseInput)?;
    let pre_token_amount = pre_token_balance
        .ui_token_amount
        .amount
        .parse::<i64>()
        .map_err(|_| Error::FailedToParseInput)?;

    let post_token_amount = post_token_balance
        .ui_token_amount
        .amount
        .parse::<i64>()
        .map_err(|_| Error::FailedToParseInput)?;

    Ok((
        pre_token_balance.mint.clone(),
        pre_token_amount,
        post_token_amount,
    ))
}

pub fn cmp_pubkey(a: &[u8], b: &[u8]) -> bool {
    sol_memcmp(a, b, 32) == 0
}
