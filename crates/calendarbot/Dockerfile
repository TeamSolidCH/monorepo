FROM gcr.io/distroless/static:debug-nonroot
WORKDIR /
ARG TARGETPATH=target/x86_64-unknown-linux-musl/release/calendarbot
COPY --chown=nonroot:nonroot ${TARGETPATH} /calendarbot
ENTRYPOINT [ "/calendarbot" ]
