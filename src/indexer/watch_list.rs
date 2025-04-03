use crate::indexer::error::{Error, Result};
use solana_sdk::{pubkey, pubkey::Pubkey};

#[derive(Debug)]
pub enum PoolName {
    Raydium,
}

#[derive(Debug)]
pub struct TargetPool {
    pub name: PoolName,
    pub address: Pubkey,
}

impl TargetPool {
    pub fn new() -> Result<Vec<TargetPool>> {
        let mut pools: Vec<TargetPool> = Vec::new();
        pools.push(TargetPool {
            name: PoolName::Raydium,
            address: pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
        });

        Ok(pools)
    }
}
