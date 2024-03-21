//! Types used in the PCIDSS Gateway.

/// Message type indicator for the ISO-8583 message, 1987 version
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MTI {
	/// 0100 - Authorization request
	AuthorizationRequest,
	/// 0110 - Authorization response
	AuthorizationResponse,
	/// 0200 - Financial request
	FinancialRequest,
	/// 0210 - Financial response
	FinancialResponse,
	/// 0400 - Reversal request
	ReversalRequest,
	/// 0410 - Reversal response
	ReversalResponse,
	/// 0800 - Network management request
	NetworkManagementRequest,
	/// 0810 - Network management response
	NetworkManagementResponse,
}

#[allow(clippy::from_over_into)]
impl Into<&str> for MTI {
	fn into(self) -> &'static str {
		match self {
			MTI::AuthorizationRequest => "0100",
			MTI::AuthorizationResponse => "0110",
			MTI::FinancialRequest => "0200",
			MTI::FinancialResponse => "0210",
			MTI::ReversalRequest => "0400",
			MTI::ReversalResponse => "0410",
			MTI::NetworkManagementRequest => "0800",
			MTI::NetworkManagementResponse => "0810",
		}
	}
}

impl TryFrom<&str> for MTI {
	type Error = ();
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			"0100" => Ok(MTI::AuthorizationRequest),
			"0110" => Ok(MTI::AuthorizationResponse),
			"0200" => Ok(MTI::FinancialRequest),
			"0210" => Ok(MTI::FinancialResponse),
			"0400" => Ok(MTI::ReversalRequest),
			"0410" => Ok(MTI::ReversalResponse),
			"0800" => Ok(MTI::NetworkManagementRequest),
			"0810" => Ok(MTI::NetworkManagementResponse),
			_ => Err(()),
		}
	}
}

/// Response codes for the ISO-8583 message, 1987 version
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseCodes {
	// 00 - Approved
	Approved,
	// 05 - Do not honor
	DoNotHonor,
	// 12 - Invalid transaction
	InvalidTransaction,
	// 14 - Invalid PAN
	InvalidCardNumber,
	// 51 - Insufficient funds, if it underflows
	InsufficientFunds,
	// 54 - Expired card
	ExpiredCard,
}

#[allow(clippy::from_over_into)]
impl Into<&str> for ResponseCodes {
	fn into(self) -> &'static str {
		match self {
			ResponseCodes::Approved => "00",
			ResponseCodes::DoNotHonor => "05",
			ResponseCodes::InvalidTransaction => "12",
			ResponseCodes::InvalidCardNumber => "14",
			ResponseCodes::InsufficientFunds => "51",
			ResponseCodes::ExpiredCard => "54",
		}
	}
}

/// Represents truncated version of dev accounts
/// Explicitly used in tests and dev mode
pub(crate) type DevAccount = (&'static str, &'static str, &'static str, u32, Option<&'static str>);

/// Constants used in the app
pub mod constants {
	/// ISO8583 Pallet ID converted to `AccountId32`
	pub const PALLET_ACCOUNT: &str =
		"6d6f646c70792f69736f38350000000000000000000000000000000000000000";

	/// Pallet prefix
	pub const PALLET_NAME: &str = "ISO8583";

	/// Field numbers that we populate in the ISO message
	pub const POPULATED_ISO_MSG_FIELD_NUMBERS: [u32; 9] = [
		0, // Message Type Indicator or MTI
		2, // Primary account number, card number
		3, // Processing code
		4, /* Amount is 20 characters long, check the length of amount and pad it with `0`
		    * from the left */
		7,   // Transmission date, combination of 13 and 12
		12,  // HHMMSS format of transaction time
		32,  // Acquiring institution ID
		35,  // Track-2 Data
		126, // Private data
	];

	/// Response Code field
	pub const RESPONSE_CODE_FIELD_NUMBER: u32 = 39;

	// Development accounts
	pub const DEV_ACCOUNTS: [crate::types::DevAccount; 8] = [
		// Healthy account
		(
			"Alice",
			"4169812345678901",
			"123",
			1000,
			Some("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"),
		),
		// Zero balance case
		(
			"Bob",
			"4169812345678902",
			"124",
			0,
			Some("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"),
		),
		(
			"Charlie",
			"4169812345678903",
			"125",
			12345,
			Some("90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22"),
		),
		(
			"Dave",
			"4169812345678904",
			"126",
			1000000,
			Some("306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20"),
		),
		// Expired card
		(
			"Eve",
			"4169812345678905",
			"127",
			1000,
			Some("e659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e"),
		),
		// Mock acquirer account, i.e merchant
		(
			"Acquirer",
			"123456",
			"000",
			1000000000,
			Some("ecd07df8b5fdd6c13e776c4720b325423d5c2449520266ca11dfd1735e28f572"),
		),
		// Alice stash
		("Alice_stash", "4169812345678908", "999", 0, None),
		// Bob stash
		("Bob_stash", "4169812345678909", "888", 0, None),
	];
}
