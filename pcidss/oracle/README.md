## PCIDSS Compliant Oracle Gateway

PCIDSS Compliant Oracle contains set of services to process ISO-8583 services, sync with Substrate chain and maintain local ledger. In a nutshell, it mocks a traditional bank which has issued some cards and is currently used for implementing a PoC implementation of ISO-8583 on a Substrate based blockchain.

### What it does?

Oracle service implements an RPC API for processing incoming ISO-8583 messages via `MsgProcessor` trait of `iso_8583rs` crate. In the future watcher service will be added, which will be used to subscribe to the Substrate chain for further integration of ISO-8583 standard.

### How to run

**Pre-requisites**

- Rust toolchain (tested with version 1.72.0)
- Postgres database (tested with version 14.9)

OR

- Docker and docker-compose

To run the oracle locally:

```bash
make run

# OR

RUST_LOG=info cargo run --release -p pcidss-oracle -- --dev
```

To build the binary for release and run it:

```bash
make build
# OR
cargo build --release


RUST_LOG=info ./target/release/pcidss-oracle
```

#### Docker and docker-compose

To start the oracle with the deployed Docker image and Postgres Docker container:
    
```bash
docker-compose up
```

If you want to build the Docker image locally, you will need to change Postgres settings to allow Docker container to connect to it (listened_addresses and trust authentication method).

To build the Docker image:

```bash
make docker-build
# OR
docker build --platform linux/x86_64 -t pcidss-oracle .
```

To run the Docker image:

```bash
make docker-run
# OR
# NOTE: your Postgres database should be accessible via host.docker.internal
docker run -p 0.0.0.0:3030:3030 --platform linux/x86_64 -it pcidss-oracle --database-host host.docker.internal --iso8583-spec /usr/bin/spec.yaml
```

#### CLI

Oracle service accepts the following arguments (which can be seen by running `pcidss-oracle --help`):

```bash
Usage: pcidss-oracle [OPTIONS]

Options:
      --database-host <DATABASE_HOST>
          Path to the Postgres database [default: localhost]
      --database-port <DATABASE_PORT>
          Port of the Postgres database [default: 5432]
      --database-user <DATABASE_USER>
          Username of the Postgres database [default: postgres]
      --database-name <DATABASE_NAME>
          Name of the Postgres database [default: postgres]
      --chain-endpoint <CHAIN_ENDPOINT>
          Substrate chain websocket endpoint [default: ws://localhost:9944]
      --iso8583-spec <ISO8583_SPEC>
          ISO-8583 specification file [default: spec.yaml]
      --rpc-port <RPC_PORT>
          RPC port [default: 3030]
      --dev
          Run in development mode (development accounts are injected)
  -h, --help
          Print help
```

> **_NOTE:_** Make sure you pass your local postgres configuration in case it differs from the default values (e.g. `pcidss-oracle --database-host localhost --database-port 5432 --database-user postgres --database-name postgres`). Otherwise, you won't be able to run the oracle.

#### Testing

Oracle service has integration tests for the ISO-8583 message processing logic. You can run them with:

```bash
make test
# OR
cargo test
```

> **_NOTE:_** Integration tests are run in parallel by default. This might cause issues with Postgres database, so we should run them in a single thread and one by one.

#### Linting and formatting

Oracle service uses `rustfmt` and `clippy` for formatting and linting. You can run them with:

```bash
make lint
```

## Documentation

More on proposed architecture (and its comparison to traditional way of doing it):

<img width="930" alt="implementation" src="https://github.com/subclone/payment-processor/assets/88332432/0a700fe7-7deb-49bb-b651-925d78cddb5b">

And more about how it works:

![Implementation plan](https://github.com/dastansam/Grants-Program/assets/88332432/8b832448-9095-4846-95ea-ccaebe5e52a5)

> **_NOTE:_** Actual implementation might slightly differ from the presented solutions above.
