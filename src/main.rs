#![allow(unused)]
mod config;
mod db;
mod error;
mod indexer;
mod routes;
mod web;

use axum::{
    Router,
    routing::{get, post},
};
use indexer::{
    Indexer, process_tx_buffer, types::WebhookTransactionPayload,
    webhook_client::HeliusWebhookClient,
};
use routes::static_routes;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{
        Mutex,
        mpsc::{Receiver, channel},
    },
    time::sleep,
};
use web::{create_webhook_hanlder, index};

#[tokio::main]
async fn main() {
    // Figure out the best size for queue
    let (sx_queue, mut rx_queue) = channel::<Vec<WebhookTransactionPayload>>(3000);

    tokio::spawn(async move {
        listen_to_tx_buffer(rx_queue).await;
    });

    {
        let indexer = Arc::new(Indexer::new());
        tokio::spawn(async move {
            loop {
                // Take a block out of queue every 15 seconds
                sleep(Duration::from_secs(15)).await;
                println!("Checking if there is a block in queue");

                if let Some(buffer) = indexer.get_buffer_to_process().await {
                    if let Err(e) = sx_queue.send(buffer).await {
                        println!("Failed to send tx buffer for processing, Error {}", e);
                    }
                }
            }
        });
    }

    let webhook_client = Arc::new(HeliusWebhookClient::new().unwrap()); //panic early

    let routes = Router::new()
        .route("/index", post(index))
        .route("/create", post(create_webhook_hanlder))
        .with_state(Arc::clone(&webhook_client));

    let app = Router::new()
        .merge(routes)
        .fallback_service(static_routes());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3005").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn listen_to_tx_buffer(mut rx_queue: Receiver<Vec<WebhookTransactionPayload>>) {
    println!("Listening to tx buffer");

    while let Some(buffer) = rx_queue.recv().await {
        process_tx_buffer(buffer).await;
        todo!()
    }
    println!("rx_queue has been closed !");
}
