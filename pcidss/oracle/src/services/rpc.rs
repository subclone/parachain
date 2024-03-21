//! PCIDSS Gateway entry point.
use async_trait::async_trait;
use chrono::{Months, Utc};
use iso8583_rs::iso8583::iso_spec::IsoMsg;
use jsonrpsee::{core::RpcResult, proc_macros::rpc, server::Server};
use jsonrpsee_types::error::ErrorCode;
use log::info;
use op_core::{
	bank_account::models::{BankAccount, BankAccountCreate},
	error::DomainError,
	transaction::models::Transaction,
};
use std::{error::Error, net::SocketAddr, sync::Arc};
use subxt::{utils::AccountId32, OnlineClient, SubstrateConfig};
use subxt_signer::{sr25519, sr25519::Signature};

use super::processor::Iso8583MessageProcessor;
use crate::{
	services::watcher::iso_8583_chain,
	types::{
		constants::{DEV_ACCOUNTS, RESPONSE_CODE_FIELD_NUMBER},
		MTI,
	},
};

/// PCIDSS Compliant Oracle RPC API
#[rpc(server, client, namespace = "pcidss")]
pub trait OracleApi {
	/// Submit ISO8583 message for processing
	#[method(name = "submit_iso8583")]
	async fn submit_iso8583(&self, iso_msg: Vec<u8>) -> RpcResult<Vec<u8>>;

	/// Get transactions by on-chain account id
	#[method(name = "get_transactions")]
	async fn get_transactions(&self, account_id: String) -> RpcResult<Option<Vec<Transaction>>>;

	/// Get bank account by on-chain account id
	#[method(name = "get_bank_account")]
	async fn get_bank_account(&self, account_id: String) -> RpcResult<Option<BankAccount>>;

	/// Get balance by on-chain account id
	///
	/// Only the OCW can call this method
	#[method(name = "get_batch_balances")]
	async fn get_batch_balances(
		&self,
		signature: Vec<u8>,
		account_ids: Vec<String>,
	) -> RpcResult<Option<Vec<(String, u32)>>>;
}

/// PCIDSS Compliant Oracle RPC API implementation
pub struct OracleApiImpl {
	/// ISO8583 message processor
	pub processor: Arc<Iso8583MessageProcessor>,
	/// Client to interact with the chain
	pub client: Arc<OnlineClient<SubstrateConfig>>,
	/// Oracle signer account
	pub keypair: sr25519::Keypair,
	/// OCW signer account
	pub signer: sr25519::PublicKey,
}

impl OracleApiImpl {
	/// Send a register extrinsic to the chain
	async fn register_on_chain(&self, iso_msg: IsoMsg) {
		let response_code =
			iso_msg.bmp_child_value(RESPONSE_CODE_FIELD_NUMBER).unwrap_or("12".to_string());
		if response_code == "00".to_string() {
			if let Ok(t) = iso_msg.get_field_value(&"message_type".to_string()) {
				let msg_type = t.as_str().try_into().unwrap_or(MTI::AuthorizationRequest);
				if msg_type == MTI::NetworkManagementResponse {
					// send `register` extrinsic to the chain
					if let Ok(account_hex) = iso_msg.bmp_child_value(126) {
						log::debug!("Registering account: {:?}", &account_hex);

						let account = AccountId32(
							hex::decode(account_hex)
								.expect("valid hex; qed") // TODO: safe unwrap
								.try_into()
								.expect("valid; qed"),
						);

						let tx = iso_8583_chain::tx().iso8583().register(account, 0);
						if let Err(e) =
							self.client.tx().sign_and_submit_default(&tx, &self.keypair).await
						{
							log::error!("Failed to submit transaction: {:?}", e);
						}
					}
				}
			}
		}
	}
}

#[async_trait]
impl OracleApiServer for OracleApiImpl {
	async fn submit_iso8583(&self, iso_msg: Vec<u8>) -> RpcResult<Vec<u8>> {
		log::debug!("Received ISO8583 message: {:?}", iso_msg);

		let mut iso_msg = iso_msg;

		match self.processor.process(&mut iso_msg).await {
			Ok((raw_iso_msg, iso_msg)) => {
				log::info!("Processed ISO8583 message: {:?}", raw_iso_msg);
				Self::register_on_chain(&self, iso_msg).await;
				Ok(raw_iso_msg)
			},
			Err(err) => {
				log::error!("Failed to process ISO8583 message: {:?}", err.to_string());
				let error_code = match err {
					DomainError::ApiError(_) => ErrorCode::InternalError,
					DomainError::InternalServerError(_) => ErrorCode::InternalError,
					DomainError::BadRequest(_) => ErrorCode::InvalidParams,
					DomainError::NotFound(_) => ErrorCode::InvalidParams,
				};

				Err(error_code.into())
			},
		}
	}

	async fn get_transactions(&self, account_id: String) -> RpcResult<Option<Vec<Transaction>>> {
		log::debug!("Received get_transactions request: {:?}", account_id);

		let bank_account = self
			.processor
			.bank_account_controller
			.find_by_account_id(&account_id)
			.await
			.map_err(|_| ErrorCode::InvalidParams)?
			.ok_or(ErrorCode::InvalidParams)?;

		let transactions = self
			.processor
			.transaction_controller
			.find_by_bank_account_id(&bank_account.id)
			.await
			.map_err(|err| match err {
				DomainError::ApiError(_) => ErrorCode::InternalError,
				DomainError::InternalServerError(_) => ErrorCode::InternalError,
				DomainError::BadRequest(_) => ErrorCode::InvalidParams,
				DomainError::NotFound(_) => ErrorCode::InvalidParams,
			})
			.unwrap_or_default();

		Ok(Some(transactions))
	}

	async fn get_bank_account(&self, account_id: String) -> RpcResult<Option<BankAccount>> {
		log::debug!("Received get_bank_account request: {:?}", account_id);

		let ba = self
			.processor
			.bank_account_controller
			.find_by_account_id(&account_id)
			.await
			.map_err(|e| {
				log::debug!("Error: {:?}", e);
				ErrorCode::InvalidParams
			})?;

		log::debug!("Bank account: {:?}", ba);
		Ok(ba)
	}

	async fn get_batch_balances(
		&self,
		signature: Vec<u8>,
		account_ids: Vec<String>,
	) -> RpcResult<Option<Vec<(String, u32)>>> {
		let signature = signature.try_into().map_err(|_| ErrorCode::InvalidParams)?;

		// message is JSON serialized array of account ids, so we need
		// to include the brackets and quotes in the message
		let message = {
			let mut message = Vec::new();
			message.push('[' as u8);
			for account_id in &account_ids {
				message.push('"' as u8);
				message.extend_from_slice(account_id.as_bytes());
				message.push('"' as u8);
				message.push(',' as u8);
			}
			message.pop();
			message.push(']' as u8);
			message
		};

		if !sr25519::verify(&Signature(signature), &message[..], &self.signer) {
			log::error!("Invalid signature");
			return Err(ErrorCode::InvalidParams.into());
		}

		let mut balances = Vec::new();

		for account_id in account_ids {
			let ba = self
				.processor
				.bank_account_controller
				.find_by_account_id(&account_id)
				.await
				.map_err(|e| {
				log::error!("Error: {:?}", e);
				ErrorCode::InvalidParams
			})?;

			if let Some(ba) = ba {
				balances.push((account_id, ba.balance));
			}
		}

		Ok(Some(balances))
	}
}

/// Run ISO8583 Message Processor
pub async fn run(
	processor: Arc<Iso8583MessageProcessor>,
	client: Arc<OnlineClient<SubstrateConfig>>,
	keypair: sr25519::Keypair,
	rpc_port: u16,
	dev_mode: bool,
	ocw_signer: sr25519::PublicKey,
) -> anyhow::Result<(), Box<dyn Error>> {
	if dev_mode {
		info!("Running in dev mode, inserting dev accounts");
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
				account_id: account.4.map(|s| s.to_string()),
			};

			let bank_account = processor.bank_account_controller.create(&bank_account_create).await;

			match bank_account {
				Ok(bank_account) => {
					assert_eq!(bank_account.card_number, account.1);
					assert_eq!(bank_account.balance, account.3);
					assert_eq!(bank_account.nonce, 0);
					info!("Inserted dev account: {:?}", bank_account);
				},
				Err(err) => {
					log::error!("Error inserting dev account: {:?}", err);
				},
			}
		}
	}

	// Run RPC server
	let addr = run_server(processor, client, keypair, rpc_port, ocw_signer).await?;
	let url = format!("ws://{}", addr);

	log::info!("RPC server listening on {}", url);

	Ok(())
}

/// Run RPC server for ISO8583 Message Processor
async fn run_server(
	processor: Arc<Iso8583MessageProcessor>,
	client: Arc<OnlineClient<SubstrateConfig>>,
	keypair: sr25519::Keypair,
	rpc_port: u16,
	ocw_signer: sr25519::PublicKey,
) -> anyhow::Result<SocketAddr> {
	let server = Server::builder().build(format!("0.0.0.0:{}", rpc_port)).await?;

	let addr = server.local_addr()?;
	let oracle_impl = OracleApiImpl { processor, signer: ocw_signer, client, keypair };

	let server_handle = server.start(oracle_impl.into_rpc());

	log::info!("Starting RPC server");
	// Wait for the server to start listening.
	tokio::spawn(server_handle.stopped());

	Ok(addr)
}
