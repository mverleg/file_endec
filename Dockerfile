
FROM ekidd/rust-musl-builder:1.48.0 AS build

ENV RUST_BACKTRACE=1
ENV CARGO_HOME=/home/rust/.cargo
ENV RUSTUP_HOME=/home/rust/.rustup
USER root

RUN rustup component add rustfmt
RUN rustup component add clippy
RUN cargo install cargo-outdated
RUN cargo install cargo-audit
RUN cargo install cargo-deny

WORKDIR /app

# Compile dependencies first

COPY ./Cargo.toml ./Cargo.lock ./

RUN mkdir -p ./src && \
    printf 'fn main() { println!("placeholder for compiling dependencies") }' | tee src/encrypt.rs | tee src/decrypt.rs | tee src/bench.rs && \
    printf '' | tee src/lib.rs

RUN cargo build --all-targets --all-features --release --tests

# Code changes invalidate cache beyond here main code separately

COPY ./src/ src/
RUN bash -c 'touch -c src/* test_files/*'

# Build

RUN cargo --offline build --all-targets --all-features --release --bin fileenc
RUN cargo --offline build --all-targets --all-features --release --bin filedec

RUN cargo --offline run --all-features --release --bin fileenc -- --help
RUN cargo --offline run --all-features --release --bin filedec -- --help

RUN mv "$(find . -executable -name fileenc)" "$(find . -executable -name filedec)" .

# Run checks

COPY ./test_files/ test_files/
ENV ENDEC_TEST_FILE_DIR=/app/test_files/
RUN cargo --offline test --release --all-targets --all-features

RUN cargo --offline clippy --release --all-targets --all-features -- -D warnings

RUN cargo --offline fmt --all -- --check

RUN cargo --offline doc --no-deps --all-features --release

RUN cargo --offline audit --deny-warnings
RUN cargo --offline deny check advisories
RUN cargo --offline deny check bans
RUN cargo --offline outdated --exit-code 1


# Executable-only image

FROM scratch as execute

WORKDIR /data

ENV RUST_BACKTRACE=1

COPY --from=build /app/encrypt /app/decrypt /

ENTRYPOINT ["printf", "use either 'encrypt --help' or 'decrypt --help' for for more information"]

