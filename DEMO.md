# Demo for M2

For demonstration purposes, `docker-compose` configuration is provided. It will start the following services:

- [Demo merchant application](./interface/README.md)
- [Payment Processor Server](./payment-processor/README.md)
- PCIDSS compliant [oracle](./pcidss/README.md)
- [ISO-8583 compliant Substrate chain](https://github.com/subclone/iso8583-chain)

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

Demo of on-chain extrinsics and their interaction with the oracle gateway is documented [here](https://github.com/subclone/iso8583-chain/blob/main/DEMO.md). This demo is aimed at showing the user facing side of the infrastracture, and in general the end product of all components.

### Prerequisites

First and foremost, insert the offchain worker key by running this command:

```bash
curl -H "Content-Type: application/json" \
 --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["'"iso8"'", "'"news slush supreme milk chapter athlete soap sausage put clutch what kitten"'", "'"0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"'"],"id":1 }' \
"http://localhost:9944"
```

#### Accounts and their roles

- `Alice`, `Bob` - oracle wallets, i.e used for submitting finality of ISO-8583 by PCIDSS oracles.
- `Charlie`, `Dave` - wallets that are associated with corresponding bank account, come with balance and ready for using.
- `Alice_stash`, `Bob_stash` - wallets that will be used to demo associating on-chain accounts to bank account
- `Eve` - an account with expired card
- `5HRD6MDjy9XjX6gNhj7wSAinvpNw1DptfR73LZBz68zH4Gex` - wallet associated with merchant's bank account, i.e it will receive payments from the checkout page.

Use these dev bank accounts for testing payment and reversal. Note that the `Demo User` account has a private key, which you have to add to `Polkadot.js` extension if you want to use it to sign transactions.

```json
[
    {
        "name": "Charlie",
        "card_number": "4169812345678903",
        "expiration_date": "03/28",
        "cvv": "125"
    },
    {
        "name": "Dave",
        "card_number": "4169812345678904",
        "expiration_date": "03/28",
        "cvv": "126"
    },    
    {
        "name": "Demo User",
        "card_number": "",
        "expiration_date": "03/28",
        "cvv": "123",
        "private_key": "intact start solar kind young network dizzy churn crisp custom fuel fabric"
    }
]
```

Use these predefined dev bank accounts for testing. They are not associated with any on-chain account.

```json
[
  {
    "name": "Alice_stash",
    "card_number": "4169812345678908",
    "expiration_date": "03/28",
    "cvv": "999"
  },
  {
    "name": "Bob_stash",
    "card_number": "4169812345678909",
    "expiration_date": "03/28",
    "cvv": "888"
  }
]
```
NOTE: expiration date is always 4 years away from current time, i.e 03/2028 assuming we are in 03/2024.

Now, everything is ready for the demo.

### On-chain address association

By opening the demo merchant application, you will see the simulation of a bank dashboard, where you can see basic details about the bank account and its transactions. To ease the testing, there is a button on the top right corner that allows you to switch between wallets. You can use development accounts to simulate different scenarios.

![Screenshot 2024-03-21 at 22 11 18](https://github.com/subclone/payment-processor/assets/88332432/02f748f4-8c1d-491e-b2aa-887b27fc8e24)

When an address you switched to is not associated with any bank account, you will be redirected to the registration page, which will ask for your card details. After submitting the form with one of the predefined bank account details from above, you will be redirected back to the dashboard. Registration request is ISO-8583 message, which is processed and settled on chain by the oracle.

![Screenshot 2024-03-21 at 22 25 17](https://github.com/subclone/payment-processor/assets/88332432/d2f04aa6-df0c-4218-a523-bd30a9957eed)

On-chain association:

<img width="1712" alt="Screenshot 2024-03-21 at 22 25 48" src="https://github.com/subclone/payment-processor/assets/88332432/55fc127d-30af-49d6-8153-fecac988f627">

### On-chain balance synchronization

If you check for account balances from the explorer, you will see that they match what is shown on the dashboard. And offchain worker periodically (every 10 blocks) runs and updates the latest balance of accounts from the bank backend.

<img width="1712" alt="Screenshot 2024-03-21 at 22 29 03" src="https://github.com/subclone/payment-processor/assets/88332432/899cda0c-8528-48e9-83ce-11d3c4c221b0">

![Screenshot 2024-03-21 at 22 29 30](https://github.com/subclone/payment-processor/assets/88332432/639a7ccc-fc3f-4998-958e-d8ac41990852)

### ISO-8583 transactions

### Payment
Now, to actually see how on-chain balance is synced and how ISO-8583 transactions are processed, we can use the checkout page. It is a simple form that asks for card details in a checkout session, i.e when paying for some goods. It is a simulation of a POS terminal, part of delivery of Milestone 1. Submit the form with any of the dev bank account details, here we will use `Charlie`:

![Screenshot 2024-03-21 at 22 32 54](https://github.com/subclone/payment-processor/assets/88332432/b7f4df39-5782-408e-a6c6-0402ba7963b9)

It will forward us back to the Dashboard where we can see that the balance has been decreased and off-chain ledger transaction is recorded.

![Screenshot 2024-03-21 at 22 33 45](https://github.com/subclone/payment-processor/assets/88332432/ca9c44ba-832b-4821-96c6-01526a819246)

By checking the explorer, you will notice that offchain worker detects change in the balance and updates on-chain balance after couple of blocks (10 blocks ~30s currently). This is not a limitation of the system, because the source of truth is always off-chain ledger.

<img width="1451" alt="Screenshot 2024-03-21 at 22 35 31" src="https://github.com/subclone/payment-processor/assets/88332432/e0ad5ea4-de32-43e4-a79b-3d9a25b8adc7">

With Milestone 2, we added the ability to trigger ISO-8583 transactions with on-chain transaction. To do that, you have to switch to `Crypto` tab and click on `Pay` button. It will trigger an extrinsic which you have to 
sign and submit (if you are using development accounts it will not prompt signature, i.e `Alice`, `Charlie`, etc.)

![Screenshot 2024-03-21 at 23 57 32](https://github.com/subclone/payment-processor/assets/88332432/0ab6d5d4-30c9-4699-a954-2377287fa9ea)

Make sure you have selected a proper wallet, in the screenshot above it is `Dave`.

#### Reversal 

Reversal can be triggered in the dashboard, similar to how it was after the Milestone 1.

![Screenshot 2024-03-22 at 0 09 07](https://github.com/subclone/payment-processor/assets/88332432/ea3670b5-72ea-42e5-975c-a579c9216a37)

Note that this is an off-chain ledger transaction and it can only be reversed once.

For triggering reversal using Polkadot.js, take a look at the [Demo of ISO-8583 chain](https://github.com/subclone/iso8583-chain/blob/main/DEMO.md)

### Settlement

Note that in the explorer, you will initially see `InitiateTransfer` event, and after couple of blocks `ProcessedTransaction` event. This is because of event driven nature of current implementation. Most of the times, however, transaction is initiated and processed in the next or 2 blocks later. Since we are using off-chain ledger as a source of truth, on-chain settlement's speed is not really important, however it is important for UX since wallets need to be notified when transaction is settled (i.e by tracking `ProcessedTransaction` event).

<img width="672" alt="Settlement" src="https://github.com/subclone/payment-processor/assets/88332432/ceb17bfc-63bf-4456-bb74-e5954eea43b3">
