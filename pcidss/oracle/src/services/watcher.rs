//! Watcher service subscribes to Substrate chain to maintain constant sync between the chain and
//! the oracle
use super::processor::Iso8583MessageProcessor;
use iso_8583_chain::iso8583::events::InitiateTransfer;
use op_core::bank_account::models::BankAccount;
use std::sync::Arc;
use subxt::{events::EventDetails, OnlineClient, SubstrateConfig};

#[subxt::subxt(runtime_metadata_path = "./iso-8583-chain.scale")]
pub mod iso_8583_chain {}

pub(crate) async fn watcher(
	processor: Arc<Iso8583MessageProcessor>,
) -> anyhow::Result<(), Box<dyn std::error::Error>> {
	let api = OnlineClient::<SubstrateConfig>::from_url("ws://localhost:9933").await?;

	// Subscribe to the oracle module
	let mut blocks_sub = api.blocks().subscribe_finalized().await?;

	let event_processor = EventProcessor::new(processor);

	// For each block, look for oracle events
	while let Some(block_result) = blocks_sub.next().await {
		match block_result {
			Ok(block) => match block.events().await {
				Ok(events) => {
					log::info!("Events: {:?}", events.len());
					for event_result in events.iter() {
						match event_result {
							Ok(event) => event_processor.process_event(&event).await?,
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

/// Processor for events
pub(crate) struct EventProcessor {
	iso_msg_processor: Arc<Iso8583MessageProcessor>,
}

impl EventProcessor {
	/// Create a new event processor
	pub(crate) fn new(iso_msg_processor: Arc<Iso8583MessageProcessor>) -> Self {
		Self { iso_msg_processor }
	}

	/// Process a single event
	pub(crate) async fn process_event(
		&self,
		event: &EventDetails<SubstrateConfig>,
	) -> anyhow::Result<(), Box<dyn std::error::Error>> {
		log::info!("Event: {:?}", event.pallet_name());

		if event.pallet_name().contains("ISO8583") {
			let event_name = event.variant_name();
			match event_name {
				x if x.contains("InitiateTransfer") => {
					let decoded_event =
						event.as_event::<InitiateTransfer>()?.ok_or("Could not decode event")?;

					// spread struct properties into variables
					let InitiateTransfer { from, to, amount } = decoded_event;

					let (from_hex, to_hex): (String, String) = (
						from.0.iter().map(|b| format!("{:02x}", b)).collect(),
						from.0.iter().map(|b| format!("{:02x}", b)).collect(),
					);

					let from_bank_account = self
						.iso_msg_processor
						.bank_account_controller
						.find_by_account_id(&from_hex.as_str())
						.await?
						.ok_or("From bank account not found")?;

					let to_bank_account = self
						.iso_msg_processor
						.bank_account_controller
						.find_by_account_id(&to_hex.as_str())
						.await?
						.ok_or("To bank account not found")?;

					// compose ISO8583 message
					let mut iso_msg =
						self.compose_iso_msg(&from_bank_account, &to_bank_account, amount)?;

					self.iso_msg_processor.process(&mut iso_msg);
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
		to: &BankAccount,
		amount: u128,
	) -> anyhow::Result<Vec<u8>, Box<dyn std::error::Error>> {
		let mut msg = vec![];

		Ok(msg)
	}
}
