ARG DEV_APP_IMAGE=node:24.22.0
ARG WEB_SERVER_IMAGE=ubuntu:26.04

FROM "${DEV_APP_IMAGE}" AS dev-app

WORKDIR /web-app
COPY web-app/package.json web-app/yarn.lock web-app/.yarnrc.yml ./
RUN corepack enable yarn
RUN yarn install

COPY web-app/. ./

RUN yarn build


FROM scratch AS web-app
COPY --from=dev-app /web-app/dist /


FROM "${WEB_SERVER_IMAGE}" AS rust

ARG RUST_ARCH=x86_64-unknown-linux-gnu
ARG RUSTUP_VERSION=1.29.1
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.99.1

RUN apt update && apt install -y --no-install-recommends wget ca-certificates
RUN update-ca-certificates 

RUN set -eux; \
    url="https://static.rust-lang.org/rustup/archive/${RUSTUP_VERSION}/${RUST_ARCH}/rustup-init"; \
    wget "$url"; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${RUST_ARCH}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;

RUN apt install -y --no-install-recommends libmysqlclient-dev build-essential libssl-dev pkg-config

FROM rust AS chef
RUN cargo install --locked cargo-chef
WORKDIR /web-server

FROM chef AS planner
COPY diesel.toml Cargo.toml Cargo.lock ./
COPY src ./src 
COPY migrations ./migrations
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /web-server/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY diesel.toml Cargo.toml Cargo.lock ./
COPY src ./src 
COPY migrations ./migrations
RUN cargo build --release


FROM "${WEB_SERVER_IMAGE}" AS web-server
RUN apt update && apt-get install -y --no-install-recommends libmysqlclient-dev
RUN apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/web-server

COPY --from=builder /web-server/target/release/gene-platform /web-server/target/release/new_admin_user /web-server/target/release/migrate_db ./

CMD [ "./gene-platform" ]
