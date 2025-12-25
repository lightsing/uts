//! Solidity contracts for UTS

/// UniversalTimestamps contract
pub mod uts {
    #[doc(hidden)]
    pub mod binding {
        use alloy_sol_types::sol;

        sol!(
            #[sol(rpc, all_derives)]
            IUniversalTimestamps,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../target/foundry/IUniversalTimestamps.sol/IUniversalTimestamps.json"
            )
        );
        sol!(
            #[sol(rpc)]
            UniversalTimestamps,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../target/foundry/UniversalTimestamps.sol/UniversalTimestamps.json"
            )
        );
    }

    pub use binding::IUniversalTimestamps::{
        Attested, IUniversalTimestampsInstance as UniversalTimestamps,
    };

    pub use binding::UniversalTimestamps::{BYTECODE, DEPLOYED_BYTECODE, deploy, deploy_builder};

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::erc1967::ERC1967ProxyInstance;
        use alloy::{
            primitives::{B256, Bytes, U256, b256},
            providers::ProviderBuilder,
        };
        use futures::StreamExt;

        const ROOT: B256 =
            b256!("5cd5c6763b9f2b3fb1cd66a15fe92b7ac913eec295d9927886e175f144ce3308");

        #[tokio::test]
        async fn test() -> eyre::Result<()> {
            let provider = ProviderBuilder::new().connect_anvil_with_wallet();
            let imp = deploy(&provider).await?;
            let proxy =
                ERC1967ProxyInstance::deploy(&provider, *imp.address(), Bytes::new()).await?;
            let uts = UniversalTimestamps::new(*proxy.address(), &provider);

            let attested_log = uts.Attested_filter().watch().await?;

            let _ = uts.attest(ROOT).send().await?.watch().await?;

            let timestamp = uts.timestamp(ROOT).call().await?;
            assert_ne!(timestamp, U256::ZERO);

            let (attested, _log) = attested_log.into_stream().next().await.unwrap()?;
            assert_eq!(attested.root, ROOT);

            Ok(())
        }
    }
}

/// ERC-1967 Proxy contract
#[cfg(any(test, feature = "erc1967"))]
pub mod erc1967 {
    mod binding {
        use alloy_sol_types::sol;

        sol!(
            #[sol(rpc)]
            ERC1967Proxy,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../target/foundry/ERC1967Proxy.sol/ERC1967Proxy.json"
            )
        );
    }

    pub use binding::ERC1967Proxy::*;
}
