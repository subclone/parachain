//! Tests for reversal transactions
use crate::{
	tests::{mock::*, prelude::*},
	types::{ResponseCodes, MTI},
};

/// Tests payment reversal
#[tokio::test]
async fn test_reversals_success() {
	let api = MockProcessorImpl::new(Some("testdb".to_string())).await;

	// make a basic transaction payment from Alice
	let spec = api.processor.spec;

	let mut new_msg = get_new_iso_msg(&spec, MTI::AuthorizationRequest, ALICE);
	new_msg.set_on(4, "00000000000000000100").unwrap();

	let mut msg_raw = new_msg.assemble().unwrap();
	let (_, msg) = api.processor.process(&mut msg_raw).await.unwrap();

	assert_eq!(msg.bmp_child_value(39).unwrap(), "00");

	let alice_account = get_bank_account_by_card_number(&api, &ALICE.1).await;
	let alice_txs = get_transactions_by_id(&api, &alice_account.id).await;

	let alice_tx = &alice_txs[0];

	assert_eq!(alice_txs.len(), 1);
	assert_eq!(alice_tx.amount, 100);

	// make a reversal transaction from Alice
	let mut reversal_new_msg = get_new_iso_msg(&spec, MTI::ReversalRequest, ALICE);

	// set tx hash on 126
	reversal_new_msg.set_on(4, "00000000000000000100").unwrap();
	reversal_new_msg.set_on(126, &alice_tx.hash).unwrap();

	let mut msg_raw = reversal_new_msg.assemble().unwrap();

	let (_, msg) = api.processor.process(&mut msg_raw).await.unwrap();

	assert_eq!(msg.bmp_child_value(39).unwrap(), "00");
	assert_eq!(msg.bmp_child_value(4).unwrap(), "00000000000000000100");

	// get alice account again
	let alice_account = get_bank_account_by_card_number(&api, &ALICE.1).await;
	let acquirer_account = get_bank_account_by_card_number(&api, &ACQUIRER.1).await;

	// balances should be the same as before
	assert_eq!(alice_account.balance, ALICE.3);
	assert_eq!(acquirer_account.balance, ACQUIRER.3);

	// get alice txs again
	let alice_txs = get_transactions_by_id(&api, &alice_account.id).await;

	// tx len unchanged, but tx is flagged as reversed now
	assert_eq!(alice_txs.len(), 1);
	assert_eq!(alice_txs[0].reversed, true);

	// VALIDATION TESTS
	// Try to reverse a transaction that doesn't exist
	let mut new_msg = get_new_iso_msg(&spec, MTI::ReversalRequest, CHARLIE);

	// set tx hash on 126
	new_msg.set_on(4, "00000000000000000100").unwrap();
	new_msg.set_on(126, &"0".repeat(64)).unwrap();

	let charlie_account = get_bank_account_by_card_number(&api, &CHARLIE.1).await;
	let charlie_txs = get_transactions_by_id(&api, &charlie_account.id).await;

	assert_noop(
		&api,
		CHARLIE,
		&new_msg,
		ResponseCodes::InvalidTransaction,
		charlie_account.clone(),
		charlie_txs.clone(),
	)
	.await;

	// Try to reverse a transaction that has already been reversed
	assert_noop(
		&api,
		ALICE,
		&reversal_new_msg,
		ResponseCodes::InvalidTransaction,
		alice_account.clone(),
		alice_txs.clone(),
	)
	.await;
}
