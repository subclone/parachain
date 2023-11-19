//! Watcher service subscribes to Substrate chain to maintain constant sync between the chain and the oracle
use subxt::{OnlineClient, PolkadotConfig};

#[subxt::subxt(runtime_metadata_path = "../artifacts/polkadot_metadata_small.scale")]
pub mod polkadot {}

async fn watcher() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
