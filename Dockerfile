FROM lukemathwalker/cargo-chef:latest-rust-alpine AS chef
WORKDIR /app
RUN apk add --no-cache build-base musl-dev openssl-dev pkgconfig fontconfig-dev openssl-libs-static

ENV CARGO_TERM_COLOR=always

FROM chef AS planner
COPY . .
RUN cargo chef prepare

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
WORKDIR /app
COPY . .
RUN cargo build --release --bin renderer

FROM alpine:latest AS runtime
COPY --from=builder /app/target/release/renderer /usr/local/bin/app

# Copy the run_app script
COPY run_app.sh /usr/local/bin/run_app.sh
RUN chmod +x /usr/local/bin/run_app.sh

ENTRYPOINT ["/usr/local/bin/run_app.sh"]
