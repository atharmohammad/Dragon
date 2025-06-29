use crate::indexer::error::{Error, Result};
use solana_pubkey::{Pubkey, pubkey};

#[derive(Debug)]
pub enum PoolName {
    Raydium,
    PumpFun,
}

#[derive(Debug)]
pub struct TargetPool {
    pub name: PoolName,
    pub address: Pubkey,
    pub instruction_discriminator: u8,
}

impl TargetPool {
    pub fn new() -> Vec<TargetPool> {
        let mut pools: Vec<TargetPool> = Vec::new();
        pools.push(TargetPool {
            name: PoolName::Raydium,
            address: pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
            instruction_discriminator: 9,
        });

        pools.push(TargetPool {
            name: PoolName::PumpFun,
            address: pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA"),
            instruction_discriminator: 6, //todo: check
        });

        pools
    }
}
