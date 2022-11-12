FROM rust:latest

WORKDIR /usr/src/app
RUN groupadd -g 999 -r artemis && useradd -u 999 --no-log-init -r -g artemis artemis
COPY . .
run cargo install --path .
USER artemis
CMD ["http"]
