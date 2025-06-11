# build container
FROM rust:1.87.0 AS builder

WORKDIR /app

# copy derive crate
COPY ./eduflow_derive/src ./eduflow_derive/src
COPY ./eduflow_derive/Cargo.toml ./eduflow_derive/.
COPY ./eduflow_derive/Cargo.lock ./eduflow_derive/.
# copy main crate
COPY ./src ./src
COPY ./Cargo.toml .
COPY ./Cargo.lock .

RUN cargo build --release

RUN cp ./target/release/eduflow-backend /eduflow-backend

# runner container
FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /eduflow-backend .
# create data directory (for db)
RUN mkdir data

EXPOSE 3000

ENTRYPOINT [ "./eduflow-backend" ]

