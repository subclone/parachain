//! Models to represent a bank account and its operations.

use chrono::{DateTime, Months, Utc};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::{error::DomainError, types::TransactionType};

/// `BankAccountCreate` is a model for creating a bank account.
#[derive(Debug, Clone)]
pub struct BankAccountCreate {
    /// Unique identifier of the bank account.
    pub id: Uuid,
    /// Card number linked to the bank account, should be 16 digits.
    pub card_number: String,
    /// Card holder first name.
    pub card_holder_first_name: String,
    /// Card holder last name.
    pub card_holder_last_name: String,
    /// Card expiration date.
    pub card_expiration_date: DateTime<Utc>,
    /// Card CVV.
    pub card_cvv: String,
    /// Balance of the bank account, can be set in test mode.
    pub balance: u32,
}

impl BankAccountCreate {
    /// Creates a new `BankAccountCreate`.
    pub fn new(
        card_number: String,
        card_holder_first_name: String,
        card_holder_last_name: String,
        card_cvv: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            card_number,
            card_holder_first_name,
            // expiration date is 4 years from now
            card_expiration_date: Utc::now()
                .checked_add_months(Months::new(48))
                .expect("valid date"),
            card_holder_last_name,
            card_cvv,
            balance: 0,
        }
    }
}

/// `BankAccountUpdate` is a model for updating a bank account.
#[derive(Debug, Clone)]
pub enum BankAccountUpdate {
    /// Balance update.
    Balance {
        /// Unique identifier of the bank account.
        id: Uuid,
        /// Amount of change to the balance.
        amount: u32,
        /// Type of change to the balance.
        transaction_type: TransactionType,
    },
    /// Update bank account info.
    Info {
        /// Unique identifier of the bank account.
        id: Uuid,
        /// AccountId on the blockchain.
        account_id: Option<String>,
    },
}

/// Extremely simplified, dummy version of a bank account model.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BankAccount {
    /// Unique identifier of the bank account.
    pub id: Uuid,
    /// Card number linked to the bank account, should be 16 digits.
    pub card_number: String,
    /// Card holder first name.
    pub card_holder_first_name: String,
    /// Card holder last name.
    pub card_holder_last_name: String,
    /// Card expiration date.
    pub card_expiration_date: DateTime<Utc>,
    /// Card CVV.
    pub card_cvv: String,
    /// Balance of the bank account.
    pub balance: u32,
    /// Nonce of the bank account.
    pub nonce: u32,
    /// Account ID on the blockchain.
    pub account_id: Option<String>,
}

impl BankAccount {
    /// Creates a new `BankAccount`.
    pub fn new(
        card_number: String,
        card_holder_first_name: String,
        card_holder_last_name: String,
        card_expiration_date: DateTime<Utc>,
        card_cvv: String,
        balance: u32,
        nonce: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            card_number,
            card_holder_first_name,
            card_holder_last_name,
            card_expiration_date,
            card_cvv,
            balance,
            nonce,
            account_id: None,
        }
    }

    /// Try updating bank account balance
    ///
    /// Simple balance update, no transaction history.
    pub async fn try_update(
        &mut self,
        bank_account_update: &BankAccountUpdate,
    ) -> Result<(), DomainError> {
        match bank_account_update {
            BankAccountUpdate::Balance {
                id: _id,
                amount,
                transaction_type,
            } => {
                self.balance = match transaction_type {
                    TransactionType::Debit => self.balance.checked_add(amount.clone()),
                    TransactionType::Credit => self.balance.checked_sub(amount.clone()),
                }
                .ok_or(DomainError::ApiError(String::from(
                    "Arithmetic underflow/overflow",
                )))?;

                self.nonce =
                    self.nonce
                        .checked_add(1)
                        .ok_or(DomainError::ApiError(String::from(
                            "Arithmetic underflow/overflow",
                        )))?;

                Ok(())
            }
            BankAccountUpdate::Info { id: _, account_id } => {
                self.account_id = account_id.clone();
                Ok(())
            }
        }
    }
}

/// Implement `From` trait for `BankAccount` from `Row`.
/// Helps with parsing database results.
impl From<&Row> for BankAccount {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            card_holder_first_name: row.get("card_holder_first_name"),
            card_holder_last_name: row.get("card_holder_last_name"),
            card_cvv: row.get("card_cvv"),
            card_expiration_date: row.get("card_expiration_date"),
            card_number: row.get("card_number"),
            balance: row.get::<&str, i32>("balance") as u32,
            nonce: row.get::<&str, i32>("nonce") as u32,
            account_id: row.get("account_id"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_successful_debit() {
        let mut bank_account = BankAccount::new(
            "1234123412341234".to_string(),
            "Alice".to_string(),
            "Smith".to_string(),
            Utc::now(),
            "123".to_string(),
            1000,
            0,
        );

        let update = BankAccountUpdate::Balance {
            id: Uuid::new_v4(),
            transaction_type: TransactionType::Debit,
            amount: 500,
        };

        bank_account
            .try_update(&update)
            .await
            .expect("Debit failed");
        assert_eq!(bank_account.balance, 1500);
        assert_eq!(bank_account.nonce, 1);
    }

    #[tokio::test]
    async fn test_successful_credit() {
        let mut bank_account = BankAccount::new(
            "1234123412341234".to_string(),
            "Alice".to_string(),
            "Smith".to_string(),
            Utc::now(),
            "123".to_string(),
            1000,
            0,
        );

        let update = BankAccountUpdate::Balance {
            id: Uuid::new_v4(),
            transaction_type: TransactionType::Credit,
            amount: 500,
        };

        bank_account
            .try_update(&update)
            .await
            .expect("Credit failed");
        assert_eq!(bank_account.balance, 500);
        assert_eq!(bank_account.nonce, 1);
    }

    #[tokio::test]
    async fn test_arithmetic_overflow_balance() {
        let mut bank_account = BankAccount::new(
            "1234123412341234".to_string(),
            "Alice".to_string(),
            "Smith".to_string(),
            Utc::now(),
            "123".to_string(),
            1000,
            0,
        );

        let update = BankAccountUpdate::Balance {
            id: Uuid::new_v4(),
            transaction_type: TransactionType::Credit,
            amount: 2000, // More than the available balance
        };

        match bank_account.try_update(&update).await {
            Ok(_) => panic!("Expected an error due to arithmetic overflow"),
            Err(e) => assert_eq!(
                e,
                DomainError::ApiError(String::from("Arithmetic underflow/overflow"))
            ),
        }
    }

    #[tokio::test]
    async fn test_arithmetic_overflow_nonce() {
        let mut bank_account = BankAccount::new(
            "1234123412341234".to_string(),
            "Alice".to_string(),
            "Smith".to_string(),
            Utc::now(),
            "123".to_string(),
            1000,
            u32::MAX,
        );

        let update = BankAccountUpdate::Balance {
            id: Uuid::new_v4(),
            transaction_type: TransactionType::Debit,
            amount: 500,
        };

        match bank_account.try_update(&update).await {
            Ok(_) => panic!("Expected an error due to arithmetic overflow"),
            Err(e) => assert_eq!(
                e,
                DomainError::ApiError(String::from("Arithmetic underflow/overflow"))
            ),
        }
    }
    #[tokio::test]
    async fn test_info_update() {
        let mut bank_account = BankAccount::new(
            "1234123412341234".to_string(),
            "Alice".to_string(),
            "Smith".to_string(),
            Utc::now(),
            "123".to_string(),
            1000,
            0,
        );

        let update = BankAccountUpdate::Info {
            id: Uuid::new_v4(),
            account_id: Some("1234".to_string()),
        };

        bank_account
            .try_update(&update)
            .await
            .expect("Info update failed");
        assert_eq!(bank_account.account_id, Some("1234".to_string()));
    }
}
