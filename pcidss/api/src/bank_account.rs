//! Defines the [`PgBankAccount`] type and its traits.
use async_trait::async_trait;
use deadpool_postgres::Pool;
use std::sync::Arc;
use uuid::Uuid;

use op_core::{
	bank_account::{
		models::{BankAccount, BankAccountCreate, BankAccountUpdate},
		traits::BankAccountTrait,
	},
	error::DomainError,
};

/// Type that will be used to interact with the database.
pub struct PgBankAccount {
	pool: Arc<Pool>,
}

impl PgBankAccount {
	pub fn new(pool: Arc<Pool>) -> Self {
		Self { pool }
	}
}

#[async_trait]
impl BankAccountTrait for PgBankAccount {
	async fn find_by_id(&self, id: &Uuid) -> Result<Option<BankAccount>, DomainError> {
		let client = self.pool.get().await?;
		let stmt = client.prepare(r#"SELECT * FROM bank_account WHERE id = $1;"#).await?;

		if let Some(result) = client.query_opt(&stmt, &[&id]).await? {
			return Ok(Some((&result).into()));
		}

		Ok(None)
	}

	async fn find_by_card_number(
		&self,
		card_number: &str,
	) -> Result<Option<BankAccount>, DomainError> {
		let client = self.pool.get().await?;
		let stmt = client.prepare(r#"SELECT * FROM bank_account WHERE card_number = $1;"#).await?;

		if let Some(result) = client.query_opt(&stmt, &[&card_number]).await? {
			return Ok(Some((&result).into()));
		}

		Ok(None)
	}

	async fn update(
		&self,
		id: &Uuid,
		bank_account_update: &BankAccountUpdate,
	) -> Result<BankAccount, DomainError> {
		let client = self.pool.get().await?;

		let mut bank_account = self
			.find_by_id(id)
			.await?
			.ok_or(DomainError::NotFound("Bank account not found".to_string()))?;

		bank_account.try_update(bank_account_update).await?;

		let stmt = client
            .prepare(
                r#"UPDATE bank_account SET balance = $1, nonce = $2, account_id = $3, updated_at = $4 WHERE id = $5 RETURNING *;"#,
            )
            .await?;

		let result = client
			.query_one(
				&stmt,
				&[
					&(bank_account.balance as i32),
					&(bank_account.nonce as i32),
					&bank_account.account_id,
					&chrono::Utc::now(),
					&id,
				],
			)
			.await?;

		Ok((&result).into())
	}

	async fn create(
		&self,
		bank_account_create: &BankAccountCreate,
	) -> Result<BankAccount, DomainError> {
		let client = self.pool.get().await?;

		let query_string = if let Some(_) = &bank_account_create.account_id {
			r#"INSERT INTO bank_account (id, card_number, card_holder_first_name, card_holder_last_name, card_expiration_date, card_cvv, balance, nonce, account_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *;"#
		} else {
			r#"INSERT INTO bank_account (id, card_number, card_holder_first_name, card_holder_last_name, card_expiration_date, card_cvv, balance, nonce) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *;"#
		};

		let stmt = client.prepare(query_string).await?;

		let row = &client
			.query_one(
				&stmt,
				&[
					&bank_account_create.id,
					&bank_account_create.card_number,
					&bank_account_create.card_holder_first_name,
					&bank_account_create.card_holder_last_name,
					&bank_account_create.card_expiration_date,
					&bank_account_create.card_cvv,
					&(bank_account_create.balance as i32), // Initial balance is 0
					&0_i32,                                // Initial nonce is 0
					&bank_account_create.account_id,
				],
			)
			.await?;

		Ok((row).into())
	}

	async fn delete(&self, id: &Uuid) -> Result<(), DomainError> {
		let client = self.pool.get().await?;
		let stmt = client.prepare("DELETE FROM bank_account WHERE id = $1;").await?;
		client.execute(&stmt, &[&id]).await?;
		Ok(())
	}

	async fn find_by_account_id(
		&self,
		on_chain_account_id: &str,
	) -> Result<Option<BankAccount>, DomainError> {
		let client = self.pool.get().await?;
		let stmt = client.prepare(r#"SELECT * FROM bank_account WHERE account_id = $1;"#).await?;

		if let Some(result) = client.query_opt(&stmt, &[&on_chain_account_id]).await? {
			return Ok(Some((&result).into()));
		}

		Ok(None)
	}
}
