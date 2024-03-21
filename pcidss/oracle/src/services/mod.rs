use std::{str::FromStr, sync::Arc};

use deadpool_postgres::Pool;
use op_api::{bank_account::PgBankAccount, transaction::PgTransaction};
use op_core::{bank_account::traits::BankAccountTrait, transaction::traits::TransactionTrait};
use subxt::{OnlineClient, SubstrateConfig};
use subxt_signer::{
	sr25519::{Keypair, PublicKey},
	SecretUri,
};

use crate::cli::Cli;

use self::processor::Iso8583MessageProcessor;

pub mod processor;
pub mod rpc;
pub mod watcher;

/// Start the suite of services for the oracle
///
/// 1. Start the ISO8583 message processor
/// 2. Start the RPC server
/// 3. Start the watcher service
pub async fn start_oracle(args: &Cli, pg_pool: Arc<Pool>) -> anyhow::Result<()> {
	let iso8583_spec = iso8583_rs::iso8583::iso_spec::spec("");

	let bank_account_trait: Arc<dyn BankAccountTrait> =
		Arc::new(PgBankAccount::new(pg_pool.clone()));
	let transaction_trait: Arc<dyn TransactionTrait> =
		Arc::new(PgTransaction::new(pg_pool.clone()));

	// Message processor
	let processor = Arc::new(Iso8583MessageProcessor {
		spec: iso8583_spec,
		bank_account_controller: bank_account_trait.clone(),
		transaction_controller: transaction_trait.clone(),
	});

	let args = args.clone();

	let client = Arc::new(
		OnlineClient::<SubstrateConfig>::from_url(&args.ws_url)
			.await
			.map_err(|_| format!("Could not connect to Substrate node at: {}", args.ws_url))
			.unwrap(),
	);

	let seed = SecretUri::from_str(&args.seed).unwrap();
	let keypair = Keypair::from_uri(&seed).map_err(|_| "Invalid seed phrase").unwrap();
	log::info!("Using keypair: {:?}", hex::encode(&keypair.public_key()));

	// spawn the RPC server
	tokio::spawn({
		let processor = Arc::clone(&processor);
		let client = Arc::clone(&client);
		let keypair = keypair.clone();
		async move {
			let ocw_signer = PublicKey(
				hex::decode(&args.ocw_signer).unwrap().try_into().expect("valid public key"),
			);
			let result =
				rpc::run(processor, client, keypair, args.rpc_port, args.dev, ocw_signer).await;
			if result.is_err() {
				log::error!("Could not start RPC: {}", result.unwrap_err().to_string());
				std::process::exit(1)
			}
		}
	});

	// spawn the watcher service
	tokio::spawn({
		let processor = Arc::clone(&processor);
		let client = Arc::clone(&client);
		async move {
			let watcher = watcher::WatcherService::new(keypair, processor, client).await.unwrap();
			let result = watcher.start().await;
			if result.is_err() {
				log::error!("Could not start watcher: {}", result.unwrap_err().to_string());
				std::process::exit(1)
			}
		}
	});

	Ok(())
}
