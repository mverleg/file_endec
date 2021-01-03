
FROM ekidd/rust-musl-builder:nightly-2020-11-19 AS build

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

RUN cargo build --all-targets --release --tests

# Code changes invalidate cache beyond here main code separately

COPY ./src/ src/
RUN bash -c 'touch -c src/*'

# Build

RUN cargo --offline build --all-targets --release --bin fileenc
RUN cargo --offline build --all-targets --release --bin filedec

RUN cargo --offline run --release --bin fileenc -- --help
RUN cargo --offline run --release --bin filedec -- --help

RUN mv "$(find . -executable -name fileenc)" "$(find . -executable -name filedec)" .

# Run checks

COPY ./test_files/ test_files/
ENV ENDEC_TEST_FILE_DIR=/app/test_files/
RUN cargo --offline test --release --all-targets

RUN cargo --offline clippy --release --all-targets -- -D warnings

RUN cargo --offline fmt --all -- --check

RUN cargo --offline doc --no-deps --release

RUN cargo --offline audit --deny warnings
RUN cargo --offline deny check advisories
RUN cargo --offline deny check bans
#RUN cargo --offline outdated --exit-code 1


# Executable-only image

FROM scratch as execute

WORKDIR /data

ENV RUST_BACKTRACE=1

COPY --from=build /app/fileenc /app/filedec /

CMD ["/fileenc", "--help"]

