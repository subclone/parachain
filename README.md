# ISO-8553 Infrastracture

This repository contains the infrastructure for ISO-8553 implementation for Substrate based chains. It contains of parts that are responsible for processing ISO-8553 messages, maintaining offchain ledger. With these components, it will be possible to mock full cycle of ISO-8583 messages.

This is the high-level overview of the infrastracture:

![iso-8583-overview](https://github.com/subclone/payment-processor/assets/88332432/01c97bed-2ec8-4041-9702-cf079477e9be)

## Run the demo

For demonstration purposes, `docker-compose` configuration is provided. It will start the following services:

- [Demo merchant application](./interface/README.md)
- [Payment Processor Server](./payment-processor/README.md)
- PCIDSS compliant [oracle](./pcidss/README.md)
- Postgres database

To start the demo, first pull the images:

```bash
docker-compose pull
```

Then start the services:

```bash
docker-compose up
```

You will be able to access the demo merchant application at `http://0.0.0.0:3001`.

## Demo flow

Initial demo documentation is located at [merchant application](./interface/README.md). You can follow it to fully test the setup. Note that, sometimes websocket connection with frontend is lost, so you might need to refresh the page.

## Notes

Some important notes about the project:

- It is a PoC implementation, so there are many places where we cut corners and some things are hard coded.
- ISO-8583:1987 standard is used.
- Unit tests are more like integration tests (somewhat similar to Substrate).

## Contents

Contains three key parts for ISO-8553 message processing:

1. [Demo merchant application](./interface/README.md)
2. [Server application](./payment-processor/README.md) for processing requests from merchant application
3. PCIDSS compliant [oracle](./pcidss/README.md)

### Demo Merchant Application

React application with mock bank interface and a demo checkout page.

### Payment Processor Server

NodeJs server that is responsible for processing requests from merchant application and sending them to the gateway simulator.

### PCIDSS Compliant Oracle Gateway

JSON-RPC based message consumer that maintains offchain ledger and a processor for ISO-8553 messages. In the future will have Substrate chain watcher service integrated.

## References

- [ISO-8583 Research](https://github.com/adit313/ISO8583-Blockchain-Integration-Plan) by Adit Patel
- [ISO-8583 Specification](https://www.iso.org/standard/15870.html) ISO-8583:1987 is used
