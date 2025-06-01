FROM rust:1.87.0-slim

# copy derive crate
COPY ./eduflow_derive/src ./eduflow_derive/src
COPY ./eduflow_derive/Cargo.toml ./eduflow_derive/.
COPY ./eduflow_derive/Cargo.lock ./eduflow_derive/.
# copy main crate
COPY ./src ./src
COPY ./Cargo.toml .
COPY ./Cargo.lock .

RUN cargo build --release

COPY ./target/release/eduflow-backend ./eduflow-backend

EXPOSE 3000

ENTRYPOINT [ "./eduflow-backend" ]

