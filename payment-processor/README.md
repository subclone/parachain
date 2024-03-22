## Payment Processor

Simulates a traditional payment processor which is used to process ISO-8583 messages from clients (i.e PoS terminals, ATMs, web payment gateways, etc.) and send them to the appropriate destination. You could think of it as a naive simulation of Stripe or PayPal. It also exposes an endpoint for the offchain worker to query the balances of the accounts.

It is part of the infrastracture for integrating ISO-8583 standard for Substrate.

### What it does?

#### Enpoints

- `/pos`: which receives metadata of a plastic card and a transaction amount from the client, forms ISO-8583 message and sends `AuthorizationRequest` to the [Oracle Gateway](../pcidss/README.md) for further processing.
- `/reverse`: which receives a transaction id from the client, forms ISO-8583 message and sends `ReversalRequest` to the Oracle Gateway for further processing.
- `/register`: which receives a card number and a public key from the client, forms ISO-8583 message and sends `RegisterRequest` to the Oracle Gateway for further processing.
- `/balances`: which receives a batch on-chain addresse from the offchain worker and returns the balances of the accounts reading it from the offchain ledger.

#### PCIDSS Compliant Oracle

It maintains a constant websocket connection to the oracle gateway RPC and sends the ISO-8583 messages to it by converting it from user requests. When oracle is done with processing the message, it sends the response back to this server which then sends it back to the client.

### How to run

It is a simple Express.js API, so you need to have `node` and `yarn/npm` installed on your machine. 

> **_NOTE:_** Was tested with `node` version `v20.5.1` and `yarn` version `1.22.10`.

```bash
yarn install

# start the dev server
yarn run dev-server
```

> **_NOTE:_** You can set `PORT`, `ORACLE_RPC_URL` and `NODE_ENV` environment variables according to your needs.

### Build and run in Docker

```bash
docker build -t payment-processor .
docker run -p 3001:3001 -e ORACLE_RPC_URL=<oracle-rpc-url> payment-processor
```
