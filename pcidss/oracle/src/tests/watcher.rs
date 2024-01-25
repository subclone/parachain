//! Tests for the watcher module

use std::{str::FromStr, sync::Arc};

use subxt_signer::SecretUri;

use crate::{
	services::watcher::WatcherService,
	tests::{mock::*, prelude::get_bank_account_by_card_number},
};

/// Test uri
const SEED: &str = "//Alice";

#[tokio::test]
async fn test_compose_iso_msg() {
	let mock_processor = Arc::new(MockProcessorImpl::new(None).await);
	let watcher = WatcherService::new(
		SecretUri::from_str(SEED).unwrap(),
		mock_processor.processor.clone(),
		"ws://localhost:9944".to_string(),
	)
	.await
	.unwrap();

	let spec = iso8583_rs::iso8583::iso_spec::spec("");

	let from = get_bank_account_by_card_number(&mock_processor, "1111222233334444").await;
	let to = get_bank_account_by_card_number(&mock_processor, "5555666677778888").await;

	let mut iso_msg = watcher.compose_iso_msg(&from, Some(&to), None, 1234, "123-1").unwrap();

	let iso_msg_parsed = spec.parse(&mut iso_msg).unwrap();

	assert_eq!(iso_msg_parsed.get_field_value(&"message_type".to_string()).unwrap(), "0200");
	assert_eq!(iso_msg_parsed.bmp_child_value(2).unwrap(), "5555666677778888");
	assert_eq!(iso_msg_parsed.bmp_child_value(3).unwrap(), "000000");
	assert_eq!(iso_msg_parsed.bmp_child_value(4).unwrap(), "00000000001234");
	assert_eq!(iso_msg_parsed.bmp_child_value(7).unwrap(), "123-1");
}
