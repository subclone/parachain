//! PCIDSS Gateway entry point.
use async_trait::async_trait;
use chrono::{Months, Utc};
use deadpool_postgres::Pool;
use iso8583_rs::iso8583::iso_spec::IsoMsg;
use jsonrpsee::server::Server;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use jsonrpsee_types::error::ErrorCode;
use log::info;
use op_api::{bank_account::PgBankAccount, transaction::PgTransaction};
use op_core::bank_account::models::{BankAccount, BankAccountCreate};
use op_core::transaction::models::Transaction;
use op_core::{
    bank_account::traits::BankAccountTrait, error::DomainError,
    transaction::traits::TransactionTrait,
};
use std::{error::Error, net::SocketAddr, sync::Arc};

use crate::constants::DEV_ACCOUNTS;
use crate::iso8583::Iso8583MessageProcessor;

/// PCIDSS Compliant Oracle RPC API
#[rpc(server, client, namespace = "pcidss")]
pub trait OracleApi<IsoMsg> {
    /// Submit ISO8583 message for processing
    #[method(name = "submit_iso8583")]
    async fn submit_iso8583(&self, iso_msg: Vec<u8>) -> RpcResult<Vec<u8>>;

    /// Get transactions by card number
    #[method(name = "get_transactions")]
    async fn get_transactions(&self, card_number: String) -> RpcResult<Vec<Transaction>>;

    /// Get bank account by card number
    #[method(name = "get_bank_account")]
    async fn get_bank_account(&self, card_number: String) -> RpcResult<BankAccount>;
}

/// PCIDSS Compliant Oracle RPC API implementation
pub struct OracleApiImpl {
    /// ISO8583 message processor
    pub processor: Arc<Iso8583MessageProcessor>,
}

#[async_trait]
impl OracleApiServer<IsoMsg> for OracleApiImpl {
    async fn submit_iso8583(&self, iso_msg: Vec<u8>) -> RpcResult<Vec<u8>> {
        log::debug!("Received ISO8583 message: {:?}", iso_msg);

        let mut iso_msg = iso_msg;

        match self.processor.process(&mut iso_msg).await {
            Ok(result) => {
                log::info!("Processed ISO8583 message: {:?}", result.0);
                Ok(result.0)
            }
            Err(err) => {
                log::error!("Failed to process ISO8583 message: {:?}", err.to_string());
                let error_code = match err {
                    DomainError::ApiError(_) => ErrorCode::InternalError,
                    DomainError::InternalServerError(_) => ErrorCode::InternalError,
                    DomainError::BadRequest(_) => ErrorCode::InvalidParams,
                    DomainError::NotFound(_) => ErrorCode::InvalidParams,
                };

                Err(error_code.into())
            }
        }
    }

    async fn get_transactions(&self, card_number: String) -> RpcResult<Vec<Transaction>> {
        log::debug!("Received get_transactions request: {:?}", card_number);

        let bank_account = self
            .processor
            .bank_account_controller
            .find_by_card_number(&card_number)
            .await
            .map_err(|_| ErrorCode::InvalidParams)?
            .ok_or(ErrorCode::InvalidParams)?;

        self.processor
            .transaction_controller
            .find_by_beneficiary(&bank_account.id)
            .await
            .map_err(|err| {
                let error_code = match err {
                    DomainError::ApiError(_) => ErrorCode::InternalError,
                    DomainError::InternalServerError(_) => ErrorCode::InternalError,
                    DomainError::BadRequest(_) => ErrorCode::InvalidParams,
                    DomainError::NotFound(_) => ErrorCode::InvalidParams,
                };

                error_code.into()
            })
    }

    async fn get_bank_account(&self, card_number: String) -> RpcResult<BankAccount> {
        log::debug!("Received get_bank_account request: {:?}", card_number);

        let ba = self
            .processor
            .bank_account_controller
            .find_by_card_number(&card_number)
            .await
            .map_err(|e| {
                log::debug!("Error: {:?}", e);
                ErrorCode::InvalidParams
            })?;

        ba.ok_or(ErrorCode::InvalidParams.into())
    }
}

/// Run ISO8583 Message Processor
pub async fn run(
    rpc_port: u16,
    pg_pool: Arc<Pool>,
    dev_mode: bool,
) -> anyhow::Result<(), Box<dyn Error>> {
    log::info!("Starting ISO8583 processor");

    // Prepare ISO8583 processor
    let iso8583_spec = iso8583_rs::iso8583::iso_spec::spec("");

    let bank_account_trait: Arc<dyn BankAccountTrait> =
        Arc::new(PgBankAccount::new(pg_pool.clone()));
    let transaction_trait: Arc<dyn TransactionTrait> =
        Arc::new(PgTransaction::new(pg_pool.clone()));

    // Message processor
    let processor = Iso8583MessageProcessor {
        spec: iso8583_spec,
        bank_account_controller: bank_account_trait.clone(),
        transaction_controller: transaction_trait.clone(),
    };

    if dev_mode {
        info!("Running in dev mode, inserting dev accounts");
        // insert dev accounts
        for account in DEV_ACCOUNTS.iter() {
            let expiration_date = if account.0 != "Eve" {
                Utc::now()
                    .checked_add_months(Months::new(48))
                    .expect("valid date")
            } else {
                Utc::now()
                    .checked_sub_months(Months::new(2))
                    .expect("safe; qed")
            };

            let bank_account_create = BankAccountCreate {
                id: uuid::Uuid::new_v4(),
                card_number: account.1.to_string(),
                card_holder_first_name: account.0.to_string(),
                card_holder_last_name: account.0.to_string(),
                card_cvv: account.2.to_string(),
                card_expiration_date: expiration_date,
                balance: account.3,
            };

            let bank_account = processor
                .bank_account_controller
                .create(&bank_account_create)
                .await;

            if let Ok(bank_account) = bank_account {
                assert_eq!(bank_account.card_number, account.1);
                assert_eq!(bank_account.balance, account.3);
                assert_eq!(bank_account.nonce, 0);
                info!("Inserted dev account: {:?}", bank_account);
            }
        }
    }

    // Run RPC server
    let addr = run_server(Arc::new(processor), rpc_port).await?;
    let url = format!("ws://{}", addr);

    log::info!("RPC server listening on {}", url);

    Ok(())
}

/// Run RPC server for ISO8583 Message Processor
async fn run_server(
    processor: Arc<Iso8583MessageProcessor>,
    rpc_port: u16,
) -> anyhow::Result<SocketAddr> {
    let server = Server::builder()
        .build(format!("0.0.0.0:{}", rpc_port))
        .await?;

    let addr = server.local_addr()?;
    let oracle_impl = OracleApiImpl { processor };

    let server_handle = server.start(oracle_impl.into_rpc());

    log::info!("Starting RPC server");
    // Wait for the server to start listening.
    tokio::spawn(server_handle.stopped());

    Ok(addr)
}
