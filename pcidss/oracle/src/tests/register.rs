//! Tests for registering an on-chain account

use crate::{
	tests::{mock::*, prelude::*},
	types::MTI,
};

/// Tests basic payment authorization and its settlement
#[tokio::test]
async fn test_register() {
	env_logger::init();
	let api = MockProcessorImpl::new(Some("register_db".to_string())).await;

	let spec = api.processor.spec;

	let mti = MTI::NetworkManagementRequest;
	let mut new_msg = get_new_iso_msg(spec, mti.clone(), ALICE);
	new_msg.set_on(4, &"0".repeat(20)).unwrap();

	let account_id_hex = format!("0x{}", "01".repeat(32));
	new_msg.set_on(126, &account_id_hex).unwrap();

	let mut msg_raw = new_msg.assemble().unwrap();

	let (_, msg) = api.processor.process(&mut msg_raw).await.unwrap();

	// Assert processing results
	assert_eq!(msg.bmp_child_value(39).unwrap(), "00");

	let alice_account = get_bank_account_by_card_number(&api, &ALICE.1).await;

	assert_eq!(alice_account.account_id, Some(account_id_hex.trim_start_matches("0x").to_string()));

	// supply invalid account id
	let mut new_msg = get_new_iso_msg(spec, mti, CHARLIE);
	new_msg.set_on(4, &"0".repeat(20)).unwrap();

	let account_id_hex = format!("0x{}", "00".repeat(31));
	new_msg.set_on(126, &account_id_hex).unwrap();

	let mut msg_raw = new_msg.assemble().unwrap();

	let (_, msg) = api.processor.process(&mut msg_raw).await.unwrap();

	// Assert processing results
	assert_eq!(msg.bmp_child_value(39).unwrap(), "12");

	let charlie_account = get_bank_account_by_card_number(&api, &CHARLIE.1).await;

	assert_eq!(
		charlie_account.account_id,
		Some(CHARLIE.4.unwrap().trim_start_matches("0x").to_string())
	);
}
