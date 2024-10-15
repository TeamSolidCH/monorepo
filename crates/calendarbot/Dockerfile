FROM node:latest AS base
WORKDIR /app

# Install moon binary
RUN npm install -g @moonrepo/cli

#### SKELETON
FROM base AS skeleton

# Copy entire repository and scaffold
COPY . .
RUN moon docker scaffold calendarbot

#### BUILD
FROM base AS build
RUN apt-get update -y && apt-get install -y --no-install-recommends libpq-dev=15.6-0+deb12u1
# Copy toolchain
COPY --from=skeleton /root/.proto /root/.proto

# Copy workspace skeleton
COPY --from=skeleton /app/.moon/docker/workspace .

# Install toolchain and dependencies
RUN moon docker setup
RUN moon run calendarbot:build
RUN moon docker prune

FROM debian:trixie-slim
WORKDIR /app

RUN apt-get update -y && apt-get install -y --no-install-recommends libpq-dev=16.2-1 \
	&& apt-get clean \
	&& rm -rf /var/lib/apt/lists/*

RUN useradd -u 1000 runner
USER runner

COPY --from=build /app/target/release/calendarbot .

ENTRYPOINT ["/app/calendarbot"]
