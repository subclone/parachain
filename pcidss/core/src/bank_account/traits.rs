//! Defines trait for bank account operations.

use async_trait::async_trait;
use uuid::Uuid;

use super::models::{BankAccount, BankAccountCreate, BankAccountUpdate};
use crate::error::DomainError;

/// `BankAccountTrait` is a trait for bank account operations.
///
/// This should be implemented by any bank account controller.
#[async_trait]
pub trait BankAccountTrait: Send + Sync {
    /// Find a bank account by unique identifier.
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<BankAccount>, DomainError>;

    /// Find a bank account by card number.
    async fn find_by_card_number(
        &self,
        card_number: &str,
    ) -> Result<Option<BankAccount>, DomainError>;

    /// Create a new bank account.
    async fn create(
        &self,
        bank_account_create: &BankAccountCreate,
    ) -> Result<BankAccount, DomainError>;

    /// Update a bank account by unique identifier.
    async fn update(
        &self,
        id: &Uuid,
        bank_account_update: &BankAccountUpdate,
    ) -> Result<BankAccount, DomainError>;

    /// Delete a bank account by unique identifier.
    async fn delete(&self, id: &Uuid) -> Result<(), DomainError>;
}
