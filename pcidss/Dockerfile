FROM docker.io/rust:slim-bullseye as builder

WORKDIR /pcidss

COPY . .

RUN  --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/pcidss/target \
    cargo build --release --locked -p pcidss-oracle

FROM docker.io/debian:bullseye-slim

COPY --from=builder /pcidss/target/release/pcidss-oracle /usr/bin

COPY --from=builder /pcidss/spec.yaml /usr/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /oracle oracle

# Create entrypoint script
RUN echo '#!/bin/bash\nset -xeu\n/usr/bin/pcidss-oracle $@' > /usr/bin/entrypoint.sh && \
    chmod +x /usr/bin/entrypoint.sh

ENV RUST_LOG=info
# Switch to user oracle
USER oracle

EXPOSE 3030
ENTRYPOINT ["/usr/bin/entrypoint.sh"]
