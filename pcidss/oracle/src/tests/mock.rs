//! Mock implementation of the Oracle API server.

use std::sync::Arc;

use chrono::{Months, Utc};
use op_api::{bank_account::PgBankAccount, transaction::PgTransaction};
use op_core::{
	bank_account::{models::BankAccountCreate, traits::BankAccountTrait},
	postgres::mock_init,
	transaction::traits::TransactionTrait,
};
use subxt::{OnlineClient, SubstrateConfig};

use crate::{
	services::{processor::Iso8583MessageProcessor, watcher::WatcherService},
	types::constants::DEV_ACCOUNTS,
};

#[subxt::subxt(runtime_metadata_path = "./iso-8583-chain.scale")]
pub mod iso_8583_chain {}

/// Mock implementation of the Oracle API server.
#[derive(Clone)]
pub struct MockProcessorImpl {
	/// ISO8583 message processor
	pub processor: Arc<Iso8583MessageProcessor>,
}

impl MockProcessorImpl {
	/// Creates a new instance of the mock processor
	pub async fn new(db_name: Option<String>) -> Self {
		let pg_pool = mock_init(db_name.unwrap_or("mockdb".to_string()))
			.await
			.expect("Error to init database to tests");
		let pg_pool = Arc::new(pg_pool);

		let bank_account_trait: Arc<dyn BankAccountTrait> =
			Arc::new(PgBankAccount::new(pg_pool.clone()));
		let transaction_trait: Arc<dyn TransactionTrait> =
			Arc::new(PgTransaction::new(pg_pool.clone()));

		std::env::set_var("SPEC_FILE", "./src/tests/test_spec.yaml");

		let iso8583_spec = iso8583_rs::iso8583::iso_spec::spec("");

		let processor = Iso8583MessageProcessor {
			spec: iso8583_spec,
			bank_account_controller: bank_account_trait,
			transaction_controller: transaction_trait,
		};

		// insert dev accounts
		for account in DEV_ACCOUNTS.iter() {
			let expiration_date = if account.0 != "Eve" {
				Utc::now().checked_add_months(Months::new(48)).expect("valid date")
			} else {
				Utc::now().checked_sub_months(Months::new(2)).expect("safe; qed")
			};

			let bank_account_create = BankAccountCreate {
				id: uuid::Uuid::new_v4(),
				card_number: account.1.to_string(),
				card_holder_first_name: account.0.to_string(),
				card_holder_last_name: account.0.to_string(),
				card_cvv: account.2.to_string(),
				card_expiration_date: expiration_date,
				balance: account.3,
				account_id: Some(account.4.trim_start_matches("0x").to_string()),
			};

			let bank_account =
				processor.bank_account_controller.create(&bank_account_create).await.unwrap();

			assert_eq!(bank_account.card_number, account.1);
			assert_eq!(bank_account.balance, account.3);
		}

		Self { processor: Arc::new(processor) }
	}
}

/// Assert an expression returns an error specified.
///
/// Used as `assert_err!(expression_to_assert, expected_error_expression)`
#[macro_export]
macro_rules! assert_err {
	( $x:expr , $y:expr $(,)? ) => {
		assert_eq!($x, Err($y.into()));
	};
}
