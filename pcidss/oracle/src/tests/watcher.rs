//! Tests for the watcher module

use std::{str::FromStr, sync::Arc};

use subxt::metadata::types::Metadata;
use subxt_signer::SecretUri;

use crate::{services::watcher::WatcherService, tests::mock::*};

/// Test uri
const SEED: &str = "//Alice";

#[tokio::test]
async fn test_process_transfer() {
	let mock_processor = Arc::new(MockProcessorImpl::new(None).await);
	let watcher = WatcherService::new(
		SecretUri::from_str(SEED).unwrap(),
		mock_processor.processor.clone(),
		"ws://localhost:9944".to_string(),
	)
	.await
	.unwrap();

	watcher.process_event(1, EventDetails{
        phase: Phase::ApplyExtrinsic(0),
        index: 0,
        all_bytes: vec![],
        start_idx: 0,
        event_stard_idx: 0,
        event_fields_start_idx: 0,
        event_fields_end_idx: 0,
        end_idx: 0,
        metadata: Metadata::default(),

    }
}
