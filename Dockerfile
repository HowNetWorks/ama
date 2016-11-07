FROM debian:latest

RUN apt-get -y update && \
    apt-get -y upgrade && \
    apt-get -y install \
        curl \
        gcc-4.9 \
    && \
    rm -rf /var/lib/apt/lists

# Ref: https://github.com/rust-lang-nursery/rustup.rs/issues/747
# We don't need or want docs. They are 100MB+. Convert to rustup or
# alternative if and when docs can be left out from installation.
RUN export RUST=rust-1.12.1-x86_64-unknown-linux-gnu ; \
    mkdir -p /tmp/rust && \
    curl -Ssf -o- https://static.rust-lang.org/dist/$RUST.tar.gz | \
    tar zxf - --strip-components 1 -C /tmp/rust && \
    /tmp/rust/install.sh --without=rust-docs && \
    rm -rf /tmp/rust

RUN mkdir -p /app /build
RUN useradd app

WORKDIR /build/
COPY Cargo.* /build/
COPY src /build/src/

RUN export RUSTFLAGS="-C linker=/usr/bin/gcc-4.9" && \
    cargo build --release && \
    cp target/release/ama-cymru /app && \
    cd / && \
    rm -rf /build /root/.cargo

USER app
WORKDIR /

EXPOSE 8080
ENTRYPOINT ["/app/ama-cymru"]
