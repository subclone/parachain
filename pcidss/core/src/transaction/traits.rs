//! Defines traits
use async_trait::async_trait;
use uuid::Uuid;

use super::models::{Transaction, TransactionCreate};
use crate::error::DomainError;

/// `TransactionTrait` is a trait for transaction operations.
///
/// This should be implemented by any transaction controller.
#[async_trait]
pub trait TransactionTrait: Send + Sync {
	/// Find a transaction by unique identifier.
	async fn find_by_id(&self, id: &Uuid) -> Result<Option<Transaction>, DomainError>;

	/// Find by beneficiary.
	async fn find_by_bank_account_id(&self, source: &Uuid) -> Result<Vec<Transaction>, DomainError>;

	/// Find a transaction by hash.
	async fn find_by_hash(&self, hash: &str) -> Result<Option<Transaction>, DomainError>;

	/// Create a new transaction.
	async fn create(&self, transaction: &TransactionCreate) -> Result<Transaction, DomainError>;

	/// Update a transaction.
	/// Mainly used to update `reversed` field.
	async fn update(&self, id: &Uuid) -> Result<Transaction, DomainError>;
}
