FROM rust:1.81 as builder
WORKDIR /usr/src/crane
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install ca-certificates && apt-get clean
COPY --from=builder /usr/local/cargo/bin/crane /usr/local/bin/crane
EXPOSE 8594
CMD ["crane"]