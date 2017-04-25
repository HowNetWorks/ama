FROM debian:jessie-slim

RUN useradd -m app
COPY target/release/ama /home/app/ama
RUN chown -R app:app /home/app/ama

USER app
WORKDIR /home/app

EXPOSE 8080
CMD ./ama