//! Unit tests (Substrate style)
#[cfg(test)]
mod mock;
mod payment;
mod register;
mod reversal;

#[cfg(test)]
mod prelude {
	use chrono::Months;
	use iso8583_rs::iso8583::iso_spec::{new_msg, IsoMsg, Spec};
	use op_core::{bank_account::models::BankAccount, transaction::models::Transaction};
	use uuid::Uuid;

	use super::mock::MockProcessorImpl;
	use crate::types::{constants::DEV_ACCOUNTS, DevAccount, ResponseCodes, MTI};

	pub const ALICE: DevAccount = DEV_ACCOUNTS[0];
	pub const _BOB: DevAccount = DEV_ACCOUNTS[1];
	pub const CHARLIE: DevAccount = DEV_ACCOUNTS[2];
	pub const DAVE: DevAccount = DEV_ACCOUNTS[3];
	pub const EVE: DevAccount = DEV_ACCOUNTS[4];
	pub const ACQUIRER: DevAccount = DEV_ACCOUNTS[5];

	/// Get bank account by card number
	pub(crate) async fn get_bank_account_by_card_number(
		api: &MockProcessorImpl,
		card_number: &str,
	) -> BankAccount {
		api.processor
			.bank_account_controller
			.find_by_card_number(card_number)
			.await
			.unwrap()
			.unwrap()
	}

	/// Get transactions by card number
	pub(crate) async fn get_transactions_by_id(
		api: &MockProcessorImpl,
		id: &Uuid,
	) -> Vec<Transaction> {
		api.processor.transaction_controller.find_by_beneficiary(id).await.unwrap()
	}

	/// Creates new mock ISO-8583 message
	///
	/// # Cases
	/// * Eve's card is expired
	/// * Bob and Dave have zero balance
	/// * Alice and Charlie have healthy accounts
	///
	/// # Arguments
	///
	/// * `spec` - ISO-8583 specification
	/// * `mti` - Message type indicator
	/// * `account` - Dev account
	pub(crate) fn get_new_iso_msg(spec: &'static Spec, mti: MTI, account: DevAccount) -> IsoMsg {
		let mut msg = new_msg(spec, spec.get_message_from_header(mti.clone().into()).unwrap());
		let (name, card_number, cvv, _) = account;

		msg.set("message_type", mti.into()).unwrap();

		msg.set_on(2, &card_number).unwrap();
		// processing code
		msg.set_on(3, "000000").unwrap();

		let now = chrono::Utc::now();

		// transmission date and time
		msg.set_on(7, &format!("{}", now.format("%m%d%H%M%S"))).unwrap();

		// time date
		msg.set_on(12, &format!("{}", now.format("%H%M%S"))).unwrap();

		// card expiration date
		let exp_date = if name == "Eve" {
			now.checked_sub_months(Months::new(2))
				.expect("safe; qed")
				.format("%m%y")
				.to_string()
		} else {
			now.checked_add_months(Months::new(48))
				.expect("valid date")
				.format("%m%y")
				.to_string()
		};

		msg.set_on(32, "123456").unwrap();
		msg.set_on(35, &format!("{}D{}C{}", card_number, exp_date, cvv)).unwrap();
		msg.set_on(126, &"0".repeat(99)).unwrap();

		msg
	}

	/// Assert ISO-8583 message processing failed with given Response Code
	/// and storage has not been altered
	pub(crate) async fn assert_noop(
		api: &MockProcessorImpl,
		beneficiary: DevAccount,
		iso_msg: &IsoMsg,
		response_code: ResponseCodes,
		previous_account_state: BankAccount,
		previous_txs: Vec<Transaction>,
	) {
		let mut msg_raw = iso_msg.assemble().unwrap();
		let msg_response = api.processor.process(&mut msg_raw).await.unwrap();

		assert_eq!(&msg_response.1.bmp_child_value(39).unwrap(), Into::<&str>::into(response_code),);

		let beneficiary_account = get_bank_account_by_card_number(api, &beneficiary.1).await;

		assert_eq!(beneficiary_account.balance, previous_account_state.balance);

		let beneficiary_txs = api
			.processor
			.transaction_controller
			.find_by_beneficiary(&beneficiary_account.id)
			.await
			.unwrap();

		for (i, tx) in beneficiary_txs.iter().enumerate() {
			assert_eq!(tx, &previous_txs[i]);
		}
	}
}
