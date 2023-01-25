FROM lukemathwalker/cargo-chef:latest-rust-1.59.0 as chef

WORKDIR /app

RUN apt update && apt install lld clang -y

# -----------------------------

FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

# -----------------------------

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . . 


ENV SQLX_OFFLINE true

RUN cargo build --release --bin audius_network_monitor

# ----------------------------

FROM debian:bullseye-slim AS runtime

WORKDIR /app

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates cron \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/audius_network_monitor audius_network_monitor

COPY migrations migrations
COPY configuration configuration
ENV APP_ENVIRONMENT stage


# Copy hello-cron file to the cron.d directory
COPY network-monitoring-cron /etc/cron.d/network-monitoring-cron

# Give exeuction rights to entrypoint script
RUN chmod 0744 /network-monitoring/new_job.sh
 
# Give execution rights on the cron job
RUN chmod 0644 /etc/cron.d/network-monitoring-cron

# Apply cron job
RUN crontab /etc/cron.d/network-monitoring-cron
 
# Create the log file to be able to run tail
RUN touch /var/log/cron.log
 
# Run the command on container startup
CMD ["cron", "&&", "tail", "-f", "/var/log/cron.log"]