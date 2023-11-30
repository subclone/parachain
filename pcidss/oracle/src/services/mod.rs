use std::sync::Arc;

use deadpool_postgres::Pool;
use op_api::{bank_account::PgBankAccount, transaction::PgTransaction};
use op_core::{bank_account::traits::BankAccountTrait, transaction::traits::TransactionTrait};

use crate::cli::Cli;

use self::processor::Iso8583MessageProcessor;

mod processor;
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

	// spawn the RPC server
	tokio::spawn({
		let processor = Arc::clone(&processor);
		async move {
			let result = rpc::run(processor, args.rpc_port, args.dev).await;
			if result.is_err() {
				log::error!("Could not start RPC: {}", result.unwrap_err().to_string());
				std::process::exit(1)
			}
		}
	});

	// start the watcher service
	let _ = watcher::watcher(Arc::clone(&processor)).await;

	Ok(())
}
