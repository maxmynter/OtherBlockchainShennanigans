use thiserror::Error;

#[derive(Error, Debug)]
pub enum BtcError {
    #[error("Invalid Transaction")]
    InvalidTransaction,

    #[error("Invalid Block")]
    InvalidBlock,

    #[error("Invalid Block Headger")]
    InvalidBlockHeader,

    #[error("Invalid transaction Input")]
    TransactionInput,

    #[error("Invalid transaction Output")]
    TransactionOutput,

    #[error("Invaild Merkle Root")]
    InvalidMerkleRoot,

    #[error("Invalid Hash")]
    InvalidHash,

    #[error("Invalid Signature")]
    InvalidSignature,

    #[error("Invalid Public Key")]
    InvalidPublicKey,

    #[error("Invalid Private Key")]
    InvalidPrivateKey,
}

pub type Result<T> = std::result::Result<T, BtcError>;
