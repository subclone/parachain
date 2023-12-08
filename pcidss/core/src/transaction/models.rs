//! Models to represent a transaction and its operations.
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{bank_account::models::BankAccountUpdate, types::TransactionType};

#[derive(Debug, Clone)]
pub struct TransactionCreate {
	/// Unique identifier of the bank account.
	pub id: Uuid,
	/// Unique identifier of the sender bank account.
	pub from: Uuid,
	/// Unique identifier of the receiving bank account, if any.
	pub to: Option<Uuid>,
	/// Amount of the transaction.
	pub amount: u32,
	/// Type of the transaction.
	pub transaction_type: TransactionType,
	/// Nonce of the transaction.
	pub nonce: u32,
	/// Raw ISO message of the transaction.
	pub iso_msg_raw: Vec<u8>,
	/// On-chain id of the transaction, it is `block_number` - `event_index`.
	pub on_chain_id: Option<String>,
}

/// `Transaction` is a model for a transaction.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
	/// Unique identifier of the transaction.
	pub id: Uuid,
	/// Unique hash of the transaction.
	pub hash: String,
	/// Unique identifier of the bank account.
	pub from: Uuid,
	/// Unique identifier of the receiving bank account, if any.
	pub to: Option<Uuid>,
	/// Amount of the transaction.
	pub amount: u32,
	/// Type of the transaction.
	pub transaction_type: u32,
	/// Is it reversed?
	pub reversed: bool,
	/// On-chain id of the transaction, it is `block_number` - `event_index`.
	pub on_chain_id: Option<String>,
}

impl From<&TransactionCreate> for Transaction {
	fn from(value: &TransactionCreate) -> Self {
		let mut hasher = Sha256::new();
		hasher.update(value.iso_msg_raw.clone());
		hasher.update(value.nonce.to_be_bytes());
		let hash = hasher.finalize();

		Self {
			id: value.id,
			hash: format!("{:x}", hash),
			from: value.from,
			to: value.to,
			amount: value.amount,
			transaction_type: value.transaction_type.clone().into(),
			reversed: false,
			on_chain_id: value.on_chain_id.clone(),
		}
	}
}

impl From<&tokio_postgres::Row> for Transaction {
	fn from(row: &tokio_postgres::Row) -> Self {
		Self {
			id: row.get("id"),
			hash: row.get("hash"),
			from: row.get("beneficiary"),
			to: row.get("recipient"),
			amount: row.get::<&str, i32>("amount") as u32,
			transaction_type: row.get::<&str, i32>("transaction_type") as u32,
			reversed: row.get("reversed"),
			on_chain_id: row.get("on_chain_id"),
		}
	}
}

#[allow(clippy::from_over_into)]
impl Into<BankAccountUpdate> for &Transaction {
	fn into(self) -> BankAccountUpdate {
		BankAccountUpdate::Balance {
			amount: self.amount,
			transaction_type: match self.transaction_type {
				1 => TransactionType::Credit,
				_ => TransactionType::Debit,
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sha2::{Digest, Sha256};

	#[test]
	fn test_conversion_from_transaction_create_to_transaction() {
		let iso_msg_raw = vec![
			48, 49, 49, 48, 242, 28, 64, 1, 34, 224, 128, 0, 0, 0, 0, 0, 0, 0, 0, 6, 49, 54, 52,
			49, 54, 57, 56, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 52, 48, 48, 48, 48, 48, 48, 48,
			48, 48, 48, 48, 48, 49, 48, 48, 48, 48, 48, 48, 57, 49, 50, 50, 51, 49, 52, 49, 54, 50,
			51, 49, 52, 49, 54, 48, 57, 49, 50, 48, 57, 50, 55, 52, 56, 49, 54, 48, 54, 49, 50, 51,
			52, 53, 54, 50, 53, 52, 49, 54, 57, 56, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 52, 68,
			48, 57, 50, 55, 67, 49, 50, 54, 48, 48, 49, 50, 51, 52, 53, 54, 55, 56, 65, 66, 67, 68,
			69, 70, 71, 72, 95, 48, 48, 48, 48, 48, 49, 68, 117, 109, 109, 121, 32, 98, 117, 115,
			105, 110, 101, 115, 115, 32, 110, 97, 109, 101, 44, 32, 68, 117, 109, 109, 121, 32, 67,
			105, 116, 121, 44, 32, 49, 50, 48, 48, 48, 48, 48, 57, 57, 55, 57, 57, 48, 48, 48, 48,
			48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
			48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
			48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
			48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
			48, 48, 48, 48, 48, 48, 48, 52, 57, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49,
			49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49,
			49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49,
		];
		let transaction_create = TransactionCreate {
			id: Uuid::new_v4(),
			iso_msg_raw,
			nonce: 12345,
			from: Uuid::new_v4(),
			to: Some(Uuid::new_v4()),
			amount: 1000,
			transaction_type: TransactionType::Debit,
			on_chain_id: None,
		};

		let transaction: Transaction = (&transaction_create).into();

		// Compute expected hash
		let mut hasher = Sha256::new();
		hasher.update(&transaction_create.iso_msg_raw);
		hasher.update(transaction_create.nonce.to_be_bytes());
		let expected_hash = format!("{:x}", hasher.finalize());

		assert_eq!(transaction.id, transaction_create.id);
		assert_eq!(transaction.hash, expected_hash);
		assert_eq!(transaction.from, transaction_create.from);
		assert_eq!(transaction.to, transaction_create.to);
		assert_eq!(transaction.amount, transaction_create.amount);
	}
}
