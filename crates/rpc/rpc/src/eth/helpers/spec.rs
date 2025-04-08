use alloy_primitives::U256;
use reth_chainspec::{ChainSpecProvider, EthereumHardforks};
use reth_network_api::NetworkInfo;
use reth_rpc_eth_api::{helpers::EthApiSpec, RpcNodeCore};
use reth_storage_api::{BlockNumReader, BlockReader, ProviderTx, StageCheckpointReader};

use crate::EthApi;

//Custom imports
use crate::trace::reward_trace;
use alloy_rpc_types_trace::parity::{LocalizedTransactionTrace, RewardAction, RewardType};
use reth_chainspec::{EthChainSpec, EthereumHardfork, SEPOLIA};
use alloy_evm::block::calc::{base_block_reward_pre_merge, block_reward, ommer_reward};
use reth_primitives_traits::BlockHeader;
use reth_rpc_eth_types::EthApiError;

impl<Provider, Pool, Network, EvmConfig> EthApiSpec for EthApi<Provider, Pool, Network, EvmConfig>
where
    Self: RpcNodeCore<
        Provider: ChainSpecProvider<ChainSpec: EthereumHardforks>
                      + BlockNumReader
                      + StageCheckpointReader,
        Network: NetworkInfo,
    >,
    Provider: BlockReader,
{
    type Transaction = ProviderTx<Provider>;

    fn starting_block(&self) -> U256 {
        self.inner.starting_block()
    }

    fn signers(
        &self,
    ) -> &parking_lot::RwLock<Vec<Box<dyn reth_rpc_eth_api::helpers::EthSigner<Self::Transaction>>>>
    {
        self.inner.signers()
    }

    fn calculate_base_block_reward<H: BlockHeader>(
        &self,
        header: &H,
    ) -> Result<Option<u128>, EthApiError> {
        let chain_spec = self.provider().chain_spec();
        let is_paris_activated = if chain_spec.chain() == reth_chainspec::MAINNET.chain() {
            Some(header.number()) >= EthereumHardfork::Paris.mainnet_activation_block()
        } else if chain_spec.chain() == SEPOLIA.chain() {
            Some(header.number()) >= EthereumHardfork::Paris.sepolia_activation_block()
        } else {
            true
        };

        if is_paris_activated {
            return Ok(None);
        }

        Ok(Some(base_block_reward_pre_merge(&chain_spec, header.number())))
    }

    fn extract_reward_traces<H: BlockHeader>(
        &self,
        header: &H,
        ommers: Option<&[H]>,
        base_block_reward: u128,
    ) -> Vec<LocalizedTransactionTrace> {
        let ommers_cnt = ommers.as_ref().map(|o| o.len()).unwrap_or_default();
        let mut traces = Vec::with_capacity(ommers_cnt + 1);

        let block_reward = block_reward(base_block_reward, ommers_cnt);
        traces.push(reward_trace(
            header,
            RewardAction {
                author: header.beneficiary(),
                reward_type: RewardType::Block,
                value: U256::from(block_reward),
            },
        ));

        let Some(ommers) = ommers else { return traces };

        for uncle in ommers {
            let uncle_reward = ommer_reward(base_block_reward, header.number(), uncle.number());
            traces.push(reward_trace(
                header,
                RewardAction {
                    author: uncle.beneficiary(),
                    reward_type: RewardType::Uncle,
                    value: U256::from(uncle_reward),
                },
            ));
        }
        traces
    }
}
