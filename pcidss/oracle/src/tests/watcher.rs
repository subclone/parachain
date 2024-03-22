//! Tests for the watcher module

use std::{str::FromStr, sync::Arc};

use subxt::{OnlineClient, SubstrateConfig};
use subxt_signer::{sr25519::Keypair, SecretUri};

use crate::{
	services::watcher::WatcherService,
	tests::{mock::*, prelude::*},
};

/// Test uri
const SEED: &str = "//Alice";

#[tokio::test]
async fn test_compose_iso_msg() {
	let mock_processor = Arc::new(MockProcessorImpl::new(Some("compose_iso".to_string())).await);
	let keypair = Keypair::from_uri(&SecretUri::from_str(SEED).unwrap())
		.map_err(|_| "Invalid seed phrase")
		.unwrap();

	let watcher = WatcherService::new(keypair, mock_processor.processor.clone(), client)
		.await
		.unwrap();

	let from = get_bank_account_by_card_number(&mock_processor, ALICE.1).await;
	let to = get_bank_account_by_card_number(&mock_processor, ACQUIRER.1).await;

	let mut iso_msg = watcher.compose_iso_msg(&from, Some(&to), None, 1234, "123-1").unwrap();

	let iso_msg_parsed = mock_processor.processor.spec.parse(&mut iso_msg).unwrap();

	assert_eq!(iso_msg_parsed.get_field_value(&"message_type".to_string()).unwrap(), "0100");
	assert_eq!(iso_msg_parsed.bmp_child_value(2).unwrap(), ALICE.1);
	assert_eq!(iso_msg_parsed.bmp_child_value(3).unwrap(), "000000");
	assert_eq!(iso_msg_parsed.bmp_child_value(4).unwrap(), "00000000000000001234");
	assert_eq!(iso_msg_parsed.bmp_child_value(127).unwrap(), "123-1");
}
