#![allow(unused)]
mod config;
mod db;
mod error;
mod indexer;

use crate::config::config;
use axum::{
    Router,
    routing::{get, post},
};
use indexer::Indexer;
use solana_client::rpc_client::RpcClient;
use solana_transaction_status_client_types::EncodedConfirmedBlock;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    time::sleep,
};

#[tokio::main]
async fn main() {
    /**
     * Picker:
     *
     * -- on avg should process 3 slots per second to not drift away:
     *
     * Every 10 second fetch all the slots after the current one and push them in the queue
     */
    // Figure out the best size for queue
    let (block_processor_sender, mut block_processor_receiver) = channel::<Vec<u64>>(3000);

    let (block_filter_sender, mut block_filter_receiver) = channel::<EncodedConfirmedBlock>(10000);

    tokio::spawn(async move {
        block_filter_and_validation(block_filter_receiver).await;
    });

    // spawn the block processor listener
    tokio::spawn(async move {
        block_processor(block_processor_receiver, block_filter_sender).await;
    });

    {
        let mut last_slot: u64 = 5000000; // Setup initial slot to start indexing mev from, can be synced from db on restart
        let rpc_client = RpcClient::new(&config().RPC_URL);

        loop {
            // fetch blocks every 10 seconds and push them in the queue
            sleep(Duration::from_secs(10)).await;

            if let Ok(blocks) = rpc_client.get_blocks(last_slot, None) {
                if let Err(e) = block_processor_sender.send(blocks).await {
                    println!("Failed to send tx buffer for processing, Error {}", e);
                }
            }
        }
    }
}

async fn block_filter_and_validation(mut block_filter_receiver: Receiver<EncodedConfirmedBlock>) {
    let indexer = Indexer::new();

    println!("Listening to block filter sender");

    while let Some(block) = block_filter_receiver.recv().await {
        indexer.find_mev_in_block(block).await;
    }
    println!("block filter receiver has been closed");
}

async fn block_processor(
    mut rx_queue: Receiver<Vec<u64>>,
    block_filter_sender: Sender<EncodedConfirmedBlock>,
) {
    let indexer = Indexer::new();

    println!("Listening to block processor");

    while let Some(buffer) = rx_queue.recv().await {
        indexer
            .process_blocks(buffer, block_filter_sender.clone())
            .await;
    }
    println!("block processor receiver has been closed");
}
