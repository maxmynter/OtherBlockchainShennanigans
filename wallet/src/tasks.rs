use crate::core::Core;
use crate::ui::run_ui;
use crate::util::big_mode_btc;
use btclib::types::Transaction;
use cursive::views::TextContent;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::{self, Duration};
use tracing::*;

pub async fn update_utxos(core: Arc<Core>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(20));
        loop {
            interval.tick().await;
            if let Err(e) = core.fetch_utxos().await {
                error!("Failed to update UTXOs: {}", e);
            }
        }
    })
}

pub async fn handle_transactions(
    rx: kanal::AnsyncReceiver<Transaction>,
    core: Arc<Core>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Ok(transaction) = rx.recv().await {
            if let Err(e) = core.send_transaction(transaction).await {
                error!("Failed to send transaction: {}", e);
            }
        }
    })
}
