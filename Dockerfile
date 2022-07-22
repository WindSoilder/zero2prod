# Builder stage
FROM rust:1.62.1 AS builder

# Let's switch our working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does not exist already.
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE true
# Let's build our binary!
# We'll use the release profile to make it fast
RUN cargo build --release

# Runtime stage
FROM rust:1.62.1 AS runtime
WORKDIR /app
# Copy the compiled binary from the builder environment to our runtime environment
COPY --from=builder /app/target/release/zero2prod zero2prod
# We need the configuration file at runtime.
COPY configuration configuration
ENV APP_ENVIRONMENT production
# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./zero2prod"]
