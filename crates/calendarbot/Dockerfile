FROM gcr.io/distroless/cc-debian12
WORKDIR /app
ARG TARGETDIR
COPY  ${TARGETDIR} /usr/local/bin
ENTRYPOINT ["/usr/local/bin/calendarbot"]
