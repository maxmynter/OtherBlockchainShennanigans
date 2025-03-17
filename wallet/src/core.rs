use anyhow::Result;
use btclib::crypto::{PrivateKey, PublicKey};
use btclib::types::{Transaction, TransactionOutput};
use btclib::util::Saveable;
use btclib::util::Saveable;
use crossbeam_skiplist::SkipMap;
use kanal::AsyncSender;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpStream;

#[derive(Clone)]
struct UtxoStore {
    my_keys: Vec<LoadedKey>,
    utxos: Arc<SkipMap<PublicKey, Vec<(bool, TransactionOutput)>>>,
}

impl UtxoStore {
    fn new() -> Self {
        todo!();
    }
    fn add_key(&mut self, key: LoadedKey) {
        todo!()
    }
}

#[derive(Clone)]
pub struct Core {
    pub config: Config,
    utxos: UtxoStore,
    pub tx_sender: AsyncSender<Transaction>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Key {
    public: PathBuf,
    private: PathBuf,
}

#[derive(Clone)]
struct LoadedKey {
    public: PublicKey,
    private: PrivateKey,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Recipient {
    pub name: String,
    pub key: PathBuf,
}

#[derive(Clone)]
pub struct LoadedRecipient {
    pub name: String,
    pub key: PublicKey,
}

impl Recipient {
    pub fn load(&self) -> Result<LoadedRecipient> {
        let key = PublicKey::load_from_file(&self.key)?;
        Ok(LoadedRecipient {
            name: self.name.clone(),
            key,
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum FeeType {
    Fixed,
    Percent,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FeeConfig {
    pub fee_type: FeeType,
    pub value: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub my_keys: Vec<Key>,
    pub contacts: Vec<Recipient>,
    pub default_node: String,
    pub fee_config: FeeConfig,
}
