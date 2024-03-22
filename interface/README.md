# Mock checkout and bank interface

This is a demo interface that uses the [PCIDSS Oracle Gateway](../pcidss/README.md) and [Payment Processor](../payment-processor/README.md) to simulate bank dashboard and demo checkout page.

## Overview

Main page is a simulation of a bank dashboard, where you can see basic details about the bank account and its transactions. To ease the testing, there is a button on the top right corner that allows you to switch between development accounts. You can use development accounts to simulate different scenarios.

## How to run

This is a React app and while running in development node, you need to run the [PCIDSS Oracle Gateway](../pcidss/README.md) and [Payment Processor](../payment-processor/README.md) in separate terminals. Please follow the instructions in the respective README files.

It also uses the payment processor server running at `http://localhost:3001` as a proxy. If you want to change the port, you need to update the `package.json` file.

1. Run `yarn` to install dependencies
2. Run `yarn start` to start the interface
3. Open `http://localhost:3002` in your browser

### Docker

You can also run the interface in Docker:

```bash
docker build -t interface .
docker run -p 3002:3002 interface
```

## How to use

Here are the test accounts you can use:

```json
[
  {
    "card_holder_first_name": "Alice",
    "card_number": "4169812345678901",
    "card_cvv": "123",
    "balance": 1000,
    "card_expiry": "03/28"
  },
  {
    "card_holder_first_name": "Bob",
    "card_number": "4169812345678902",
    "card_cvv": "124",
    "balance": 0,
    "card_expiry": "03/28"
  },
  {
    "card_holder_first_name": "Charlie",
    "card_number": "4169812345678903",
    "card_cvv": "125",
    "balance": 12345,
    "card_expiry": "03/28"
  },
  {
    "card_holder_first_name": "Dave",
    "card_number": "4169812345678904",
    "card_cvv": "126",
    "balance": 1000000,
    "card_expiry": "03/28"
  },
  {
    "card_holder_first_name": "Eve",
    "card_number": "4169812345678905",
    "card_cvv": "127",
    "balance": 1000,
    "card_expiry": "06/23"
  },
  {
    "card_holder_first_name": "Acquirer",
    "card_number": "123456",
    "card_cvv": "000",
    "balance": 1000000000,
    "card_expiry": "03/28"
  },
  {
    "card_holder_first_name": "Alice_stash",
    "card_number": "4169812345678908",
    "card_cvv": "999",
    "balance": 0,
    "card_expiry": "03/28"
  },
  {
    "card_holder_first_name": "Bob_stash",
    "card_number": "4169812345678909",
    "card_cvv": "888",
    "balance": 0,
    "card_expiry": "03/28"
  },
]
```

#### Main page

Open `http://localhost:3001` in your browser:

<img width="1704" alt="Main page" src="https://github.com/subclone/payment-processor/assets/88332432/869e84eb-d75a-42b6-bb23-c56ec819df90">

#### Payment

Click on `Go To Checkout` and fill out with Alice card details above:

<img width="1704" alt="Alice" src="https://github.com/subclone/payment-processor/assets/88332432/5a39beaa-c36c-4370-8e22-fafabf989a7e">

Once the payment is processed and approved, you will be redirected back to the `Dashboard`:

<img width="1704" alt="Back-to-main" src="https://github.com/subclone/payment-processor/assets/88332432/4cd5699f-e906-49c4-88f3-06afbc60bf9f">

#### Reversal

Once you are in the main page, you will see that each transaction has a `Reverse` button.

https://github.com/subclone/payment-processor/assets/88332432/e66d0df1-fbc9-4cf7-aca2-76cbe92da2a9

If the reversal is successfull, then `Reversed` field of `transaction` will be set to `True`:

<img width="1704" alt="reversed" src="https://github.com/subclone/payment-processor/assets/88332432/e85c469b-c094-4284-8f23-0110b453a3bf">

#### Failures

Expired card:

<img width="1704" alt="expired-card" src="https://github.com/subclone/payment-processor/assets/88332432/2a7e8b91-feb8-43d0-aff3-12e94222605f">

Wrong CVV:

<img width="1704" alt="wrong-cvv" src="https://github.com/subclone/payment-processor/assets/88332432/aa35187e-224e-4241-8ad0-1b2fa1bf30de">

Non-existing card number:

<img width="1704" alt="wrong-card-number" src="https://github.com/subclone/payment-processor/assets/88332432/a2a5ce53-9b91-4248-8b4a-b513529a6763">

