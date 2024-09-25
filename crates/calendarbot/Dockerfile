
FROM rust:1.76 AS build
WORKDIR /app

RUN apt-get update -y && apt-get install -y --no-install-recommends libpq-dev=15.6-0+deb12u1
RUN touch .env

COPY . .
RUN cargo build --release 


FROM debian:trixie-slim
WORKDIR /app

RUN apt-get update -y && apt-get install -y --no-install-recommends libpq-dev=16.2-1 \
	&& apt-get clean \
	&& rm -rf /var/lib/apt/lists/*

RUN useradd -u 1000 runner
USER runner


COPY --from=build /app/target/release/calendarbot .

COPY --from=build /app/.env ./.env

ENTRYPOINT ["/app/calendarbot"]

