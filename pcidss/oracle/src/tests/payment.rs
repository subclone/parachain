//! Tests for payment transactions

use crate::{
	tests::{mock::*, prelude::*},
	types::{ResponseCodes, MTI},
};

/// Tests basic payment authorization and its settlement
#[tokio::test]
async fn test_payment() {
	let api = MockProcessorImpl::new(None).await;

	let spec = api.processor.spec;

	let mut new_msg = get_new_iso_msg(&spec, MTI::AuthorizationRequest, ALICE);
	new_msg.set_on(4, "00000000000000000100").unwrap();

	let mut msg_raw = new_msg.assemble().unwrap();
	let (_, msg) = api.processor.process(&mut msg_raw).await.unwrap();

	assert_eq!(msg.bmp_child_value(39).unwrap(), "00");
	assert_eq!(msg.bmp_child_value(4).unwrap(), "00000000000000000100");
	// transaction hash is set
	assert_ne!(msg.bmp_child_value(126).unwrap(), "0".repeat(99));

	let alice_account = get_bank_account_by_card_number(&api, &ALICE.1).await;
	let alice_txs = get_transactions_by_id(&api, &alice_account.id).await;

	assert_eq!(alice_txs.len(), 1);
	let alice_tx = &alice_txs[0];

	assert_eq!(alice_tx.amount, 100);
	assert_eq!(alice_tx.from, alice_account.id);
	assert!(alice_tx.to.is_some());
	assert_eq!(alice_tx.transaction_type, 1); // Credit transaction

	// INSUFFICIENT FUNDS
	// Make sure alice can't spend more than she has
	let mut new_msg = get_new_iso_msg(&spec, MTI::AuthorizationRequest, ALICE);
	new_msg.set_on(4, "00000000000000001100").unwrap();

	assert_noop(
		&api,
		ALICE,
		&new_msg,
		ResponseCodes::InsufficientFunds,
		alice_account.clone(),
		alice_txs.clone(),
	)
	.await;

	// EXPIRED CARD
	// Make sure Eve can't spend anything
	let mut new_msg = get_new_iso_msg(&spec, MTI::AuthorizationRequest, EVE);
	new_msg.set_on(4, "00000000000000000100").unwrap();

	let eve_account = get_bank_account_by_card_number(&api, &EVE.1).await;

	assert_noop(&api, EVE, &new_msg, ResponseCodes::ExpiredCard, eve_account.clone(), vec![]).await;

	// INVALID CARD NUMBER
	// Make sure any msg with invalid card number is rejected
	let mut new_msg = get_new_iso_msg(&spec, MTI::AuthorizationRequest, ALICE);
	new_msg.set_on(2, "1234567890123456").unwrap();
	new_msg.set_on(4, "00000000000000000100").unwrap();

	assert_noop(
		&api,
		ALICE,
		&new_msg,
		ResponseCodes::InvalidCardNumber,
		alice_account.clone(),
		alice_txs.clone(),
	)
	.await;

	// INVALID TRANSACTION
	// Make sure any msg with invalid transaction is rejected
	// This is triggered when timestamp is not within 30 seconds of now
	let mut new_msg = get_new_iso_msg(&spec, MTI::AuthorizationRequest, CHARLIE);
	new_msg.set_on(7, "1109010101").unwrap();
	new_msg.set_on(4, "00000000000000000100").unwrap();

	let charlie_account = get_bank_account_by_card_number(&api, &CHARLIE.1).await;

	assert_noop(
		&api,
		CHARLIE,
		&new_msg,
		ResponseCodes::InvalidTransaction,
		charlie_account.clone(),
		vec![],
	)
	.await;

	// DO NOT HONOR
	// Can be caused by wrong cvv
	let mut new_msg = get_new_iso_msg(&spec, MTI::AuthorizationRequest, ALICE);
	new_msg
		.set_on(
			35,
			&format!(
				"{}D{}C{}",
				ALICE.1,
				alice_account.card_expiration_date.format("%m%y").to_string(),
				"999" // correct cvv is 123
			),
		)
		.unwrap();
	new_msg.set_on(4, "00000000000000000100").unwrap();

	assert_noop(
		&api,
		ALICE,
		&new_msg,
		ResponseCodes::DoNotHonor,
		alice_account.clone(),
		alice_txs.clone(),
	)
	.await;

	// And now finally, DAVE makes big payment
	let mut new_msg = get_new_iso_msg(&spec, MTI::AuthorizationRequest, DAVE);
	new_msg.set_on(4, "00000000000000100000").unwrap();

	let mut msg_raw = new_msg.assemble().unwrap();
	let (_, msg) = api.processor.process(&mut msg_raw).await.unwrap();

	assert_eq!(msg.bmp_child_value(39).unwrap(), "00");
	assert_eq!(msg.bmp_child_value(4).unwrap(), "00000000000000100000");

	let dave_account = get_bank_account_by_card_number(&api, &DAVE.1).await;

	assert_eq!(dave_account.balance, DAVE.3 - 100_000);

	let dave_txs = api
		.processor
		.transaction_controller
		.find_by_beneficiary(&dave_account.id)
		.await
		.unwrap();

	assert_eq!(dave_txs.len(), 1);
	let dave_tx = &dave_txs[0];

	assert_eq!(dave_tx.amount, 100_000);

	// Settlement is `on-us` since merchant is hard coded as `ACQUIRER`
	let acquirer = get_bank_account_by_card_number(&api, &ACQUIRER.1).await;

	assert_eq!(acquirer.balance, ACQUIRER.3 + 100_000 + 100);

	for (alice_tx, dave_tx) in alice_txs.iter().zip(dave_txs.iter()) {
		assert_eq!(alice_tx.to, Some(acquirer.id));
		assert_eq!(dave_tx.to, Some(acquirer.id));
	}
}
