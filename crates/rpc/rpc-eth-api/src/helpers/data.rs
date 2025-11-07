/// Helper data structures for RPC
use jsonrpsee::core::Serialize;


use serde::Deserialize;

use alloy_rpc_types::Block;
use alloy_rpc_types_trace::parity::{LocalizedTransactionTrace, TraceResults};

/// `EnrichedTransaction` object used in RPC
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichedTransaction {
    ///Alloy ETH transaction
    #[cfg(not(feature = "optimism"))]
    #[serde(flatten)]
    pub inner: alloy_rpc_types_eth::Transaction,
    ///Alloy Optimism transaction
    #[cfg(feature = "optimism")]
    #[serde(flatten)]
    pub inner: op_alloy_rpc_types::Transaction,

    ///compressed public key
    pub public_key: String,

    ///Alloy ETH receipts
    #[cfg(not(feature = "optimism"))]
    pub receipts: alloy_rpc_types_eth::TransactionReceipt,
    ///Alloy Optimism receipts
    #[cfg(feature = "optimism")]
    pub receipts: op_alloy_rpc_types::OpTransactionReceipt,

    ///Alloy traces
    pub trace: TraceResults,
}

/// `EnrichedBlock` object used in RPC
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichedBlock {
    ///Alloy block
    #[serde(flatten)]
    pub inner: Block<EnrichedTransaction>,

    ///static block rewards
    pub rewards: Vec<LocalizedTransactionTrace>,
}