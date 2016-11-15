FROM eu.gcr.io/hownetworks-148514/rust-base:v1.13.0

RUN mkdir -p /app /build && useradd app

WORKDIR /build/
COPY Cargo.* /build/
COPY src /build/src/

RUN cargo build --release && \
    cp target/release/ama /app && \
    cd / && \
    rm -rf /build /root/.cargo

USER app
WORKDIR /

EXPOSE 8080
ENTRYPOINT ["/app/ama"]
