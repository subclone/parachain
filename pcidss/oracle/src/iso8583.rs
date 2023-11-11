//! ISO-8583 message parsing and formatting.

use std::sync::Arc;

use chrono::Utc;
use iso8583_rs::iso8583::iso_spec::{new_msg, IsoMsg, Spec};
use log::info;
use tracing::debug;

use op_core::bank_account::models::BankAccount;
use op_core::{
    bank_account::{models::BankAccountUpdate, traits::BankAccountTrait},
    error::DomainError,
    transaction::{models::TransactionCreate, traits::TransactionTrait},
    types::TransactionType,
};

use super::types::*;
use crate::constants::{POPULATED_ISO_MSG_FIELD_NUMBERS, RESPONSE_CODE_FIELD_NUMBER};

/// ISO-8583 message processor
#[derive(Clone)]
pub struct Iso8583MessageProcessor {
    /// Spec for the ISO-8583 message
    pub spec: &'static Spec,
    /// Bank account controller
    pub bank_account_controller: Arc<dyn BankAccountTrait>,
    /// Transaction controller
    pub transaction_controller: Arc<dyn TransactionTrait>,
}

impl Iso8583MessageProcessor {
    /// Process the encoded ISO-8583 message and return the response
    pub async fn process(&self, msg: &mut Vec<u8>) -> Result<(Vec<u8>, IsoMsg), DomainError> {
        match self.spec.parse(msg) {
            Ok(iso_msg) => {
                debug!("parsed incoming request - message = \"{}\" successfully. \n : parsed message: \n --- \n {} \n ----\n",
                       iso_msg.msg.name(), iso_msg);

                let req_msg_type = iso_msg.get_field_value(&"message_type".to_string())?;

                let res_msg_type = match req_msg_type.as_str().try_into() {
                    Ok(MTI::AuthorizationRequest) => MTI::AuthorizationResponse,
                    Ok(MTI::ReversalRequest) => MTI::ReversalResponse,
                    _ => {
                        return Err(DomainError::ApiError(
                            "Unsupported message type".to_string(),
                        ))
                    }
                };

                // Create a new response message
                let mut res_iso_msg = new_msg(
                    self.spec,
                    self.spec.get_message_from_header(res_msg_type.into())?,
                );

                // don't copy the fields that we have already set
                res_iso_msg.echo_from(&iso_msg, &POPULATED_ISO_MSG_FIELD_NUMBERS[1..])?;

                // handle authorization request
                match req_msg_type
                    .as_str()
                    .try_into()
                    .expect("Validated above; qed")
                {
                    MTI::AuthorizationRequest => {
                        self.handle_authorization_request(&mut res_iso_msg).await?
                    }
                    MTI::ReversalRequest => self.handle_reversal_request(&mut res_iso_msg).await?,
                    _ => {
                        return Err(DomainError::ApiError(
                            "Unsupported message type".to_string(),
                        ))
                    }
                };

                if let Ok(res_data) = res_iso_msg.assemble() {
                    return Ok((res_data, res_iso_msg));
                }

                Err(DomainError::ApiError(
                    "Failed to assemble new ISO message".to_string(),
                ))
            }
            Err(e) => {
                debug!("Failed to parse incoming request - message = \"{}\". \n : error: \n --- \n {} \n ----\n",
                       String::from_utf8_lossy(msg), e);
                Err(DomainError::ApiError(
                    "Failed to parse incoming request".to_string(),
                ))
            }
        }
    }

    /// Handle authorization request
    ///
    /// Extracts necessary fields from the ISO message and performs authorization.
    async fn handle_authorization_request(&self, iso_msg: &mut IsoMsg) -> Result<(), DomainError> {
        iso_msg.set("message_type", MTI::AuthorizationResponse.into())?;

        // extract necessary fields from the ISO message
        let card_number = iso_msg.bmp_child_value(2)?;
        let acquiring_institution_id = iso_msg.bmp_child_value(32)?;

        let (maybe_beneficiary_account, maybe_recipient_account) = futures::join!(
            self.bank_account_controller
                .find_by_card_number(&card_number),
            self.bank_account_controller
                .find_by_card_number(&acquiring_institution_id)
        );

        if let Ok(Some(bank_account)) = maybe_beneficiary_account {
            let validation_result = self
                .validate_with_bank_account(iso_msg, &bank_account)
                .await?;

            // early return if not approved
            if validation_result != ResponseCodes::Approved {
                iso_msg.set_on(RESPONSE_CODE_FIELD_NUMBER, validation_result.into())?;
                return Ok(());
            }

            let amount: u32 = iso_msg.bmp_child_value(4)?.trim().parse()?;

            // perform the transaction
            let update_beneficiary_account = BankAccountUpdate::Balance {
                id: bank_account.id,
                amount,
                transaction_type: TransactionType::Credit,
            };

            match self
                .bank_account_controller
                .update(&bank_account.id, &update_beneficiary_account)
                .await
            {
                Ok(updated_bank_account) => {
                    info!(
                        "Transaction successful, updated bank account: {:?}",
                        updated_bank_account
                    );

                    let iso_msg_raw = iso_msg.assemble().expect("should be working");
                    let mut recipient_id = None;

                    if let Ok(Some(recipient_account)) = maybe_recipient_account {
                        let update_recipient_account = BankAccountUpdate::Balance {
                            id: recipient_account.id,
                            amount,
                            transaction_type: TransactionType::Debit,
                        };
                        self.bank_account_controller
                            .update(&recipient_account.id, &update_recipient_account)
                            .await?;
                        recipient_id = Some(recipient_account.id);
                    }

                    // Insert the transaction into the database
                    let transaction = self
                        .transaction_controller
                        .create(&TransactionCreate::new(
                            bank_account.id,
                            recipient_id,
                            amount,
                            TransactionType::Credit,
                            bank_account.nonce,
                            iso_msg_raw,
                        ))
                        .await?;

                    // set the transaction hash in the ISO message
                    iso_msg.set_on(126, &transaction.hash)?;
                    iso_msg.set_on(RESPONSE_CODE_FIELD_NUMBER, ResponseCodes::Approved.into())?;
                }
                Err(e) => {
                    log::error!("Transaction failed: {:?}", e);
                    iso_msg.set_on(
                        RESPONSE_CODE_FIELD_NUMBER,
                        ResponseCodes::InvalidTransaction.into(),
                    )?;
                }
            }
        } else {
            iso_msg.set_on(
                RESPONSE_CODE_FIELD_NUMBER,
                ResponseCodes::InvalidCardNumber.into(),
            )?;
        }

        Ok(())
    }

    /// Handle reversal request
    ///
    /// Extracts necessary fields from the ISO message and performs reversal.
    async fn handle_reversal_request(&self, iso_msg: &mut IsoMsg) -> Result<(), DomainError> {
        iso_msg.set("message_type", MTI::ReversalResponse.into())?;

        // extract transaction hash from the ISO message
        // first 64 characters are the hash
        let private_data = iso_msg.bmp_child_value(126)?;

        let (tx_hash, _) = private_data.split_at(64);

        let validation_result = self.validate(iso_msg).await?;

        // early return if not approved
        if validation_result != ResponseCodes::Approved {
            iso_msg.set_on(RESPONSE_CODE_FIELD_NUMBER, validation_result.into())?;
            return Ok(());
        }

        if let Ok(Some(transaction)) = self.transaction_controller.find_by_hash(tx_hash).await {
            // early return if already reversed
            if transaction.reversed {
                debug!("Transaction already reversed {:?}", &transaction.hash);
                iso_msg.set_on(
                    RESPONSE_CODE_FIELD_NUMBER,
                    ResponseCodes::InvalidTransaction.into(),
                )?;
                return Ok(());
            }

            // updates to bank accounts
            let update_account = BankAccountUpdate::Balance {
                id: transaction.from,
                amount: transaction.amount,
                transaction_type: TransactionType::Debit,
            };

            if self
                .bank_account_controller
                .update(&transaction.from, &update_account)
                .await
                .is_ok()
            {
                if let Some(beneficiary_id) = transaction.to {
                    let update_recipient_account = BankAccountUpdate::Balance {
                        id: beneficiary_id,
                        amount: transaction.amount,
                        transaction_type: TransactionType::Credit,
                    };
                    self.bank_account_controller
                        .update(&beneficiary_id, &update_recipient_account)
                        .await?;
                }

                // Flags the transaction as reversed
                self.transaction_controller.update(&transaction.id).await?;

                iso_msg.set_on(RESPONSE_CODE_FIELD_NUMBER, ResponseCodes::Approved.into())?;
            } else {
                iso_msg.set_on(
                    RESPONSE_CODE_FIELD_NUMBER,
                    ResponseCodes::InvalidTransaction.into(),
                )?;
            }
        } else {
            iso_msg.set_on(
                RESPONSE_CODE_FIELD_NUMBER,
                ResponseCodes::InvalidTransaction.into(),
            )?;
        }

        Ok(())
    }

    /// Validate Iso8583 message
    ///
    /// Does some sanity checks:
    ///
    /// - Timestamp should be valid
    /// - Card expiration date should match and be in the future
    /// - CVV should match
    /// - Amount should be less than or equal to the balance
    ///
    /// Returns the response code according to ISO-8583 specification
    async fn validate(&self, iso_msg: &IsoMsg) -> Result<ResponseCodes, DomainError> {
        // extract necessary fields from the ISO message
        let card_number = iso_msg.bmp_child_value(2)?;

        if let Ok(Some(bank_account)) = self
            .bank_account_controller
            .find_by_card_number(&card_number)
            .await
        {
            self.validate_with_bank_account(iso_msg, &bank_account)
                .await
        } else {
            Ok(ResponseCodes::InvalidCardNumber)
        }
    }

    /// Same as [`self.validate`] but with already queried [`BankAccount`]
    async fn validate_with_bank_account(
        &self,
        iso_msg: &IsoMsg,
        bank_account: &BankAccount,
    ) -> Result<ResponseCodes, DomainError> {
        // format is: "{}D{}C{}", card_number, exp_date, cvv
        let track_2_data = iso_msg.bmp_child_value(35)?;
        let mut parts = track_2_data.split('D');
        let _ = parts.next().unwrap_or("");
        let remainder = parts.next().unwrap_or("");
        let mut parts = remainder.split('C');
        let card_expiration = parts.next().unwrap_or("");
        let cvv = parts.next().unwrap();

        // %m%d%H%M%S format
        let transaction_timestamp = iso_msg.bmp_child_value(7)?;

        let amount: u32 = iso_msg.bmp_child_value(4)?.trim().parse()?;

        let now = Utc::now();

        // validate the transaction timestamp
        // MMDDHHMMSS format, parse it
        if !utils::validate_timestamp(transaction_timestamp.clone()) {
            log::info!("Invalid timestamp: {:?}", transaction_timestamp);
            return Ok(ResponseCodes::InvalidTransaction);
        }

        // validate the card expiration date
        if card_expiration != bank_account.card_expiration_date.format("%m%y").to_string()
            || bank_account.card_expiration_date <= now
        {
            return Ok(ResponseCodes::ExpiredCard);
        }

        // validate the CVV
        if cvv != bank_account.card_cvv {
            return Ok(ResponseCodes::DoNotHonor);
        }

        // validate the amount
        if amount > bank_account.balance {
            return Ok(ResponseCodes::InsufficientFunds);
        }

        Ok(ResponseCodes::Approved)
    }
}

/// Utility functions
mod utils {
    use chrono::Datelike;

    /// Validate timestamp
    /// MMDDHHMMSS format, parse it
    pub(crate) fn validate_timestamp(timestamp: String) -> bool {
        // %m%d%H%M%S format
        let (mo, rest) = timestamp.split_at(2);
        let (dd, rest) = rest.split_at(2);
        let (_hh, rest) = rest.split_at(2);
        let (_mi, rest) = rest.split_at(2);
        let (_ss, _) = rest.split_at(2);

        // we allow 30 seconds of difference
        let now = chrono::Utc::now();

        timestamp.len() == 10 && Ok(now.day()) == dd.parse() && Ok(now.month()) == mo.parse()
    }
}
