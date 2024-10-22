FROM gcr.io/distroless/static:debug-nonroot
WORKDIR /
COPY --chown=nonroot:nonroot target/x86_64-unknown-linux-musl/release/calendarbot /calendarbot
ENTRYPOINT [ "/calendarbot" ]
