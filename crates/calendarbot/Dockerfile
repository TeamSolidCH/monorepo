FROM debian:bookworm-slim
RUN apt-get update -y && apt-get install -y --no-install-recommends libpq-dev=15.8-0+deb12u1 \
	&& apt-get clean \
	&& rm -rf /var/lib/apt/lists/*

WORKDIR /app
ARG TARGETDIR
COPY  ${TARGETDIR} /usr/local/bin
COPY  /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
ENTRYPOINT ["/usr/local/bin/calendarbot"]
