//! End-to-end tests for PCIDSS oracle and client.

#![cfg(test)]

use jsonrpsee::core::client::ClientT;
use op_core::bank_account::models::BankAccount;
use std::{str::FromStr, sync::Arc};
use subxt::{config::substrate::H256, utils::AccountId32, OnlineClient, SubstrateConfig};

const SEED: &str = "intact start solar kind young network dizzy churn crisp custom fuel fabric";
const CHARLIE: &str = "90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22";
const DAVE: &str = "306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20";

type Oracle = jsonrpsee::ws_client::WsClient;
type Substrate = OnlineClient<SubstrateConfig>;

#[subxt::subxt(runtime_metadata_path = "../oracle/iso8583-chain.scale")]
pub mod iso_8583_chain {}

struct TestEnv {
	oracle: Arc<Oracle>,
	substrate: Arc<Substrate>,
	keypair: subxt_signer::sr25519::Keypair,
}

impl TestEnv {
	async fn new() -> Self {
		let chain = Arc::new(
			OnlineClient::<SubstrateConfig>::from_url("ws://localhost:9944")
				.await
				.expect("Could not connect to Substrate node at: ws://localhost:9944"),
		);

		let oracle = Arc::new(
			jsonrpsee::ws_client::WsClientBuilder::new()
				.build("ws://localhost:3030")
				.await
				.expect("Could not connect to Oracle at: ws://localhost:3030"),
		);

		let seed = subxt_signer::SecretUri::from_str(SEED).unwrap();
		let keypair = subxt_signer::sr25519::Keypair::from_uri(&seed).expect("Invalid seed phrase");

		Self { oracle, substrate: chain, keypair }
	}
}

/// Simply append 6 zeros to the balance
fn format_balance(balance: u32) -> u32 {
	balance * 1_000_000
}

#[tokio::test]
async fn test_full_lifecycle() {
	let env = TestEnv::new().await;
	let charlie = hex::decode(CHARLIE).unwrap();

	// get initial balance
	let initial_bank_account: BankAccount = env
		.oracle
		.request("pcidss_get_bank_account", [hex::encode(env.keypair.public_key().0)])
		.await
		.expect("ok");

	let balance_query =
		iso_8583_chain::storage().system().account(&env.keypair.public_key().into());

	// check on-chain balance
	let initial_on_chain_account = env
		.substrate
		.storage()
		.at_latest()
		.await
		.unwrap()
		.fetch(&balance_query)
		.await
		.unwrap()
		.unwrap();

	assert_eq!(
		initial_on_chain_account.data.free as u32,
		format_balance(initial_bank_account.balance)
	);

	// initiate transfer
	let transfer = iso_8583_chain::tx().iso8583().initiate_transfer(
		AccountId32(env.keypair.public_key().0),
		AccountId32(charlie.try_into().expect("ok")),
		1_000_0000u128.into(),
	);

	let result = env.substrate.tx().sign_and_submit_default(&transfer, &env.keypair).await;
	assert!(result.is_ok());

	'outer: while let Some(block) =
		env.substrate.blocks().subscribe_finalized().await.unwrap().next().await
	{
		let block = block.unwrap();
		for event in block.events().await.unwrap().iter() {
			let event = event.unwrap();

			if event.pallet_name() == "ISO8583" &&
				event.variant_name().contains("ProcessedTransaction")
			{
				break 'outer;
			}
		}
	}

	// check on-chain balance
	let on_chain_account = env
		.substrate
		.storage()
		.at_latest()
		.await
		.unwrap()
		.fetch(&balance_query)
		.await
		.unwrap()
		.unwrap();

	println!("on_chain_account.data.free: {:?}", on_chain_account.data.free);
	assert_eq!(
		on_chain_account.data.free as u32,
		format_balance(initial_bank_account.balance) - 1_000_0000u128 as u32
	);

	// get list of transactions
	let transactions: Vec<op_core::transaction::models::Transaction> = env
		.oracle
		.request("pcidss_get_transactions", [hex::encode(env.keypair.public_key().0)])
		.await
		.expect("ok");

	// revert transfer

	let revert = iso_8583_chain::tx()
		.iso8583()
		.initiate_revert(H256::from_str(&transactions[0].hash).unwrap());

	let result = env.substrate.tx().sign_and_submit_default(&revert, &env.keypair).await;

	assert!(result.is_ok());

	// wait for `ProcessedTransaction` event
	'outer: while let Some(block) =
		env.substrate.blocks().subscribe_finalized().await.unwrap().next().await
	{
		let block = block.unwrap();
		for event in block.events().await.unwrap().iter() {
			let event = event.unwrap();
			if event.pallet_name() == "ISO8583" && event.variant_name() == "ProcessedTransaction" {
				break 'outer;
			}
		}
	}

	// check balance
	let bank_account: BankAccount = env
		.oracle
		.request("pcidss_get_bank_account", [hex::encode(env.keypair.public_key().0)])
		.await
		.expect("ok");

	assert_eq!(format_balance(bank_account.balance), initial_on_chain_account.data.free as u32);
}
