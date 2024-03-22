# Mock Bank API

Represents a mock of a bank API that is used by the [Oracle](../oracle/README.md) to simulate a real bank API. In a nutshell, it's an implementation of `BankAccount` and `Transaction` traits.

In a real world scenario, this API would be provided by a bank and would be used by the oracle to fetch the account balance and to send the transaction details.

## How to run

Currently it's only used as a service by oracle, so there is no way to run it separately. Please refer to the [Oracle](../oracle/README.md) README for more information.

## Testing

Since it is used by the oracle, it's functionality is tested in the oracle [integration tests](../oracle/src/tests/mod.rs).
