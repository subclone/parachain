//! Watcher service subscribes to Substrate chain to maintain constant sync between the chain and
//! the oracle
use crate::types::{
	constants::{PALLET_ACCOUNT, PALLET_NAME},
	MTI,
};

use self::iso_8583_chain::runtime_types::bounded_collections::bounded_vec::BoundedVec;

use super::processor::Iso8583MessageProcessor;
use iso8583_rs::iso8583::{
	iso_spec::{new_msg, IsoMsg},
	IsoError,
};
use iso_8583_chain::{
	iso8583::events::{InitiateRevert, InitiateTransfer},
	runtime_types::pallet_iso_8583::types::{
		FinalisedTransaction, ISO8583FailureReason, ISO8583Status,
	},
};
use op_core::bank_account::models::BankAccount;
use std::{str::FromStr, sync::Arc};
use subxt::{
	config::substrate::H256, events::EventDetails, utils::AccountId32, OnlineClient,
	SubstrateConfig,
};
use subxt_signer::{sr25519::Keypair, SecretUri};

#[subxt::subxt(runtime_metadata_path = "./iso-8583-chain.scale")]
pub mod iso_8583_chain {}

/// Service for consuming events and submitting finalities of ISO8583 messages on-chain
pub struct WatcherService {
	/// Keypair for signing transactions
	pub keypair: Keypair,
	/// ISO8583 message processor
	pub processor: Arc<Iso8583MessageProcessor>,
	/// Substrate client
	pub client: OnlineClient<SubstrateConfig>,
}

impl WatcherService {
	/// Create a new watcher service
	pub(crate) async fn new(
		seed: SecretUri,
		processor: Arc<Iso8583MessageProcessor>,
		ws_url: String,
	) -> Result<Self, &'static str> {
		let keypair = Keypair::from_uri(&seed).map_err(|_| "Invalid seed phrase")?;

		let client = OnlineClient::<SubstrateConfig>::from_url(&ws_url)
			.await
			.map_err(|_| format!("Could not connect to Substrate node at: {}", ws_url))
			.unwrap();

		Ok(Self { keypair, processor, client })
	}

	/// Start the main processing loop
	pub async fn start(&self) -> anyhow::Result<()> {
		// Subscribe to the oracle module
		let mut blocks_sub = self.client.blocks().subscribe_finalized().await?;

		// For each block, look for oracle events
		while let Some(block_result) = blocks_sub.next().await {
			match block_result {
				Ok(block) => match block.events().await {
					// get block here
					Ok(events) => {
						let block_number = block.number();
						for event_result in events.iter() {
							match event_result {
								Ok(event) => {
									if let Err(_) = self.process_event(block_number, &event).await {
										log::error!("Error processing event: {:?}", event.index());
									}
								},
								Err(e) => log::error!("Error decoding event: {}", e),
							}
						}
					},
					Err(e) => log::error!("Error retrieving events: {}", e),
				},
				Err(e) => log::error!("Error processing block: {}", e),
			}
		}
		Ok(())
	}

	/// Process a single event
	pub(crate) async fn process_event(
		&self,
		block_number: u32,
		event: &EventDetails<SubstrateConfig>,
	) -> anyhow::Result<(), Box<dyn std::error::Error>> {
		log::info!("Event: {:?}", event.pallet_name());

		if event.pallet_name().contains(PALLET_NAME) {
			let event_name = event.variant_name();

			let event_id = format!("{}-{}", block_number, event.index());
			match event_name {
				x if x.contains("InitiateTransfer") => {
					let decoded_event =
						event.as_event::<InitiateTransfer>()?.ok_or("Could not decode event")?;

					// spread struct properties into variables
					let InitiateTransfer { from, to, amount } = decoded_event;

					Self::process_transfer(&self, from, to, amount, &event_id).await?
				},
				x if x.contains("InitiateRevert") => {
					let decoded_event =
						event.as_event::<InitiateRevert>()?.ok_or("Could not decode event")?;

					let InitiateRevert { who, hash } = decoded_event;

					Self::process_revert(&self, who, hash, &event_id).await?
				},
				_ => (),
			}
		}

		Ok(())
	}

	/// Given a `from` and `to` bank account, compose an ISO8583 message
	pub(crate) fn compose_iso_msg(
		&self,
		from: &BankAccount,
		to: Option<&BankAccount>,
		hash: Option<&str>,
		amount: u128,
		event_id: &str,
	) -> anyhow::Result<Vec<u8>, IsoError> {
		// if tx_hash is set, then it's a revert
		let mti = {
			if hash.is_some() {
				MTI::ReversalRequest
			} else {
				MTI::AuthorizationRequest
			}
		};

		let spec = self.processor.spec;
		let mut msg = new_msg(spec, spec.get_message_from_header(mti.clone().into())?);

		msg.set("message_type", mti.into())?;
		msg.set_on(2, &from.card_number)?;
		msg.set_on(3, "000000")?;
		msg.set_on(4, &format!("{:020}", amount))?;

		let now = chrono::Utc::now();

		msg.set_on(7, &format!("{}", now.format("%m%d%H%M%S")))?;
		msg.set_on(12, &format!("{}", now.format("%H%M%S")))?;

		if let Some(to) = to {
			msg.set_on(32, &to.id.to_string())?;
		}

		msg.set_on(
			35,
			&format!("{}D{}C{}", from.card_number, from.card_expiration_date, from.card_cvv),
		)?;

		msg.set_on(127, &event_id)?;

		if let Some(hash) = hash {
			msg.set_on(126, hash)?;
		} else {
			msg.set_on(126, &"0".repeat(99))?;
		}

		let iso_msg_raw = msg.assemble()?;
		Ok(iso_msg_raw)
	}
}

// Separate utility functions into a separate module
impl WatcherService {
	/// Process a transfer event
	pub(crate) async fn process_transfer(
		&self,
		from: AccountId32,
		to: AccountId32,
		amount: u128,
		event_id: &str,
	) -> anyhow::Result<(), Box<dyn std::error::Error>> {
		let (from_hex, to_hex): (String, String) = (
			from.0.iter().map(|b| format!("{:02x}", b)).collect(),
			to.0.iter().map(|b| format!("{:02x}", b)).collect(),
		);

		let (from_bank_account, to_bank_account) = futures::join!(
			self.processor.bank_account_controller.find_by_account_id(&from_hex.as_str()),
			self.processor.bank_account_controller.find_by_account_id(&to_hex.as_str())
		);

		let from_bank_account = from_bank_account?.ok_or("From bank account not found")?;
		let to_bank_account = to_bank_account?.ok_or("To bank account not found")?;

		log::debug!(
			"Processing transaction from: {}, to: {}, amount: {}, event_id: {}",
			from_hex,
			to_hex,
			amount,
			&event_id
		);

		// compose ISO8583 message
		let mut iso_msg_raw = self
			.compose_iso_msg(&from_bank_account, Some(&to_bank_account), None, amount, &event_id)
			.map_err(|_| "Could not compose ISO8583 message")?;

		let (_, iso_msg) = self.processor.process(&mut iso_msg_raw).await?;

		// submit finality
		self.submit_finality(from, to, amount, iso_msg, &event_id).await
	}

	/// Process a revert event
	pub(crate) async fn process_revert(
		&self,
		from: AccountId32,
		hash: H256,
		event_id: &str,
	) -> anyhow::Result<(), Box<dyn std::error::Error>> {
		let (who_hex, hash_hex): (String, String) = (
			from.0.iter().map(|b| format!("{:02x}", b)).collect(),
			hash.0.iter().map(|b| format!("{:02x}", b)).collect(),
		);

		log::debug!("Reverting transaction from: {}, hash: {}", who_hex, hash_hex);

		let (from_bank_account, maybe_transaction) = futures::join!(
			self.processor.bank_account_controller.find_by_account_id(&who_hex.as_str()),
			self.processor.transaction_controller.find_by_hash(&hash_hex.as_str())
		);

		let from_bank_account = from_bank_account?.ok_or("From bank account not found")?;

		// simply try to unwrap the transaction, if it's not found, then it's an error
		// we do this to avoid doing unnecessary ISO8583 processing and as a naive DDOS
		// protection
		let _ = maybe_transaction?.ok_or("Transaction not found")?;

		let mut iso_msg_raw = self
			.compose_iso_msg(&from_bank_account, None, Some(&hash_hex), 0, &event_id)
			.map_err(|_| "Could not compose ISO8583 message")?;

		let (_, iso_msg) = self.processor.process(&mut iso_msg_raw).await?;

		self.submit_finality(
			from,
			AccountId32(PALLET_ACCOUNT.as_bytes().try_into().expect("valid account;qed")),
			0,
			iso_msg,
			&event_id,
		)
		.await
	}

	/// Submit a processed ISO8583 message on-chain
	///
	/// This is called after an ISO-8583 message has been processed by the ISO8583 message
	/// processor, both from the watcher service or the RPC server
	pub(crate) async fn submit_finality(
		&self,
		from: AccountId32,
		to: AccountId32,
		amount: u128,
		iso_msg: IsoMsg,
		event_id: &str,
	) -> anyhow::Result<(), Box<dyn std::error::Error>> {
		let private_data =
			iso_msg.bmp_child_value(126).map_err(|_| "Could not get private data")?;

		let (tx_hash, _) = private_data.split_at(64);

		let response_code =
			iso_msg.bmp_child_value(39).map_err(|_| "Could not get response code")?;

		let status = match &response_code[..] {
			"00" => ISO8583Status::Approved,
			"05" => ISO8583Status::Failed(ISO8583FailureReason::DoNotHonor),
			"12" => ISO8583Status::Failed(ISO8583FailureReason::InvalidTransaction),
			"14" => ISO8583Status::Failed(ISO8583FailureReason::InvalidCardNumber),
			"51" => ISO8583Status::Failed(ISO8583FailureReason::InsufficientFunds),
			"54" => ISO8583Status::Failed(ISO8583FailureReason::ExpiredCard),
			_ => ISO8583Status::Failed(ISO8583FailureReason::Other),
		};

		let finalised_transacton = FinalisedTransaction {
			hash: H256::from_str(tx_hash.strip_prefix("0x").unwrap_or_default())
				.expect("valid hash; qed"),
			event_id: BoundedVec::<u8>(event_id.as_bytes().to_vec()),
			from,
			to,
			amount,
			status,
		};

		log::debug!("Submitting finality: {:?}", finalised_transacton);

		let finalised_transaction_tx =
			iso_8583_chain::tx().iso8583().submit_finality(finalised_transacton);

		// don't wait for the transaction to be included in a block, submit and forget
		self.client
			.tx()
			.sign_and_submit_default(&finalised_transaction_tx, &self.keypair)
			.await?;

		Ok(())
	}
}
