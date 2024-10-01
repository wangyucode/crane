FROM rust:1.81-bullseye AS builder
WORKDIR /app
COPY . .
RUN cargo build --release


FROM scratch
WORKDIR /app
COPY --from=builder /app/target/release/crane .
EXPOSE 8594
CMD ["./crane"]