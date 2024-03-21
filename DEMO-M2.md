# Demo for M2

For demonstration purposes, `docker-compose` configuration is provided. It will start the following services:

- [Demo merchant application](./interface/README.md)
- [Payment Processor Server](./payment-processor/README.md)
- PCIDSS compliant [oracle](./pcidss/README.md)
- Substrate chain

To start the demo, first pull the images:

```bash
docker-compose pull
```

Then start the services:

```bash
docker-compose up
```

You will be able to access the demo merchant application at `http://localhost:3002`.

And for the Substrate chain, you can access the explorer [here](https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9944#/explorer).

## Milestone Goals

1. On-chain addresses can be associated with bank accounts
2. On-chain balance is synced with off-chain balance, off-chain ledger serves as a source of truth
3. It is possible to trigger ISO-8583 transactions (both payment and reversal) both from POS and on-chain transactions
4. On-chain messages are converted to ISO-8583 format and processed by the oracle
5. Oracles settle finalised ISO-8583 transactions on-chain

## Demo flow

### On-chain address association

By opening the demo merchant application, main page is a simulation of a bank dashboard, where you can see basic details about the bank account and its transactions. To ease the testing, there is a button on the top right corner that allows you to switch between development accounts. You can use development accounts to simulate different scenarios.

When switching between accounts, you will notice that balance and transactions are different.

When an address you switched to is not associated with any bank account, you will be redirected to the registration page, which will ask for your card details. After submitting the form, you will be redirected back to the dashboard. Registration request is ISO-8583 message, which is processed and settled on chain by the oracle.

And now when you select the address, it will show bank account details and transactions.

### On-chain balance synchronization

If you check for account balances from the explorer, you will see that they match what is shown on the dashboard.

### ISO-8583 transactions

Now, to actually see how on-chain balance is synced and how ISO-8583 transactions are processed, you can use the checkout page. It is a simple form that asks for card details and amount to transfer. It is a simulation of a POS terminal, part of delivery of Milestone 1. Submit the form with one of the test accounts and you will see the transaction on the dashboard.

By checking the explorer, you will notice that on-chain balance is updated after couple of blocks (10 blocks ~30s currently). This is not a limitation of the system, because the source of truth is always off-chain ledger.

With Milestone 2, we added the ability to trigger ISO-8583 transactions with on-chain transaction. To do that, you have to switch to `Crypto` tab and click on `Pay` button. It will trigger an extrinsic which you have to sign and submit. After that, you will see the transaction both on the dashboard and in the explorer.

### Settlement

Note that in the explorer, you will initially see `InitiateTransfer` event, and after couple of blocks `ProcessedTransaction` event. This is because of event driven nature of current implementation. Most of the times, however, transaction is initiated and processed in the same or next block. Since we are using off-chain ledger as a source of truth, on-chain settlement is not really important, however it is important for UX since wallets need to be notified when transaction is settled (i.e by tracking `ProcessedTransaction` event).

## Notes
