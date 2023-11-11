//! Types used in the PCIDSS Gateway.

/// Message type indicator for the ISO-8583 message, 1987 version
#[derive(Debug, Clone)]
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
